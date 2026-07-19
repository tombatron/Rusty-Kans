use crate::errors::KanbanError;
use crate::models::{Card, List};
use crate::state::ApplicationState;
use askama::Template;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::{AssertSqlSafe, Row};

#[derive(Debug, Template)]
#[template(path = "board.html")]
struct IndexTemplate {
    id: u64,
    name: String,
    lists: Vec<List>
}

pub async fn get_index(
    State(db): State<ApplicationState>,
) -> Result<impl IntoResponse, KanbanError> {
    let board = sqlx::query("SELECT board_id, name FROM boards LIMIT 1")
        .fetch_one(&db)
        .await?;

    let mut lists: Vec<List> =
        sqlx::query_as::<_, List>("SELECT list_id, name FROM lists WHERE board_id = ?")
            .bind(board.get::<i64, _>("board_id"))
            .fetch_all(&db)
            .await?;

    let list_ids_placeholders = lists.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let query_text = format!("SELECT card_id, list_id, title, description, status FROM cards WHERE list_id in ({})", list_ids_placeholders);
    let mut query = sqlx::query_as::<_, Card>(AssertSqlSafe(query_text));

    for id in &lists {
        query = query.bind(id.id as i64);
    }

    let cards: Vec<Card> = query.fetch_all(&db).await?;

    for l in lists.iter_mut() {
        l.cards = cards.iter().filter(|c|c.list_id == l.id).cloned().collect()
    }

    let response_template = IndexTemplate {
        id: board.get::<u64, _>("board_id"),
        name: board.get::<String, _>("name"),
        lists
    };

    Ok(Html(response_template.render()?))
}

pub async fn get_health() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

pub async fn get_board(State(db): State<ApplicationState>) -> Result<Json<Value>, KanbanError> {
    let board = sqlx::query("SELECT board_id, name FROM boards WHERE board_id = 1;")
        .fetch_one(&db)
        .await?;

    let lists = sqlx::query("SELECT list_id, name FROM lists WHERE board_id = ?")
        .bind(board.get::<i64, _>("board_id"))
        .fetch_all(&db)
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
    State(db): State<ApplicationState>,
    Path(id): Path<u64>,
) -> Result<Json<Value>, KanbanError> {
    let list = sqlx::query("SELECT list_id, board_id, name FROM lists WHERE list_id = ?")
        .bind(id as i64)
        .fetch_optional(&db)
        .await?
        .ok_or(KanbanError::ListNotFound(id))?;

    let cards = sqlx::query(
        "SELECT card_id, list_id, title, description, status FROM cards WHERE list_id = ?",
    )
    .bind(id as i64)
    .fetch_all(&db)
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
    State(db): State<ApplicationState>,
    Json(list): Json<CreateListRequest>,
) -> Result<Json<Value>, KanbanError> {
    let insert_list_result =
        sqlx::query("INSERT INTO lists (board_id, name) SELECT board_id, ? FROM boards LIMIT 1;")
            .bind(&list.name)
            .execute(&db)
            .await?;

    let list_id = insert_list_result.last_insert_rowid();

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
    State(db): State<ApplicationState>,
    Path(list_id): Path<u64>,
    Json(card): Json<CreateCardRequest>,
) -> Result<Json<Value>, KanbanError> {
    let result = sqlx::query("INSERT INTO cards (list_id, title, description) VALUES (?, ?, ?);")
        .bind(list_id as i64)
        .bind(&card.title)
        .bind(&card.description)
        .execute(&db)
        .await?;

    let card_id = result.last_insert_rowid();

    Ok(Json(json!({
        "id": card_id,
        "title": card.title,
        "description": card.description,
    })))
}

pub async fn post_move_card(
    State(db): State<ApplicationState>,
    Path((list_id, card_id)): Path<(u64, u64)>,
) -> Result<StatusCode, KanbanError> {
    sqlx::query("UPDATE cards SET list_id = ? WHERE card_id = ?;")
        .bind(list_id as i64)
        .bind(card_id as i64)
        .execute(&db)
        .await?;

    Ok(StatusCode::OK)
}

pub async fn get_card_by_id(
    State(db): State<ApplicationState>,
    Path(card_id): Path<u64>,
) -> Result<Json<Value>, KanbanError> {
    let card_result = sqlx::query(
        "SELECT card_id, list_id, title, description, status FROM cards WHERE card_id = ?;",
    )
    .bind(card_id as i64)
    .fetch_optional(&db)
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
    State(db): State<ApplicationState>,
    Query(search_query): Query<SearchQuery>,
) -> Result<Json<Value>, KanbanError> {
    let search_result = sqlx::query(
        "SELECT card_id, list_id, title, description, status FROM cards WHERE title LIKE ?;",
    )
    .bind(format!("%{}%", &search_query.keyword))
    .fetch_all(&db)
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
