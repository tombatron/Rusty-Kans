use crate::data;
use crate::errors::KanbanError;
use crate::state::{ApplicationState, UserDb};
use axum::extract::Path;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/api/board/{board_id}", get(get_board))
}

async fn get_board(UserDb(db): UserDb, Path(board_id): Path<u64>) -> Result<Json<Value>, KanbanError> {
    let result = data::get_board_with_lists(db, board_id).await?;

    Ok(Json(json!({
        "id": board_id,
        "name": result.board.name,
        "lists": result.lists
    })))
}

#[cfg(test)]
mod tests {
    use crate::handlers::api::boards::get_board;
    use crate::state::UserDb;
    use axum::extract::Path;
    use sqlx::SqlitePool;

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn get_board_returns_a_board(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let response = get_board(db, Path(1)).await.unwrap();

        assert_eq!(1, response["id"]);
        assert_eq!("Whatever", response["name"]);
        assert_eq!(2, response["lists"].as_array().unwrap().len());
        assert_eq!("Card 1", response["lists"].as_array().unwrap()[0]["cards"][0]["title"]);

        Ok(())
    }
}