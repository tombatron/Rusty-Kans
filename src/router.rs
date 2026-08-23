use crate::handlers::*;
use crate::state::ApplicationState;
use axum::Router;
use tower_http::services::ServeDir;
use tower_sessions::cookie::time::Duration;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer, SessionStore};
use tower_sessions_redis_store::RedisStore;

pub fn create_router(application_state: ApplicationState) -> Router<()> {
    match application_state.clone().redis_pool {
        Some(pool) => create_router_with_session(application_state, RedisStore::new(pool)),

        None => create_router_with_session(application_state, MemoryStore::default())
    }
}

pub fn create_router_with_session<S>(
    application_state: ApplicationState,
    session_store: S,
) -> Router<()>
where
    S: SessionStore + Clone,
{
    let auth = auth::get_router_configuration();
    let api = api::get_router_configuration();
    let utility = utility::get_router_configuration();
    let web = web::get_router_configuration();
    let ws = ws::get_router_configuration();

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // This needs to be `true` for production.
        .with_expiry(Expiry::OnInactivity(Duration::minutes(30)));

    Router::new()
        .nest_service("/static", ServeDir::new("static"))
        .merge(auth)
        .merge(web)
        .merge(ws)
        .layer(session_layer)
        .merge(api)
        .merge(utility)
        .with_state(application_state)
}

#[cfg(test)]
mod tests {
    use crate::router::create_router;
    use crate::state::create_application_state;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_check_returns_200() {
        let state = create_application_state().await;
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn api_rejects_unauthenticated_requests() {
        let state = create_application_state().await;
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/board/1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn api_accepts_authenticated_requests() {
        let state = create_application_state().await;
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/board/1")
                    .header("Authorization", "Bearer super-secret")
                    .header("delegated-user-id", i64::MIN)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
