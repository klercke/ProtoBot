CREATE TABLE santa_participants_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    guild_id TEXT NOT NULL
    REFERENCES santa_guilds (guild_id)
    ON DELETE CASCADE,
    user_id TEXT NOT NULL,
    registered_at INTEGER NOT NULL,
    steam TEXT,
    UNIQUE (guild_id, user_id)
);

INSERT INTO santa_participants_new (id, guild_id, user_id, registered_at, steam)
SELECT
    id,
    guild_id,
    user_id,
    registered_at,
    steam
FROM santa_participants;

DROP TABLE santa_participants;

ALTER TABLE santa_participants_new RENAME TO santa_participants;
