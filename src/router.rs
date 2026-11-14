use crate::handlers::*;
use crate::state::ApplicationState;
use axum::Router;
use axum::http::{HeaderValue, header};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_sessions::cookie::SameSite;
use tower_sessions::cookie::time::Duration;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer, SessionStore};
use tower_sessions_redis_store::RedisStore;

pub fn create_router(application_state: ApplicationState) -> Router<()> {
    match application_state.clone().redis_pool {
        Some(pool) => create_router_with_session(application_state, RedisStore::new(pool)),

        None => create_router_with_session(application_state, MemoryStore::default()),
    }
}

pub fn create_router_with_session<S>(application_state: ApplicationState, session_store: S) -> Router<()>
where
    S: SessionStore + Clone,
{
    let auth = auth::get_router_configuration();
    let api = api::get_router_configuration(application_state.clone());
    let utility = utility::get_router_configuration();
    let web = web::get_router_configuration();
    let ws = ws::get_router_configuration();
    let share = share::get_router_configuration(application_state.clone());

    let session_layer = SessionManagerLayer::new(session_store)
        .with_same_site(SameSite::Lax)
        .with_secure(!cfg!(debug_assertions))
        .with_expiry(Expiry::OnInactivity(Duration::minutes(30)));

    let router = Router::new()
        .nest_service("/static", ServeDir::new("static"))
        .merge(auth)
        .merge(web)
        .merge(ws)
        .merge(api)
        .layer(session_layer)
        .merge(utility)
        .merge(share);

    // In debug builds, prevent the browser from caching anything so template/CSS
    // changes are always visible on the next reload without manual cache-busting.
    let router = if cfg!(debug_assertions) {
        router.layer(SetResponseHeaderLayer::overriding(header::CACHE_CONTROL, HeaderValue::from_static("no-store")))
    } else {
        router
    };

    router.with_state(application_state)
}

#[cfg(test)]
mod tests {
    use crate::handlers::auth::{AUTHENTICATED_USER_KEY, AuthSources};
    use crate::handlers::tests::TestDatabaseGuard;
    use crate::router::{create_router, create_router_with_session};
    use crate::state::create_application_state;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum_test::TestServer;
    use reqwest::Method;
use serde_json::json;
use sqlx::SqlitePool;
    use std::sync::Arc;
    use tower::ServiceExt;
    use tower_sessions::{MemoryStore, Session, SessionStore};

    #[tokio::test]
    async fn health_check_returns_200() {
        let state = create_application_state().await;
        let app = create_router(state);

        let response = app.oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap()).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn api_rejects_unauthenticated_requests() {
        let state = create_application_state().await;
        let app = create_router(state);

        let response = app.oneshot(
            Request::builder()
                .uri("/api/whatever-test-db/board/1")
                .body(Body::empty())
                .unwrap()
            ).await.unwrap();

        assert_eq!(StatusCode::FORBIDDEN, response.status());
    }

    #[sqlx::test(fixtures(path = "./fixtures", scripts("boards")))]
    async fn api_accepts_authenticated_requests(db_pool: SqlitePool) -> sqlx::Result<()> {
        let user_id: i64 = -20000;

        let _use_me = TestDatabaseGuard::new(user_id);

        let store = MemoryStore::default();
        let state = create_application_state().await;

        // Insert the db_pool into the map of cached db pools.
        state.db_pools.insert("whatever-test-db".to_string(), db_pool.clone());

        let mut server_builder = TestServer::builder();
        server_builder = server_builder.save_cookies();

        let server = server_builder.build(create_router_with_session(state.clone(), store.clone()));

        // Establish a session.
        let response = server.get("/auth/login").await;

        // Pull the session ID out of the cookie.
        let cookie = response.cookie("id");
        let session_id = cookie.value().parse().unwrap();

        // Load the record and insert user data directly into the store, becase
        // if you are authed, we don't even check to see if you passed a csrf token.
        let mut record = store.load(&session_id).await.unwrap().unwrap();
        record.data.insert(AUTHENTICATED_USER_KEY.to_string(), serde_json::json!({"id": user_id, "name": "whatever", "source": AuthSources::DEV }));
        store.save(&record).await.unwrap();

        let session = Session::new(Some(session_id), Arc::new(store.clone()), None);

        let _ = session.save().await;

        let response = server.get("/api/whatever-test-db/board/1").await;

        response.assert_status_success();

        Ok(())
    }

    #[sqlx::test(fixtures(path = "./fixtures", scripts("boards")))]
    async fn api_rejects_authenticated_requests_with_no_board_access(db_pool: SqlitePool) -> sqlx::Result<()> {
        let user_id: i64 = -7000;

        let _use_me = TestDatabaseGuard::new(user_id);

        let store = MemoryStore::default();
        let state = create_application_state().await;

        // Insert the db_pool into the map of cached db pools.
        state.db_pools.insert("whatever-test-db".to_string(), db_pool.clone());

        let mut server_builder = TestServer::builder();
        server_builder = server_builder.save_cookies();

        let server = server_builder.build(create_router_with_session(state.clone(), store.clone()));

        // Establish a session.
        let response = server.get("/auth/login").await;

        // Pull the session ID out of the cookie.
        let cookie = response.cookie("id");
        let session_id = cookie.value().parse().unwrap();

        // Load the record and insert user data directly into the store, becase
        // if you are authed, we don't even check to see if you passed a csrf token.
        let mut record = store.load(&session_id).await.unwrap().unwrap();
        record.data.insert(AUTHENTICATED_USER_KEY.to_string(), serde_json::json!({"id": user_id, "name": "whatever", "source": AuthSources::DEV }));
        store.save(&record).await.unwrap();

        let session = Session::new(Some(session_id), Arc::new(store.clone()), None);

        let _ = session.save().await;

        let response = server.get("/api/whatever-test-db/board/1").await;

        response.assert_status_forbidden();

        Ok(())
    }

    #[sqlx::test(fixtures(path = "./fixtures", scripts("boards")))]
    async fn api_rejects_authenticated_edit_requests_with_no_edit_access(db_pool: SqlitePool) -> sqlx::Result<()> {
        let user_id: i64 = -40000; // This user should only have view access.

        let _use_me = TestDatabaseGuard::new(user_id);

        let store = MemoryStore::default();
        let state = create_application_state().await;

        // Insert the db_pool into the map of cached db pools.
        state.db_pools.insert("whatever-test-db".to_string(), db_pool.clone());

        let mut server_builder = TestServer::builder();
        server_builder = server_builder.save_cookies();

        let server = server_builder.build(create_router_with_session(state.clone(), store.clone()));

        // Establish a session.
        let response = server.get("/auth/login").await;

        // Pull the session ID out of the cookie.
        let cookie = response.cookie("id");
        let session_id = cookie.value().parse().unwrap();

        // Load the record and insert user data directly into the store, becase
        // if you are authed, we don't even check to see if you passed a csrf token.
        let mut record = store.load(&session_id).await.unwrap().unwrap();
        record.data.insert(AUTHENTICATED_USER_KEY.to_string(), serde_json::json!({"id": user_id, "name": "whatever", "source": AuthSources::DEV }));
        store.save(&record).await.unwrap();

        let session = Session::new(Some(session_id), Arc::new(store.clone()), None);

        let _ = session.save().await;

        let response = server
            .post("/api/whatever-test-db/1/lists/1/cards")
            .json(&json!({
                "title": "whatever",
                "description": "this doesn't even matter"
            }))
            .await;

        response.assert_status_forbidden();

        Ok(())
    }

    #[tokio::test]
    async fn require_delegated_rejects_missing_auth_header_as_forbidden() {
        let state = create_application_state().await;
        let app = create_router(state);

        let request = Request::builder()
            .method(Method::POST)
            .uri("/backend/whatever_db/redeem")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn require_delegated_rejects_wrong_auth_header_as_unauthorized() {
        let state = create_application_state().await;
        let app = create_router(state);

        let request = Request::builder()
            .method(Method::POST)
            .uri("/backend/whatever_db/redeem")
            .header("X-backend-key", "not even close")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn request_delegated_rejects_unknown_db_as_bad_request() {
        let state = create_application_state().await;
        let app = create_router(state);

        let request = Request::builder()
            .method(Method::POST)
            .uri("/backend/whatever_db/redeem")
            .header("X-backend-key", "change_this_in_production")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
