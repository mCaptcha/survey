CREATE TABLE IF NOT EXISTS survey_response_tokens (
	resp_id INTEGER NOT NULL references survey_responses(ID) ON DELETE CASCADE,
	user_id UUID NOT NULL references survey_users(ID) ON DELETE CASCADE,
    ID UUID PRIMARY KEY NOT NULL UNIQUE
);
