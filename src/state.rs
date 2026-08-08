use std::str::FromStr;
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;
use crate::models::CardMoveEvent;

#[derive(Clone)]
pub struct ApplicationState {
    pub db: SqlitePool,
    pub tx: tokio::sync::broadcast::Sender<CardMoveEvent>,
}

pub async fn create_application_state() -> ApplicationState {
    let options = SqliteConnectOptions::from_str("sqlite:./kanban.db")
        .unwrap()
        .foreign_keys(true);

    let db = SqlitePool::connect_with(options).await.unwrap();

    let (tx, _) = tokio::sync::broadcast::channel::<CardMoveEvent>(512);

    ApplicationState {
        db,
        tx
    }
}
