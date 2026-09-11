pub mod boards;
pub mod lists;
pub mod cards;

use crate::data;
use crate::errors::KanbanError;
use crate::middleware::{require_csrf_token, require_web_auth};
use crate::models::Board;
use crate::state::{ApplicationState, CsrfTokenValue, UserDb};
use askama::Template;
use axum::Router;
use axum::response::Html;
use axum::routing::get;
use crate::validation::FormErrors;

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/", get(get_landing))
        .merge(boards::get_router_configuration())
        .merge(lists::get_router_configuration())
        .merge(cards::get_router_configuration())
        .layer(axum::middleware::from_fn(require_csrf_token))
        .layer(axum::middleware::from_fn(require_web_auth))
}

#[derive(Debug, Template)]
#[template(path = "landing.html")]
struct LandingTemplate {
    boards: Vec<Board>,
    new_board: NewContainerFormTemplate<Board>,
    csrf_token: String,
}

async fn get_landing(CsrfTokenValue(csrf_token): CsrfTokenValue, UserDb(db): UserDb) -> Result<Html<String>, KanbanError> {
    let boards = data::get_all_boards(db).await?;

    let new_board = NewContainerFormTemplate {
        sub_id: "board".to_string(),
        action: "/boards".to_string(),
        place_holder: "New board&hellip;".to_string(),
        button_sub_label: "Board".to_string(),
        errors: None,
        request: None,
    };

    let template = LandingTemplate {
        boards,
        new_board,
        csrf_token,
    };
    
    Ok(Html(template.render()?))
}

#[derive(Debug)]
pub struct NewContainerFormTemplate<T> {
    pub sub_id: String,
    pub action: String,
    pub place_holder: String,
    pub button_sub_label: String,
    pub errors: Option<FormErrors>,
    pub request: Option<T>,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::handlers::web::get_landing;
    use crate::router::{create_router, create_router_with_session};
    use crate::state::{create_application_state, CsrfTokenValue, UserDb};
    use axum::http::StatusCode;
    use axum::response::Response;
    use axum_test::TestServer;
    use sqlx::SqlitePool;
    use tower_sessions::{MemoryStore, Session, SessionStore};
    use crate::csrf::{get_or_create_secret, mask};
    use crate::handlers::auth::AUTHENTICATED_USER_KEY;

    pub async fn get_response_body(response: Response) -> String {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        String::from_utf8(body.to_vec()).unwrap()
    }

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

    pub async fn base_csrf_rejection_assertion(path: &str) {
        let store = MemoryStore::default();
        let state = create_application_state().await;

        let server = TestServer::builder()
            .save_cookies()
            .build(create_router_with_session(state, store.clone()));

        // Establish a session.
        let response = server.get("/auth/login").await;

        // Pull the session ID out of the cookie.
        let cookie = response.cookie("id");
        let session_id = cookie.value().parse().unwrap();

        // Load the record and insert user data directly into the store, because
        // if you are authed, we don't even check to see if you passed a csrf token.
        let mut record = store.load(&session_id).await.unwrap().unwrap();
        record.data.insert(
            AUTHENTICATED_USER_KEY.to_string(),
            serde_json::json!({ "id": 1, "name": "testuser", "source": "dev" }),
        );
        store.save(&record).await.unwrap();

        let response = server.post(path).await;

        response.assert_status_bad_request();
        response.assert_text_contains("No CSRF token found.");
    }

    pub async fn base_csrf_acceptance_redirect_assertion(path: &str, redirect_url: &str, form: Vec<(String, String)>) {
        let store = MemoryStore::default();
        let state = create_application_state().await;

        let server = TestServer::builder()
            .save_cookies()
            .build(create_router_with_session(state, store.clone()));

        // Establish a session.
        let response = server.get("/auth/login").await;

        // Pull the session ID out of the cookie.
        let cookie = response.cookie("id");
        let session_id = cookie.value().parse().unwrap();

        // Load the record and insert user data directly into the store, because
        // if you are authed, we don't even check to see if you passed a csrf token.
        let mut record = store.load(&session_id).await.unwrap().unwrap();
        record.data.insert(
            AUTHENTICATED_USER_KEY.to_string(),
            serde_json::json!({ "id": 1, "name": "testuser", "source": "dev" }),
        );
        store.save(&record).await.unwrap();

        let session = Session::new(Some(session_id), Arc::new(store.clone()), None);
        let secret = get_or_create_secret(&session).await.unwrap();

        let _ = session.save().await;

        let token = mask(&secret);

        let mut csrf_form: Vec<(String, String)> = vec!();
        csrf_form.push(("csrf_token".to_string(), token));
        csrf_form.append(form.clone().as_mut());

        let response = server.post(path).form(&csrf_form).await;

        // TODO: I think we need two different kinds of utility methods, one for posts that reeturn
        //      ok, and one that detects redirects correctly.
        response.assert_status(StatusCode::SEE_OTHER);
        response.assert_header("location", redirect_url);
    }

    pub async fn base_csrf_acceptance_assertion(path: &str, form: Vec<(String, String)>) {
        let store = MemoryStore::default();
        let state = create_application_state().await;

        let server = TestServer::builder()
            .save_cookies()
            .build(create_router_with_session(state, store.clone()));

        // Establish a session.
        let response = server.get("/auth/login").await;

        // Pull the session ID out of the cookie.
        let cookie = response.cookie("id");
        let session_id = cookie.value().parse().unwrap();

        // Load the record and insert user data directly into the store, because
        // if you are authed, we don't even check to see if you passed a csrf token.
        let mut record = store.load(&session_id).await.unwrap().unwrap();
        record.data.insert(
            AUTHENTICATED_USER_KEY.to_string(),
            serde_json::json!({ "id": 1, "name": "testuser", "source": "dev" }),
        );
        store.save(&record).await.unwrap();

        let session = Session::new(Some(session_id), Arc::new(store.clone()), None);
        let secret = get_or_create_secret(&session).await.unwrap();

        let _ = session.save().await;

        let token = mask(&secret);

        let mut csrf_form: Vec<(String, String)> = vec!();
        csrf_form.push(("csrf_token".to_string(), token));
        csrf_form.append(form.clone().as_mut());

        let response = server.post(path).form(&csrf_form).await;

        // TODO: I think we need two different kinds of utility methods, one for posts that reeturn
        //      ok, and one that detects redirects correctly.
        response.assert_status_success();
    }

    #[sqlx::test(fixtures(path="../fixtures", scripts("boards")))]
    async fn get_landing_returns_all_boards(db: SqlitePool) -> sqlx::Result<()> {
        let db = UserDb(db);

        let response = get_landing(CsrfTokenValue("token".to_string()), db).await.unwrap().0;

        assert!(response.contains("board-1"));
        assert!(response.contains("board-2"));
        assert!(response.contains("board-3"));

        Ok(())
    }
}