pub mod boards;
pub mod lists;
pub mod cards;

use crate::errors::KanbanError;
use crate::models::Board;
use crate::state::ApplicationState;
use askama::Template;
use axum::extract::State;
use axum::response::Html;
use axum::routing::get;
use axum::Router;
use crate::data;

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

async fn get_landing(State(state): State<ApplicationState>) -> Result<Html<String>, KanbanError> {
    let boards = data::get_all_boards(&state.db).await?;

    let template = LandingTemplate { boards };
    
    Ok(Html(template.render()?))
}

#[cfg(test)]
mod tests {
    use axum::extract::State;
    use sqlx::SqlitePool;
    use crate::handlers::tests::get_fake_application_state;
    use crate::handlers::web::get_landing;

    #[sqlx::test(fixtures(path="../fixtures", scripts("boards")))]
    async fn get_landing_returns_all_boards(db: SqlitePool) -> sqlx::Result<()> {
        let state = State(get_fake_application_state(db));

        let response = get_landing(state).await.unwrap().0;

        assert!(response.contains("board-1"));
        assert!(response.contains("board-2"));
        assert!(response.contains("board-3"));

        Ok(())
    }
}