use serde::{Deserialize, Serialize};

use serde_yaml::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WebhookConfig {
    pub url: String,
    pub content_type: String,
    pub events: Vec<String>,
}

// Extensible settings: arbitrary key-value pairs for repo settings
pub type RepoSettings = HashMap<String, Value>;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BranchProtectionRule {
    pub pattern: String, // branch name or glob
    #[serde(default)]
    pub enforce_admins: bool,
    #[serde(default)]
    pub allow_deletions: bool,
    #[serde(default)]
    pub allow_force_pushes: bool,
}

impl Default for BranchProtectionRule {
    fn default() -> Self {
        BranchProtectionRule {
            pattern: "main".to_string(),
            enforce_admins: true,
            allow_deletions: false,
            allow_force_pushes: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Repo {
    pub name: String,
    #[serde(default)]
    pub settings: RepoSettings, // Now extensible
    #[serde(default)]
    pub visibility: Option<String>,
    #[serde(default)]
    pub webhook: Option<WebhookConfig>,
    #[serde(default)]
    pub branch_protections: Vec<BranchProtectionRule>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>, // For arbitrary fields/extensions
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
    #[serde(default)]
    pub repos: Vec<Repo>,
    #[serde(default)]
    pub teams: Vec<Team>,
    #[serde(default)]
    pub users: Vec<User>,
    #[serde(default)]
    pub assignments: Vec<Assignment>,
    #[serde(default)]
    pub default_webhook: Option<WebhookConfig>,
    #[serde(default)]
    pub default_branch_protections: Vec<BranchProtectionRule>,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>, // For extensibility and custom/policy fields
}

impl Config {
    /// Loads config from main file, optionally merging with defaults.config.yaml if present.
    pub fn from_file_with_defaults(main_path: &str, defaults_path: Option<&str>) -> crate::error::AppResult<Self> {
        let main_file = std::fs::File::open(main_path).map_err(crate::error::AppError::Io)?;
        let mut main_config: Value = serde_yaml::from_reader(main_file).map_err(crate::error::AppError::Serialization)?;

        if let Some(defaults_path) = defaults_path {
            if let Ok(defaults_file) = std::fs::File::open(defaults_path) {
                let defaults_config: Value = serde_yaml::from_reader(defaults_file).map_err(crate::error::AppError::Serialization)?;
                main_config = merge_with_defaults(main_config, defaults_config);
            }
        }

        // Deserialize the merged config into Config struct
        let config: Config = serde_yaml::from_value(main_config).map_err(crate::error::AppError::Serialization)?;
        Ok(config)
    }
}

/// Recursively merges defaults into main config (main config takes precedence).
fn merge_with_defaults(mut main: Value, defaults: Value) -> Value {
    match (main, defaults) {
        (Value::Mapping(mut main_map), Value::Mapping(defaults_map)) => {
            for (k, v) in defaults_map {
                main_map.entry(k.clone()).or_insert(v);
            }
            Value::Mapping(main_map)
        }
        (main, _) => main, // If not both mappings, main wins
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BranchProtection {
    #[serde(default)]
    pub enforce_admins: bool,
    #[serde(default)]
    pub allow_deletions: bool,
    #[serde(default)]
    pub allow_force_pushes: bool,
}

impl Default for BranchProtection {
    fn default() -> Self {
        BranchProtection {
            enforce_admins: true,
            allow_deletions: false,
            allow_force_pushes: false,
        }
    }
}

#[cfg(test)]
mod branch_protection_tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_merge_with_defaults() {
        let defaults_yaml = r#"
org: test-org
repos:
  - name: repo1
    settings:
      allow_merge_commit: false
      allow_squash_merge: true
      custom_default: 42
    custom_policy: "default"
extra_default: "foo"
"#;
        let main_yaml = r#"
org: test-org
repos:
  - name: repo1
    settings:
      allow_merge_commit: true
      allow_rebase_merge: true
    custom_policy: "main"
"#;
        let mut defaults_file = tempfile::NamedTempFile::new().expect("create temp file");
        write!(defaults_file, "{}", defaults_yaml).expect("write defaults yaml");
        let mut main_file = tempfile::NamedTempFile::new().expect("create temp file");
        write!(main_file, "{}", main_yaml).expect("write main yaml");

        let config = crate::config::Config::from_file_with_defaults(
            main_file.path().to_str().unwrap(),
            Some(defaults_file.path().to_str().unwrap())
        ).expect("parse merged config");

        // Main config takes precedence
        let repo = &config.repos[0];
        assert_eq!(repo.settings.get("allow_merge_commit").unwrap().as_bool().unwrap(), true);
        assert_eq!(repo.settings.get("allow_squash_merge").unwrap().as_bool().unwrap(), true);
        assert_eq!(repo.settings.get("allow_rebase_merge").unwrap().as_bool().unwrap(), true);
        // Custom default is filled in
        assert_eq!(repo.settings.get("custom_default").unwrap().as_i64().unwrap(), 42);
        // Custom policy: main config wins
        assert_eq!(repo.extra.get("custom_policy").unwrap().as_str().unwrap(), "main");
        // Top-level extra field from defaults
        assert_eq!(config.extra.get("extra_default").unwrap().as_str().unwrap(), "foo");
    }

    #[test]
    fn test_branch_protection_rule_deserialization() {
        let yaml = r#"
pattern: main
enforce_admins: true
allow_deletions: false
allow_force_pushes: false
"#;

        let rule: BranchProtectionRule = serde_yaml::from_str(yaml).expect("deserialize");
        assert_eq!(rule.pattern, "main");
        assert!(rule.enforce_admins);
        assert!(!rule.allow_deletions);
        assert!(!rule.allow_force_pushes);
    }

    #[test]
    fn test_config_with_branch_protections() {
        let yaml = r#"
org: test-org
default_branch_protections:
  - pattern: main
    enforce_admins: true
    allow_deletions: false
    allow_force_pushes: false
repos:
  - name: repo1
    branch_protections:
      - pattern: release/*
        enforce_admins: true
        allow_deletions: false
        allow_force_pushes: false
    settings:
      allow_merge_commit: true
      allow_squash_merge: true
      allow_rebase_merge: true
teams: []
users: []
assignments: []
"#;

        let mut tmpfile = tempfile::NamedTempFile::new().expect("create temp file");
        write!(tmpfile, "{}", yaml).expect("write yaml");

        let config = crate::config::Config::from_file_with_defaults(tmpfile.path().to_str().unwrap(), None).expect("parse config");
        assert_eq!(config.org, "test-org");
        assert_eq!(config.default_branch_protections.len(), 1);
        assert_eq!(config.repos.len(), 1);
        let repo = &config.repos[0];
        assert_eq!(repo.branch_protections.len(), 1);
        assert_eq!(repo.branch_protections[0].pattern, "release/*");
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_from_file_deserializes_valid_yaml() {
        // Sample YAML content matching Config structure
        let yaml = r#"
org: test-org
repos:
  - name: repo1
    settings:
      allow_merge_commit: true
      allow_squash_merge: false
      allow_rebase_merge: true
    visibility: public
    webhook:
      url: "http://example.com"
      content_type: "json"
      events: ["push", "pull_request"]
teams:
  - name: team1
    members: ["alice", "bob"]
users:
  - login: alice
    role: admin
  - login: bob
    role: member
assignments:
  - repo: repo1
    team: team1
    permission: admin
default_webhook:
  url: "http://default.com"
  content_type: "json"
  events: ["push"]
"#;

        // Create a temporary file with the YAML content
        let mut tmpfile = tempfile::NamedTempFile::new().expect("create temp file");
        write!(tmpfile, "{}", yaml).expect("write yaml to temp file");

        // Call from_file
        let config = crate::config::Config::from_file_with_defaults(tmpfile.path().to_str().unwrap(), None).expect("parse config");

        // Basic assertions
        assert_eq!(config.org, "test-org");
        assert_eq!(config.repos.len(), 1);
        assert_eq!(config.teams.len(), 1);
        assert_eq!(config.users.len(), 2);
        assert_eq!(config.assignments.len(), 1);
        assert!(config.default_webhook.is_some());
    }
}