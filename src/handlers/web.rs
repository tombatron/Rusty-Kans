use crate::errors::KanbanError;
use crate::handlers::{create_card_common, create_list_common, delete_card_common, move_card_common, CreateCardRequest, CreateListRequest};
use crate::models::{Card, CardMoveEvent, List};
use crate::state::ApplicationState;
use crate::turbo::TurboStream;
use askama::Template;
use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse};
use axum::routing::{delete, get, post};
use axum::{Form, Router};
use sqlx::{AssertSqlSafe, Row};

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/", get(get_index))

        .route("/lists", post(post_list_form))
        .route("/lists/{list_id}/cards", post(post_card_form))
        .route("/lists/{list_id}/cards/{card_id}/move", post(post_move_card_action))
        .route("/lists/{list_id}/cards/{card_id}", delete(delete_card_form))
}

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

#[derive(Debug, Template)]
#[template(path = "turbo_card.html")]
struct CreateCardTemplate {
    card: Card
}

pub async fn post_card_form(State(state): State<ApplicationState>, Path(list_id): Path<u64>, Form(card): Form<CreateCardRequest>) -> Result<TurboStream, KanbanError> {
    let card = create_card_common(State(state), list_id, card).await?;

    let template = CreateCardTemplate { card };

    Ok(TurboStream(template.render()?))
}

pub async fn delete_card_form(State(state): State<ApplicationState>, Path((list_id, card_id)): Path<(u64, u64)>) -> Result<TurboStream, KanbanError> {
    delete_card_common(State(state), card_id).await?;
    
    let response = format!("<turbo-stream action=\"remove\" target=\"card-{card_id}\"></turbo-stream>");
    
    Ok(TurboStream(response))
}