use askama::Template;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use crate::data;
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
    list_id: u64,
    name: String,
}

#[derive(Deserialize)]
pub struct CreateListRequest {
    name: String,
}

async fn create_list_common(
    State(state): State<ApplicationState>,
    board_id: u64,
    list_info: CreateListRequest,
) -> Result<ListItemTemplate, KanbanError> {
    let list_id = data::insert_list(&state.db, board_id, &list_info.name).await?;

    Ok(ListItemTemplate {
        list_id,
        name: list_info.name,
    })
}

async fn move_card_common(State(state): State<ApplicationState>, list_id: u64, card_id: u64) -> Result<(), KanbanError> {
    data::update_card_list(&state.db, list_id, card_id).await?;

    Ok(())
}

#[derive(Deserialize)]
pub struct CreateCardRequest {
    title: String,
    description: Option<String>,
}

async fn create_card_common(State(state): State<ApplicationState>, list_id: u64, card: CreateCardRequest) -> Result<Card, KanbanError> {
    let card_id = data::insert_card(&state.db, list_id, &card.title, &card.description).await?;

    Ok(Card {
        id: card_id,
        list_id,
        title: card.title,
        description: card.description,
        status: Status::Todo
    })
}

async fn delete_card_common(State(state): State<ApplicationState>, card_id: u64) -> Result<(), KanbanError> {
    data::delete_card(&state.db, card_id).await?;

    Ok(())
}

async fn patch_card_common(State(state): State<ApplicationState>, card: Card) -> Result<(), KanbanError> {
    let result = data::update_card(&state.db, card.id, card.title, card.description, card.status).await?;

    if result != 1 {
        return Err(KanbanError::DatabaseError("Update failed. Refresh and try again.".to_string()));
    }

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use sqlx::SqlitePool;
    use crate::models::CardMoveEvent;
    use crate::state::ApplicationState;

    pub fn get_fake_application_state(db: SqlitePool) -> ApplicationState {
        let (tx, _) = tokio::sync::broadcast::channel::<CardMoveEvent>(512);

        ApplicationState { db, tx }
    }
}