use crate::errors::KanbanError;
use crate::state::ApplicationState;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use serde_json::{Value, json};

const BOARD_FILENAME: &str = "board.json";

pub async fn get_health() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

pub async fn get_board(State(board): State<ApplicationState>) -> Json<Value> {
    let board = board.lock().unwrap();

    let lists: Vec<Value> = board
        .lists
        .values()
        .map(|l| {
            json!({
                "id": l.id,
                "name": l.name
            })
        })
        .collect();

    Json(json!({
        "name": board.name,
        "lists": lists
    }))
}

pub async fn get_list_by_id(
    State(board): State<ApplicationState>,
    Path(id): Path<u64>,
) -> Result<Json<Value>, KanbanError> {
    let board = board.lock().unwrap();

    board
        .lists
        .get(&id)
        .map(|list| Json(json!(list)))
        .ok_or(KanbanError::ListNotFound(id))
}

#[derive(Deserialize)]
pub struct CreateListRequest {
    name: String,
}

pub async fn post_list(
    State(board): State<ApplicationState>,
    Json(list): Json<CreateListRequest>,
) -> Result<Json<Value>, KanbanError> {
    let (mutated_board, list_id) = {
        let mut board = board.lock().unwrap();

        let list_id = board.add_list(&list.name);

        (serde_json::to_string(&*board)?, list_id)
    };

    tokio::fs::write(BOARD_FILENAME, mutated_board).await?;

    Ok(Json(json!({
        "id": list_id,
        "name": list.name
    })))
}

#[derive(Deserialize)]
pub struct CreateCardRequest {
    title: String,
    description: Option<String>,
}

pub async fn post_card(
    State(board): State<ApplicationState>,
    Path(list_id): Path<u64>,
    Json(card): Json<CreateCardRequest>,
) -> Result<Json<Value>, KanbanError> {
    let (mutated_board, result) = {
        let mut board = board.lock().unwrap();

        let result = board
            .add_card(list_id, &card.title, card.description.clone())
            .map(|card_id| {
                Json(json!({
                    "id": card_id,
                    "title": card.title,
                    "description": card.description
                }))
            })?;

        (serde_json::to_string(&*board)?, result)
    };

    tokio::fs::write(BOARD_FILENAME, mutated_board).await?;

    Ok(result)
}

pub async fn post_move_card(
    State(board): State<ApplicationState>,
    Path((list_id, card_id)): Path<(u64, u64)>,
) -> Result<StatusCode, KanbanError> {
    let mutated_board = {
        let mut board = board.lock().unwrap();

        board.move_card(card_id, list_id)?;

        serde_json::to_string(&*board)?
    };

    tokio::fs::write(BOARD_FILENAME, mutated_board).await?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct SearchQuery {
    keyword: String,
}

pub async fn get_card_search(
    State(board): State<ApplicationState>,
    Query(search_query): Query<SearchQuery>,
) -> Json<Value> {
    let board = board.lock().unwrap();

    Json(json!(board.search(&search_query.keyword)))
}
