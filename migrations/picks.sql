CREATE TABLE IF NOT EXISTS picks (
	id     			SERIAL NOT NULL PRIMARY KEY,
	book_id			INT8 NOT NULL REFERENCES books(id),
	chapter_id		INT8 NOT NULL REFERENCES chapters(id),
	event_id		INT8 NOT NULL REFERENCES events(id),
	user_id			INT8 NOT NULL REFERENCES users(id),
	choice			TEXT NOT NULL,
	wager			INT8 NOT NULL,
	correct			BOOLEAN DEFAULT FALSE
);
