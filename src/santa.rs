use crate::{Context, Error};

use poise::serenity_prelude::{self as serenity, model::guild};
use rusqlite::{self, OptionalExtension};
use tracing::{debug, info, error, warn};

// Represents a Secret Santa participant (SQLite table santa_participants)
#[derive(Debug)]
struct Participant {
    id: i64,
    guild_id: String,
    user_id: String,
    registered_at: i64,
    steam_code: Option<String>,
}

// Represents a Secret Santa assignment (SQLite table santa_assignments)
#[derive(Debug)]
struct  Assignment {
    participant_id: i64,
    giftee_id: i64,
    assigned_at: i64,
}

// Represents a Secret Santa guild (SQLite table santa_guilds)
#[derive(Debug)]
struct Guild {
    id: i64,
    guild_id: String,
    drawing_time: i64,
    gifting_time: i64,
}

impl Participant {
    fn insert(db: &rusqlite::Connection, p: &Participant) -> rusqlite::Result<i64> {
        db.execute(
            "INSERT INTO santa_participants (guild_id, user_id, registered_at, steam_code)
            VALUES (?1, ?2, ?3, ?4)", 
            (&p.guild_id, &p.user_id, p.registered_at, &p.steam_code),
        )?;
        Ok(db.last_insert_rowid())
    }

    fn get(db: &rusqlite::Connection, guild_id: &str, user_id: &str) -> rusqlite::Result<Option<Participant>> {
        let mut stmt = db.prepare(
            "SELECT id, guild_id, user_id, registered_at, steam_code
            FROM santa_participants
            WHERE guild_id = ?1 AND user_id = ?2",
        )?;

        let row = stmt.query_row([guild_id, user_id], |row| {
            Ok(Participant {
                id: row.get(0)?,
                guild_id: row.get(1)?,
                user_id: row.get(2)?,
                registered_at: row.get(3)?,
                steam_code: row.get(4)?,
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

    fn get_for_participant(db: &rusqlite::Connection, participant_id: i64) -> rusqlite::Result<Option<Assignment>> {
        let mut stmt = db.prepare(
            "SELECT participant_id, giftee_id, assigned_at
            FROM santa_assignments
            WHERE participant_id = ?1"
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
            (&guild.guild_id, guild.drawing_time, guild.gifting_time)
        )?;
        Ok(db.last_insert_rowid())
    }

    fn get(db: &rusqlite::Connection, guild_id: &str) -> rusqlite::Result<Option<Guild>> {
        let mut stmt = db.prepare(
            "SELECT id, guild_id, drawing_time, gifting_time
            FROM santa_servers
            WHERE guild_id = ?1",
        )?;

        let row = stmt.query_row([guild_id], |row| {
            Ok(Guild {
                id: row.get(0)?,
                guild_id: row.get(1)?,
                drawing_time: row.get(2)?,
                gifting_time: row.get(3)?,
            })
        });

        match row {
            Ok(g) => Ok(Some(g)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

/// Initialize a Secret Santa event in a server (requires Administrator permissions in the server).
#[poise::command(slash_command, prefix_command, required_permissions = "ADMINISTRATOR")]
pub async fn santa_init(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id()
        .ok_or("Command must be used in a guild")?
        .to_string();

    info!("Recieved request to initialize Secret Santa in guild {} by user {}", ctx.channel_id(), ctx.author());

    let response = "Initializing secret santa for this server...";
    ctx.say(response).await?;

    // Show bot as typing. Ignore any errors that come from this
    let typing = ctx.channel_id().broadcast_typing(&ctx).await;
    if let Err(e) = typing {
        warn!(?e, "Failed to start typing indicator for Secret Santa init");
    }

    // Get a database handle and lock connection
    let db = ctx.data().db.clone();
    let db = db.lock().await;
    debug!("Acquired database lock for Secret Santa initialization.");

    let now = chrono::Utc::now().timestamp();

    db.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS santa_participants (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            guild_id        INTEGER NOT NULL
                                REFERENCES santa_guilds(id)
                                ON DELETE CASCADE,
            user_id         TEXT NOT NULL,
            registered_at   INTEGER NOT NULL,
            steam_code      TEXT,
            UNIQUE (guild_id, user_id)
        );

        CREATE TABLE IF NOT EXISTS santa_assignments (
            participant_id  INTEGER NOT NULL
                                REFERENCES santa_participants(id)
                                ON DELETE CASCADE,
            giftee_id       INTEGER NOT NULL
                                REFERENCES santa_participants(id)
                                ON DELETE CASCADE,
            PRIMARY KEY (participant_id)
        );

        CREATE TABLE IF NOT EXISTS santa_guilds (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            guild_id    TEXT NOT NULL UNIQUE,
            draw_at     INTEGER,
            gift_at     INTEGER,
            created_at  INTEGER
        );
    "#)
    .map_err(|e| {
        error!(?e, "Failed to create Secret Santa tables");
        e
    })?;

    let guild_exists: Option<i64> = db.query_row(
        "SELECT id FROM santa_guilds WHERE guild_id = ?1",
        [&guild_id],
        |row| row.get(0),
    ).optional()
    .map_err(|e| {
        error!(?e, "Failed to query existing guilds");
        e
    })?;

    if guild_exists.is_some() {
        ctx.say("A Secret Santa already exists for this guild!").await?;
        return Ok(());
    }

    db.execute(
        "INSERT INTO santa_guilds (guild_id, created_at) VALUES (?1, ?2)",
        (&guild_id, &now),
    )
    .map_err(|e| {
        error!(?e, "Failed to insert guild into Secret Santa table for guild {}", guild_id);
        e
    })?;

    debug!("Released database lock after Secret Santa initialization.");
    ctx.say("Secret Santa initialized for this server!").await?;
    info!("Successfully initialized Secret Santa in guild {}", guild_id);
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn santa_register(ctx: Context<'_>) -> Result<(), Error> {
    let response = "Pong!";
    ctx.say(response).await?;
    Ok(())
}
