use crate::handlers::auth::AUTHENTICATED_USER_KEY;
use crate::handlers::ws::SocketEvents;
use crate::models::security::LoggedInUser;
use axum::Extension;
use axum::extract::{FromRef, FromRequestParts, Path};
use axum::http::request::Parts;
use axum::http::StatusCode;
use dashmap::DashMap;
use dotenvy::dotenv;
use oauth2::basic::{BasicClient, BasicErrorResponse, BasicRevocationErrorResponse, BasicTokenIntrospectionResponse, BasicTokenResponse};
use oauth2::{AuthUrl, ClientId, ClientSecret, EndpointNotSet, EndpointSet, RedirectUrl, StandardRevocableToken, TokenUrl};
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{AssertSqlSafe, SqlitePool};
use std::str::FromStr;
use std::sync::Arc;
use std::{env, fs};
use std::path::PathBuf;
use sha2::{Digest, Sha256};
use tower_sessions::Session;
use tower_sessions_redis_store::fred::prelude::{ClientLike, Config, Pool};
use crate::csrf;
use crate::csrf::get_or_create_secret;
use crate::errors::KanbanError;

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

pub type SocketEventsSender = tokio::sync::broadcast::Sender<SocketEvents>;

#[derive(Clone)]
pub struct ApplicationState {
    pub db_pools: Arc<DashMap<String, SqlitePool>>,
    pub tx_pool: Arc<DashMap<String, SocketEventsSender>>,
    pub oauth_client: GitHubOAuthClient,
    pub redis_pool: Option<Pool>,
    pub base_url: String,
}

pub async fn get_or_create_pool(db_pools: &Arc<DashMap<String, SqlitePool>>, user_id: i64, source: String) -> Result<SqlitePool, KanbanError> {
    let database_id = get_database_id(user_id, source);

    Ok(get_or_create_pool_with_id(&db_pools, database_id).await?)
}

async fn get_or_create_pool_with_id(db_pools: &Arc<DashMap<String, SqlitePool>>, database_id: String) -> Result<SqlitePool, KanbanError> {
    if let Some(pool) = db_pools.get(&database_id) {
        return Ok(pool.clone());
    }

    let db_pool = get_database(database_id.clone(), true).await?;
    db_pools.insert(database_id, db_pool.clone());

    Ok(db_pool)
}

pub async fn get_pool_with_id(db_pools: &Arc<DashMap<String, SqlitePool>>, database_id: String) -> Option<SqlitePool> {
    if let Some(pool) = db_pools.get(&database_id) {
        return Some(pool.clone());
    }

    let db_pool = get_database(database_id.clone(), false).await.ok();

    if let Some(pool) = db_pool {
        db_pools.insert(database_id, pool.clone());
        return Some(pool);
    }

    None
}

pub fn get_database_id(user_id: i64, source: String) -> String {
    let unhashed_result = format!("{source}:{user_id}");

    if source.starts_with("dev") {
        return format!("dev-{:x}", Sha256::digest(unhashed_result))
    }

    format!("{:x}", Sha256::digest(unhashed_result))
}

pub fn get_database_path(database_id: &String) -> PathBuf { PathBuf::from(format!("./databases/{}.kanban.db", database_id)) }

#[derive(Clone)]
pub struct UserDb(pub SqlitePool);

impl<S> FromRequestParts<S> for UserDb
where
    ApplicationState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let unauthed_response = || (StatusCode::UNAUTHORIZED, "There doesn't seem to be anyone logged in...".to_string());

        let session = Session::from_request_parts(parts, state).await
            .map_err(|_| unauthed_response())?;

        let state = ApplicationState::from_ref(state);

        let current_user = session.get::<LoggedInUser>(AUTHENTICATED_USER_KEY)
            .await
            .map_err(|_| unauthed_response())?
            .ok_or(unauthed_response())?;

        let db_pool = get_or_create_pool(&state.db_pools, current_user.id, current_user.source).await;

        Ok(UserDb(db_pool?))
    }
}

#[derive(Clone)]
pub struct ApiDb(pub SqlitePool);

impl<S> FromRequestParts<S> for ApiDb
where
    ApplicationState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // When you forget what's happening here... We've already resolved the database in the middleware and stored it in an
        // extension, we're just retrieving it here. 
        let Extension(delegated_sqlpool) = Extension::<SqlitePool>::from_request_parts(parts, state)
            .await
            .map_err(|_| (StatusCode::BAD_REQUEST, "No way partner, I didn't see a delegated database...".to_string()))?;

        Ok(ApiDb(delegated_sqlpool))
    }
}

#[derive(serde::Deserialize)]
struct BackendPathParams {
    owner_database_id: String,
}

#[derive(Clone)]
pub struct BackendDb(pub SqlitePool);

impl<S> FromRequestParts<S> for BackendDb
where 
    ApplicationState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Path(params) = Path::<BackendPathParams>::from_request_parts(parts, state)
            .await
            .map_err(|_| (StatusCode::BAD_REQUEST, "Request was bad yo.".to_string()))?;

        let state = ApplicationState::from_ref(state);

        let pool = get_pool_with_id(&state.db_pools, params.owner_database_id)
            .await
            .ok_or((StatusCode::BAD_REQUEST, "Not sure where you got that ID, but I don't know what to do with it.".to_string()))?;

        Ok(BackendDb(pool))
    }
}

async fn get_database(database_id: String, create_if_missing: bool) -> Result<SqlitePool, KanbanError> {
    let database_path = get_database_path(&database_id);

    if !create_if_missing && !database_path.exists() {
        return Err(KanbanError::DatabaseNotFound(format!("Couldn't find a database file for {database_id}.")));
    }

    let connection_string = format!("sqlite:{}", database_path.to_str().unwrap());
    let options = SqliteConnectOptions::from_str(connection_string.as_str())?
        .foreign_keys(true)
        .create_if_missing(create_if_missing);

    let db_pool = SqlitePool::connect_with(options).await?;

    sqlx::migrate!().run(&db_pool).await?;

    // If the database ID starts with "dev" we'll assume we might have to seed the database.
    if database_id.starts_with("dev") {
        // First check to see if there are any boards...
        let count = sqlx::query_scalar::<_, u64>("SELECT COUNT(*) FROM boards;").fetch_one(&db_pool).await?;

        // No boards? No data. Let's seed it.
        if count == 0 {
            let seed_script = fs::read_to_string("./src/fixtures/boards.sql")?;

            let _ = sqlx::query(AssertSqlSafe(seed_script)).execute(&db_pool).await;
        }
    }

    Ok(db_pool)
}

async fn create_redis_pool(redis_connection_string: String) -> Result<Pool, KanbanError> {
    let config = Config::from_url(redis_connection_string.as_str())?;
    let pool = Pool::new(config, None, None, None, 6)?;

    pool.connect();
    pool.wait_for_connect().await?;

    Ok(pool)
}

pub async fn create_application_state() -> ApplicationState {
    dotenv().ok();

    // If we panic because we can't get the necessary environmental variables then so be it.
    let github_client_id = env::var("GITHUB_CLIENT_ID").unwrap();
    let github_client_secret = env::var("GITHUB_CLIENT_SECRET").unwrap();
    let github_redirect_url = RedirectUrl::new(env::var("GITHUB_REDIRECT_URL").unwrap());
    let base_url = env::var("APP_BASE_URL").unwrap().trim_matches('/').to_string();

    let oauth_client = BasicClient::new(ClientId::new(github_client_id))
        .set_client_secret(ClientSecret::new(github_client_secret))
        .set_auth_uri(AuthUrl::new("https://github.com/login/oauth/authorize".to_string()).unwrap())
        .set_token_uri(
            TokenUrl::new("https://github.com/login/oauth/access_token".to_string()).unwrap(),
        )
        .set_redirect_uri(github_redirect_url.unwrap());

    let db_pools = Arc::new(DashMap::new());

    let tx_pool = Arc::new(DashMap::new());

    let redis_connection_string = env::var("REDIS_CONNECTION_STRING");

    let redis_pool: Option<Pool> = match redis_connection_string {
        Ok(conn_string) => {
            create_redis_pool(conn_string).await.ok()
        },
        Err(_) => None
    };

    ApplicationState {
        db_pools,
        tx_pool,
        oauth_client,
        redis_pool,
        base_url,
    }
}

#[derive(Clone)]
pub struct CsrfTokenValue(pub String);

impl<S> FromRequestParts<S> for CsrfTokenValue where S: Send + Sync {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state).await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "No session available for this request.".to_string()))?;

        let csrf_secret = get_or_create_secret(&session).await?;
        
        let masked_secret = csrf::mask(&csrf_secret);

        Ok(CsrfTokenValue(masked_secret))
    }
}

fn get_or_create_ws_sender(tx_pool: &Arc<DashMap<String, SocketEventsSender>>, user_id: i64, source: String) -> SocketEventsSender {
    let tx_instance_id = get_database_id(user_id, source);

    if let Some(tx) = tx_pool.get(&tx_instance_id) {
        return tx.clone();
    }

    let tx = SocketEventsSender::new(512);

    tx_pool.insert(tx_instance_id, tx.clone());

    tx
}

#[derive(Clone)]
pub struct UserBroadcast(pub SocketEventsSender);

impl<S> FromRequestParts<S> for UserBroadcast 
where 
    ApplicationState: FromRef<S>, 
    S: Send + Sync, 
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let unauthed_response = || (StatusCode::UNAUTHORIZED, "There doesn't seem to be anyone logged in...".to_string());

        let session = Session::from_request_parts(parts, state).await.map_err(|_| unauthed_response())?;

        let state = ApplicationState::from_ref(state);

        let current_user = session.get::<LoggedInUser>(AUTHENTICATED_USER_KEY).await.map_err(|_| unauthed_response())?.ok_or(unauthed_response())?;

        let tx = get_or_create_ws_sender(&state.tx_pool, current_user.id, current_user.source);

        Ok(UserBroadcast(tx))
    }
}


#[cfg(test)]
pub mod tests  {
    use crate::{errors::KanbanError, state::{get_database, get_database_id}};

    #[test]
    fn get_database_id_returns_hashed_value() {
        let result_database_id = get_database_id(123, "whatever".to_string());

        assert_eq!("a032aacdf2aaa5fb2dd0903d2123ae0c06a7a095a86a8834125cc706254bfa40", result_database_id);
    }

    #[tokio::test]
    async fn get_database_returns_error_if_not_found_when_create_if_not_exists_is_false() {
        let database_result = get_database("this_doesnt_exist".to_string(), false).await;

        assert!(matches!(database_result, Err(KanbanError::DatabaseNotFound(_))));
    }
}

