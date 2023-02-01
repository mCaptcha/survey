CREATE TABLE IF NOT EXISTS survey_bench_type (
	name VARCHAR(30) UNIQUE NOT NULL,
	ID SERIAL PRIMARY KEY NOT NULL
);

INSERT INTO survey_bench_type (name) VALUES ('wasm');
INSERT INTO survey_bench_type (name) VALUES ('js');


CREATE OR REPLACE FUNCTION id_in_survey_bench_type(iname varchar) 
RETURNS int LANGUAGE SQL AS $$
   SELECT ID FROM survey_bench_type WHERE name = name;
$$;


ALTER TABLE survey_responses
	ADD  COLUMN submission_bench_type_id INTEGER references survey_bench_type(ID) NOT NULL
	DEFAULT id_in_survey_bench_type('wasm');
