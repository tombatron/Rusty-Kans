use crate::data;
use crate::errors::KanbanError;
use crate::state::ApplicationState;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/api/board/{board_id}", get(get_board))
}

async fn get_board(State(state): State<ApplicationState>, Path(board_id): Path<u64>) -> Result<Json<Value>, KanbanError> {
    let result = data::get_board_with_lists(state.db, board_id).await?;

    Ok(Json(json!({
        "id": board_id,
        "name": result.board.name,
        "lists": result.lists
    })))
}

#[cfg(test)]
mod tests {
    use crate::handlers::api::boards::get_board;
    use crate::handlers::tests::get_fake_application_state;
    use axum::extract::{Path, State};
    use sqlx::SqlitePool;

    #[sqlx::test(fixtures(path = "../../fixtures", scripts("boards")))]
    async fn get_board_returns_a_board(db: SqlitePool) -> sqlx::Result<()> {
        let state = State(get_fake_application_state(db));

        let response = get_board(state, Path(1)).await.unwrap();

        assert_eq!(1, response["id"]);
        assert_eq!("Whatever", response["name"]);
        assert_eq!(2, response["lists"].as_array().unwrap().len());
        assert_eq!("Card 1", response["lists"].as_array().unwrap()[0]["cards"][0]["title"]);

        Ok(())
    }
}