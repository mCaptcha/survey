CREATE TABLE IF NOT EXISTS survey_users (
    ID UUID PRIMARY KEY NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL
)
