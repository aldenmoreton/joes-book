CREATE TABLE IF NOT EXISTS events (
	id     			SERIAL NOT NULL PRIMARY KEY,
	book_id			INT8 NOT NULL REFERENCES books(id),
	chapter_id		INT8 NOT NULL REFERENCES chapters(id),
	is_open			BOOLEAN NOT NULL,
	event_type		TEXT NOT NULL,
	contents		TEXT NOT NULL,
	answer			TEXT DEFAULT NULL,
	closing_time	TIMESTAMPTZ NOT NULL,
	created_at 		TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
