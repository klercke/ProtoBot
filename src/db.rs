use rusqlite::{Connection, Result};
use tracing::{info, warn};

const MIGRATIONS: &[&str] = &[include_str!("../migrations/0001_baseline.sql")];

pub fn migrate(conn: &mut Connection) -> Result<()> {
    // Read which schema version this database file is at.
    let current: usize = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

    let num_migrations = MIGRATIONS.len();

    info!(current, latest = num_migrations, "Checking database schema");

    // Check to make sure the current DB version is not newer than the code expects, indicating
    // code rollback or similar.
    if current > num_migrations {
        warn!(
            current,
            latest = num_migrations,
            "Database schema is newer than this build"
        );
        return Ok(());
    }

    for (i, sql) in MIGRATIONS.iter().enumerate().skip(current) {
        let version = i + 1;
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", version)?;
        tx.commit()?;
        info!(version, "Applied database migration");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(conn: &Connection) -> usize {
        conn.pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap()
    }

    #[test]
    fn fresh_database_reaches_latest() {
        let mut conn = Connection::open_in_memory().unwrap();

        migrate(&mut conn).unwrap();

        assert_eq!(version(&conn), MIGRATIONS.len());
    }

    #[test]
    fn migrate_is_idempotent() {
        let mut conn = Connection::open_in_memory().unwrap();

        migrate(&mut conn).unwrap();

        assert!(migrate(&mut conn).is_ok());

        assert_eq!(version(&conn), MIGRATIONS.len());
    }

    #[test]
    fn existing_schema_keeps_data() {
        let mut conn = Connection::open_in_memory().unwrap();

        // The `INTERER` typo is on purpose (this time ;))
        conn.execute_batch(
            r"
                CREATE TABLE IF NOT EXISTS santa_participants (
                    id              INTEGER PRIMARY KEY AUTOINCREMENT,
                    guild_id        TEXT NOT NULL
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
                    drawing_event_id    INTEGER,
                    gifting_event_id    INTERER
                );
            ",
        )
        .unwrap();

        conn.execute("INSERT INTO santa_guilds (guild_id) VALUES (?1)", ["123"])
            .unwrap();

        let guild_id: String = conn
            .query_row("SELECT guild_id FROM santa_guilds", [], |row| row.get(0))
            .unwrap();

        assert_eq!(guild_id, "123");

        assert_eq!(version(&conn), 0);

        migrate(&mut conn).unwrap();

        assert_eq!(version(&conn), MIGRATIONS.len());
    }
}
