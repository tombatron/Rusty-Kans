mod errors;
mod models;

use crate::models::Board;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};
use std::fs;
use std::sync::{Arc, Mutex};

async fn get_health() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

async fn get_board(State(board): State<Arc<Mutex<Board>>>) -> Json<Value> {
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

async fn get_list_by_id(
    State(board): State<Arc<Mutex<Board>>>,
    Path(id): Path<u64>,
) -> Result<Json<Value>, StatusCode> {
    let board = board.lock().unwrap();

    board
        .lists
        .get(&id)
        .map(|list| Json(json!(list)))
        .ok_or(StatusCode::NOT_FOUND)
}

fn load_existing_board_configuration() -> Option<Board> {
    let saved_board_configuration = fs::read_to_string("board.json").ok()?;
    serde_json::from_str(&saved_board_configuration).ok()
}

#[tokio::main]
async fn main() {
    let board = load_existing_board_configuration()
        .unwrap_or_else(|| Board::new(String::from("Web board")));

    let shared_board = Arc::new(Mutex::new(board));

    // Router with initial health endpoint.
    let app = Router::new()
        .route("/health", get(get_health))
        .route("/board", get(get_board))
        .route("/lists/{id}", get(get_list_by_id))
        .with_state(shared_board);

    // Setup the listener on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
