pub mod boards;
pub mod lists;
pub mod cards;

use crate::errors::KanbanError;
use crate::models::Board;
use crate::state::ApplicationState;
use askama::Template;
use axum::extract::State;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/", get(get_landing))

        .merge(boards::get_router_configuration())
        .merge(lists::get_router_configuration())
        .merge(cards::get_router_configuration())
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