use axum::{Form, Json, Router, extract::State, routing::post};
use garde::Validate;
use serde::Deserialize;
use serde_json::Value;

use crate::{data, errors::KanbanError, middleware::require_backend_access, state::{ApplicationState, BackendDb}};

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/backend/{owner_database_id}/redeem", post(post_redeem_share_code))
        .route_layer(axum::middleware::from_fn(require_backend_access))
}

#[derive(Debug, Validate, Deserialize)]
struct RedeemShareRequest { 
    #[garde(skip)]
    share_code: String, 
    #[garde(skip)]
    display_name: String,
    #[garde(skip)]
    avatar_url: Option<String>,
}

#[axum::debug_handler]
// Need to get owner db access.
async fn post_redeem_share_code(State(state): State<ApplicationState>, BackendDb(db): BackendDb, Form(redeem_share): Form<RedeemShareRequest>) -> Result<Json<Value>, KanbanError> {
    // Redeem the share link. 
    let redeemed_share_link = data::redeem_share_link(db, redeem_share.share_code).await?;

    // Create the user record, or return an existing.

    // Write the board access record.

    // Return, owner_database_id, the shared board_id and shared board name.

    Err(KanbanError::GeneralError("Whatever".to_string()))
}