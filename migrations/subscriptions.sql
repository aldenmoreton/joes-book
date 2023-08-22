CREATE TABLE IF NOT EXISTS subscriptions (
	user_id     	INT8 NOT NULL REFERENCES users(id),
	book_id     	INT8 NOT NULL REFERENCES books(id),
	role       		TEXT NOT NULL
);