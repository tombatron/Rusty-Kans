use crate::models::CardMoveEvent;
use dotenvy::dotenv;
use oauth2::basic::{BasicClient, BasicErrorResponse, BasicRevocationErrorResponse, BasicTokenIntrospectionResponse, BasicTokenResponse};
use oauth2::{AuthUrl, ClientId, ClientSecret, EndpointNotSet, EndpointSet, RedirectUrl, StandardRevocableToken, TokenUrl};
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;
use std::env;
use std::str::FromStr;

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
    pub db: SqlitePool,
    pub tx: tokio::sync::broadcast::Sender<CardMoveEvent>,
    pub oauth_client: GitHubOAuthClient,
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

    let options = SqliteConnectOptions::from_str("sqlite:./kanban.db")
        .unwrap()
        .foreign_keys(true);

    let db = SqlitePool::connect_with(options).await.unwrap();

    let (tx, _) = tokio::sync::broadcast::channel::<CardMoveEvent>(512);

    ApplicationState {
        db,
        tx,
        oauth_client,
    }
}
