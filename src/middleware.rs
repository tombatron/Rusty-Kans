use crate::errors::KanbanError;
use crate::handlers::auth::AUTHENTICATED_USER_KEY;
use crate::models::security::LoggedInUser;
use axum::body::to_bytes;
use axum::extract::Request;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Redirect, Response};
use tower_sessions::Session;
use axum::http::Method;
use axum::body::Body;
use crate::csrf::{get_or_create_secret, verify};

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
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

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

        let csrf_token:String = if let Some(token) = header_token {
            token.to_str().unwrap_or("").to_string()
        } else {
            let mut posted_form = form_urlencoded::parse(&body_bytes);

            let token = posted_form
                .find(|(key, _) | key == "csrf_token")
                .map(|(_, value)| value.to_string())
                .unwrap_or_default().to_string();

            token
        };

        let csrf_secret = get_or_create_secret(&session).await?;

        if verify(&csrf_token, &csrf_secret) {
            Ok(next.run(Request::from_parts(parts, Body::from(body_bytes))).await)
        } else {
            Err(KanbanError::RequestError("No CSRF token found.".to_string()))
        }
    } else {
        Ok(next.run(request).await)
    }
}
