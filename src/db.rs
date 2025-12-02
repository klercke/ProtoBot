use rusqlite::{Connection, Result};
use tracing::{error, info};

pub fn init(conn: &Connection) -> Result<()> {
    info!("Initializing SQLite schema…");

    let res = conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS santa_participants (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            guild_id        INTEGER NOT NULL
                                REFERENCES santa_guilds(id)
                                ON DELETE CASCADE,
            user_id         TEXT NOT NULL,
            registered_at   INTEGER NOT NULL,
            steam           TEXT,
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
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            guild_id            TEXT NOT NULL UNIQUE,
            draw_at             INTEGER,
            gift_at             INTEGER,
            created_at          INTEGER,
            drawing_event_id    TEXT,
            gifting_event_id    TEXT
        );
    "#,
    );

    match res {
        Ok(_) => {
            info!("SQLite schema initialization OK");
            Ok(())
        }
        Err(e) => {
            error!(?e, "Failed to init SQLite schema");
            Err(e)
        }
    }
}
