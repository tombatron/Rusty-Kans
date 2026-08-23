use crate::models::CardMoveEvent;
use dotenvy::dotenv;
use oauth2::basic::{BasicClient, BasicErrorResponse, BasicRevocationErrorResponse, BasicTokenIntrospectionResponse, BasicTokenResponse};
use oauth2::{AuthUrl, ClientId, ClientSecret, EndpointNotSet, EndpointSet, RedirectUrl, StandardRevocableToken, TokenUrl};
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;
use std::env;
use std::str::FromStr;
use axum::extract::{FromRef, FromRequest, FromRequestParts, Request};
use axum::http::request::Parts;
use axum::http::StatusCode;
use dashmap::DashMap;
use tower_sessions::Session;
use crate::handlers::auth::GITHUB_USER_KEY;
use crate::models::security::GitHubUser;

pub type GitHubOAuthClient = oauth2::Client<
    BasicErrorResponse,
    BasicTokenResponse,
    BasicTokenIntrospectionResponse,
    StandardRevocableToken,
    BasicRevocationErrorResponse,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
>;

#[derive(Clone)]
pub struct ApplicationState {
    pub db_pools: DashMap<i64, SqlitePool>,
    pub tx: tokio::sync::broadcast::Sender<CardMoveEvent>,
    pub oauth_client: GitHubOAuthClient,
}

#[derive(Clone)]
pub struct UserDb(pub SqlitePool);

impl<S> FromRequestParts<S> for UserDb
where
    ApplicationState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let unauthed_response = (StatusCode::UNAUTHORIZED, "There doesn't seem to be anyone logged in...");

        let session = Session::from_request_parts(parts, state).await?;
        let state = ApplicationState::from_ref(state);

        let current_user = session.get::<GitHubUser>(GITHUB_USER_KEY)
            .await
            .map_err(|_| unauthed_response)?
            .ok_or(unauthed_response)?;

        let user_db_pool = state.db_pools.get(&(current_user.id));

        match user_db_pool {
            Some(db) => Ok(UserDb(db.clone())),
            None => {
                let options = SqliteConnectOptions::from_str(format!("sqlite:./database/{}.kanban.db", current_user.id)
                    .as_str()).unwrap()
                    .foreign_keys(true);

                let db_pool = SqlitePool::connect_with(options).await.unwrap();

                state.db_pools.insert(current_user.id, db_pool.clone());

                Ok(UserDb(db_pool))
            }
        }
    }
}

#[derive(Clone)]
pub struct ApiDb(pub SqlitePool);

impl<S> FromRequest<S> for ApiDb
where
    ApplicationState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let unauthed_response = (StatusCode::UNAUTHORIZED, "There doesn't seem to be a delegated user...");

        let state = ApplicationState::from_ref(state);
        let user_id: i64 = req.headers()
            .get("delegated-user-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.parse::<i64>().unwrap())
            .ok_or(unauthed_response)
            .map_err(|_| unauthed_response)?;

        let user_db_pool = state.db_pools.get(&user_id);

        match user_db_pool {
            Some(db) => Ok(ApiDb(db.clone())),
            None => {
                let options = SqliteConnectOptions::from_str(format!("sqlite:./database/{}.kanban.db", user_id)
                    .as_str()).unwrap()
                    .foreign_keys(true);

                let db_pool = SqlitePool::connect_with(options).await.unwrap();

                state.db_pools.insert(user_id, db_pool.clone());

                Ok(ApiDb(db_pool))
            }
        }
    }
}

pub async fn create_application_state() -> ApplicationState {
    dotenv().ok();

    // If we panic because we can't get the necessary environmental variables then so be it.
    let github_client_id = env::var("GITHUB_CLIENT_ID").unwrap();
    let github_client_secret = env::var("GITHUB_CLIENT_SECRET").unwrap();
    let github_redirect_url = RedirectUrl::new(env::var("GITHUB_REDIRECT_URL").unwrap());

    let oauth_client = BasicClient::new(ClientId::new(github_client_id))
        .set_client_secret(ClientSecret::new(github_client_secret))
        .set_auth_uri(AuthUrl::new("https://github.com/login/oauth/authorize".to_string()).unwrap())
        .set_token_uri(
            TokenUrl::new("https://github.com/login/oauth/access_token".to_string()).unwrap(),
        )
        .set_redirect_uri(github_redirect_url.unwrap());

    // let options = SqliteConnectOptions::from_str("sqlite:./kanban.db")
    //     .unwrap()
    //     .foreign_keys(true);

    //let db = SqlitePool::connect_with(options).await.unwrap();
    let db_pools = DashMap::new();

    let (tx, _) = tokio::sync::broadcast::channel::<CardMoveEvent>(512);

    ApplicationState {
        db_pools,
        tx,
        oauth_client,
    }
}
