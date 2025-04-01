use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WebhookConfig {
    pub url: String,
    pub content_type: String,
    pub events: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RepoSettings {
    pub allow_merge_commit: bool,
    pub allow_squash_merge: bool,
    pub allow_rebase_merge: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)] // Added Clone
pub struct Repo {
    pub name: String,
    pub settings: RepoSettings,
    #[serde(default)]
    pub visibility: Option<String>, // "public" or "private"
    #[serde(default)]
    pub webhook: Option<WebhookConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)] // Added Clone
pub struct Team {
    pub name: String,
    pub members: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)] // Added Clone
pub struct User {
    pub login: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)] // Added Clone
pub struct Assignment {
    pub repo: String,
    pub team: String,
    pub permission: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub org: String,
    pub repos: Vec<Repo>,
    pub teams: Vec<Team>,
    pub users: Vec<User>,
    pub assignments: Vec<Assignment>,
    #[serde(default)]
    pub default_webhook: Option<WebhookConfig>,
}

impl Config {
    pub fn from_file(path: &str) -> crate::error::AppResult<Self> {
        let file = std::fs::File::open(path).map_err(crate::error::AppError::Io)?;
        let config = serde_yaml::from_reader(file).map_err(crate::error::AppError::Serialization)?;
        Ok(config)
    }
}