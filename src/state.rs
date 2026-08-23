use crate::handlers::auth::GITHUB_USER_KEY;
use crate::models::CardMoveEvent;
use crate::models::security::GitHubUser;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode};
use dashmap::DashMap;
use dotenvy::dotenv;
use oauth2::basic::{BasicClient, BasicErrorResponse, BasicRevocationErrorResponse, BasicTokenIntrospectionResponse, BasicTokenResponse};
use oauth2::{AuthUrl, ClientId, ClientSecret, EndpointNotSet, EndpointSet, RedirectUrl, StandardRevocableToken, TokenUrl};
use sqlx::{AssertSqlSafe, SqlitePool};
use sqlx::sqlite::SqliteConnectOptions;
use std::{env, fs};
use std::str::FromStr;
use std::sync::Arc;
use tower_sessions::Session;
use tower_sessions_redis_store::fred::prelude::*;

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
    pub db_pools: Arc<DashMap<i64, SqlitePool>>,
    pub tx: tokio::sync::broadcast::Sender<CardMoveEvent>,
    pub oauth_client: GitHubOAuthClient,
    pub redis_pool: Option<Pool>
}

async fn get_or_create_pool(db_pools: &Arc<DashMap<i64, SqlitePool>>, user_id: i64) -> SqlitePool {
    if let Some(pool) = db_pools.get(&user_id) {
        return pool.clone();
    }

    let db_pool = get_database(user_id).await;
    db_pools.insert(user_id, db_pool.clone());

    db_pool
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

        let db_pool = get_or_create_pool(&state.db_pools, current_user.id).await;

        Ok(UserDb(db_pool))
    }
}

#[derive(Clone)]
pub struct ApiDb(pub SqlitePool);

impl<S> FromRequestParts<S> for ApiDb
where
    ApplicationState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let unauthed_response = (StatusCode::UNAUTHORIZED, "There doesn't seem to be a delegated user...");

        let headers = HeaderMap::from_request_parts(parts, state).await.map_err(|_| unauthed_response)?;
        let state = ApplicationState::from_ref(state);

        let user_id = headers
            .get("delegated-user-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.parse::<i64>().unwrap())
            .ok_or(unauthed_response)
            .map_err(|_| unauthed_response)?;

        let user_db_pool = get_or_create_pool(&state.db_pools, user_id).await;

        Ok(ApiDb(user_db_pool))
    }
}

async fn get_database(user_id: i64) -> SqlitePool {
    let options = SqliteConnectOptions::from_str(format!("sqlite:./databases/{}.kanban.db", user_id)
        .as_str()).unwrap()
        .foreign_keys(true)
        .create_if_missing(true);

    let db_pool = SqlitePool::connect_with(options).await.unwrap();

    sqlx::migrate!().run(&db_pool).await.unwrap();

    // If the user id is i64::MIN then we'll assume this is a test user and seed the database.
    if user_id == i64::MIN {
        // First check to see if there are any boards...
        let count = sqlx::query_scalar::<_, u64>("SELECT COUNT(*) FROM boards;").fetch_one(&db_pool).await.unwrap();

        // No boards? No data. Let's seed it.
        if count == 0 {
            let seed_script = fs::read_to_string("./src/fixtures/boards.sql").unwrap();

            let _ = sqlx::query(AssertSqlSafe(seed_script)).execute(&db_pool).await;
        }
    }

    db_pool
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

    let db_pools = Arc::new(DashMap::new());

    let (tx, _) = tokio::sync::broadcast::channel::<CardMoveEvent>(512);

    let redis_connection_string = env::var("REDIS_CONNECTION_STRING");

    let redis_pool: Option<Pool> = match redis_connection_string {
        Ok(conn_string) => {
            let config = Config::from_url(conn_string.as_str()).unwrap();
            let pool = Pool::new(config, None, None, None, 6).unwrap();
            pool.connect();
            pool.wait_for_connect().await.unwrap();

            Some(pool)
        },
        Err(_) => None
    };

    ApplicationState {
        db_pools,
        tx,
        oauth_client,
        redis_pool
    }
}
