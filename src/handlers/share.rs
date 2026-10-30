use axum::{Form, Json, Router, extract::{Path, State}, routing::post};
use garde::Validate;
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::types::chrono::Utc;

use crate::{data, errors::KanbanError, middleware::require_backend_access, models::{BoardAccess, User}, state::{ApplicationState, BackendDb}};

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/backend/{owner_database_id}/redeem", post(post_redeem_share_code))
        .route_layer(axum::middleware::from_fn(require_backend_access))
}

#[derive(Debug, Clone, Validate, Deserialize)]
struct RedeemShareRequest { 
    #[garde(skip)]
    share_code: String, 
    #[garde(skip)]
    display_name: String,
    #[garde(skip)]
    user_id: i64,
    #[garde(skip)]
    source: String,
    #[garde(skip)]
    avatar_url: Option<String>,
}

impl From<RedeemShareRequest> for User {
    fn from(value: RedeemShareRequest) -> Self {
        User {
            user_id: value.user_id,
            source: value.source,
            oauth_login: "".to_string(),
            display_name: Some(value.display_name),
            avatar_url: value.avatar_url,
        }
    }
}

// Need to get owner db access.
async fn post_redeem_share_code(
    State(_): State<ApplicationState>, 
    BackendDb(db): BackendDb, 
    Path(owner_database_id): Path<String>,
    Form(redeem_share): Form<RedeemShareRequest>,
) -> Result<Json<Value>, KanbanError> {
    // Redeem the share link. 
    let redeemed_share_link = data::redeem_share_link(db.clone(), redeem_share.share_code.clone()).await?;

    // Create the user record, or return an existing.
    let _ = data::upsert_user(db.clone(), User::from(redeem_share.clone())).await?;

    // Write the board access record.
    let board_access = BoardAccess {
        board_id: redeemed_share_link.board_id,
        granted_to_user_id: redeem_share.user_id,
        granted_to_user_source: redeem_share.source,
        permission: redeemed_share_link.permission,
        granted_at: Utc::now()
    };

    let _ = data::create_board_access(db.clone(), board_access).await?;

    // Need to get the board name...
    let board = data::get_board(db, redeemed_share_link.board_id).await?;

    // Return, owner_database_id, the shared board_id and shared board name.
    Ok(Json(json!({
        "owner_database_id": owner_database_id,
        "remote_board_id": redeemed_share_link.board_id,
        "remote_board_name": board.name,
    })))
}