CREATE TABLE IF NOT EXISTS todos (
	id     			SERIAL NOT NULL PRIMARY KEY,
	user_id       	INT8 NOT NULL,
	title       	TEXT NOT NULL,
	completed 		BOOL,
	created_at 		TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);