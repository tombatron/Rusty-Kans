use askama::Template;
use axum::extract::{Path, State};
use axum::{Form, Router};
use axum::response::{Html, Redirect};
use axum::routing::{get, post};
use serde::Deserialize;
use sqlx::AssertSqlSafe;
use crate::errors::KanbanError;
use crate::models::{Board, Card, List};
use crate::state::ApplicationState;
use crate::turbo::TurboStream;

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/boards", post(post_board_form))
        .route("/boards/{board_id}", get(get_board))
        .route("/boards/{board_id}/edit", get(get_board_edit))
        .route("/boards/{board_id}/header", get(get_board_header))
        .route("/boards/{board_id}/rename", post(post_board_rename))
        .route("/boards/{board_id}/delete", post(post_board_delete))
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

async fn post_board_form(State(state): State<ApplicationState>, Form(board): Form<CreateBoardRequest>) -> Result<TurboStream, KanbanError> {
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
    board: Board,
    lists: Vec<List>,
}

pub async fn get_board(
    State(state): State<ApplicationState>,
    Path(board_id): Path<u64>
) -> Result<Html<String>, KanbanError> {
    let board = sqlx::query_as::<_, Board>("SELECT board_id, name FROM boards WHERE board_id = ?;")
        .bind(board_id as i64)
        .fetch_optional(&state.db)
        .await?
        .ok_or(KanbanError::BoardNotFound(board_id))?;

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
        board,
        lists,
    };

    Ok(Html(response_template.render()?))
}

#[derive(Debug, Template)]
#[template(path = "turbo_board_title_edit.html")]
struct BoardHeaderEdit {
    board: Board
}

async fn get_board_edit(State(state): State<ApplicationState>, Path(board_id): Path<u64>) -> Result<Html<String>, KanbanError> {
    let board = sqlx::query_as::<_, Board>("SELECT board_id, name FROM boards WHERE board_id = ?;")
        .bind(board_id as i64)
        .fetch_optional(&state.db)
        .await?
        .ok_or(KanbanError::BoardNotFound(board_id))?;

    let template = BoardHeaderEdit { board };

    Ok(Html(template.render()?))
}

#[derive(Debug, Template)]
#[template(path = "turbo_board_title.html")]
struct BoardHeader {
    board: Board
}

async fn get_board_header(State(state): State<ApplicationState>, Path(board_id): Path<u64>) -> Result<Html<String>, KanbanError> {
    let board = sqlx::query_as::<_, Board>("SELECT board_id, name FROM boards WHERE board_id = ?;")
        .bind(board_id as i64)
        .fetch_optional(&state.db)
        .await?
        .ok_or(KanbanError::BoardNotFound(board_id))?;

    let template = BoardHeader { board };

    Ok(Html(template.render()?))
}

#[derive(Debug, Deserialize)]
struct BoardRename {
    id: u64,
    name: String,
}

async fn post_board_rename(State(state): State<ApplicationState>, Path(board_id):Path<u64>, Form(board): Form<BoardRename>) -> Result<Redirect, KanbanError> {
    if board_id != board.id {
        return Err(KanbanError::RequestError(format!("Ambiguous board specified, path says `{}` and the form says `{}", board_id, board.id)));
    }

    let result = sqlx::query("UPDATE boards SET name = ? WHERE board_id = ?;")
        .bind(board.name)
        .bind(board_id as i64)
        .execute(&state.db)
        .await?;

    if result.rows_affected() != 1 {
        return Err(KanbanError::DatabaseError("Update failed please try again.".to_string()));
    }

    Ok(Redirect::to(format!("/boards/{}/header", board_id).as_str()))
}

async fn post_board_delete(State(state): State<ApplicationState>, Path(board_id):Path<u64>) -> Result<Redirect, KanbanError> {
    let result = sqlx::query("DELETE FROM boards WHERE board_id = ?;")
        .bind(board_id as i64)
        .execute(&state.db)
        .await?;

    if result.rows_affected() != 1 {
        return Err(KanbanError::BoardNotFound(board_id));
    }

    Ok(Redirect::to("/"))
}