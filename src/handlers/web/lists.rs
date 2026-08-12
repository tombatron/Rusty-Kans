use askama::Template;
use axum::extract::{Path, State};
use axum::{Form, Router};
use axum::response::{Html, Redirect};
use axum::routing::{get, post};
use serde::Deserialize;
use crate::data;
use crate::errors::KanbanError;
use crate::handlers::{create_list_common, CreateListRequest};
use crate::models::List;
use crate::state::ApplicationState;
use crate::turbo::TurboStream;

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/boards/{board_id}/lists", post(post_list_form))
        .route("/lists/{list_id}/edit", get(get_list_edit))
        .route("/lists/{list_id}/rename", post(post_list_rename))
        .route("/lists/{list_id}/header", get(get_list_header))
        .route("/lists/{list_id}/delete", post(post_list_delete))
}

async fn post_list_form(
    State(state): State<ApplicationState>,
    Path(board_id): Path<u64>,
    Form(list_info): Form<CreateListRequest>,
) -> Result<TurboStream, KanbanError> {
    let created_list = create_list_common(State(state), board_id, list_info).await?;

    Ok(TurboStream(created_list.render()?))
}

#[derive(Debug, Template)]
#[template(path = "turbo_list_title.html")]
struct ListHeader {
    list: List
}

async fn get_list_header(State(state): State<ApplicationState>, Path(list_id): Path<u64>) -> Result<Html<String>, KanbanError> {
    let list = data::get_list_header(&state.db, list_id).await?;

    let template = ListHeader { list };

    Ok(Html(template.render()?))
}

#[derive(Debug, Template)]
#[template(path = "turbo_list_title_edit.html")]
struct ListHeaderEdit {
    list: List
}

async fn get_list_edit(State(state): State<ApplicationState>, Path(list_id): Path<u64>) -> Result<Html<String>, KanbanError> {
    let list = data::get_list_header(&state.db, list_id).await?;

    let template = ListHeaderEdit { list };

    Ok(Html(template.render()?))
}

#[derive(Debug, Deserialize)]
struct ListRename {
    name: String
}

async fn post_list_rename(State(state): State<ApplicationState>, Path(list_id): Path<u64>, Form(list_header): Form<ListRename>) -> Result<Redirect, KanbanError> {
    let result = data::update_list(&state.db, list_id, list_header.name).await?;

    if result != 1 {
        return Err(KanbanError::DatabaseError("Update failed please try again.".to_string()));
    }

    Ok(Redirect::to(format!("/lists/{list_id}/header").as_str()))
}

async fn post_list_delete(State(state): State<ApplicationState>, Path(list_id): Path<u64>) -> Result<TurboStream, KanbanError> {
    data::delete_list(&state.db, list_id).await?;

    let result = format!("<turbo-stream action=\"remove\" target=\"list-{list_id}\"></turbo-stream>");

    Ok(TurboStream(result))
}