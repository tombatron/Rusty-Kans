use axum::{Json, Router};
use axum::routing::get;
use serde_json::{json, Value};
use crate::state::ApplicationState;

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/health", get(get_health))
}

pub async fn get_health() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

#[cfg(test)]
mod tests {
    use crate::handlers::utility::get_health;

    #[tokio::test]
    async fn get_health_returns_health_response() {
        let response = get_health().await.0;

        assert_eq!("ok", response["status"]);
    }
}