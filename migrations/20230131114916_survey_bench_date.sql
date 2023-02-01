ALTER TABLE survey_responses
	ADD COLUMN submitted_at TIMESTAMPTZ NOT NULL DEFAULT now();
