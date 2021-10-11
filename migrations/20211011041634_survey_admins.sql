-- Add migration script here
CREATE TABLE IF NOT EXISTS survey_admins (
	name VARCHAR(100) NOT NULL UNIQUE,
	email VARCHAR(100) UNIQUE DEFAULT NULL,
	email_verified BOOLEAN DEFAULT NULL,
    secret varchar(50) NOT NULL UNIQUE,
	password TEXT NOT NULL,
	ID SERIAL PRIMARY KEY NOT NULL
);

CREATE TABLE IF NOT EXISTS survey_campaigns (
    ID UUID PRIMARY KEY NOT NULL UNIQUE,
	user_id INTEGER NOT NULL references survey_admins(ID) ON DELETE CASCADE,
	name VARCHAR(200) NOT NULL,
    difficulties INTEGER[] NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS survey_responses (
	user_id UUID NOT NULL references survey_users(ID) ON DELETE CASCADE,
	campaign_id UUID NOT NULL references survey_campaigns(ID) ON DELETE CASCADE,
	device_user_provided VARCHAR(400) NOT NULL,
	device_software_recognised VARCHAR(400) NOT NULL,
	ID SERIAL PRIMARY KEY NOT NULL,
	threads INTEGER
);

CREATE TABLE IF NOT EXISTS survey_benches (
	resp_id INTEGER NOT NULL references survey_responses(ID) ON DELETE CASCADE,
	difficulty INTEGER NOT NULL,
	duration FLOAT(8) NOT NULL
);


CREATE TABLE IF NOT EXISTS survey_response_tokens (
	resp_id INTEGER NOT NULL references survey_responses(ID) ON DELETE CASCADE,
	user_id UUID NOT NULL references survey_users(ID) ON DELETE CASCADE,
    ID UUID PRIMARY KEY NOT NULL UNIQUE
);
