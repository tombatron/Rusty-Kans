use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: i64,
    pub login: String,
}

impl From<GitHubUser> for LoggedInUser {
    fn from(value: GitHubUser) -> Self {
        LoggedInUser {
            id: value.id,
            name: value.login,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LoggedInUser {
    pub id: i64,
    pub name: String,
}
