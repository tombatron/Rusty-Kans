use serde::{Deserialize, Serialize};

use crate::handlers::auth::AuthSources;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: i64,
    pub login: String,
}

impl From<GitHubUser> for LoggedInUser {
    fn from(value: GitHubUser) -> Self {
        LoggedInUser {
            id: value.id,
            name: value.login,
            source: AuthSources::GITHUB.to_string()
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LoggedInUser {
    pub id: i64,
    pub name: String,
    pub source: String,
}
