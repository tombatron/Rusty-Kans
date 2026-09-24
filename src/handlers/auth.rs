use crate::data;
use crate::errors::KanbanError;
#[cfg(debug_assertions)]
use crate::models::User;
use crate::models::security::{GitHubUser, LoggedInUser};
use crate::state::{ApplicationState, get_or_create_pool};
#[cfg(debug_assertions)]
use crate::state::CsrfTokenValue;
use askama::Template;
use axum::extract::{Query, State};
use axum::response::{Html, Redirect};
use axum::routing::{get, post};
#[cfg(debug_assertions)]
use axum::Form;
use axum::Router;
use oauth2::{
    AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse,
};
use serde::Deserialize;
use tower_sessions::Session;
use crate::middleware::require_csrf_token;

const CSRF_TOKEN_KEY: &str = "CSRF_TOKEN";
const PKCE_VERIFIER_KEY: &str = "PKCE_VERIFIER";
pub const AUTHENTICATED_USER_KEY: &str = "AUTHENTICATED_USER";
const GITHUB_USER_API_URL: &str = "https://api.github.com/user";

pub struct AuthSources;

impl AuthSources {
    pub const GITHUB: &'static str = "github";
    #[allow(dead_code)] // only referenced from the debug-only dev auth path and from tests
    pub const DEV: &'static str = "dev";
}

pub fn get_router_configuration() -> Router<ApplicationState> {
    let router = Router::new()
        .route("/auth", get(get_auth_landing))
        .route("/auth/login", get(get_login))
        .route("/auth/callback", get(get_callback))
        .route("/auth/logout", post(post_logout))
        .route_layer(axum::middleware::from_fn(require_csrf_token));

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
    session.save().await?;

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
    let stored_csrf: Option<CsrfToken> = session.get(CSRF_TOKEN_KEY).await?;
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
            .await?
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

    let db_pool = get_or_create_pool(&state.db_pools, github_user.id, AuthSources::GITHUB.to_string()).await?;

    let _ = data::upsert_user(db_pool, github_user.clone().into()).await?;

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
struct AuthDevTemplate {
    csrf_token: String,
}

#[cfg(debug_assertions)]
async fn get_auth_dev(CsrfTokenValue(csrf_token): CsrfTokenValue) -> Result<Html<String>, KanbanError> {
    let template = AuthDevTemplate { csrf_token };
    Ok(Html(template.render()?))
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone, Deserialize)]
struct AuthDevForm {
    id: i64,
    name: String,
}

#[cfg(debug_assertions)]
impl From<AuthDevForm> for LoggedInUser {
    fn from(value: AuthDevForm) -> Self {
        LoggedInUser {
            id: value.id,
            name: value.name,
            source: AuthSources::DEV.to_string(),
        }
    }
}

#[cfg(debug_assertions)]
impl From<AuthDevForm> for User {
    fn from(value: AuthDevForm) -> Self {
        User {
            user_id: value.id,
            source: AuthSources::DEV.to_string(),
            oauth_login: format!("dev--{}", value.id),
            display_name: None, // For now...
            avatar_url: None, // For now...
        }
    }
}

#[cfg(debug_assertions)]
async fn post_auth_dev(
    State(state): State<ApplicationState>,
    session: Session,
    Form(auth_dev): Form<AuthDevForm>,
) -> Result<Redirect, KanbanError> {
    session
        .insert(AUTHENTICATED_USER_KEY, LoggedInUser::from(auth_dev.clone()))
        .await?;

    let db = get_or_create_pool(&state.db_pools, auth_dev.id, AuthSources::DEV.to_string()).await?;

    let _ = data::upsert_user(db, auth_dev.into()).await?;

    Ok(Redirect::to("/"))
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;
    #[cfg(debug_assertions)]
    use crate::data;
    use crate::handlers::tests::get_fake_application_state;
    use crate::router::create_router_with_session;
    #[cfg(debug_assertions)]
    use axum::Form;
    #[cfg(debug_assertions)]
    use axum::extract::State;
    use axum_test::TestServer;
    #[cfg(debug_assertions)]
    use sqlx::SqlitePool;
    #[cfg(debug_assertions)]
    use tower_sessions::session::Id;
    use tower_sessions::{MemoryStore, Session, SessionStore};
    use crate::csrf::{get_or_create_secret, mask};
    use crate::handlers::auth::AUTHENTICATED_USER_KEY;
    #[cfg(debug_assertions)]
    use crate::handlers::auth::AuthSources;
    #[cfg(debug_assertions)]
    use crate::handlers::auth::post_auth_dev;

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
            AUTHENTICATED_USER_KEY.to_string(),
            serde_json::json!({ "id": 1, "login": "testuser" }),
        );
        store.save(&record).await.unwrap();

        let session = Session::new(Some(session_id), Arc::new(store.clone()), None);

        let secret = get_or_create_secret(&session).await.unwrap();

        let _save_result = session.save().await;

        let token = mask(&secret);

        let mut form: Vec<(String, String)> = vec!();
        
        form.push(("csrf_token".to_string(), token));

        // Logout - axum-test carries the cookie automatically.
        server.post("/auth/logout", )
            .form(&form).await;

        // Verify the session record is gone
        let result = store.load(&session_id).await.unwrap();

        assert!(result.is_none());
    }

    #[sqlx::test(fixtures(path = "../fixtures", scripts("boards")))]
    #[cfg(debug_assertions)]
    async fn post_auth_dev_persists_new_user_record(db_pool: SqlitePool) -> sqlx::Result<()> {
        use crate::{handlers::auth::AuthDevForm, state::get_database_id};

        let state = get_fake_application_state();

        state.db_pools.insert(get_database_id(-1000, AuthSources::DEV.to_string()), db_pool.clone());

        let session_store = MemoryStore::default();
        let session = Session::new(Some(Id(67)), Arc::new(session_store.clone()), None);

        let auth_dev_form = AuthDevForm {
            id: -1000,
            name: "your face".to_string(),
        };

        let result = post_auth_dev(State(state), session, Form(auth_dev_form)).await.unwrap();

        assert_eq!("/", result.location());

        let persisted_user = data::get_user(db_pool, -1000, AuthSources::DEV.to_string()).await.unwrap().unwrap();

        assert_eq!(-1000, persisted_user.user_id);
        assert_eq!("dev", persisted_user.source);
        assert_eq!("dev---1000", persisted_user.oauth_login);
        assert!(persisted_user.display_name.is_none());
        assert!(persisted_user.avatar_url.is_none());

        Ok(())
    }
}
