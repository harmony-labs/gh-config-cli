///
/// Configuration models for gh-config-cli.
///
/// This module defines the data structures used for representing repository, team, user, and webhook
/// configuration. All structs are serializable/deserializable for use with YAML and JSON configuration files.
///

use serde::{Deserialize, Serialize};

use serde_yaml::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
///
/// Configuration for a GitHub webhook.
///
/// Represents the configuration for a GitHub webhook, including the endpoint URL,
/// content type, and the list of events that trigger the webhook.
///
pub struct WebhookConfig {
    /// The webhook endpoint URL.
    pub url: String,
    /// The content type for webhook payloads (e.g., "json").
    pub content_type: String,
    /// List of events that trigger the webhook.
    pub events: Vec<String>,
}

// Extensible settings: arbitrary key-value pairs for repo settings
/// Arbitrary key-value pairs for repository settings (extensible).
pub type RepoSettings = HashMap<String, Value>;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
///
/// Branch protection rule for a repository.
///
/// Represents a single branch protection rule, including the branch pattern and
/// enforcement options for admins, deletions, and force pushes.
///
pub struct BranchProtectionRule {
    /// Branch name or glob pattern to match.
    pub pattern: String,
    /// Whether to enforce admin restrictions.
    #[serde(default)]
    pub enforce_admins: bool,
    /// Whether to allow branch deletions.
    #[serde(default)]
    pub allow_deletions: bool,
    /// Whether to allow force pushes.
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
///
/// Repository configuration.
///
/// Represents the configuration for a single repository, including its name,
/// settings, visibility, webhook, branch protections, and any extra fields.
///
pub struct Repo {
    /// Name of the repository.
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
///
/// Represents a team within the organization.
///
/// Each team has a name and a list of member usernames.
///
pub struct Team {
    /// Name of the team.
    pub name: String,
    /// List of usernames belonging to the team.
    pub members: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)] // Added Clone
///
/// Represents a user within the organization.
///
/// Each user has a login (username) and a role (e.g., admin, member).
///
pub struct User {
    /// The user's GitHub login/username.
    pub login: String,
    /// The user's role within the organization.
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)] // Added Clone
///
/// Represents a team assignment to a repository with a specific permission level.
///
/// Each assignment links a team to a repository and specifies the permission granted.
///
pub struct Assignment {
    /// The name of the repository.
    pub repo: String,
    /// The name of the team.
    pub team: String,
    /// The permission level (e.g., admin, write, read).
    pub permission: String,
}

#[derive(Debug, Serialize, Deserialize)]
///
/// Top-level configuration for gh-config-cli.
///
/// This struct represents the full configuration for an organization, including
/// repositories, teams, users, assignments, default webhook, and branch protections.
///
pub struct Config {
    /// The name of the GitHub organization.
    pub org: String,
    /// List of repository configurations.
    #[serde(default)]
    pub repos: Vec<Repo>,
    /// List of team configurations.
    #[serde(default)]
    pub teams: Vec<Team>,
    /// List of user configurations.
    #[serde(default)]
    pub users: Vec<User>,
    /// List of team-to-repo assignments.
    #[serde(default)]
    pub assignments: Vec<Assignment>,
    /// Default webhook configuration for all repositories (if not overridden).
    #[serde(default)]
    pub default_webhook: Option<WebhookConfig>,
    /// Default branch protection rules for all repositories (if not overridden).
    #[serde(default)]
    pub default_branch_protections: Vec<BranchProtectionRule>,
    /// Extra fields for extensibility and custom/policy fields.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
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
fn merge_with_defaults(main: Value, defaults: Value) -> Value {
    match (main, defaults) {
        (Value::Mapping(mut main_map), Value::Mapping(defaults_map)) => {
            for (k, v_default) in defaults_map {
                if let Some(v_main) = main_map.remove(&k) {
                    // Special case: if v_main is an empty sequence and v_default is a sequence,
                    // use v_default directly
                    let merged_value = match (v_main, &v_default) {
                        (Value::Sequence(seq), Value::Sequence(default_seq)) if seq.is_empty() && !default_seq.is_empty() => {
                            v_default.clone()
                        }
                        (v_main, _) => {
                            merge_with_defaults(v_main, v_default.clone())
                        }
                    };
                    main_map.insert(k, merged_value);
                } else {
                    main_map.insert(k, v_default);
                }
            }
            Value::Mapping(main_map)
        }
        (Value::Sequence(main_seq), Value::Sequence(defaults_seq)) => {
            // If main_seq is empty, use defaults_seq directly
            if main_seq.is_empty() {
                return Value::Sequence(defaults_seq);
            }
            // Merge lists of mappings by "name" key (for repos, teams, etc.)
            let mut merged = vec![];
            let mut used = vec![false; main_seq.len()];
            for d in &defaults_seq {
                if let Value::Mapping(d_map) = d {
                    if let Some(Value::String(d_name)) = d_map.get(&Value::String("name".to_string())) {
                        // Try to find a matching item in main_seq
                        let mut found = false;
                        for (i, m) in main_seq.iter().enumerate() {
                            if let Value::Mapping(m_map) = m {
                                if let Some(Value::String(m_name)) = m_map.get(&Value::String("name".to_string())) {
                                    if m_name == d_name {
                                        // Merge recursively
                                        merged.push(merge_with_defaults(m.clone(), d.clone()));
                                        used[i] = true;
                                        found = true;
                                        break;
                                    }
                                }
                            }
                        }
                        if !found {
                            merged.push(d.clone());
                        }
                    } else {
                        merged.push(d.clone());
                    }
                } else {
                    merged.push(d.clone());
                }
            }
            // Add all main_seq items that were not matched
            for (i, m) in main_seq.into_iter().enumerate() {
                if !used[i] {
                    merged.push(m);
                }
            }
            Value::Sequence(merged)
        }
        (main, _) => main, // If not both mappings or sequences, main wins
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
///
/// Branch protection settings for a repository.
///
/// Represents enforcement options for admins, deletions, and force pushes for a branch.
///
pub struct BranchProtection {
    /// Whether to enforce admin restrictions.
    #[serde(default)]
    pub enforce_admins: bool,
    /// Whether to allow branch deletions.
    #[serde(default)]
    pub allow_deletions: bool,
    /// Whether to allow force pushes.
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
        println!("config.repos.len() = {}", config.repos.len());
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