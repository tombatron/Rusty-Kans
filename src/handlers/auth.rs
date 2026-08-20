use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::Router;
use axum::routing::{get, post};
use oauth2::{CsrfToken, PkceCodeChallenge, Scope};
use tower_sessions::Session;
use crate::errors::KanbanError;
use crate::state::ApplicationState;

const CSRF_TOKEN_KEY: &str = "CSRF_TOKEN";
const PKCE_VERIFIER_KEY: &str = "PKCE_VERIFIER";

pub fn get_router_configuration() -> Router<ApplicationState> {
    Router::new()
        .route("/auth/login", get(get_login))
        .route("/auth/callback", get(get_callback))
        .route("/auth/logout", post(post_logout))
}

async fn get_login(State(state): State<ApplicationState>, session: Session) -> Result<Redirect, KanbanError> {
    // Generate PKCE challenge.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let (auth_url, csrf_token) = state.oauth_client
        .authorize_url(CsrfToken::new_random)
        // Set the desired scopes.
        .add_scope(Scope::new("read".to_string()))
        .add_scope(Scope::new("write".to_string()))
        // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();
    
    session.insert(CSRF_TOKEN_KEY, csrf_token).await.unwrap();
    session.insert(PKCE_VERIFIER_KEY, pkce_verifier).await.unwrap();

    Ok(Redirect::to(auth_url.as_str()))
}

async fn get_callback(State(state): State<ApplicationState>) -> Result<StatusCode, KanbanError> {
    Ok(StatusCode::OK)
}

async fn post_logout(State(state): State<ApplicationState>) -> Result<StatusCode, KanbanError> {
    Ok(StatusCode::OK)
}

#[cfg(test)]
pub mod tests {

}