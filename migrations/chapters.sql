CREATE TABLE IF NOT EXISTS chapters (
	id     			SERIAL NOT NULL PRIMARY KEY,
	title       	TEXT NOT NULL,
	book_id			INT8 NOT NULL REFERENCES books(id),
	is_open			BOOLEAN NOT NULL,
	closing_time	TIMESTAMPTZ DEFAULT NULL,
	created_at 		TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
	notes 			TEXT
);