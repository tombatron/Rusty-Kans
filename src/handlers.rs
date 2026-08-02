use askama::Template;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use crate::errors::KanbanError;
use crate::models::{Card, Status};
use crate::state::ApplicationState;

pub mod api;
pub mod web;
pub mod utility;
pub mod ws;

#[derive(Debug, Serialize, Template)]
#[template(path = "turbo_list_item.html")]
struct ListItemTemplate {
    id: u64,
    name: String,
}

#[derive(Deserialize)]
pub struct CreateListRequest {
    name: String,
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

#[derive(Deserialize)]
pub struct CreateCardRequest {
    title: String,
    description: Option<String>,
}

async fn create_card_common(State(state): State<ApplicationState>, list_id: u64, card: CreateCardRequest) -> Result<Card, KanbanError> {
    let result = sqlx::query("INSERT INTO cards (list_id, title, description) VALUES (?, ?, ?);")
        .bind(list_id as i64)
        .bind(&card.title)
        .bind(&card.description)
        .execute(&state.db)
        .await?;

    let card_id = result.last_insert_rowid();

    Ok(Card {
        id: card_id as u64,
        list_id,
        title: card.title,
        description: card.description,
        status: Status::Todo
    })
}

async fn delete_card_common(State(state): State<ApplicationState>, card_id: u64) -> Result<(), KanbanError> {
    sqlx::query("DELETE FROM cards WHERE card_id = ?")
        .bind(card_id as i64)
        .execute(&state.db)
        .await?;

    Ok(())
}

async fn patch_card_common(State(state): State<ApplicationState>, card: Card) -> Result<(), KanbanError> {
    let update_result = sqlx::query("UPDATE cards SET title = ?, description = ?, status = ? WHERE card_id = ?;")
        .bind(card.title)
        .bind(card.description)
        .bind(card.status)
        .bind(card.id as i64)
        .execute(&state.db)
        .await?;

    if update_result.rows_affected() != 1 {
        return Err(KanbanError::DatabaseError("Update failed. Refresh and try again.".to_string()));
    }

    Ok(())
}