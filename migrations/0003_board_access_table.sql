CREATE TABLE IF NOT EXISTS board_access (
    board_id INTEGER NOT NULL,
    granted_to_user_id INTEGER NOT NULL,
    granted_to_user_source TEXT NOT NULL,
    permission TEXT NOT NULL,
    granted_at TEXT NOT NULL,

    PRIMARY KEY (granted_to_user_id, granted_to_user_source),

    CHECK (permission IN ('view', 'edit')),
    CHECK (granted_to_user_source IN ('github', 'dev'))
)