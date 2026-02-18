CREATE TABLE IF NOT EXISTS users
(
	id                    SERIAL PRIMARY KEY,
	username              VARCHAR(255) NOT NULL UNIQUE,
	password              VARCHAR(255) NOT NULL,
	permissions           INT NOT NULL DEFAULT 0,
	join_date             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	last_online           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	needs_password_change SMALLINT NOT NULL DEFAULT 0,
	is_active             SMALLINT NOT NULL DEFAULT 1
);
