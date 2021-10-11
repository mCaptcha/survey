-- Add migration script here
CREATE TABLE IF NOT EXISTS survey_admins (
	name VARCHAR(100) NOT NULL UNIQUE,
	email VARCHAR(100) UNIQUE DEFAULT NULL,
	email_verified BOOLEAN DEFAULT NULL,
    secret varchar(50) NOT NULL UNIQUE,
	password TEXT NOT NULL,
	ID SERIAL PRIMARY KEY NOT NULL
);

CREATE TABLE IF NOT EXISTS survey_challenges (
    ID UUID PRIMARY KEY NOT NULL UNIQUE,
	user_id INTEGER NOT NULL references survey_admins(ID) ON DELETE CASCADE,
	name VARCHAR(200) NOT NULL,
    difficulties INTEGER[] NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
)
