use crate::handlers::*;
use crate::state::ApplicationState;
use axum::Router;
use tower_http::services::ServeDir;

pub fn create_router(application_state: ApplicationState) -> Router<()> {
    let api = api::get_router_configuration();
    let utility = utility::get_router_configuration();
    let web = web::get_router_configuration();
    let ws = ws::get_router_configuration();
    
    Router::new()
        .nest_service("/static", ServeDir::new("static"))
        .merge(api)
        .merge(utility)
        .merge(web)
        .merge(ws)
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
                    .unwrap()
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
                    .uri("/api/board")
                    .body(Body::empty())
                    .unwrap()
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
                    .uri("/api/board")
                    .header("Authorization", "Bearer super-secret")
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn landing_page_returns_200() {
        let state = create_application_state().await;
        let app = create_router(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}