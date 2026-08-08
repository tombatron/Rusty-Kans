use crate::errors::KanbanError;
use crate::handlers::{create_card_common, create_list_common, delete_card_common, move_card_common, patch_card_common, CreateCardRequest, CreateListRequest};
use crate::models::{Board, Card, CardMoveEvent, List};
use crate::state::ApplicationState;
use crate::turbo::TurboStream;
use askama::Template;
use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::{Form, Router};
use serde::Deserialize;
use sqlx::AssertSqlSafe;

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/", get(get_landing))
        .route("/boards", post(post_board_form))
        .route("/boards/{board_id}", get(get_board))

        .route("/boards/{board_id}/lists", post(post_list_form))
        .route("/lists/{list_id}/cards", post(post_card_form))
        .route(
            "/lists/{list_id}/cards/{card_id}/move",
            post(post_move_card_action),
        )
        .route("/lists/{list_id}/cards/{card_id}/delete", post(delete_card_form))

        .route("/lists/{list_id}/edit", get(get_list_edit))
        .route("/lists/{list_id}/rename", post(post_list_rename))
        .route("/lists/{list_id}/header", get(get_list_header))
        .route("/lists/{list_id}/delete", post(post_list_delete))

        .route("/cards/{card_id}/edit", get(get_card_edit).post(patch_card_form))
        .route("/cards/{card_id}/view", get(get_card_by_id))
}

#[derive(Debug, Template)]
#[template(path = "landing.html")]
struct LandingTemplate {
    boards: Vec<Board>
}

async fn get_landing(State(state): State<ApplicationState>) -> Result<impl IntoResponse, KanbanError> {
    let boards = sqlx::query_as::<_, Board>("SELECT board_id, name FROM boards;")
        .fetch_all(&state.db)
        .await?;

    let template = LandingTemplate { boards };
    
    Ok(Html(template.render()?))
}

#[derive(Debug, Deserialize)]
struct CreateBoardRequest {
    name: String,
}

#[derive(Debug, Template)]
#[template(path = "turbo_new_board.html")]
struct NewBoardTemplate {
    id: u64,
    name: String
}

async fn post_board_form(State(state): State<ApplicationState>, Form(board): Form<CreateBoardRequest>) -> Result<impl IntoResponse, KanbanError> {
    let result = sqlx::query("INSERT INTO boards (name) VALUES (?);")
        .bind(&board.name)
        .execute(&state.db)
        .await?;

    let board_id = result.last_insert_rowid();

    let template = NewBoardTemplate {
        id: board_id as u64,
        name: board.name,
    };

    Ok(TurboStream(template.render()?))
}



#[derive(Debug, Template)]
#[template(path = "board.html")]
struct BoardTemplate {
    id: u64,
    name: String,
    lists: Vec<List>,
}

pub async fn get_board(
    State(state): State<ApplicationState>,
    Path(board_id): Path<u64>
) -> Result<impl IntoResponse, KanbanError> {
    let board = sqlx::query_as::<_, Board>("SELECT board_id, name FROM boards WHERE board_id = ?;")
        .bind(board_id as i64)
        .fetch_one(&state.db)
        .await?;

    let mut lists: Vec<List> =
        sqlx::query_as::<_, List>("SELECT list_id, board_id, name FROM lists WHERE board_id = ?")
            .bind(board.id as i64)
            .fetch_all(&state.db)
            .await?;

    if !lists.is_empty() {
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
    }

    let response_template = BoardTemplate {
        id: board.id,
        name: board.name,
        lists,
    };

    Ok(Html(response_template.render()?))
}

pub async fn post_list_form(
    State(state): State<ApplicationState>,
    Path(board_id): Path<u64>,
    Form(list_info): Form<CreateListRequest>,
) -> Result<TurboStream, KanbanError> {
    let created_list = create_list_common(State(state), board_id, list_info).await?;

    Ok(TurboStream(created_list.render()?))
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

pub async fn post_move_card_action(
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

pub async fn post_card_form(
    State(state): State<ApplicationState>,
    Path(list_id): Path<u64>,
    Form(card): Form<CreateCardRequest>,
) -> Result<TurboStream, KanbanError> {
    let card = create_card_common(State(state), list_id, card).await?;

    let template =  NewCardTemplate { card };

    Ok(TurboStream(template.render()?))
}

pub async fn delete_card_form(
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

pub async fn get_card_edit(
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

pub async fn patch_card_form(State(state): State<ApplicationState>, Path(card_id): Path<i64>, Form(card): Form<Card>) -> Result<Redirect, KanbanError> {
    patch_card_common(State(state), card).await?;

    Ok(Redirect::to(format!("/cards/{card_id}/view").as_str()))
}

pub async fn get_card_by_id(State(state): State<ApplicationState>, Path(card_id): Path<u64>) -> Result<Html<String>, KanbanError> {
    let card = sqlx::query_as::<_, Card>("SELECT card_id, list_id, title, description, status FROM cards WHERE card_id = ?;")
        .bind(card_id as i64)
        .fetch_one(&state.db)
        .await?;

    let template = CardTemplate { card};

    Ok(Html(template.render()?))
}

#[derive(Debug, Template)]
#[template(path = "turbo_list_title.html")]
struct ListHeader {
    list: List
}

pub async fn get_list_header(State(state): State<ApplicationState>, Path(list_id): Path<u64>) -> Result<Html<String>, KanbanError> {
    let list = sqlx::query_as::<_, List>("SELECT list_id, board_id, name FROM lists WHERE list_id = ?;")
        .bind(list_id as i64)
        .fetch_one(&state.db)
        .await?;

    let template = ListHeader { list };

    Ok(Html(template.render()?))
}

#[derive(Debug, Template)]
#[template(path = "turbo_list_title_edit.html")]
struct ListHeaderEdit {
    list: List
}

pub async fn get_list_edit(State(state): State<ApplicationState>, Path(list_id): Path<u64>) -> Result<Html<String>, KanbanError> {
    let list = sqlx::query_as::<_, List>("SELECT list_id, board_id, name FROM lists WHERE list_id = ?;")
        .bind(list_id as i64)
        .fetch_one(&state.db)
        .await?;

    let template = ListHeaderEdit { list };

    Ok(Html(template.render()?))
}

#[derive(Debug, Deserialize)]
pub struct ListRename {
    name: String
}

pub async fn post_list_rename(State(state): State<ApplicationState>, Path(list_id): Path<u64>, Form(list_header): Form<ListRename>) -> Result<Redirect, KanbanError> {
    let result = sqlx::query("UPDATE lists SET name = ? WHERE list_id = ?")
        .bind(list_header.name)
        .bind(list_id as i64)
        .execute(&state.db)
        .await?;

    if result.rows_affected() != 1 {
        return Err(KanbanError::DatabaseError("Update failed please try again.".to_string()));
    }

    Ok(Redirect::to(format!("/lists/{list_id}/header").as_str()))
}

pub async fn post_list_delete(State(state): State<ApplicationState>, Path(list_id): Path<u64>) -> Result<TurboStream, KanbanError> {
    sqlx::query("DELETE FROM lists WHERE list_id = ?;")
        .bind(list_id as i64)
        .execute(&state.db)
        .await?;

    let result = format!("<turbo-stream action=\"remove\" target=\"list-{list_id}\"></turbo-stream>");

    Ok(TurboStream(result))
}