CREATE TABLE IF NOT EXISTS books (
	id     			SERIAL NOT NULL PRIMARY KEY,
	name       		TEXT NOT NULL,
	created_at 		TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);