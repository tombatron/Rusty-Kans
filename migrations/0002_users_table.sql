CREATE TABLE IF NOT EXISTS users (
    user_id INTEGER NOT NULL,
    source TEXT NOT NULL,
    oauth_login TEXT NOT NULL,
    display_name TEXT,
    avatar_url TEXT,

    PRIMARY KEY (user_id, source)
);