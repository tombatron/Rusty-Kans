use crate::errors::KanbanError;
use crate::models::{Card, CardMoveEvent, List};
use crate::state::ApplicationState;
use crate::turbo::TurboStream;
use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::{Form, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{AssertSqlSafe, Row};

#[derive(Debug, Template)]
#[template(path = "board.html")]
struct IndexTemplate {
    id: u64,
    name: String,
    lists: Vec<List>,
}

pub async fn get_index(
    State(state): State<ApplicationState>,
) -> Result<impl IntoResponse, KanbanError> {
    let board = sqlx::query("SELECT board_id, name FROM boards LIMIT 1")
        .fetch_one(&state.db)
        .await?;

    let mut lists: Vec<List> =
        sqlx::query_as::<_, List>("SELECT list_id, name FROM lists WHERE board_id = ?")
            .bind(board.get::<i64, _>("board_id"))
            .fetch_all(&state.db)
            .await?;

    let list_ids_placeholders = lists.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let query_text = format!(
        "SELECT card_id, list_id, title, description, status FROM cards WHERE list_id in ({})",
        list_ids_placeholders
    );
    let mut query = sqlx::query_as::<_, Card>(AssertSqlSafe(query_text));

    for id in &lists {
        query = query.bind(id.id as i64);
    }

    let cards: Vec<Card> = query.fetch_all(&state.db).await?;

    for l in lists.iter_mut() {
        l.cards = cards
            .iter()
            .filter(|c| c.list_id == l.id)
            .cloned()
            .collect()
    }

    let response_template = IndexTemplate {
        id: board.get::<u64, _>("board_id"),
        name: board.get::<String, _>("name"),
        lists,
    };

    Ok(Html(response_template.render()?))
}

#[derive(Debug, Serialize, Template)]
#[template(path = "turbo_list_item.html")]
struct ListItemTemplate {
    id: u64,
    name: String,
}

pub async fn post_list_form(
    State(state): State<ApplicationState>,
    Form(list_info): Form<CreateListRequest>,
) -> Result<TurboStream, KanbanError> {
    let created_list = create_list_common(State(state), list_info).await?;

    Ok(TurboStream(created_list.render()?))
}

#[derive(Debug, Clone, Template)]
#[template(path = "turbo_move_card.html")]
struct MoveCardTemplate {
    card_id: u64,
    to_list_id: u64,
    card: Card
}

impl Into<CardMoveEvent> for MoveCardTemplate {
    fn into(self) -> CardMoveEvent {
        CardMoveEvent {
            card_id: self.card_id,
            to_list_id: self.to_list_id,
            card: self.card,
        }
    }
}

pub async fn post_move_card_action(State(state): State<ApplicationState>, Path((list_id, card_id)): Path<(u64, u64)>) -> Result<TurboStream, KanbanError> {
    move_card_common(State(state.clone()), list_id, card_id).await?;

    let card = sqlx::query_as::<_, Card>("SELECT card_id, list_id, title, description, status FROM cards WHERE card_id = ?;")
        .bind(card_id as i64)
        .fetch_optional(&state.db)
        .await?
        .ok_or(KanbanError::CardNotFound(card_id))?;

    let response = MoveCardTemplate {
        card_id,
        to_list_id: list_id,
        card
    };

    // Discard the potential error response because send will return an error if there are zero
    // active receivers.
    let _ = state.tx.send(response.clone().into());

    Ok(TurboStream(response.render()?))
}

pub async fn get_health() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

pub async fn get_board(State(state): State<ApplicationState>) -> Result<Json<Value>, KanbanError> {
    let board = sqlx::query("SELECT board_id, name FROM boards WHERE board_id = 1;")
        .fetch_one(&state.db)
        .await?;

    let lists = sqlx::query("SELECT list_id, name FROM lists WHERE board_id = ?")
        .bind(board.get::<i64, _>("board_id"))
        .fetch_all(&state.db)
        .await?;

    let lists_json: Vec<Value> = lists
        .iter()
        .map(|l| {
            json!({
                "id": l.get::<i64, _>("list_id"),
                "name": l.get::<String, _>("name")
            })
        })
        .collect();

    Ok(Json(json!({
        "id": board.get::<i64, _>("board_id"),
        "name": board.get::<String, _>("name"),
        "lists": lists_json
    })))
}

pub async fn get_list_by_id(
    State(state): State<ApplicationState>,
    Path(id): Path<u64>,
) -> Result<Json<Value>, KanbanError> {
    let list = sqlx::query("SELECT list_id, board_id, name FROM lists WHERE list_id = ?")
        .bind(id as i64)
        .fetch_optional(&state.db)
        .await?
        .ok_or(KanbanError::ListNotFound(id))?;

    let cards = sqlx::query(
        "SELECT card_id, list_id, title, description, status FROM cards WHERE list_id = ?",
    )
    .bind(id as i64)
    .fetch_all(&state.db)
    .await?;

    let cards_json: Vec<Value> = cards
        .iter()
        .map(|c| {
            json!({
                "card_id": c.get::<i64, _>("card_id"),
                "title": c.get::<String, _>("title"),
                "description": c.get::<Option<String>, _>("description"),
                "status": c.get::<String, _>("status")
            })
        })
        .collect();

    Ok(Json(json!({
        "list_id": list.get::<i64, _>("list_id"),
        "board_id": list.get::<i64, _>("board_id"),
        "name": list.get::<String, _>("name"),
        "cards": cards_json
    })))
}

#[derive(Deserialize)]
pub struct CreateListRequest {
    name: String,
}

pub async fn post_list(
    State(state): State<ApplicationState>,
    Json(list): Json<CreateListRequest>,
) -> Result<Json<Value>, KanbanError> {
    let created_list = create_list_common(State(state), list).await?;

    Ok(Json(json!(created_list)))
}

#[derive(Deserialize)]
pub struct CreateCardRequest {
    title: String,
    description: Option<String>,
}

pub async fn post_card(
    State(state): State<ApplicationState>,
    Path(list_id): Path<u64>,
    Json(card): Json<CreateCardRequest>,
) -> Result<Json<Value>, KanbanError> {
    let result = sqlx::query("INSERT INTO cards (list_id, title, description) VALUES (?, ?, ?);")
        .bind(list_id as i64)
        .bind(&card.title)
        .bind(&card.description)
        .execute(&state.db)
        .await?;

    let card_id = result.last_insert_rowid();

    Ok(Json(json!({
        "id": card_id,
        "title": card.title,
        "description": card.description,
    })))
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
    let card_result = sqlx::query(
        "SELECT card_id, list_id, title, description, status FROM cards WHERE card_id = ?;",
    )
    .bind(card_id as i64)
    .fetch_optional(&state.db)
    .await?
    .ok_or(KanbanError::CardNotFound(card_id))?;

    Ok(Json(json!({
        "card_id": card_result.get::<i64, _>("card_id"),
        "list_id": card_result.get::<i64, _>("list_id"),
        "title": card_result.get::<String, _>("title"),
        "description": card_result.get::<Option<String>, _>("description"),
        "status": card_result.get::<String, _>("status")
    })))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    keyword: String,
}

pub async fn get_card_search(
    State(state): State<ApplicationState>,
    Query(search_query): Query<SearchQuery>,
) -> Result<Json<Value>, KanbanError> {
    let search_result = sqlx::query(
        "SELECT card_id, list_id, title, description, status FROM cards WHERE title LIKE ?;",
    )
    .bind(format!("%{}%", &search_query.keyword))
    .fetch_all(&state.db)
    .await?;

    let cards = search_result
        .iter()
        .map(|c| {
            json!({
                "card_id": c.get::<i64, _>("card_id"),
                "list_id": c.get::<i64, _>("list_id"),
                "title": c.get::<String, _>("title"),
                "description": c.get::<Option<String>, _>("description"),
                "status": c.get::<String, _>("status")
            })
        })
        .collect();

    Ok(Json(cards))
}

async fn create_list_common(
    State(state): State<ApplicationState>,
    list_info: CreateListRequest,
) -> Result<ListItemTemplate, KanbanError> {
    let insert_list_result =
        sqlx::query("INSERT INTO lists (board_id, name) SELECT board_id, ? FROM boards LIMIT 1;")
            .bind(&list_info.name)
            .execute(&state.db)
            .await?;

    let list_id = insert_list_result.last_insert_rowid();

    Ok(ListItemTemplate {
        id: list_id as u64,
        name: list_info.name,
    })
}

async fn move_card_common(State(state): State<ApplicationState>, list_id: u64, card_id: u64) -> Result<(), KanbanError> {
    sqlx::query("UPDATE cards SET list_id = ? WHERE card_id = ?;")
        .bind(list_id as i64)
        .bind(card_id as i64)
        .execute(&state.db)
        .await?;

    Ok(())
}