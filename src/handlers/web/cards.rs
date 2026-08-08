use askama::Template;
use axum::extract::{Path, State};
use axum::{Form, Router};
use axum::response::{Html, Redirect};
use axum::routing::{get, post};
use crate::errors::KanbanError;
use crate::handlers::{create_card_common, delete_card_common, move_card_common, patch_card_common, CreateCardRequest};
use crate::models::{Card, CardMoveEvent};
use crate::state::ApplicationState;
use crate::turbo::TurboStream;

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/lists/{list_id}/cards", post(post_card_form))
        .route("/lists/{list_id}/cards/{card_id}/move", post(post_move_card_action))
        .route("/cards/{card_id}/edit", get(get_card_edit).post(patch_card_form))
        .route("/cards/{card_id}/view", get(get_card_by_id))
        .route("/lists/{list_id}/cards/{card_id}/delete", post(delete_card_form))
}

#[derive(Debug, Clone, Template)]
#[template(path = "turbo_move_card.html")]
struct MoveCardTemplate {
    card_id: u64,
    to_list_id: u64,
    card: Card,
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

async fn post_move_card_action(
    State(state): State<ApplicationState>,
    Path((list_id, card_id)): Path<(u64, u64)>,
) -> Result<TurboStream, KanbanError> {
    move_card_common(State(state.clone()), list_id, card_id).await?;

    let card = sqlx::query_as::<_, Card>(
        "SELECT card_id, list_id, title, description, status FROM cards WHERE card_id = ?;",
    )
        .bind(card_id as i64)
        .fetch_optional(&state.db)
        .await?
        .ok_or(KanbanError::CardNotFound(card_id))?;

    let response = MoveCardTemplate {
        card_id,
        to_list_id: list_id,
        card,
    };

    // Discard the potential error response because send will return an error if there are zero
    // active receivers.
    let _ = state.tx.send(response.clone().into());

    Ok(TurboStream(response.render()?))
}

#[derive(Debug, Template)]
#[template(path = "turbo_new_card.html")]
struct NewCardTemplate {
    card: Card,
}

async fn post_card_form(
    State(state): State<ApplicationState>,
    Path(list_id): Path<u64>,
    Form(card): Form<CreateCardRequest>,
) -> Result<TurboStream, KanbanError> {
    let card = create_card_common(State(state), list_id, card).await?;

    let template =  NewCardTemplate { card };

    Ok(TurboStream(template.render()?))
}

async fn delete_card_form(
    State(state): State<ApplicationState>,
    Path((_list_id, card_id)): Path<(u64, u64)>,
) -> Result<TurboStream, KanbanError> {
    delete_card_common(State(state), card_id).await?;

    let response =
        format!("<turbo-stream action=\"remove\" target=\"card-{card_id}\"></turbo-stream>");

    Ok(TurboStream(response))
}

#[derive(Debug, Template)]
#[template(path = "turbo_card_edit.html")]
struct EditCardTemplate {
    card: Card,
}

async fn get_card_edit(
    State(state): State<ApplicationState>,
    Path(card_id): Path<u64>,
) -> Result<Html<String>, KanbanError> {
    let card = sqlx::query_as::<_, Card>(
        "SELECT card_id, list_id, title, description, status FROM cards WHERE card_id = ?",
    )
        .bind(card_id as i64)
        .fetch_optional(&state.db)
        .await?
        .ok_or(KanbanError::CardNotFound(card_id))?;

    let template = EditCardTemplate { card };

    Ok(Html(template.render()?))
}

#[derive(Debug, Template)]
#[template(path = "turbo_card.html")]
struct CardTemplate {
    card: Card,
}

async fn patch_card_form(State(state): State<ApplicationState>, Path(card_id): Path<i64>, Form(card): Form<Card>) -> Result<Redirect, KanbanError> {
    patch_card_common(State(state), card).await?;

    Ok(Redirect::to(format!("/cards/{card_id}/view").as_str()))
}

async fn get_card_by_id(State(state): State<ApplicationState>, Path(card_id): Path<u64>) -> Result<Html<String>, KanbanError> {
    let card = sqlx::query_as::<_, Card>("SELECT card_id, list_id, title, description, status FROM cards WHERE card_id = ?;")
        .bind(card_id as i64)
        .fetch_one(&state.db)
        .await?;

    let template = CardTemplate { card};

    Ok(Html(template.render()?))
}