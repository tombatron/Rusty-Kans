use crate::data;
use crate::errors::KanbanError;
use crate::handlers::{CreateCardRequest, CreateListRequest, create_card_common, create_list_common, delete_card_common, move_card_common, patch_card_common};
use crate::middleware::require_auth;
use crate::models::Card;
use crate::state::ApplicationState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/api/board/{board_id}", get(get_board))
        .route("/api/lists/{id}", get(get_list_by_id))
        .route("/api/board/{board_id}/lists", post(post_list))
        .route("/api/lists/{id}/cards", post(post_card))
        .route("/api/lists/{list_id}/cards/{card_id}/move", post(post_move_card))
        .route("/api/lists/{list_id}/cards/{card_id}", delete(delete_card))
        .route("/api/cards/search", get(get_card_search))
        .route("/api/cards/{card_id}", get(get_card_by_id))
        .route("/api/cards/{card_id}", patch(patch_card))

        .layer(axum::middleware::from_fn(require_auth))
}

pub async fn get_board(State(state): State<ApplicationState>, Path(board_id): Path<u64>) -> Result<Json<Value>, KanbanError> {
    let result = data::get_board_with_lists(&state.db, board_id).await?;

    Ok(Json(json!({
        "id": board_id,
        "name": result.board.name,
        "lists": result.lists
    })))
}

pub async fn get_list_by_id(
    State(state): State<ApplicationState>,
    Path(id): Path<u64>,
) -> Result<Json<Value>, KanbanError> {
    let result = data::get_list_with_cards(&state.db, id).await?;

    let cards_json: Vec<Value> = result.cards
        .iter()
        .map(|c| json!(c))
        .collect();

    Ok(Json(json!({
        "list_id": id,
        "board_id": result.list.board_id,
        "name": result.list.name,
        "cards": cards_json
    })))
}

pub async fn post_list(
    State(state): State<ApplicationState>,
    Path(board_id): Path<u64>,
    Json(list): Json<CreateListRequest>,
) -> Result<Json<Value>, KanbanError> {
    let created_list = create_list_common(State(state), board_id, list).await?;

    Ok(Json(json!(created_list)))
}

pub async fn post_card(
    State(state): State<ApplicationState>,
    Path(list_id): Path<u64>,
    Json(card): Json<CreateCardRequest>,
) -> Result<Json<Value>, KanbanError> {
    let card = create_card_common(State(state), list_id, card).await?;

    Ok(Json(json!(card)))
}

pub async fn post_move_card(
    State(state): State<ApplicationState>,
    Path((list_id, card_id)): Path<(u64, u64)>,
) -> Result<StatusCode, KanbanError> {
    move_card_common(State(state), list_id, card_id).await?;

    Ok(StatusCode::OK)
}

pub async fn get_card_by_id(
    State(state): State<ApplicationState>,
    Path(card_id): Path<u64>,
) -> Result<Json<Value>, KanbanError> {
    let result = data::get_card(&state.db, card_id).await?;

    Ok(Json(json!(result)))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    keyword: String,
}

pub async fn get_card_search(
    State(state): State<ApplicationState>,
    Query(search_query): Query<SearchQuery>,
) -> Result<Json<Value>, KanbanError> {
    let search_result = data::get_cards_by_title_submatch(&state.db, search_query.keyword).await?;

    let cards = search_result
        .iter()
        .map(|c| json!(c))
        .collect();

    Ok(Json(cards))
}

pub async fn delete_card(State(state): State<ApplicationState>, Path((_list_id, card_id)): Path<(u64, u64)>) -> Result<StatusCode, KanbanError> {
    delete_card_common(State(state), card_id).await?;

    Ok(StatusCode::OK)
}

pub async fn patch_card(State(state): State<ApplicationState>, Path(card_id): Path<u64>, Json(card): Json<Card>) -> Result<Redirect, KanbanError> {
    patch_card_common(State(state), card).await?;

    Ok(Redirect::to(format!("/api/cards/{card_id}").as_str()))
}