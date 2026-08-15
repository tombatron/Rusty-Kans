use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::routing::get;
use serde_json::{json, Value};
use crate::data;
use crate::errors::KanbanError;
use crate::state::ApplicationState;

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/api/board/{board_id}", get(get_board))
        
}

async fn get_board(State(state): State<ApplicationState>, Path(board_id): Path<u64>) -> Result<Json<Value>, KanbanError> {
    let result = data::get_board_with_lists(&state.db, board_id).await?;

    Ok(Json(json!({
        "id": board_id,
        "name": result.board.name,
        "lists": result.lists
    })))
}