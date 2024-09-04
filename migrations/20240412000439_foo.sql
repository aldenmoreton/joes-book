
CREATE TABLE IF NOT EXISTS users (
	"id"			SERIAL NOT NULL PRIMARY KEY,
	"username"		TEXT NOT NULL UNIQUE,
	"password"		TEXT NOT NULL DEFAULT md5((gen_random_uuid())::text),
	"created_at"	TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS oauth (
    user_id integer REFERENCES users(id),
    sub text NOT NULL,
    provider text NOT NULL,
    content jsonb NOT NULL,
    created_at timestamp DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT oauth_pkey PRIMARY KEY (sub, provider)
	CONSTRAINT oauth_user_id_fkey FOREIGN KEY (user_id)
        REFERENCES users (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION
);

CREATE TABLE IF NOT EXISTS signup_tokens (
    sub text NOT NULL,
    provider text NOT NULL,
    token text NOT NULL DEFAULT md5((gen_random_uuid())::text),
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT signup_tokens_pkey PRIMARY KEY (token)
	CONSTRAINT signup_tokens_sub_provider_fkey FOREIGN KEY (sub, provider)
        REFERENCES oauth (sub, provider) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION
        NOT VALID
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
	CONSTRAINT subscriptions_book_id_fkey FOREIGN KEY (book_id)
        REFERENCES books (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION,
    CONSTRAINT subscriptions_user_id_fkey FOREIGN KEY (user_id)
        REFERENCES users (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION
);

CREATE TABLE IF NOT EXISTS chapters (
	"id"     			SERIAL NOT NULL PRIMARY KEY,
	"title"       		TEXT NOT NULL,
	"book_id"			SERIAL NOT NULL REFERENCES books(id),
	"is_open"			BOOLEAN NOT NULL DEFAULT FALSE,
	"is_visible"		BOOLEAN NOT NULL DEFAULT FALSE,
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
	"points"		INTEGER,
	"correct"		BOOLEAN
);

ALTER TABLE IF EXISTS picks
ADD UNIQUE (book_id, chapter_id, event_id, user_id);

CREATE TABLE IF NOT EXISTS added_points(
    id SERIAL NOT NULL,
    user_id integer NOT NULL,
    book_id integer NOT NULL,
    points integer NOT NULL,
    reason text NOT NULL,
    CONSTRAINT added_points_pkey PRIMARY KEY (id),
    CONSTRAINT added_points_book_id_fkey FOREIGN KEY (book_id)
        REFERENCES books (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION
        NOT VALID,
    CONSTRAINT added_points_user_id_fkey FOREIGN KEY (user_id)
        REFERENCES users (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION
        NOT VALID
);
