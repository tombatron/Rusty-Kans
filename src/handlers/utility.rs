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