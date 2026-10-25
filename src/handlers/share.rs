use axum::{Json, Router, routing::post};
use serde_json::Value;

use crate::{errors::KanbanError, middleware::require_backend_access, state::ApplicationState};

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/backend/{owner_database_id}/redeem", post(post_redeem_share_code))
        .route_layer(axum::middleware::from_fn(require_backend_access))
}

// Need to get owner db access.
async fn post_redeem_share_code() -> Result<Json<Value>, KanbanError> {
    Err(KanbanError::GeneralError("Whatever".to_string()))
}