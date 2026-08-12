use askama::Template;
use axum::extract::{Path, State};
use axum::{Form, Router};
use axum::response::{Html, Redirect};
use axum::routing::{get, post};
use serde::Deserialize;
use crate::data;
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
    let board_id = data::insert_board(&state.db, &board.name).await?;

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
    let board = data::get_board_with_lists(&state.db, board_id).await?;

    let response_template = BoardTemplate {
        board: board.board,
        lists: board.lists,
    };

    Ok(Html(response_template.render()?))
}

#[derive(Debug, Template)]
#[template(path = "turbo_board_title_edit.html")]
struct BoardHeaderEdit {
    board: Board
}

async fn get_board_edit(State(state): State<ApplicationState>, Path(board_id): Path<u64>) -> Result<Html<String>, KanbanError> {
    let board = data::get_board(&state.db, board_id).await?;

    let template = BoardHeaderEdit { board };

    Ok(Html(template.render()?))
}

#[derive(Debug, Template)]
#[template(path = "turbo_board_title.html")]
struct BoardHeader {
    board: Board
}

async fn get_board_header(State(state): State<ApplicationState>, Path(board_id): Path<u64>) -> Result<Html<String>, KanbanError> {
    let board = data::get_board(&state.db, board_id).await?;

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

    let result = data::update_board(&state.db, board_id, board.name).await?;

    if result != 1 {
        return Err(KanbanError::DatabaseError("Update failed please try again.".to_string()));
    }

    Ok(Redirect::to(format!("/boards/{}/header", board_id).as_str()))
}

async fn post_board_delete(State(state): State<ApplicationState>, Path(board_id):Path<u64>) -> Result<Redirect, KanbanError> {
    let result = data::delete_board(&state.db, board_id).await?;

    if result != 1 {
        return Err(KanbanError::BoardNotFound(board_id));
    }

    Ok(Redirect::to("/"))
}