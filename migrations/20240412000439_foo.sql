
CREATE TABLE IF NOT EXISTS users (
	"id"			SERIAL NOT NULL PRIMARY KEY,
	"username"		TEXT NOT NULL UNIQUE,
	"password"		TEXT NOT NULL,
	"created_at"	TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS user_permissions (
	"user_id"     SERIAL NOT NULL REFERENCES users(id),
	"token"       TEXT NOT NULL,
	PRIMARY KEY ("user_id", "token")
);

CREATE TABLE IF NOT EXISTS books (
	"id"     			SERIAL NOT NULL PRIMARY KEY,
	"name"       		TEXT NOT NULL,
	"created_at" 		TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS subscriptions (
	"user_id"	SERIAL NOT NULL REFERENCES users(id),
	"book_id"	SERIAL NOT NULL REFERENCES books(id),
	"role"		TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS chapters (
	"id"     			SERIAL NOT NULL PRIMARY KEY,
	"title"       		TEXT NOT NULL,
	"book_id"			SERIAL NOT NULL REFERENCES books(id),
	"is_open"			BOOLEAN NOT NULL DEFAULT FALSE,
	"is_visible"	BOOLEAN NOT NULL DEFAULT FALSE,
	"created_at" 		TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
	"notes" 			TEXT
);

CREATE TABLE IF NOT EXISTS teams (
	"id"				SERIAL NOT NULL PRIMARY KEY,
	"name"				TEXT NOT NULL,
	"logo"				TEXT
);

CREATE TYPE event_types AS ENUM ('spread_group', 'user_input');

CREATE TABLE IF NOT EXISTS events (
	"id"     		SERIAL NOT NULL PRIMARY KEY,
	"book_id"		SERIAL NOT NULL REFERENCES books(id),
	"chapter_id"	SERIAL NOT NULL REFERENCES chapters(id),
	"is_open"		BOOLEAN NOT NULL DEFAULT FALSE,
	"event_type"	event_types NOT NULL,
	"contents"		JSONB NOT NULL,
	"created_at" 	TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS picks (
	"id"     		SERIAL NOT NULL PRIMARY KEY,
	"book_id"		SERIAL NOT NULL REFERENCES books(id),
	"chapter_id"	SERIAL NOT NULL REFERENCES chapters(id),
	"event_id"		SERIAL NOT NULL REFERENCES events(id),
	"user_id"		SERIAL NOT NULL REFERENCES users(id),
	"choice"		JSONB NOT NULL,
	"wager"			JSONB NOT NULL,
	"correct"		BOOLEAN
);

ALTER TABLE IF EXISTS picks
ADD UNIQUE (book_id, chapter_id, event_id, user_id);
