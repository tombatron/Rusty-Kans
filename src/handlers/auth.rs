use crate::errors::KanbanError;
use crate::models::security::{GitHubUser, LoggedInUser};
use crate::state::ApplicationState;
use askama::Template;
use axum::extract::{Query, State};
use axum::response::{Html, Redirect};
use axum::routing::{get, post};
use axum::{Form, Router};
use oauth2::{
    AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse,
};
use serde::Deserialize;
use tower_sessions::Session;

const CSRF_TOKEN_KEY: &str = "CSRF_TOKEN";
const PKCE_VERIFIER_KEY: &str = "PKCE_VERIFIER";
pub const AUTHENTICATED_USER_KEY: &str = "AUTHENTICATED_USER";
const GITHUB_USER_API_URL: &str = "https://api.github.com/user";

pub fn get_router_configuration() -> Router<ApplicationState> {
    let router = Router::new()
        .route("/auth", get(get_auth_landing))
        .route("/auth/login", get(get_login))
        .route("/auth/callback", get(get_callback))
        .route("/auth/logout", post(post_logout));

    #[cfg(debug_assertions)]
    let router = router
        .route("/auth/dev", get(get_auth_dev))
        .route("/auth/dev", post(post_auth_dev));

    router
}

#[derive(Template, Debug)]
#[template(path = "auth.html")]
struct AuthTemplate {
    dev: bool
}

async fn get_auth_landing() -> Result<Html<String>, KanbanError> {
    let template = AuthTemplate {
        dev: cfg!(debug_assertions)
    };
    Ok(Html(template.render()?))
}

async fn get_login(
    State(state): State<ApplicationState>,
    session: Session,
) -> Result<Redirect, KanbanError> {
    // Generate PKCE challenge.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let (auth_url, csrf_token) = state
        .oauth_client
        .authorize_url(CsrfToken::new_random)
        // Set the desired scopes.
        .add_scope(Scope::new("read:user".to_string()))
        // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();

    session.insert(CSRF_TOKEN_KEY, csrf_token.secret()).await?;
    session.insert(PKCE_VERIFIER_KEY, pkce_verifier.into_secret()).await?;

    Ok(Redirect::to(auth_url.as_str()))
}

#[derive(Deserialize)]
struct CallbackParams {
    code: String,
    state: String,
}

async fn get_callback(
    State(state): State<ApplicationState>,
    session: Session,
    Query(params): Query<CallbackParams>,
) -> Result<Redirect, KanbanError> {
    let stored_csrf: Option<CsrfToken> = session.get(CSRF_TOKEN_KEY).await.unwrap();
    let stored_csrf =
        stored_csrf.ok_or(KanbanError::RequestError("Missing CSRF token.".to_string()))?;

    if stored_csrf.secret() != &params.state {
        return Err(KanbanError::RequestError(
            "CSRF token mismatch.".to_string(),
        ));
    }

    let http_client = oauth2::reqwest::ClientBuilder::new()
        .redirect(oauth2::reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build.");

    let pkce_verifier_secret =
        session
            .get(PKCE_VERIFIER_KEY)
            .await
            .unwrap()
            .ok_or(KanbanError::RequestError(
                "Missing PKCE verifier.".to_string(),
            ))?;

    let pkce_verifier = PkceCodeVerifier::new(pkce_verifier_secret);

    let token_result = state
        .oauth_client
        .exchange_code(AuthorizationCode::new(params.code))
        .set_pkce_verifier(pkce_verifier)
        .request_async(&http_client)
        .await
        .map_err(|e| KanbanError::RequestError(e.to_string()))?;

    let github_user = reqwest::Client::new()
        .get(GITHUB_USER_API_URL)
        .header(
            "Authorization",
            format!("Bearer {}", token_result.access_token().secret()),
        )
        .header("User-agent", "kanban-app")
        .send()
        .await
        .map_err(|e| KanbanError::RequestError(e.to_string()))?
        .json::<GitHubUser>()
        .await
        .map_err(|e| KanbanError::RequestError(e.to_string()))?;

    session
        .insert(AUTHENTICATED_USER_KEY, LoggedInUser::from(github_user))
        .await?;

    Ok(Redirect::to("/"))
}

async fn post_logout(session: Session) -> Result<Redirect, KanbanError> {
    session.delete().await?;
    Ok(Redirect::to("/"))
}

#[cfg(debug_assertions)]
#[derive(Template, Debug)]
#[template(path = "auth_dev.html")]
struct AuthDevTemplate {}

#[cfg(debug_assertions)]
async fn get_auth_dev() -> Result<Html<String>, KanbanError> {
    let template = AuthDevTemplate {};
    Ok(Html(template.render()?))
}

#[cfg(debug_assertions)]
#[derive(Deserialize)]
struct AuthDevForm {
    id: i64,
    name: String,
}

impl From<AuthDevForm> for LoggedInUser {
    fn from(value: AuthDevForm) -> Self {
        LoggedInUser {
            id: value.id,
            name: value.name,
            source: "dev".to_string(),
        }
    }
}

#[cfg(debug_assertions)]
async fn post_auth_dev(
    session: Session,
    Form(auth_dev): Form<AuthDevForm>,
) -> Result<Redirect, KanbanError> {
    session
        .insert(AUTHENTICATED_USER_KEY, LoggedInUser::from(auth_dev))
        .await?;

    Ok(Redirect::to("/"))
}

#[cfg(test)]
pub mod tests {
    use crate::handlers::tests::get_fake_application_state;
    use crate::router::create_router_with_session;
    use axum_test::TestServer;
    use tower_sessions::{MemoryStore, SessionStore};

    #[tokio::test]
    async fn post_logout_deletes_sessions() {
        let store = MemoryStore::default();
        let state = get_fake_application_state();

        let server = TestServer::builder()
            .save_cookies()
            .build(create_router_with_session(state, store.clone()));

        // Establish a session...
        let response = server.get("/auth/login").await;

        // Pull the session ID out of the cookie.
        let cookie = response.cookie("id");
        let session_id = cookie.value().parse().unwrap();

        // Load the record and insert user data directly into the store.
        let mut record = store.load(&session_id).await.unwrap().unwrap();
        record.data.insert(
            "GITHUB_USER".to_string(),
            serde_json::json!({ "id": 1, "login": "testuser" }),
        );
        store.save(&record).await.unwrap();

        // Logout - axum-test carries the cookie automatically.
        server.post("/auth/logout").await;

        // Verify the session record is gone
        let result = store.load(&session_id).await.unwrap();

        assert!(result.is_none());
    }
}
