-- This SQL script creates a table for managing Minecraft servers in a database.
-- It includes fields for server configuration, status, and ownership.
-- Ensure you have a `users` table created before running this script, as it references the `users` table for the owner_id foreign key.

CREATE TABLE IF NOT EXISTS `servers`
(
    `id`                INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,       -- unique identifier for the server
    `name`              TEXT    NOT NULL,                                 -- e.g. 'My Minecraft Server'
    `directory`         TEXT    NOT NULL,                                 -- the directory name where the server files are stored, e.g. 'my_minecraft_server' or 'my_minecraft_server (2)'
    `java_executable`   TEXT    NOT NULL,                                 -- path to the Java executable, e.g. '/usr/bin/java' or 'java' for system PATH
    `java_args`         TEXT    NOT NULL DEFAULT '',                      -- additional arguments for Java excluding -Xmx and -Xms, e.g. '-XX:+UseG1GC -XX:MaxGCPauseMillis=200'
    `max_memory`        tinyint NOT NULL DEFAULT 4,                       -- in GB (e.g. 4GB, 8GB, etc.) this will translate to -Xmx${max_memory}G
    `min_memory`        tinyint NOT NULL DEFAULT 2,                       -- in GB (e.g. 2GB, 4GB, etc.) this will translate to -Xms${min_memory}G
    `minecraft_args`    TEXT    NOT NULL DEFAULT 'nogui',
    `server_jar`        TEXT    NOT NULL DEFAULT '',
    `upnp`              tinyint NOT NULL DEFAULT 0,                       -- 0 = false, 1 = true
    `status`            TEXT    NOT NULL DEFAULT 'stopped',               -- e.g. 'stopped', 'starting', 'running', 'stopping', 'error'
    `auto_start`        BOOLEAN NOT NULL DEFAULT 0,                       -- whether the server should start automatically on boot
    `auto_restart`      BOOLEAN NOT NULL DEFAULT 1,                       -- whether the server should restart automatically if it crashes
    `backup_enabled`    BOOLEAN NOT NULL DEFAULT 1,                       -- whether the server should create backups
    `backup_interval`   INTEGER NOT NULL DEFAULT 1440,                    -- in minutes (1440 minutes = 24 hours)
    `description`       TEXT             DEFAULT '',                      -- a short description of the server
    `minecraft_version` TEXT             DEFAULT '',                      -- e.g. '1.20.1', '1.19.4', or `custom`
    `server_type`       TEXT             DEFAULT 'vanilla',               -- e.g. 'vanilla', 'fabric', 'forge', 'neoforge', 'quilt', or `custom`
    `loader_version`    TEXT             DEFAULT NULL,                    -- e.g. '0.14.0', '1.20.1-44.1.23', or `custom`
    `owner_id`          INTEGER NOT NULL,                                 -- the ID of the user who owns the server
    `created_at`        INTEGER NOT NULL DEFAULT (STRFTIME('%s', 'now')), -- timestamp in seconds since epoch
    `updated_at`        INTEGER NOT NULL DEFAULT (STRFTIME('%s', 'now')), -- timestamp in seconds since epoch
    `last_started`      INTEGER          DEFAULT NULL,                    -- timestamp in seconds since epoch, NULL if never started
    FOREIGN KEY (`owner_id`) REFERENCES `users` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
)