CREATE TABLE IF NOT EXISTS `server_access`
(
    `id`        INTEGER unsigned PRIMARY KEY AUTOINCREMENT,
    `server_id` INTEGER unsigned NOT NULL,
    `user_id`   INTEGER unsigned NOT NULL
)