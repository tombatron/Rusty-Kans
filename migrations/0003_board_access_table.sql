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
)