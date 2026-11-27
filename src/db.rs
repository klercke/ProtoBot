use crate::Error;
use rusqlite::Connection;
use tracing::{info, warn};

const MIGRATIONS: &[&str] = &[
    include_str!("../migrations/0001_baseline.sql"),
    include_str!("../migrations/0002_santa_participant_guild_fk.sql"),
    include_str!("../migrations/0003_santa_auto_draw.sql"),
    include_str!("../migrations/0004_santa_budget_rules.sql"),
    include_str!("../migrations/0005_santa_reminders.sql"),
];

pub fn migrate(conn: &mut Connection) -> Result<(), Error> {
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

    // Skip running migrations if the current DB is up to date
    if current == num_migrations {
        info!(
            current,
            latest = num_migrations,
            "Database schema is up to date, skipping migrations"
        );
        return Ok(());
    }

    // Disable foreign key enforcement to avoid deletion cascades
    conn.pragma_update(None, "foreign_keys", false)?;

    // Apply migrations
    let result = apply_pending(conn, current);

    // Re-enable FK enforcement
    conn.pragma_update(None, "foreign_keys", true)?;

    result
}

fn apply_pending(conn: &mut Connection, current: usize) -> Result<(), Error> {
    let tx = conn.transaction()?;

    for (i, sql) in MIGRATIONS.iter().enumerate().skip(current) {
        let version = i + 1;
        tx.execute_batch(sql)?;
        info!(version, "Running database migration");
    }

    tx.pragma_update(None, "user_version", MIGRATIONS.len())?;
    let broken = tx.prepare("PRAGMA foreign_key_check")?.exists([])?;
    if broken {
        return Err("Migrations left broken foreign keys".into());
    }
    tx.commit()?;

    info!(version = MIGRATIONS.len(), "Database upgraded");

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

        conn.pragma_update(None, "foreign_keys", false).unwrap();

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

        conn.execute(
            "INSERT INTO santa_participants (guild_id, user_id, registered_at) VALUES (?1, ?2, ?3)",
            ("123", "1", 0),
        )
        .unwrap();
        let p1_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO santa_participants (guild_id, user_id, registered_at) VALUES (?1, ?2, ?3)",
            ("123", "2", 0),
        )
        .unwrap();
        let p2_id = conn.last_insert_rowid();

        conn.execute(
            "INSERT INTO santa_assignments (participant_id, giftee_id) VALUES (?1, ?2)",
            [p1_id, p2_id],
        )
        .unwrap();

        assert_eq!(version(&conn), 0);

        migrate(&mut conn).unwrap();

        let guild_id: String = conn
            .query_row("SELECT guild_id FROM santa_guilds", [], |row| row.get(0))
            .unwrap();

        let assignment_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM santa_assignments", [], |row| {
                row.get(0)
            })
            .unwrap();

        assert_eq!(guild_id, "123");
        assert_eq!(assignment_count, 1);
        assert_eq!(version(&conn), MIGRATIONS.len());
    }

    #[test]
    fn foreign_keys_enforced_after_migrate() {
        let mut conn = Connection::open_in_memory().unwrap();

        migrate(&mut conn).unwrap();

        // Test that FK enforcement is on and guild_id points to the santa_guilds table
        let result = conn.execute(
            "INSERT INTO santa_participants (guild_id, user_id, registered_at) VALUES (?1, ?2, ?3)",
            ("999", "42", 0),
        );

        assert!(result.is_err());
    }

    #[test]
    fn participant_insert_succeeds_after_migrate() {
        let mut conn = Connection::open_in_memory().unwrap();

        migrate(&mut conn).unwrap();

        conn.execute("INSERT INTO santa_guilds (guild_id) VALUES (?1)", ["123"])
            .unwrap();

        conn.execute(
            "INSERT INTO santa_participants (guild_id, user_id, registered_at) VALUES (?1, ?2, ?3)",
            ("123", "1", 0),
        )
        .unwrap();
    }
}
