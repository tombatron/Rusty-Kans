use sqlx::SqlitePool;

pub type ApplicationState = SqlitePool;

pub async fn create_application_state() -> ApplicationState {
    let pool = SqlitePool::connect("sqlite:./kanban.db").await.unwrap();

    sqlx::query("INSERT OR IGNORE INTO boards (board_id, name) VALUES (1, 'My Board')")
        .execute(&pool)
        .await
        .unwrap();

    pool
}
