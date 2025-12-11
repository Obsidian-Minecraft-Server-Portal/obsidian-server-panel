-- This SQL script creates a table for managing Minecraft servers in a database.
-- It includes fields for server configuration, status, and ownership.
-- Ensure you have a `users` table created before running this script, as it references the `users` table for the owner_id foreign key.

CREATE TABLE IF NOT EXISTS `servers`
(
    `id`                INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,       -- unique identifier for the server
    `name`              TEXT    NOT NULL,                                 -- e.g. 'My Minecraft Server'
    `directory`         TEXT    NOT NULL,                                 -- the directory name where the server files are stored, e.g. 'my_minecraft_server' or 'my_minecraft_server (2)'
    `java_executable`   TEXT    NOT NULL,                                 -- path to the Java executable, e.g. '/usr/bin/java'
    `java_args`         TEXT    NOT NULL DEFAULT '',                      -- additional arguments for Java excluding -Xmx and -Xms, e.g. '-XX:+UseG1GC -XX:MaxGCPauseMillis=200'
    `max_memory`        tinyint NOT NULL DEFAULT 4,                       -- in GB (e.g. 4GB, 8GB, etc.) this will translate to -Xmx${max_memory}G
    `min_memory`        tinyint NOT NULL DEFAULT 2,                       -- in GB (e.g. 2GB, 4GB, etc.) this will translate to -Xms${min_memory}G
    `minecraft_args`    TEXT    NOT NULL DEFAULT 'nogui',
    `server_jar`        TEXT    NOT NULL DEFAULT '',
    `upnp`              tinyint NOT NULL DEFAULT 0,                       -- 0 = false, 1 = true
    `status`            tinyint NOT NULL DEFAULT 0,                       -- 0 => idle, 1 => running, 2 => stopped, 3 => error, 4 => starting, 5 => stopping
    `auto_start`        BOOLEAN NOT NULL DEFAULT 0,                       -- whether the server should start automatically on boot
    `auto_restart`      BOOLEAN NOT NULL DEFAULT 1,                       -- whether the server should restart automatically if it crashes
    `backup_enabled`    BOOLEAN NOT NULL DEFAULT 1,                       -- whether the server should create backups
    `backup_cron`       TEXT    NOT NULL DEFAULT '0 0 * * * *',           -- cron string (0 0 * * * * = every hour)
    `backup_retention`  INTEGER NOT NULL DEFAULT 7,                       -- number of backups to keep, e.g. 7 for weekly backups
    `description`       TEXT             DEFAULT '',                      -- a short description of the server
    `minecraft_version` TEXT             DEFAULT '',                      -- e.g. '1.20.1', '1.19.4', or `custom`
    `server_type`       tinyint NOT NULL DEFAULT 0,                       -- 0 => self::vanilla, 1 => self::forge, 2 => self::fabric, 3 => self::neoforge, 4 => self::quilt, >=5 => self::custom,
    `loader_version`    TEXT             DEFAULT NULL,                    -- e.g. '0.14.0', '1.20.1-44.1.23', or `custom`
    `owner_id`          INTEGER NOT NULL,                                 -- the ID of the user who owns the server
    `created_at`        INTEGER NOT NULL DEFAULT (STRFTIME('%s', 'now')), -- timestamp in seconds since epoch
    `updated_at`        INTEGER NOT NULL DEFAULT (STRFTIME('%s', 'now')), -- timestamp in seconds since epoch
    `last_started`      INTEGER          DEFAULT NULL,                    -- timestamp in seconds since epoch, NULL if never started
    `last_update_check` INTEGER          DEFAULT NULL,                    -- timestamp in seconds since epoch, NULL if never checked
    `update_available`  BOOLEAN NOT NULL DEFAULT 0,                       -- whether an update is available for this server
    `latest_version`    TEXT             DEFAULT NULL,                    -- the latest version available for this server (loader or minecraft version)
    FOREIGN KEY (`owner_id`) REFERENCES `users` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
)