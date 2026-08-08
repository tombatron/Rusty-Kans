use sqlx::SqlitePool;
use crate::models::CardMoveEvent;

#[derive(Clone)]
pub struct ApplicationState {
    pub db: SqlitePool,
    pub tx: tokio::sync::broadcast::Sender<CardMoveEvent>,
}

pub async fn create_application_state() -> ApplicationState {
    let db = SqlitePool::connect("sqlite:./kanban.db").await.unwrap();

    let (tx, _) = tokio::sync::broadcast::channel::<CardMoveEvent>(512);

    ApplicationState {
        db,
        tx
    }
}
