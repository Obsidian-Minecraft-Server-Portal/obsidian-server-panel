-- This SQL script creates tables for managing server backups in a database.
-- Backup schedules for automated backups are stored in the database.
-- Actual backup metadata is managed by the obsidian-backups crate using git.

CREATE TABLE IF NOT EXISTS `backup_schedules`
(
    `id`              INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,       -- unique identifier for the schedule
    `server_id`       INTEGER NOT NULL,                                 -- reference to the server this schedule belongs to
    `interval_amount` INTEGER NOT NULL,                                 -- interval amount (e.g., 6 for "every 6 hours")
    `interval_unit`   TEXT    NOT NULL,                                 -- interval unit: 'hours', 'days', or 'weeks'
    `backup_type`     tinyint NOT NULL DEFAULT 0,                       -- 0 => full backup, 1 => incremental backup, 2 => world backup
    `enabled`         BOOLEAN NOT NULL DEFAULT 1,                       -- whether this schedule is active
    `retention_days`  INTEGER          DEFAULT 7,                       -- number of days to retain backups before deletion (NULL = keep forever)
    `last_run`        INTEGER          DEFAULT NULL,                    -- timestamp of last execution in seconds since epoch
    `next_run`        INTEGER          DEFAULT NULL,                    -- timestamp of next scheduled run in seconds since epoch
    `created_at`      INTEGER NOT NULL DEFAULT (STRFTIME('%s', 'now')), -- timestamp in seconds since epoch when schedule was created
    `updated_at`      INTEGER NOT NULL DEFAULT (STRFTIME('%s', 'now')), -- timestamp in seconds since epoch when schedule was last updated
    FOREIGN KEY (`server_id`) REFERENCES `servers` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
)