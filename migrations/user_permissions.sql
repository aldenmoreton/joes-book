CREATE TABLE IF NOT EXISTS user_permissions (
	user_id     INT NOT NULL REFERENCES users(id),
	token       TEXT NOT NULL
);