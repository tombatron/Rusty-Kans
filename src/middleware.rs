use axum::extract::Request;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::Response;

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
