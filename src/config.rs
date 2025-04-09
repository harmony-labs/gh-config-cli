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

#[derive(Debug, Serialize, Deserialize, Clone)] // Added Clone
pub struct Repo {
    pub name: String,
    pub settings: RepoSettings,
    #[serde(default)]
    pub visibility: Option<String>, // "public" or "private"
    #[serde(default)]
    pub webhook: Option<WebhookConfig>,
    #[serde(default)]
    pub branch_protections: Vec<BranchProtectionRule>,
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
    #[serde(default)]
    pub default_branch_protections: Vec<BranchProtectionRule>,
}

impl Config {
    pub fn from_file(path: &str) -> crate::error::AppResult<Self> {
        let file = std::fs::File::open(path).map_err(crate::error::AppError::Io)?;
        let config = serde_yaml::from_reader(file).map_err(crate::error::AppError::Serialization)?;
        Ok(config)
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

        let config = Config::from_file(tmpfile.path().to_str().unwrap()).expect("parse config");
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
        let config = Config::from_file(tmpfile.path().to_str().unwrap()).expect("parse config");

        // Basic assertions
        assert_eq!(config.org, "test-org");
        assert_eq!(config.repos.len(), 1);
        assert_eq!(config.teams.len(), 1);
        assert_eq!(config.users.len(), 2);
        assert_eq!(config.assignments.len(), 1);
        assert!(config.default_webhook.is_some());
    }
}