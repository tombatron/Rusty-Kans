CREATE TABLE IF NOT EXISTS boards (
    board_id INTEGER PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS lists (
    list_id INTEGER PRIMARY KEY,
    board_id INTEGER,
    name TEXT NOT NULL,

    FOREIGN KEY (board_id) REFERENCES boards(board_id) ON DELETE CASCADE ON UPDATE NO ACTION
);

CREATE TABLE IF NOT EXISTS cards (
    card_id INTEGER PRIMARY KEY,
    list_id INTEGER,
    title TEXT NOT NULL,
    description TEXT NULL,
    sort_order INTEGER NULL,

    FOREIGN KEY (list_id) REFERENCES lists(list_id) ON DELETE CASCADE ON UPDATE NO ACTION
);

CREATE TABLE IF NOT EXISTS users (
    user_id INTEGER NOT NULL,
    source TEXT NOT NULL,
    oauth_login TEXT NOT NULL,
    display_name TEXT,
    avatar_url TEXT,

    PRIMARY KEY (user_id, source)
);

CREATE TABLE IF NOT EXISTS board_access (
    board_id INTEGER NOT NULL,
    granted_to_user_id INTEGER NOT NULL,
    granted_to_user_source TEXT NOT NULL,
    permission TEXT NOT NULL,
    granted_at TEXT NOT NULL,

    PRIMARY KEY (board_id, granted_to_user_id, granted_to_user_source),

    FOREIGN KEY (board_id) REFERENCES boards(board_id) ON DELETE CASCADE ON UPDATE NO ACTION,
    FOREIGN KEY (granted_to_user_id, granted_to_user_source) REFERENCES users(user_id, source) ON DELETE CASCADE ON UPDATE NO ACTION,

    CHECK (permission IN ('view', 'edit')),
    CHECK (granted_to_user_source IN ('github', 'dev'))
);

CREATE TABLE IF NOT EXISTS board_access_grants (
    owner_database_id TEXT NOT NULL,
    remote_board_id INTEGER NOT NULL,
    remote_board_name TEXT NOT NULL,
    created_at TEXT NOT NULL,

    PRIMARY KEY (owner_database_id, remote_board_id)
);

CREATE TABLE IF NOT EXISTS share_links (
    share_id INTEGER NOT NULL,
    board_id INTEGER NOT NULL,
    permission TEXT NOT NULL,
    share_code TEXT NOT NULL,
    expires_at TEXT NOT NULL,

    PRIMARY KEY (share_id),
    FOREIGN KEY (board_id) REFERENCES boards(board_id) ON DELETE CASCADE ON UPDATE NO ACTION,

    CHECK (permission IN ('view', 'edit'))
);

CREATE INDEX idx_share_code ON share_links (share_code);