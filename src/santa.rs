use crate::{Context, Error};

use chrono::{prelude::*, Duration};
use poise::serenity_prelude::{self as serenity, CreateScheduledEvent, EditScheduledEvent};
use rusqlite::{self, OptionalExtension};
use tracing::{debug, error, info, warn};

// Represents a Secret Santa participant (SQLite table santa_participants)
#[derive(Debug)]
struct Participant {
    id: i64,
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
    assigned_at: i64,
}

// Represents a Secret Santa guild (SQLite table santa_guilds)
#[derive(Debug)]
struct Guild {
    id: i64,
    guild_id: String,
    drawing_time: Option<i64>,
    gifting_time: Option<i64>,
    drawing_event_id: Option<u64>,
    gifting_event_id: Option<u64>,
}

impl Participant {
    fn insert(db: &rusqlite::Connection, p: &Participant) -> rusqlite::Result<i64> {
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
    ) -> rusqlite::Result<Option<Participant>> {
        let mut stmt = db.prepare(
            "SELECT id, guild_id, user_id, registered_at, steam
            FROM santa_participants
            WHERE guild_id = ?1 AND user_id = ?2",
        )?;

        let row = stmt.query_row([guild_id, user_id], |row| {
            Ok(Participant {
                id: row.get(0)?,
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
    fn insert(db: &rusqlite::Connection, a: &Assignment) -> rusqlite::Result<()> {
        db.execute(
            "INSERT INTO santa_assignments (participant_id, giftee_id, assigned_at)
            VALUES (?1, ?2, ?3)",
            (&a.participant_id, &a.giftee_id, &a.assigned_at),
        )?;
        Ok(())
    }

    fn get_for_participant(
        db: &rusqlite::Connection,
        participant_id: i64,
    ) -> rusqlite::Result<Option<Assignment>> {
        let mut stmt = db.prepare(
            "SELECT participant_id, giftee_id, assigned_at
            FROM santa_assignments
            WHERE participant_id = ?1",
        )?;

        let row = stmt.query_row([participant_id], |row| {
            Ok(Assignment {
                participant_id: row.get(0)?,
                giftee_id: row.get(1)?,
                assigned_at: row.get(3)?,
            })
        });

        match row {
            Ok(a) => Ok(Some(a)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

impl Guild {
    fn insert(db: &rusqlite::Connection, guild: &Guild) -> rusqlite::Result<i64> {
        db.execute(
            "INSERT INTO santa_guilds (guild_id, drawing_time, gifting_time)
            VALUES (?1, ?2, ?3)",
            (&guild.guild_id, guild.drawing_time, guild.gifting_time),
        )?;
        Ok(db.last_insert_rowid())
    }

    fn get(db: &rusqlite::Connection, guild_id: &str) -> rusqlite::Result<Option<Guild>> {
        let mut stmt = db.prepare(
            r#"SELECT id, guild_id, drawing_time, gifting_time, drawing_event_id, gifting_event_id
            FROM santa_guilds
            WHERE guild_id = ?1"#,
        )?;

        let row = stmt.query_row([guild_id], |row| {
            Ok(Guild {
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
    ) -> rusqlite::Result<i64> {
        let now = chrono::Utc::now().timestamp();
        db.execute(
            "INSERT INTO santa_guilds (guild_id, draw_at, gift_at, created_at, drawing_event_id, gifting_event_id)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (guild_id, draw_at, gift_at, now, draw_event_id, gift_event_id),
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

/// Initialize a Secret Santa event in a server (requires Administrator permissions in the server).
#[poise::command(slash_command, prefix_command, required_permissions = "ADMINISTRATOR")]
pub async fn santa_create(
    ctx: Context<'_>,
    #[description = "The time that you will assign Santas (Unix timestamp)"] draw_at: Option<i64>,
    #[description = "The time when gifts are to be sent (Unix timestamp)"] gift_at: Option<i64>,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("Command must be used in a guild")?
        .to_string();

    info!(
        "Recieved request to create Secret Santa in guild {} by user {}",
        guild_id,
        ctx.author()
    );

    let response = "Initializing Secret Santa for this server...";
    ctx.say(response).await?;

    // Show bot as typing. Ignore any errors that come from this
    let typing = ctx.channel_id().broadcast_typing(&ctx).await;
    if let Err(e) = typing {
        warn!(?e, "Failed to start typing indicator for Secret Santa init");
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
            error!(?e, "Failed to query existing servers!");
            e
        })?;

    // Skip doing anything if guild already exists
    if guild_exists.is_some() {
        ctx.say("A Secret Santa event already exists for this server!")
            .await?;
        info!(
            "Secret Santa already exists in guild {}. Ignoring request...",
            guild_id
        );
        return Ok(());
    }

    // Create guild in Santa database
    let _id = Guild::create(&db, &guild_id, draw_at, gift_at, None, None).map_err(|e| {
        error!(?e, "Failed to create Secret Santa for guild {}", guild_id);
        e
    })?;
    info!("Created Secret Santa datbase entry for guild {}", guild_id);

    // Get HTTP to create events
    let http = ctx.serenity_context().http.clone();

    // Create drawing event
    if draw_at.is_some() {
        let event_time = chrono::Utc.timestamp_opt(draw_at.unwrap(), 0).unwrap();
        let event = CreateScheduledEvent::new(
            serenity::ScheduledEventType::External,
            "Secret Santa Drawing",
            event_time,
        );
        let event = event.location("The North Pole");
        let event = event.description("This is the deadline to sign up for Secret Santa. Santas will be assigned at this time.");
        let event = event.end_time(event_time + Duration::minutes(5));

        match serenity::Builder::execute(event, &http, ctx.guild_id().unwrap()).await {
            Ok(event) => {
                let _r = Guild::set_draw_event(&db, &guild_id, &event.id.get().to_string());
                info!(
                    "Successfully created Secret Santa drawing event for guild {}. Event ID: {}",
                    guild_id,
                    event.id.get()
                );
            }
            Err(e) => {
                error!(
                    "Failed to create Secret Santa drawing event for guild {}: {}",
                    guild_id, e
                );
                ctx.say(format!(
                    "Failed to create Discord event for Secret Santa drawing: {}",
                    e
                ))
                .await?;
            }
        }
    }

    // Create gifting event
    if gift_at.is_some() {
        let event_time = chrono::Utc.timestamp_opt(gift_at.unwrap(), 0).unwrap();
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

        match serenity::Builder::execute(event, &http, ctx.guild_id().unwrap()).await {
            Ok(event) => {
                let _r = Guild::set_gift_event(&db, &guild_id, &event.id.get().to_string());
                info!(
                    "Successfully created Secret Santa gifting event for guild {}. Event ID: {}",
                    guild_id,
                    event.id.get()
                );
            }
            Err(e) => {
                error!(
                    "Failed to create Secret Santa gifting event for guild {}: {}",
                    guild_id, e
                );
                ctx.say(format!(
                    "Failed to create Discord event for Secret Santa gifting: {}",
                    e
                ))
                .await?;
            }
        }
    }

    debug!("Released database lock after Secret Santa initialization.");
    ctx.say("Secret Santa initialized for this server!").await?;
    info!(
        "Successfully initialized Secret Santa in guild {}",
        guild_id
    );
    Ok(())
}

/// Sets the drawing and/or gifting time for the Secret Santa
#[poise::command(slash_command, prefix_command, required_permissions = "ADMINISTRATOR")]
pub async fn santa_set_time(
    ctx: Context<'_>,
    #[description = "Drawing time (Unix timestamp)"] draw_at: Option<i64>,
    #[description = "Gifting time (Unix timestamp)"] gift_at: Option<i64>,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("Command must be used in a guild")?
        .to_string();

    let _ = ctx.channel_id().broadcast_typing(&ctx).await;
    ctx.defer_ephemeral().await?;

    let db = ctx.data().db.clone();
    let db = db.lock().await;

    let guild = Guild::get(&db, &guild_id).unwrap();
    let mut guild = match guild {
        Some(g) => g,
        None => {
            ctx.say("No Secret Santa event exists for this server! Run `/santa_create` first.")
                .await?;
            return Ok(());
        }
    };

    // Update struct values and event times
    if let Some(ts) = draw_at {
        if ts == 0 {
            guild.drawing_time = None;
            info!(
                "User {} cleared Secret Santa drawing time for guild {}",
                ctx.author(),
                guild_id
            );
        } else {
            // TODO: Make this also update the Discord Scheduled event
            info!(
                "User {} updated Secret Santa drawing time for guild {} to {}",
                ctx.author(),
                guild_id,
                &ts.to_string()
            );
        }
    }
    if let Some(ts) = gift_at {
        if ts == 0 {
            guild.gifting_time = None;
            info!(
                "User {} cleared Secret Santa gifting time for guild {}",
                ctx.author(),
                guild_id
            );
        } else {
            // TODO: Make this also update the Discord Scheduled event
            guild.gifting_time = Some(ts);
            info!(
                "User {} updated Secret Santa gifting time for guild {} to {}",
                ctx.author(),
                guild_id,
                &ts.to_string()
            );
        }
    }

    // Write updated times to DB
    db.execute(
        "UPDATE santa_guilds
         SET draw_at = ?1, gift_at = ?2
         WHERE id = ?3",
        (&guild.drawing_time, &guild.gifting_time, &guild.id),
    )?;

    let draw_display = guild
        .drawing_time
        .map(|ts| format!("<t:{}:F> (<t:{}:R>)", ts, ts))
        .unwrap_or("Not set".to_string());
    let gift_display = guild
        .gifting_time
        .map(|ts| format!("<t:{}:F> (<t:{}:R>)", ts, ts))
        .unwrap_or("Not set".to_string());

    ctx.say(format!(
        "Secret Santa times for this guild:\n**Draw:** {}\n**Gift:** {}",
        draw_display, gift_display
    ))
    .await?;
    Ok(())
}

/// Prints info about the current guild's Secret Santa event
#[poise::command(slash_command, prefix_command)]
pub async fn santa_info(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("Command must be used in a guild")?
        .to_string();

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

    let guild = match guild {
        Some(g) => g,
        None => {
            ctx.say("No Secret Santa event exists for this guild.")
                .await?;
            return Ok(());
        }
    };

    // Fetch number of participants
    let num_participants: i64 = db.query_row(
        "SELECT COUNT(*) FROM santa_participants WHERE guild_id = ?1",
        [&guild.id],
        |row| row.get(0),
    )?;

    // Get times
    let draw_display = guild
        .drawing_time
        .map(|ts| format!("<t:{}:F> (<t:{}:R>)", ts, ts))
        .unwrap_or("Not set".to_string());
    let gift_display = guild
        .gifting_time
        .map(|ts| format!("<t:{}:F> (<t:{}:R>)", ts, ts))
        .unwrap_or("Not set".to_string());

    // Build
    let mut msg = format!(
        "🎅🕵️ Secret Santa Info 🎄🎁\n\
        **Draw Time:** {}\n\
        **Gift Time:** {}\n\
        **Participants:** {}",
        draw_display, gift_display, num_participants
    );

    // Get event links and add to message
    if guild.drawing_event_id.is_some() {
        let draw_event = String::from("https://discord.com/events/")
            + &guild_id
            + "/"
            + &guild.drawing_event_id.unwrap().to_string();
        msg.push_str(&format!("\n**[Drawing event](<{}>)**", draw_event));
    }
    if guild.gifting_event_id.is_some() {
        let gift_event = String::from("https://discord.com/events/")
            + &guild_id
            + "/"
            + &guild.gifting_event_id.unwrap().to_string();
        msg.push_str(&format!("\n**[Gifting event](<{}>)**", gift_event));
    }

    ctx.say(msg).await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn santa_register(ctx: Context<'_>) -> Result<(), Error> {
    let response = "Pong!";
    ctx.say(response).await?;
    Ok(())
}
