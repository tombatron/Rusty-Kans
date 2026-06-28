mod models;
mod errors;

use axum::{Json, Router};
use axum::routing::get;
use serde_json::{json, Value};

async fn get_health() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

#[tokio::main]
async fn main() {
    // Router with initial health endpoint.
    let app = Router::new()
        .route("/health", get(get_health));

    // Setup the listener on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
