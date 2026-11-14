use axum::{
    Form, Json, Router,
    extract::{Path, State},
    routing::post,
};
use garde::Validate;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::types::chrono::Utc;

use crate::{
    data,
    errors::KanbanError,
    middleware::require_backend_access,
    models::{BoardAccess, User},
    state::{ApplicationState, BackendDb},
};

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/backend/{owner_database_id}/redeem", post(post_redeem_share_code))
        .route("/backend/{owner_database_id}/revoke/{shared_database_id}/{remote_board_id}", post(post_revoke_grantee_access))
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

async fn post_redeem_share_code(State(_): State<ApplicationState>, BackendDb(db): BackendDb, Path(owner_database_id): Path<String>, Form(redeem_share): Form<RedeemShareRequest>) -> Result<Json<Value>, KanbanError> {
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
        granted_at: Utc::now(),
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

async fn post_revoke_grantee_access(
    State(_): State<ApplicationState>, 
    BackendDb(db): BackendDb, 
    Path((_, shared_database_id, remote_board_id)
): Path<(String, String, i64)>) -> Result<StatusCode, KanbanError> {
    let _ = data::delete_board_access_grant(db, shared_database_id, remote_board_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
pub mod tests {
    use axum::{Form, extract::{Path, State}};
    use reqwest::StatusCode;
use sqlx::SqlitePool;

    use crate::{
        errors::KanbanError, handlers::{
            share::{RedeemShareRequest, post_redeem_share_code, post_revoke_grantee_access}, tests::get_fake_application_state,
        }, state::BackendDb,
    };

    #[sqlx::test(fixtures(path = "../fixtures", scripts("boards")))]
    async fn post_redeem_share_code_succeeds_when_code_is_valid(pool: SqlitePool) -> sqlx::Result<()> {
        let fake_state = State(get_fake_application_state());

        let redeem_request = RedeemShareRequest {
            share_code: "valid-share-code".to_string(),
            display_name: "whatever".to_string(),
            user_id: -10000,
            source: "dev".to_string(),
            avatar_url: None,
        };

        let result = post_redeem_share_code(fake_state, BackendDb(pool), Path("whatever_id".to_string()), Form(redeem_request))
            .await
            .unwrap();

        assert_eq!("whatever_id", result["owner_database_id"]);
        assert_eq!(1, result["remote_board_id"]);
        assert_eq!("Whatever", result["remote_board_name"]);

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../fixtures", scripts("boards")))]
    async fn post_redeem_share_code_fails_when_code_is_invalid(pool: SqlitePool) -> sqlx::Result<()> {
        let fake_state = State(get_fake_application_state());

        let redeem_request = RedeemShareRequest {
            share_code: "invalid-share-code".to_string(),
            display_name: "whatever".to_string(),
            user_id: -10000,
            source: "dev".to_string(),
            avatar_url: None,
        };

        let result = post_redeem_share_code(fake_state, BackendDb(pool), Path("whatever_id".to_string()), Form(redeem_request))
            .await;

        assert!(result.is_err());

        assert!(matches!(result, Err(KanbanError::DatabaseError(_))));
        assert_eq!(result.unwrap_err().to_string(), "There was a database error: no rows returned by a query that expected to return at least one row".to_string());

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../fixtures", scripts("boards")))]
    async fn post_redeem_share_code_fails_when_code_is_expired(pool: SqlitePool) -> sqlx::Result<()> {
        let fake_state = State(get_fake_application_state());

        let redeem_request = RedeemShareRequest {
            share_code: "expired-share-code".to_string(),
            display_name: "whatever".to_string(),
            user_id: -10000,
            source: "dev".to_string(),
            avatar_url: None,
        };

        let result = post_redeem_share_code(fake_state, BackendDb(pool), Path("whatever_id".to_string()), Form(redeem_request))
            .await;

        assert!(result.is_err());

        assert!(matches!(result, Err(KanbanError::DatabaseError(_))));
        assert_eq!(result.unwrap_err().to_string(), "There was a database error: no rows returned by a query that expected to return at least one row".to_string());

        Ok(())
    }

    #[sqlx::test(fixtures(path = "../fixtures", scripts("boards")))]
    async fn post_revoke_grantee_access_succeeds(pool: SqlitePool) -> sqlx::Result<()> {
        let fake_state = State(get_fake_application_state());
        let database = BackendDb(pool.clone());
        let request_path = Path(("empty".to_string(), "whatever".to_string(), 1));

        let response = post_revoke_grantee_access(fake_state, database, request_path).await.unwrap();

        let count:i64 = sqlx::query_scalar("SELECT COUNT(*) FROM board_access_grants WHERE owner_database_id = $1;")
            .bind("whatever")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(StatusCode::NO_CONTENT, response);
        assert_eq!(0, count);

        Ok(())
    }
}
