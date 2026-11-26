use crate::{Context, Error};

use chrono::{prelude::*, Duration};
use poise::serenity_prelude::UserId;
use poise::serenity_prelude::{self as serenity, CreateScheduledEvent};
use regex::Regex;
use rusqlite::{self, OptionalExtension};
use std::fmt::Write;
use tracing::{debug, error, info, warn};

// Represents a Secret Santa participant (SQLite table santa_participants)
#[derive(Debug)]
struct Participant {
    guild_id: String,
    user_id: String,
    registered_at: i64,
    steam: Option<String>,
}

// Represents a Secret Santa assignment (SQLite table santa_assignments)
#[derive(Debug)]
struct Assignment {
    participant_id: i64,
    giftee_id: i64,
}

// Represents a Secret Santa guild (SQLite table santa_guilds)
#[derive(Debug)]
#[expect(
    clippy::struct_field_names,
    reason = "Mirrors the santa_guilds.guild_id column. Will be renamed in #88"
)]
struct Guild {
    id: i64,
    guild_id: String,
    drawing_time: Option<i64>,
    gifting_time: Option<i64>,
    drawing_event_id: Option<u64>,
    gifting_event_id: Option<u64>,
}

/// One Santa's assignment, ready to send as a DM.
#[derive(Debug)]
struct AssignmentDm {
    giver_id: u64,
    giftee_id: u64,
    giftee_steam: String,
}

// Used to determine whether Secret Santa registrations can change in a server
#[derive(Debug, PartialEq, Eq)]
enum RegistrationState {
    NoEvent,
    Open,
    AlreadyDrawn,
    PastDeadline,
}

impl Participant {
    fn insert(db: &rusqlite::Connection, p: &Self) -> rusqlite::Result<i64> {
        db.execute(
            "INSERT INTO santa_participants (guild_id, user_id, registered_at, steam)
            VALUES (?1, ?2, ?3, ?4)",
            (&p.guild_id, &p.user_id, p.registered_at, &p.steam),
        )?;
        Ok(db.last_insert_rowid())
    }

    fn get(
        db: &rusqlite::Connection,
        guild_id: &str,
        user_id: &str,
    ) -> rusqlite::Result<Option<Self>> {
        let mut stmt = db.prepare(
            "SELECT id, guild_id, user_id, registered_at, steam
            FROM santa_participants
            WHERE guild_id = ?1 AND user_id = ?2",
        )?;

        let row = stmt.query_row([guild_id, user_id], |row| {
            Ok(Self {
                guild_id: row.get(1)?,
                user_id: row.get(2)?,
                registered_at: row.get(3)?,
                steam: row.get(4)?,
            })
        });

        match row {
            Ok(p) => Ok(Some(p)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

impl Assignment {
    fn insert(db: &rusqlite::Connection, a: &Self) -> rusqlite::Result<()> {
        db.execute(
            "INSERT INTO santa_assignments (participant_id, giftee_id)
            VALUES (?1, ?2)",
            (&a.participant_id, &a.giftee_id),
        )?;
        Ok(())
    }
}

impl Guild {
    fn get(db: &rusqlite::Connection, guild_id: &str) -> rusqlite::Result<Option<Self>> {
        let mut stmt = db.prepare(
            r"SELECT id, guild_id, draw_at, gift_at, drawing_event_id, gifting_event_id
            FROM santa_guilds
            WHERE guild_id = ?1",
        )?;

        let row = stmt.query_row([guild_id], |row| {
            Ok(Self {
                id: row.get(0)?,
                guild_id: row.get(1)?,
                drawing_time: row.get(2)?,
                gifting_time: row.get(3)?,
                drawing_event_id: row.get(4)?,
                gifting_event_id: row.get(5)?,
            })
        });

        match row {
            Ok(g) => Ok(Some(g)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn create(
        db: &rusqlite::Connection,
        guild_id: &str,
        draw_at: Option<i64>,
        gift_at: Option<i64>,
        draw_event_id: Option<u64>,
        gift_event_id: Option<u64>,
        announce_channel_id: Option<u64>,
    ) -> rusqlite::Result<i64> {
        let now = chrono::Utc::now().timestamp();
        db.execute(
            "INSERT INTO santa_guilds (guild_id, draw_at, gift_at, created_at, drawing_event_id, gifting_event_id, announce_channel_id)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (guild_id, draw_at, gift_at, now, draw_event_id, gift_event_id, announce_channel_id),
        )?;
        Ok(db.last_insert_rowid())
    }

    fn set_draw_event(
        db: &rusqlite::Connection,
        guild_id: &str,
        event_id: &str,
    ) -> rusqlite::Result<i64> {
        db.execute(
            "UPDATE santa_guilds
            SET drawing_event_id = ?1
            WHERE guild_id = ?2",
            (&event_id, &guild_id),
        )?;
        Ok(db.last_insert_rowid())
    }

    fn set_gift_event(
        db: &rusqlite::Connection,
        guild_id: &str,
        event_id: &str,
    ) -> rusqlite::Result<i64> {
        db.execute(
            "UPDATE santa_guilds
            SET gifting_event_id = ?1
            WHERE guild_id = ?2",
            (&event_id, &guild_id),
        )?;
        Ok(db.last_insert_rowid())
    }
}

/// Initialize a Secret Santa event in a server.
#[poise::command(
    slash_command,
    default_member_permissions = "MANAGE_EVENTS",
    required_bot_permissions = "MANAGE_EVENTS"
)]
#[expect(
    clippy::too_many_lines,
    reason = "This will be split up during the v0.13 module refactor"
)]
pub async fn santa_create(
    ctx: Context<'_>,
    #[description = "The time that you will assign Santas (Unix timestamp)"] draw_at: Option<i64>,
    #[description = "The time when gifts are to be sent (Unix timestamp)"] gift_at: Option<i64>,
) -> Result<(), Error> {
    let discord_guild = ctx.guild_id().ok_or("Command must be used in a server")?;
    let guild_id = discord_guild.to_string();

    info!(
        %guild_id,
        user_id = %ctx.author().id,
        "Received request to create Secret Santa"
    );

    let response = "Initializing Secret Santa for this server...";
    ctx.say(response).await?;

    // Show bot as typing. Ignore any errors that come from this
    let typing = ctx.channel_id().broadcast_typing(&ctx).await;
    if let Err(e) = typing {
        warn!(?e, %guild_id, "Failed to start typing indicator for Secret Santa init");
    }

    // Get a database handle and lock connection
    let db = ctx.data().db.clone();
    let db = db.lock().await;
    debug!("Acquired database lock for Secret Santa initialization");

    // Check if guild already exists in Santa database
    let guild_exists: Option<i64> = db
        .query_row(
            "SELECT id FROM santa_guilds WHERE guild_id = ?1",
            [&guild_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| {
            error!(?e, %guild_id, "Failed to query existing servers");
            e
        })?;

    // Skip doing anything if guild already exists
    if guild_exists.is_some() {
        ctx.say("A Secret Santa event already exists for this server!")
            .await?;
        info!(%guild_id, "Secret Santa already exists, ignoring request");
        return Ok(());
    }

    // Create guild in Santa database
    let _id = Guild::create(
        &db,
        &guild_id,
        draw_at,
        gift_at,
        None,
        None,
        Some(ctx.channel_id().get()),
    )
    .map_err(|e| {
        error!(?e, %guild_id, "Failed to create Secret Santa");
        e
    })?;
    info!(%guild_id, "Created Secret Santa database entry");

    // Get HTTP to create events
    let http = ctx.serenity_context().http.clone();

    // Create drawing event
    if let Some(timestamp) = draw_at {
        let event_time = chrono::Utc
            .timestamp_opt(timestamp, 0)
            .single()
            .ok_or("Invalid drawing timestamp")?;
        let event = CreateScheduledEvent::new(
            serenity::ScheduledEventType::External,
            "Secret Santa Drawing",
            event_time,
        );
        let event = event.location("The North Pole");
        let event = event.description("This is the deadline to sign up for Secret Santa. Santas will be assigned at this time.");
        let event = event.end_time(event_time + Duration::minutes(5));

        match serenity::Builder::execute(event, &http, discord_guild).await {
            Ok(event) => {
                let _r = Guild::set_draw_event(&db, &guild_id, &event.id.get().to_string());
                info!(
                    %guild_id,
                    event_id = event.id.get(),
                    "Created Secret Santa drawing event"
                );
            }
            Err(e) => {
                error!(?e, %guild_id, "Failed to create Secret Santa drawing event");
                ctx.say(format!(
                    "Failed to create Discord event for Secret Santa drawing: {e}"
                ))
                .await?;
            }
        }
    }

    // Create gifting event
    if let Some(timestamp) = gift_at {
        let event_time = chrono::Utc
            .timestamp_opt(timestamp, 0)
            .single()
            .ok_or("Invalid gifting timestamp")?;
        let event = CreateScheduledEvent::new(
            serenity::ScheduledEventType::External,
            "Secret Santa Gifting",
            event_time,
        );
        let event = event.location("The North Pole");
        let event = event.description(
            "This is time you should schedule your Secret Santa gifts to be delivered.",
        );
        let event = event.end_time(event_time + Duration::minutes(5));

        match serenity::Builder::execute(event, &http, discord_guild).await {
            Ok(event) => {
                let _r = Guild::set_gift_event(&db, &guild_id, &event.id.get().to_string());
                info!(
                    %guild_id,
                    event_id = event.id.get(),
                    "Created Secret Santa gifting event"
                );
            }
            Err(e) => {
                error!(?e, %guild_id, "Failed to create Secret Santa gifting event");
                ctx.say(format!(
                    "Failed to create Discord event for Secret Santa gifting: {e}"
                ))
                .await?;
            }
        }
    }

    debug!("Released database lock after Secret Santa initialization.");
    ctx.say("Secret Santa initialized for this server!").await?;
    info!(%guild_id, "Initialized Secret Santa");
    Ok(())
}

/// Sets the drawing and/or gifting time for the Secret Santa.
#[expect(
    clippy::too_many_lines,
    reason = "This will be split up during the v0.13 module refactor"
)]
#[poise::command(
    slash_command,
    default_member_permissions = "MANAGE_EVENTS",
    required_bot_permissions = "MANAGE_EVENTS"
)]
pub async fn santa_set_time(
    ctx: Context<'_>,
    #[description = "Drawing time (Unix timestamp)"] draw_at: Option<i64>,
    #[description = "Gifting time (Unix timestamp)"] gift_at: Option<i64>,
) -> Result<(), Error> {
    let discord_guild = ctx.guild_id().ok_or("Command must be used in a server")?;
    let guild_id = discord_guild.to_string();

    let _ = ctx.channel_id().broadcast_typing(&ctx).await;
    ctx.defer_ephemeral().await?;

    let db = ctx.data().db.clone();
    let db = db.lock().await;

    let Some(mut guild) = Guild::get(&db, &guild_id)? else {
        ctx.say("No Secret Santa event exists for this server! Run `/santa_create` first.")
            .await?;
        return Ok(());
    };

    // Check for out-of-range timestamps before changing anything
    for ts in [draw_at, gift_at].into_iter().flatten() {
        if ts != 0 {
            event_start(ts)?;
        }
    }

    let http = ctx.serenity_context().http.clone();
    let mut notes: Vec<String> = Vec::new();

    // Update struct values and event times
    if let Some(ts) = draw_at {
        notes.extend(
            apply_time(
                &http,
                discord_guild,
                &mut guild.drawing_time,
                guild.drawing_event_id,
                ts,
                "drawing",
            )
            .await,
        );
        if ts == 0 {
            info!(
                %guild_id,
                user_id = %ctx.author().id,
                "Cleared Secret Santa drawing time"
            );
        } else {
            info!(
                %guild_id,
                user_id = %ctx.author().id,
                draw_at = ts,
                "Set Secret Santa drawing time"
            );
        }
    }
    if let Some(ts) = gift_at {
        notes.extend(
            apply_time(
                &http,
                discord_guild,
                &mut guild.gifting_time,
                guild.gifting_event_id,
                ts,
                "gifting",
            )
            .await,
        );
        if ts == 0 {
            info!(
                %guild_id,
                user_id = %ctx.author().id,
                "Cleared Secret Santa gifting time"
            );
        } else {
            info!(
                %guild_id,
                user_id = %ctx.author().id,
                gift_at = ts,
                "Set Secret Santa gifting time"
            );
        }
    }

    // Write updated times to DB, and re-arm the automatic draw if the draw time changed
    db.execute(
        "UPDATE santa_guilds
         SET draw_at = ?1,
             gift_at = ?2,
             auto_draw_attempted_at = CASE WHEN ?4 THEN NULL ELSE auto_draw_attempted_at END
         WHERE id = ?3",
        (
            &guild.drawing_time,
            &guild.gifting_time,
            &guild.id,
            draw_at.is_some(),
        ),
    )?;

    let draw_display = guild.drawing_time.map_or_else(
        || "Not set".to_string(),
        |ts| format!("<t:{ts}:F> (<t:{ts}:R>)"),
    );
    let gift_display = guild.gifting_time.map_or_else(
        || "Not set".to_string(),
        |ts| format!("<t:{ts}:F> (<t:{ts}:R>)"),
    );

    let mut msg = format!(
        "Secret Santa times for this guild:\n**Draw:** {draw_display}\n**Gift:** {gift_display}"
    );
    for note in notes {
        let _ = write!(msg, "\n-# {note}");
    }
    ctx.say(msg).await?;

    Ok(())
}

/// Sets or clears the time for a single Secret Santa event and moves its Discord event to match.
/// Returns a note to be presented to the user when their attention is needed.
async fn apply_time(
    http: &serenity::Http,
    guild: serenity::GuildId,
    time: &mut Option<i64>,
    event_id: Option<u64>,
    ts: i64,
    label: &str,
) -> Option<String> {
    if ts == 0 {
        *time = None;
        return event_id.map(|_| {
            format!(
                "The Discord {label} event has had its time removed, but the event was not deleted. Delete it by hand if you don't want it."
            )
        });
    }

    *time = Some(ts);
    let Some(event_id) = event_id else {
        return Some(format!("There's no Discord {label} event to update."));
    };
    if let Err(e) = move_event(http, guild, event_id, ts).await {
        warn!(?e, guild_id = %guild, label, "Failed to move Secret Santa event");
        return Some(format!("Couldn't update the Discord {label} event."));
    }
    None
}

/// Converts a Unix timestamp to an event start time.
/// Fails for a timestamp that chrono can't represent.
fn event_start(timestamp: i64) -> Result<DateTime<Utc>, Error> {
    Ok(Utc
        .timestamp_opt(timestamp, 0)
        .single()
        .ok_or("Invalid timestamp")?)
}

/// Moves an existing Discord scheduled event to a new start time.
async fn move_event(
    http: &serenity::Http,
    guild: serenity::GuildId,
    event_id: u64,
    timestamp: i64,
) -> Result<(), Error> {
    let start = event_start(timestamp)?;
    let current = guild.scheduled_event(http, event_id, false).await?;
    let duration = current.end_time.map_or(5 * 60, |end| {
        end.unix_timestamp() - current.start_time.unix_timestamp()
    });
    let builder = serenity::EditScheduledEvent::new()
        .start_time(start)
        .end_time(start + Duration::seconds(duration));
    guild.edit_scheduled_event(http, event_id, builder).await?;
    Ok(())
}

/// Checks the state of Secret Santa registrations for a guild
fn registration_state(
    db: &rusqlite::Connection,
    guild_id: &str,
) -> rusqlite::Result<RegistrationState> {
    // Check if an event exists
    let Some(guild) = Guild::get(db, guild_id)? else {
        return Ok(RegistrationState::NoEvent);
    };

    let drawn = has_assignments(db, guild_id)?;
    if drawn {
        return Ok(RegistrationState::AlreadyDrawn);
    }

    // Check if the deadline is passed, but the drawing hasn't happened yet
    if guild
        .drawing_time
        .is_some_and(|ts| ts <= Utc::now().timestamp())
    {
        return Ok(RegistrationState::PastDeadline);
    }

    // None of the above are true, registration is open
    Ok(RegistrationState::Open)
}

/// Checks to see if the Secret Santa drawing has already happened for a guild
fn has_assignments(db: &rusqlite::Connection, guild_id: &str) -> rusqlite::Result<bool> {
    let query = r"
        SELECT EXISTS(
            SELECT 1
            FROM santa_assignments AS a
            JOIN santa_participants AS p
                ON a.participant_id = p.id
            WHERE p.guild_id = ?1
        )
    ";

    db.query_row(query, [guild_id], |row| row.get(0))
}

/// Prints info about the current guild's Secret Santa event
#[poise::command(slash_command)]
pub async fn santa_info(ctx: Context<'_>) -> Result<(), Error> {
    let discord_guild = ctx.guild_id().ok_or("Command must be used in a server")?;
    let guild_id = discord_guild.to_string();

    let db = ctx.data().db.clone();
    let db = db.lock().await;

    let guild: Option<Guild> = {
        let mut stmt = db.prepare(
            "SELECT id, guild_id, draw_at, gift_at, drawing_event_id, gifting_event_id FROM santa_guilds WHERE guild_id = ?1",
        )?;
        stmt.query_row([&guild_id], |row| {
            Ok(Guild {
                id: row.get(0)?,
                guild_id: row.get(1)?,
                drawing_time: row.get(2)?,
                gifting_time: row.get(3)?,
                drawing_event_id: row.get(4)?,
                gifting_event_id: row.get(5)?,
            })
        })
        .optional()?
    };

    let Some(guild) = guild else {
        ctx.say("No Secret Santa event exists for this guild.")
            .await?;
        return Ok(());
    };

    // Fetch number of participants
    let num_participants: i64 = db.query_row(
        "SELECT COUNT(*) FROM santa_participants WHERE guild_id = ?1",
        [&guild.guild_id],
        |row| row.get(0),
    )?;

    // Get times
    let draw_display = guild.drawing_time.map_or_else(
        || "Not set".to_string(),
        |ts| format!("<t:{ts}:F> (<t:{ts}:R>)"),
    );
    let gift_display = guild.gifting_time.map_or_else(
        || "Not set".to_string(),
        |ts| format!("<t:{ts}:F> (<t:{ts}:R>)"),
    );

    // Build
    let mut msg = format!(
        "🎅🕵️ Secret Santa Info 🎄🎁\n\
        **Draw Time:** {draw_display}\n\
        **Gift Time:** {gift_display}\n\
        **Participants:** {num_participants}"
    );

    // Get event links and add to message
    if let Some(id) = guild.drawing_event_id {
        let _ = write!(
            msg,
            "\n**[Drawing event](<https://discord.com/events/{guild_id}/{id}>)**"
        );
    }
    if let Some(id) = guild.gifting_event_id {
        let _ = write!(
            msg,
            "\n**[Gifting event](<https://discord.com/events/{guild_id}/{id}>)**"
        );
    }

    ctx.say(msg).await?;

    Ok(())
}

/// Deletes the Secret Santa event for this server.
#[poise::command(
    slash_command,
    default_member_permissions = "MANAGE_EVENTS",
    required_bot_permissions = "MANAGE_EVENTS"
)]
pub async fn santa_delete(ctx: Context<'_>) -> Result<(), Error> {
    let discord_guild = ctx.guild_id().ok_or("Command must be used in a server")?;
    let guild_id = discord_guild.to_string();

    let db = ctx.data().db.clone();
    let db = db.lock().await;

    info!(
        %guild_id,
        user_id = %ctx.author().id,
        "Requested removal of Secret Santa event"
    );

    // Fetch guild row
    let Some(guild) = Guild::get(&db, &guild_id)? else {
        ctx.say("No Secret Santa event exists for this guild.")
            .await?;
        return Ok(());
    };

    // Delete Discord scheduled events
    let http = ctx.serenity_context().http.clone();
    if let Some(event_id) = guild.drawing_event_id {
        if let Err(e) = discord_guild.delete_scheduled_event(&http, event_id).await {
            warn!(?e, %guild_id, event_id, "Failed to delete drawing event");
        }
    }
    if let Some(event_id) = guild.gifting_event_id {
        if let Err(e) = discord_guild.delete_scheduled_event(&http, event_id).await {
            warn!(?e, %guild_id, event_id, "Failed to delete gifting event");
        }
    }

    debug!(%guild_id, "Clearing Secret Santa rows from database");
    // Delete rows from DB
    db.execute("DELETE FROM santa_guilds WHERE guild_id = ?1", [&guild_id])?;
    db.execute(
        "DELETE FROM santa_assignments WHERE participant_id IN (SELECT id FROM santa_participants WHERE guild_id = ?1)",
        [&guild_id],
    )?;
    db.execute(
        "DELETE FROM santa_participants WHERE guild_id = ?1",
        [&guild_id],
    )?;

    ctx.say("Secret Santa event deleted successfully!").await?;
    info!(%guild_id, "Deleted Secret Santa event");

    Ok(())
}

/// Register for this server's Secret Santa
#[poise::command(slash_command)]
pub async fn santa_register(
    ctx: Context<'_>,
    #[description = "Your Steam profile URL"] steam_url: String,
) -> Result<(), Error> {
    let discord_guild = ctx.guild_id().ok_or("Command must be used in a server")?;
    let guild_id = discord_guild.to_string();

    let user_id = ctx.author().id.to_string();

    // Validate URL (simple regex)
    let steam_regex = Regex::new(r"^https?://steamcommunity\.com/(id|profiles)/[a-zA-Z0-9_-]+/?$")?;

    if !steam_regex.is_match(&steam_url) {
        ctx.say("Invalid Steam profile URL. It should look like `https://steamcommunity.com/id/yourname` or `https://steamcommunity.com/profiles/12345678901234567`")
            .await?;
        return Ok(());
    }

    // Get DB lock
    let db = ctx.data().db.clone();
    let db = db.lock().await;

    // Check if registration is open
    match registration_state(&db, &guild_id)? {
        RegistrationState::Open => {}
        RegistrationState::NoEvent => {
            info!(
                guild_id,
                user_id, "User tried to register for Secret Santa, but no guild event exists"
            );
            ctx.say(
                "No Secret Santa event exists in this server. Ask an admin to run `/santa_create`.",
            )
            .await?;
            return Ok(());
        }
        RegistrationState::AlreadyDrawn => {
            info!(
                guild_id,
                user_id, "User tried to register for Secret Santa, but guild has already drawn"
            );
            ctx.say("Sorry, your server has already drawn Secret Santas this year. Contact a server admin if you think this is a mistake.")
                .await?;
            return Ok(());
        }
        RegistrationState::PastDeadline => {
            info!(
                guild_id,
                user_id, "User tried to register for Secret Santa, but guild registration deadline has already passed"
            );
            ctx.say("Sorry, your server's Secret Santa registration deadline has passed. Contact a server admin if you think this is a mistake.")
                .await?;
            return Ok(());
        }
    }

    // Check if user already registered
    if Participant::get(&db, &guild_id, &user_id)?.is_some() {
        ctx.say("You are already registered for Secret Santa in this server!")
            .await?;
        return Ok(());
    }

    // Insert participant
    let participant = Participant {
        guild_id: guild_id.clone(),
        user_id: user_id.clone(),
        registered_at: Utc::now().timestamp(),
        steam: Some(steam_url.clone()),
    };
    Participant::insert(&db, &participant)?;

    info!(%guild_id, %user_id, "User registered for Secret Santa");

    ctx.say(format!(
        "Successfully registered for Secret Santa! Your Steam profile: {steam_url}"
    ))
    .await?;

    Ok(())
}

/// List all Secret Santa participants in this server
#[poise::command(slash_command)]
pub async fn santa_list(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("Command must be used in a server")?
        .to_string();

    // Get participants from the DB
    let participants: Vec<(String, Option<String>)> = {
        let db = ctx.data().db.clone();
        let db = db.lock().await;

        let mut stmt =
            db.prepare("SELECT user_id, steam FROM santa_participants WHERE guild_id = ?1")?;
        let rows = stmt.query_map([&guild_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
        })?;

        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };

    if participants.is_empty() {
        ctx.say("No participants registered for Secret Santa.")
            .await?;
        return Ok(());
    }

    // Build messages, auto-split at 2000 chars
    let mut msg = String::new();
    for (user_id, steam) in participants {
        let line = format!(
            "- <@{}>: <{}>\n",
            user_id,
            steam.unwrap_or_else(|| "No Steam URL".into())
        );
        if msg.len() + line.len() > 2000 {
            ctx.say(&msg).await?;
            msg.clear();
        }
        msg.push_str(&line);
    }
    if !msg.is_empty() {
        ctx.say(&msg).await?;
    }

    Ok(())
}

/// Unregisters you from this server's Secret Santa
#[poise::command(slash_command)]
pub async fn santa_unregister(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("Command must be used in a server")?
        .to_string();
    let user_id = ctx.author().id.to_string();

    let db = ctx.data().db.clone();
    let db = db.lock().await;

    // Check if registration is open (users can't unregister after registration closes)
    match registration_state(&db, &guild_id)? {
        RegistrationState::Open => {}
        RegistrationState::NoEvent => {
            info!(
                guild_id,
                user_id, "User tried to unregister from Secret Santa, but no guild event exists"
            );
            ctx.say("No Secret Santa event exists in this server, so there's nothing to leave.")
                .await?;
            return Ok(());
        }
        RegistrationState::AlreadyDrawn => {
            info!(
                guild_id,
                user_id, "User tried to unregister from Secret Santa, but guild has already drawn"
            );
            ctx.say("Sorry, your server has already drawn Secret Santas this year. Contact a server admin if you need to withdraw from the event.")
                .await?;
            return Ok(());
        }
        RegistrationState::PastDeadline => {
            info!(
                guild_id,
                user_id, "User tried to unregister from Secret Santa, but guild registration deadline has already passed"
            );
            ctx.say("Sorry, your server's Secret Santa registration deadline has passed. Contact a server admin if you need to withdraw from the event.")
                .await?;
            return Ok(());
        }
    }

    let deleted = db.execute(
        "DELETE FROM santa_participants WHERE guild_id = ?1 AND user_id = ?2",
        [&guild_id, &user_id],
    )?;

    if deleted == 0 {
        ctx.say("You were not registered for Secret Santa.").await?;
        info!(
            %guild_id,
            %user_id,
            "User tried to unregister from Secret Santa, but was not registered"
        );
    } else {
        ctx.say("You have been unregistered from Secret Santa.")
            .await?;
        info!(%guild_id, %user_id, "User unregistered from Secret Santa");
    }

    Ok(())
}

/// Assign giftees to all Secret Santa participants.
#[poise::command(slash_command, default_member_permissions = "MANAGE_EVENTS")]
pub async fn santa_shuffle(
    ctx: Context<'_>,
    #[description = "Replace existing assignments and DM everyone again"] force: Option<bool>,
) -> Result<(), Error> {
    let discord_guild = ctx.guild_id().ok_or("Command must be used in a server")?;
    let guild_id = discord_guild.to_string();

    let guild_name = ctx
        .serenity_context()
        .http
        .get_guild(discord_guild)
        .await
        .map_or_else(|_| "this server".to_string(), |g| g.name);

    info!(
        %guild_id,
        user_id = %ctx.author().id,
        "Requested Secret Santa shuffle"
    );

    let db = ctx.data().db.clone();

    // Check to make sure participants aren't already assigned
    let drawn = {
        let db = db.lock().await;
        has_assignments(&db, &guild_id)?
    };
    if drawn && !force.unwrap_or(false) {
        info!(
            %guild_id,
            user_id = %ctx.author().id,
            "User tried to shuffle after guild was already shuffled"
        );
        let r = poise::CreateReply::default().content("This server has already had its Secret Santas assigned. If you want to re-assign, re-run the command with `force: True`").ephemeral(true);
        ctx.send(r).await?;
        return Ok(());
    }

    // Get participants, shuffle, and assign giftees
    let guild_id_clone = guild_id.clone();
    let (dm_info, replaced) = tokio::task::spawn_blocking(move || {
        let mut db = db.blocking_lock();
        draw_assignments(&mut db, &guild_id_clone)
    })
    .await??;

    if dm_info.is_empty() {
        ctx.say("Not enough participants to shuffle Secret Santa (need at least 2).")
            .await?;
        return Ok(());
    }

    // Notify caller of success
    let response = if replaced > 0 {
        format!("{replaced} Secret Santa assignments reshuffled and stored in the database! DMs will be sent to participants.")
    } else {
        "Secret Santa assignments shuffled and stored in the database! DMs will be sent to participants.".to_string()
    };
    ctx.say(response).await?;

    // Build DM intro
    let intro = if replaced > 0 {
        format!("Secret Santas have been reshuffled in {guild_name}! Your new assigned giftee is")
    } else {
        format!("It's Secret Santa time in {guild_name}! Your assigned giftee is")
    };

    // Send DMs
    let failed = send_assignment_dms(
        &ctx.serenity_context().http,
        &guild_id,
        &guild_name,
        &intro,
        dm_info,
    )
    .await;

    if let Some(report) = failed_dm_report(&failed) {
        let r = poise::CreateReply::default()
            .content(report)
            .ephemeral(true);
        ctx.send(r).await?;
    }

    info!(%guild_id, "Secret Santa shuffle completed");
    Ok(())
}

/// Draws Secret Santa pairs for a guild, replacing any existing ones.
/// Returns the DM info for each pair and how many old pairs were replaced.
fn draw_assignments(
    db: &mut rusqlite::Connection,
    guild_id: &str,
) -> rusqlite::Result<(Vec<AssignmentDm>, usize)> {
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    // Get participants
    let participants: Vec<(i64, String)> = {
        let mut stmt =
            db.prepare("SELECT id, user_id FROM santa_participants WHERE guild_id = ?1")?;
        let rows = stmt
            .query_map([&guild_id], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        rows
    };

    if participants.len() < 2 {
        return Ok((vec![], 0));
    }

    let tx = db.transaction()?;

    // Shuffle
    let mut shuffled = participants;
    shuffled.shuffle(&mut thread_rng());

    // Clear old assignments
    let replaced = tx.execute(
            "DELETE FROM santa_assignments WHERE participant_id IN (SELECT id FROM santa_participants WHERE guild_id = ?1)",
            [&guild_id],
        )?;

    // Assign in a loop, collect DM info
    let mut dm_info = Vec::new();
    for i in 0..shuffled.len() {
        let giver = &shuffled[i];
        let giftee = &shuffled[(i + 1) % shuffled.len()];

        // Insert into DB
        Assignment::insert(
            &tx,
            &Assignment {
                participant_id: giver.0,
                giftee_id: giftee.0,
            },
        )?;

        // Fetch giftee Steam
        let mut stmt2 = tx.prepare("SELECT steam FROM santa_participants WHERE id = ?1")?;
        let giftee_steam: String = stmt2
            .query_row([giftee.0], |r| r.get::<_, Option<String>>(0))?
            .unwrap_or_else(|| "No Steam URL".to_string());

        dm_info.push(AssignmentDm {
            giver_id: giver.1.parse().unwrap_or(0),
            giftee_id: giftee.1.parse().unwrap_or(0),
            giftee_steam,
        });
    }

    tx.commit()?;

    Ok((dm_info, replaced))
}

/// Sends each Santa their assignment by DM.
/// Returns a Vec of UIDs for users who the bot failed to DM.
async fn send_assignment_dms(
    http: &serenity::Http,
    guild_id: &str,
    guild_name: &str,
    intro: &str,
    dm_info: Vec<AssignmentDm>,
) -> Vec<u64> {
    let mut failed = Vec::new();
    for AssignmentDm {
        giver_id,
        giftee_id,
        giftee_steam,
    } in dm_info
    {
        let user_id = UserId::new(giver_id);
        match user_id.to_user(http).await {
            Ok(user) => {
                let wishlist = format!("{giftee_steam}/wishlist");
                let dm_message = serenity::CreateMessage::default()
                    .content(format!(
                        "🎅 {intro} <@{giftee_id}>. Check out their [Steam wishlist]({wishlist}) and run `/santa_info` in {guild_name} to check when your gift should be scheduled to arrive!"
                    ));

                if let Err(e) = user.dm(http, dm_message).await {
                    warn!(
                        ?e,
                        %guild_id,
                        user_id = giver_id,
                        "Failed to DM participant their Secret Santa assignment"
                    );
                    failed.push(giver_id);
                } else {
                    debug!(%guild_id, user_id = giver_id, "Sent Secret Santa assignment DM");
                }
            }
            Err(e) => {
                warn!(
                    ?e,
                    %guild_id,
                    user_id = giver_id,
                    "Failed to fetch user for Secret Santa DM"
                );
                failed.push(giver_id);
            }
        }
    }

    failed
}

/// Takes the `failed` Vec from `send_assignment_dms` and builds a string listing all of them.
fn failed_dm_report(failed: &[u64]) -> Option<String> {
    if failed.is_empty() {
        return None;
    }

    let failed_list = failed
        .iter()
        .map(|id| format!("<@{id}>"))
        .collect::<Vec<_>>()
        .join(", ");

    Some(format!("Failed to DM the following users: {failed_list}. Please ask them to ensure they have DMs enabled from server members."))
}

#[cfg(test)]
mod tests {
    use crate::db;

    use super::*;

    fn setup() -> rusqlite::Connection {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();

        db::migrate(&mut conn).unwrap();

        conn
    }

    fn add_participant(conn: &rusqlite::Connection, user_id: &str) -> i64 {
        let p = Participant {
            guild_id: String::from("8675309"),
            user_id: String::from(user_id),
            registered_at: Utc::now().timestamp(),
            steam: None,
        };
        Participant::insert(conn, &p).unwrap()
    }

    #[test]
    fn guild_event_missing() {
        let conn = setup();

        assert_eq!(
            registration_state(&conn, "8675309").unwrap(),
            RegistrationState::NoEvent
        );
    }

    #[test]
    fn guild_event_open() {
        let conn = setup();

        Guild::create(&conn, "8675309", None, None, None, None, None).unwrap();

        assert_eq!(
            registration_state(&conn, "8675309").unwrap(),
            RegistrationState::Open
        );
    }

    #[test]
    fn guild_event_deadline_passed() {
        let conn = setup();

        Guild::create(
            &conn,
            "8675309",
            Some(Utc::now().timestamp() - 60),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        assert_eq!(
            registration_state(&conn, "8675309").unwrap(),
            RegistrationState::PastDeadline
        );
    }

    #[test]
    fn guild_event_drawn() {
        let conn = setup();

        Guild::create(&conn, "8675309", None, None, None, None, None).unwrap();

        let p1_id = add_participant(&conn, "1");
        let p2_id = add_participant(&conn, "2");

        Assignment::insert(
            &conn,
            &Assignment {
                participant_id: p1_id,
                giftee_id: p2_id,
            },
        )
        .unwrap();

        assert_eq!(
            registration_state(&conn, "8675309").unwrap(),
            RegistrationState::AlreadyDrawn
        );
    }

    #[test]
    fn no_assignments_before_draw() {
        let conn = setup();

        Guild::create(&conn, "8675309", None, None, None, None, None).unwrap();

        add_participant(&conn, "1");
        add_participant(&conn, "2");

        assert!(!has_assignments(&conn, "8675309").unwrap());
    }

    #[test]
    fn assignments_after_draw() {
        let conn = setup();

        Guild::create(&conn, "8675309", None, None, None, None, None).unwrap();

        let p1_id = add_participant(&conn, "1");
        let p2_id = add_participant(&conn, "2");

        Assignment::insert(
            &conn,
            &Assignment {
                participant_id: p1_id,
                giftee_id: p2_id,
            },
        )
        .unwrap();

        assert!(has_assignments(&conn, "8675309").unwrap());
    }

    #[test]
    fn dm_report_empty_returns_none() {
        let failed: Vec<u64> = Vec::new();

        assert!(failed_dm_report(&failed).is_none());
    }

    #[test]
    fn dm_report_contains_ids() {
        let failed = [1, 2];

        let report = failed_dm_report(&failed).unwrap();
        assert!(report.contains("<@1>"));
        assert!(report.contains("<@2>"));
    }

    #[test]
    fn draw_pairs_everyone_once() {
        let mut conn = setup();

        Guild::create(&conn, "8675309", None, None, None, None, None).unwrap();
        add_participant(&conn, "1");
        add_participant(&conn, "2");
        add_participant(&conn, "3");

        let (dms, replaced) = draw_assignments(&mut conn, "8675309").unwrap();

        assert_eq!(dms.len(), 3);
        assert_eq!(replaced, 0);

        // Nobody drew themselves
        assert!(dms.iter().all(|d| d.giver_id != d.giftee_id));

        // Everyone gives exactly once and receives exactly once
        let mut givers: Vec<u64> = dms.iter().map(|d| d.giver_id).collect();
        let mut giftees: Vec<u64> = dms.iter().map(|d| d.giftee_id).collect();
        givers.sort_unstable();
        giftees.sort_unstable();
        assert_eq!(givers, [1, 2, 3]);
        assert_eq!(giftees, [1, 2, 3]);
    }

    #[test]
    fn draw_needs_two_participants() {
        let mut conn = setup();

        Guild::create(&conn, "8675309", None, None, None, None, None).unwrap();
        add_participant(&conn, "1");

        let (dms, replaced) = draw_assignments(&mut conn, "8675309").unwrap();

        assert!(dms.is_empty());
        assert_eq!(replaced, 0);
    }

    #[test]
    fn draw_replaces_existing() {
        let mut conn = setup();

        Guild::create(&conn, "8675309", None, None, None, None, None).unwrap();
        add_participant(&conn, "1");
        add_participant(&conn, "2");
        add_participant(&conn, "3");

        draw_assignments(&mut conn, "8675309").unwrap();
        let (dms, replaced) = draw_assignments(&mut conn, "8675309").unwrap();

        assert_eq!(dms.len(), 3);
        assert_eq!(replaced, 3);
    }
}
