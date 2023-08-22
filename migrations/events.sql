CREATE TABLE IF NOT EXISTS events (
	id     			SERIAL NOT NULL PRIMARY KEY,
	title       	TEXT NOT NULL,
	book_id			INT8 NOT NULL REFERENCES books(id),
	chapter_id		INT8 NOT NULL REFERENCES chapters(id),
	status			TEXT NOT NULL,
	event_type		TEXT NOT NULL,
	contents		TEXT NOT NULL,
	closing_time	TIMESTAMP NOT NULL,
	created_at 		TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
	notes 			TEXT
);