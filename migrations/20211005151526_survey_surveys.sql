--CREATE TYPE survey_bench AS (
--	difficulty INTEGER,
--	duration FLOAT(8)
--);

CREATE EXTENSION IF NOT EXISTS hstore;

CREATE TABLE IF NOT EXISTS survey_responses (
	user_id UUID NOT NULL references survey_users(ID) ON DELETE CASCADE,
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
