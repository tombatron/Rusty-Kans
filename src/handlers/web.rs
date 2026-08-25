pub mod boards;
pub mod lists;
pub mod cards;

use crate::data;
use crate::errors::KanbanError;
use crate::middleware::require_web_auth;
use crate::models::Board;
use crate::state::{ApplicationState, UserDb};
use askama::Template;
use axum::Router;
use axum::response::Html;
use axum::routing::get;

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/", get(get_landing))
        .merge(boards::get_router_configuration())
        .merge(lists::get_router_configuration())
        .merge(cards::get_router_configuration())
        .layer(axum::middleware::from_fn(require_web_auth))
}

#[derive(Debug, Template)]
#[template(path = "landing.html")]
struct LandingTemplate {
    boards: Vec<Board>
}

async fn get_landing(UserDb(db): UserDb) -> Result<Html<String>, KanbanError> {
    let boards = data::get_all_boards(db).await?;

    let template = LandingTemplate { boards };
    
    Ok(Html(template.render()?))
}

#[cfg(test)]
mod tests {
    use crate::handlers::web::get_landing;
    use crate::router::create_router;
    use crate::state::{UserDb, create_application_state};
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use sqlx::SqlitePool;

    pub async fn base_auth_get_assertion(path: &str) {
        let state = create_application_state().await;

        let server = TestServer::builder().build(create_router(state));

        let response = server.get(path).await;

        response.assert_status(StatusCode::SEE_OTHER);
        response.assert_header("location", "/auth");
    }

    pub async fn base_auth_post_assertion(path: &str) {
        let state = create_application_state().await;

        let server = TestServer::builder().build(create_router(state));

        let response = server.post(path).await;

        response.assert_status(StatusCode::SEE_OTHER);
        response.assert_header("location", "/auth");
    }

    #[sqlx::test(fixtures(path="../fixtures", scripts("boards")))]
    async fn get_landing_returns_all_boards(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let response = get_landing(db).await.unwrap().0;

        assert!(response.contains("board-1"));
        assert!(response.contains("board-2"));
        assert!(response.contains("board-3"));

        Ok(())
    }
}