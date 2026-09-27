CREATE TABLE IF NOT EXISTS board_grants (
    owner_database_id TEXT NOT NULL,
    remote_board_id INTEGER NOT NULL,
    remote_board_name TEXT NOT NULL,
    created_at TEXT NOT NULL,

    PRIMARY KEY (owner_database_id, remote_board_id)
);