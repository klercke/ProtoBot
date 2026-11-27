ALTER TABLE santa_guilds ADD COLUMN reminder_hours INTEGER NOT NULL DEFAULT 24;
ALTER TABLE santa_guilds ADD COLUMN reminder_sent_at INTEGER;
