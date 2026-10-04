use crate::csrf::{get_or_create_secret, verify};
use crate::data;
use crate::errors::KanbanError;
use crate::handlers::auth::AUTHENTICATED_USER_KEY;
use crate::models::Permission;
use crate::models::security::LoggedInUser;
use crate::state::{ApplicationState, get_pool_with_id};
use axum::body::Body;
use axum::body::to_bytes;
use axum::extract::{Path, Request, State};
use axum::http::Method;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Redirect, Response};
use tower_sessions::Session;

pub async fn require_web_auth(session: Session, request: Request, next: Next) -> Response {
    let user: Option<LoggedInUser> = session
        .get(AUTHENTICATED_USER_KEY)
        .await
        .unwrap_or_default();

    match user {
        Some(_) => next.run(request).await,
        _ => Redirect::to("/auth").into_response(),
    }
}

pub async fn require_csrf_token(
    session: Session,
    request: Request,
    next: Next,
) -> Result<Response, KanbanError> {
    if request.method() == Method::POST {
        let (parts, body) = request.into_parts();
        let body_bytes = to_bytes(body, 50000).await?;

        let header_token = parts.headers.get("X-CSRF-TOKEN");

        let csrf_token: String = if let Some(token) = header_token {
            token.to_str().unwrap_or("").to_string()
        } else {
            let mut posted_form = form_urlencoded::parse(&body_bytes);

            let token = posted_form
                .find(|(key, _)| key == "csrf_token")
                .map(|(_, value)| value.to_string())
                .unwrap_or_default()
                .to_string();

            token
        };

        let csrf_secret = get_or_create_secret(&session).await?;

        if verify(&csrf_token, &csrf_secret) {
            Ok(next
                .run(Request::from_parts(parts, Body::from(body_bytes)))
                .await)
        } else {
            Err(KanbanError::RequestError(
                "No CSRF token found.".to_string(),
            ))
        }
    } else {
        Ok(next.run(request).await)
    }
}

pub async fn require_delegated_access(
    State(state): State<ApplicationState>,
    Path((owner_database_id, board_id)): Path<(String, i64)>,
    session: Session,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    // This should be the user account of the user requesting access to the endpoint.
    let user: LoggedInUser = session
        .get(AUTHENTICATED_USER_KEY)
        .await
        // No user to access...
        .map_err(|_| {
            (
                StatusCode::FORBIDDEN,
                "Something isn't right... you aren't logged in.".to_string(),
            )
        })?
        .ok_or((StatusCode::FORBIDDEN, "Auth required brah.".to_string()))?;

    // Let's go get a database.
    let database_pool = get_pool_with_id(&state.db_pools, owner_database_id.clone())
        .await
        .ok_or((
            StatusCode::NOT_FOUND,
            "Not here. Try something else".to_string(),
        ))?;

    // We've got a database, let's check access...
    let access = data::get_board_access_for_user(database_pool, board_id, user.id, user.source)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something isn't right here...".to_string(),
            )
        })?
        .ok_or((StatusCode::FORBIDDEN, "Sorry brah, no access.".to_string()))?;

    if access.permission < Permission::Edit
        && !(request.method() == Method::GET || request.method() == Method::HEAD)
    {
        return Err((StatusCode::FORBIDDEN, "You don't have the powa!".to_string()));
    }

    // We we're here, let's proceed.
    Ok(next.run(request).await)
}
