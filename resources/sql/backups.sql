-- This SQL script creates a table for managing server backups in a database.
-- It stores metadata about backup files including their location, size, and type.

CREATE TABLE IF NOT EXISTS `backups`
(
    `id`            INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT, -- unique identifier for the backup
    `server_id`     INTEGER NOT NULL,                           -- reference to the server this backup belongs to
    `filename`      TEXT    NOT NULL,                           -- name of the backup file (e.g., 'backup_2025-08-03_11-58-00.zip')
    `backup_type`   tinyint NOT NULL DEFAULT 0,                 -- 0 => full backup, 1 => incremental backup, 2 => world backup
    `file_size`     INTEGER NOT NULL DEFAULT 0,                 -- size of the backup file in bytes
    `created_at`    INTEGER NOT NULL DEFAULT (STRFTIME('%s', 'now')), -- timestamp in seconds since epoch when backup was created
    `description`   TEXT             DEFAULT NULL,              -- optional description of the backup
    `git_commit_id` TEXT             DEFAULT NULL,              -- git commit ID for incremental backups (NULL for ZIP-based backups)
    FOREIGN KEY (`server_id`) REFERENCES `servers` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
)