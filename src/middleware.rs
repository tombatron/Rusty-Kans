use crate::handlers::auth::GITHUB_USER_KEY;
use axum::extract::Request;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Redirect, Response};
use tower_sessions::Session;

pub async fn require_auth(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    
    match token {
        Some(t) if t == "super-secret" => Ok(next.run(request).await),
        _ => Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn require_web_auth(session: Session, request: Request, next: Next) -> Result<Response, StatusCode> {
    let user: Option<String> = session.get(GITHUB_USER_KEY)
        .await
        .unwrap_or_default();

    match user {
        Some(_) => Ok(next.run(request).await),
        _ => Ok(Redirect::to("/auth/login").into_response())
    }
}
