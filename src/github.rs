/*!
    GitHub API client module.

    This module provides the `GitHubClient` struct and related types for interacting with the GitHub REST API.
    It supports operations such as updating repository settings, managing teams, users, and webhooks, and
    generating configuration from a GitHub organization. All API interactions are authenticated and support
    dry-run mode for previewing changes.

    Public APIs are documented for maintainability. Internal response structs are used for deserialization.
*/

use crate::config::{Assignment, Repo, RepoSettings, Team, User, WebhookConfig, Config};
use crate::api_mapping_generated::get_github_api_mapping;
use crate::error::{AppError, AppResult};
use colored::*;
use log::{debug, info, error};
use reqwest::Client;
use serde_json::json;
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use std::fs::File;
use std::io::Write;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RepoResponse {
    allow_merge_commit: bool,
    allow_squash_merge: bool,
    allow_rebase_merge: bool,
    private: bool,
}

#[derive(Debug, Deserialize)]
struct TeamResponse {
    name: String,
}

#[derive(Debug, Deserialize)]
struct MembershipResponse {
    role: String,
}

#[derive(Debug, Deserialize)]
struct TeamRepoResponse {
    name: String,
    permissions: PermissionDetails,
}

#[derive(Debug, Deserialize)]
struct PermissionDetails {
    pull: bool,
    push: bool,
    admin: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WebhookResponse {
    id: Option<i64>,
    url: String,
    config: WebhookConfigResponse,
    events: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WebhookConfigResponse {
    url: String,
    content_type: String,
}

/**
 * GitHub API client for organization-level operations.
 *
 * This struct provides methods to interact with the GitHub REST API, including repository, team,
 * user, and webhook management. All requests are authenticated using a personal access token.
 */
pub struct GitHubClient {
    /// Reqwest HTTP client for making API requests.
    client: Client,
    /// GitHub personal access token for authentication.
    token: String,
    /// Name of the GitHub organization to operate on.
    pub org: String,
}

impl GitHubClient {
    /// Create a new GitHubClient for the given organization and token.
    ///
    /// # Arguments
    /// * `token` - GitHub personal access token.
    /// * `org` - Name of the GitHub organization.
    pub fn new(token: &str, org: &str) -> Self {
        GitHubClient {
            client: Client::new(),
            token: token.to_string(),
            org: org.to_string(),
        }
    }

    async fn send_patch(&self, url: &str, body: serde_json::Value) -> AppResult<()> {
        let response = self
            .client
            .patch(url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "gh-config")
            .json(&body)
            .send()
            .await?;
        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await?;
            error!("PATCH {} failed with status {}: {}", url, status, text);
            Err(AppError::GitHubApi(text))
        }
    }

    async fn send_post(&self, url: &str, body: serde_json::Value) -> AppResult<()> {
        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "gh-config")
            .json(&body)
            .send()
            .await?;
        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await?;
            error!("POST {} failed with status {}: {}", url, status, text);
            Err(AppError::GitHubApi(text))
        }
    }

    async fn send_put(&self, url: &str, body: Option<serde_json::Value>) -> AppResult<()> {
        let mut request = self
            .client
            .put(url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "gh-config");
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await?;
        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await?;
            error!("PUT {} failed with status {}: {}", url, status, text);
            Err(AppError::GitHubApi(text))
        }
    }

    async fn get(&self, url: &str) -> AppResult<reqwest::Response> {
        let response = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "gh-config")
            .send()
            .await?;
        let status = response.status();
        debug!("GET {} returned status: {}", url, status);
        if status.is_success() {
            Ok(response)
        } else {
            let text = response.text().await?;
            error!("GET {} failed with status {}: {}", url, status, text);
            Err(AppError::GitHubApi(text))
        }
    }

    /// Fetch all settings for a repo as a HashMap<String, serde_yaml::Value>
    async fn get_repo_settings(&self, repo_name: &str) -> AppResult<RepoSettings> {
        let url = format!("https://api.github.com/repos/{}/{}", self.org, repo_name);
        let response = self.get(&url).await?;
        let text = response.text().await?;
        if text.is_empty() {
            error!("Empty response body from GET {}", url);
            return Err(AppError::GitHubApi("Empty response body".to_string()));
        }
        debug!("Raw response for {}: {}", url, text);
        let repo_json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| AppError::GitHubApi(format!("Failed to parse response from {}: {}", url, e)))?;
        // Convert to serde_yaml::Value for consistency with config
        let repo_yaml: serde_yaml::Value = serde_yaml::to_value(repo_json)
            .map_err(|e| AppError::GitHubApi(format!("Failed to convert repo JSON to YAML: {}", e)))?;
        // Flatten to a HashMap<String, Value>
        let mut settings = RepoSettings::new();
        if let serde_yaml::Value::Mapping(map) = repo_yaml {
            for (k, v) in map {
                if let serde_yaml::Value::String(key) = k {
                    settings.insert(key, v);
                }
            }
        }
        Ok(settings)
    }

    #[allow(dead_code)]
    async fn get_repo_visibility(&self, repo_name: &str) -> AppResult<String> {
        let url = format!("https://api.github.com/repos/{}/{}", self.org, repo_name);
        let response = self.get(&url).await?;
        let text = response.text().await?;
        if text.is_empty() {
            error!("Empty response body from GET {}", url);
            return Err(AppError::GitHubApi("Empty response body".to_string()));
        }
        let repo: RepoResponse = serde_json::from_str(&text)
            .map_err(|e| AppError::GitHubApi(format!("Failed to parse response from {}: {}", url, e)))?;
        Ok(if repo.private { "private" } else { "public" }.to_string())
    }

    async fn get_team(&self, team_name: &str) -> AppResult<Option<TeamResponse>> {
        let url = format!("https://api.github.com/orgs/{}/teams/{}", self.org, team_name);
        match self.get(&url).await {
            Ok(response) => {
                let text = response.text().await?;
                if text.is_empty() {
                    error!("Empty response body from GET {}", url);
                    return Err(AppError::GitHubApi("Empty response body".to_string()));
                }
                let team: TeamResponse = serde_json::from_str(&text)
                    .map_err(|e| AppError::GitHubApi(format!("Failed to parse response from {}: {}", url, e)))?;
                if team.name == team_name {
                    Ok(Some(team))
                } else {
                    Ok(None)
                }
            }
            Err(AppError::GitHubApi(e)) if e.contains("404") => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn get_user_membership(&self, login: &str) -> AppResult<Option<String>> {
        let url = format!("https://api.github.com/orgs/{}/memberships/{}", self.org, login);
        match self.get(&url).await {
            Ok(response) => {
                let text = response.text().await?;
                if text.is_empty() {
                    error!("Empty response body from GET {}", url);
                    return Err(AppError::GitHubApi("Empty response body".to_string()));
                }
                let membership: MembershipResponse = serde_json::from_str(&text)
                    .map_err(|e| AppError::GitHubApi(format!("Failed to parse response from {}: {}", url, e)))?;
                Ok(Some(membership.role))
            }
            Err(AppError::GitHubApi(e)) if e.contains("404") => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn get_team_repos(&self, team_name: &str) -> AppResult<Vec<TeamRepoResponse>> {
        let url = format!("https://api.github.com/orgs/{}/teams/{}/repos?per_page=100", self.org, team_name);
        let response = self.get(&url).await?;
        let text = response.text().await?;
        let repos: Vec<TeamRepoResponse> = serde_json::from_str(&text)
            .map_err(|e| AppError::GitHubApi(format!("Failed to parse team repos: {}", e)))?;
        Ok(repos)
    }

    /**
     * Retrieve all webhooks configured for a given repository.
     *
     * # Arguments
     * * `repo_name` - The name of the repository.
     *
     * # Returns
     * * `Ok(Vec<WebhookResponse>)` containing all webhooks for the repository.
     * * `Err(AppError)` if the API call or deserialization fails.
     */
    pub async fn get_webhooks(&self, repo_name: &str) -> AppResult<Vec<WebhookResponse>> {
        let url = format!("https://api.github.com/repos/{}/{}/hooks", self.org, repo_name);
        let response = self.get(&url).await?;
        let text = response.text().await?;
        let webhooks: Vec<WebhookResponse> = serde_json::from_str(&text)
            .map_err(|e| AppError::GitHubApi(format!("Failed to parse webhooks: {}", e)))?;
        Ok(webhooks)
    }

    async fn create_webhook(&self, repo_name: &str, webhook: &WebhookConfig) -> AppResult<()> {
        let url = format!("https://api.github.com/repos/{}/{}/hooks", self.org, repo_name);
        let body = json!({
            "name": "web",
            "active": true,
            "events": webhook.events,
            "config": {
                "url": webhook.url,
                "content_type": webhook.content_type,
                "insecure_ssl": "0"
            }
        });
        debug!("Webhook create payload: {}", serde_json::to_string(&body)?);
        info!("Creating webhook for {}/{}", self.org, repo_name);
        self.send_post(&url, body).await?;
        Ok(())
    }

    async fn update_webhook(&self, repo_name: &str, hook_id: i64, webhook: &WebhookConfig) -> AppResult<()> {
        let url = format!("https://api.github.com/repos/{}/{}/hooks/{}", self.org, repo_name, hook_id);
        let body = json!({
            "active": true,
            "events": webhook.events,
            "config": {
                "url": webhook.url,
                "content_type": webhook.content_type,
                "insecure_ssl": "0"
            }
        });
        debug!("Webhook update payload: {}", serde_json::to_string(&body)?);
        info!("Updating webhook for {}/{}", self.org, repo_name);
        self.send_patch(&url, body).await?;
        Ok(())
    }

    async fn manage_webhooks(&self, repo_name: &str, webhook: &WebhookConfig, dry_run: bool) -> AppResult<()> {
        let current_hooks = self.get_webhooks(repo_name).await?;
        let existing = current_hooks.iter().find(|h| h.config.url == webhook.url);

        if dry_run {
            match existing {
                Some(hook) if hook.events != webhook.events || hook.config.content_type != webhook.content_type => {
                    info!(
                        "[Dry Run] Would update webhook for {}/{}: events {:?} -> {:?}, content_type {} -> {}",
                        self.org, repo_name, hook.events, webhook.events, hook.config.content_type, webhook.content_type
                    );
                }
                Some(_) => debug!("[Dry Run] Webhook for {}/{} already matches desired config", self.org, repo_name),
                None => info!(
                    "[Dry Run] Would create webhook for {}/{} with config: {:?}",
                    self.org, repo_name, webhook
                ),
            }
        } else {
            match existing {
                Some(hook) if hook.events != webhook.events || hook.config.content_type != webhook.content_type => {
                    self.update_webhook(repo_name, hook.id.unwrap(), webhook).await?;
                }
                Some(_) => debug!("Webhook for {}/{} already up to date", self.org, repo_name),
                None => self.create_webhook(repo_name, webhook).await?,
            }
        }
        Ok(())
    }

    /**
     * Update repository settings on GitHub to match the desired configuration.
     *
     * This method compares the current repository settings with the desired settings and applies only the
     * necessary changes using the appropriate GitHub API endpoints. The mapping table determines which
     * settings correspond to which API endpoints and JSON fields.
     *
     * # Arguments
     * * `repo` - The repository whose settings should be updated.
     * * `dry_run` - If true, no changes are made; actions are logged for preview.
     *
     * # Returns
     * * `Ok(())` if all updates succeed or are skipped in dry-run mode.
     * * `Err(AppError)` if any API call fails.
     *
     * # Behavior
     * - Only settings that differ from the current state are updated.
     * - Supports PATCH, PUT, and POST methods as defined in the mapping.
     * - Logs actions in dry-run mode instead of performing them.
     * - Also manages webhooks if defined in the repo configuration.
     */
    pub async fn update_repo_settings(&self, repo: &Repo, dry_run: bool) -> AppResult<()> {
        let current = self.get_repo_settings(&repo.name).await?;
        let desired = &repo.settings;
        let mapping = get_github_api_mapping();

        // For each mapped field, if it differs from the current value, send a PATCH/PUT/POST to the correct endpoint.
        for (k, v_desired) in desired.iter() {
            if let Some(field_map) = mapping.get(k.as_str()) {
                let v_current = current.get(k);
                if v_current != Some(v_desired) {
                    // Build the PATCH/PUT/POST payload for this field.
                    let mut patch_body = serde_json::Map::new();
                    patch_body.insert(
                        field_map.json_path.to_string(),
                        serde_json::to_value(v_desired).unwrap_or(serde_json::Value::Null),
                    );
                    let url = field_map
                        .endpoint
                        .replace("{org}", &self.org)
                        .replace("{repo}", &repo.name);
                    let body = serde_json::Value::Object(patch_body);

                    if dry_run {
                        // Log the intended action instead of performing it.
                        info!(
                            "[Dry Run] Would {} {} with body: {:?}",
                            field_map.method, url, body
                        );
                    } else {
                        info!("{} {} with body: {:?}", field_map.method, url, body);
                        match field_map.method {
                            "PATCH" => self.send_patch(&url, body).await?,
                            "PUT" => self.send_put(&url, Some(body)).await?,
                            "POST" => self.send_post(&url, body).await?,
                            _ => error!("Unsupported HTTP method: {}", field_map.method),
                        }
                    }
                }
            } else {
                // No mapping for this setting; skip it.
                debug!("No API mapping for repo setting '{}', skipping.", k);
            }
        }

        // If a webhook is defined in the repo config, manage it as well.
        if let Some(webhook) = repo.webhook.as_ref() {
            self.manage_webhooks(&repo.name, webhook, dry_run).await?;
        }
        Ok(())
    }

    pub async fn create_team(&self, team: &Team, dry_run: bool) -> AppResult<()> {
        let existing = self.get_team(&team.name).await?;
        if dry_run {
            if existing.is_none() {
                info!("[Dry Run] Would create team: {}", team.name);
                info!(
                    "[Dry Run] Would add members to {}: {:?}",
                    team.name, team.members
                );
            } else {
                debug!("[Dry Run] Team {} already exists", team.name);
                info!(
                    "[Dry Run] Would ensure members in {}: {:?}",
                    team.name, team.members
                );
            }
            Ok(())
        } else if existing.is_none() {
            let url = format!("https://api.github.com/orgs/{}/teams", self.org);
            let body = json!({
                "name": team.name,
                "privacy": "closed"
            });
            info!("Creating team: {}", team.name);
            self.send_post(&url, body).await?;
            for member in &team.members {
                let member_url = format!(
                    "https://api.github.com/orgs/{}/teams/{}/memberships/{}",
                    self.org, team.name, member
                );
                self.send_put(&member_url, None).await?;
                info!("Added {} to team {}", member, team.name);
            }
            Ok(())
        } else {
            info!("Team {} already exists, updating members", team.name);
            for member in &team.members {
                let member_url = format!(
                    "https://api.github.com/orgs/{}/teams/{}/memberships/{}",
                    self.org, team.name, member
                );
                match self.send_put(&member_url, None).await {
                    Ok(()) => info!("Added or confirmed {} in team {}", member, team.name),
                    Err(e) => error!("Failed to add {} to team {}: {}", member, team.name, e),
                }
            }
            Ok(())
        }
    }

    pub async fn add_user_to_org(&self, user: &User, dry_run: bool) -> AppResult<()> {
        if dry_run {
            let current_role = self.get_user_membership(&user.login).await?;
            match current_role {
                Some(role) if role != user.role => {
                    info!(
                        "[Dry Run] Would update {} role: {} -> {}",
                        user.login, role, user.role
                    );
                }
                None => {
                    info!("[Dry Run] Would add {} with role {}", user.login, user.role);
                }
                _ => debug!("[Dry Run] No changes needed for user {}", user.login),
            }
            Ok(())
        } else {
            let url = format!(
                "https://api.github.com/orgs/{}/memberships/{}",
                self.org, user.login
            );
            let body = json!({
                "role": user.role
            });
            info!("Adding {} to org with role {}", user.login, user.role);
            self.send_put(&url, Some(body)).await?;
            Ok(())
        }
    }

    pub async fn assign_team_to_repo(&self, assignment: &Assignment, dry_run: bool) -> AppResult<()> {
        if dry_run {
            let current_perm = self
                .get_team_repo_permission(&assignment.team, &assignment.repo)
                .await?;
            match current_perm {
                Some(perm) if perm != assignment.permission => {
                    info!(
                        "[Dry Run] Would update {}/{} permission for team {}: {} -> {}",
                        self.org, assignment.repo, assignment.team, perm, assignment.permission
                    );
                }
                None => {
                    info!(
                        "[Dry Run] Would assign team {} to {}/{} with permission {}",
                        assignment.team, self.org, assignment.repo, assignment.permission
                    );
                }
                _ => debug!(
                    "[Dry Run] No changes needed for team {} on {}/{}",
                    assignment.team, self.org, assignment.repo
                ),
            }
            Ok(())
        } else {
            let url = format!(
                "https://api.github.com/orgs/{}/teams/{}/repos/{}/{}",
                self.org, assignment.team, self.org, assignment.repo
            );
            let body = json!({
                "permission": assignment.permission
            });
            info!(
                "Assigning team {} to repo {} with permission {}",
                assignment.team, assignment.repo, assignment.permission
            );
            self.send_put(&url, Some(body)).await?;
            Ok(())
        }
    }

    async fn get_team_repo_permission(&self, team: &str, repo: &str) -> AppResult<Option<String>> {
        let url = format!(
            "https://api.github.com/orgs/{}/teams/{}/repos/{}/{}",
            self.org, team, self.org, repo
        );
        match self.get(&url).await {
            Ok(response) => {
                let text = response.text().await?;
                if text.is_empty() {
                    debug!("Empty response body from GET {}, assuming permission exists but not detailed", url);
                    return Ok(Some("push".to_string()));
                }
                let perms: TeamRepoResponse = serde_json::from_str(&text)
                    .map_err(|e| AppError::GitHubApi(format!("Failed to parse response from {}: {}", url, e)))?;
                let permission = if perms.permissions.admin {
                    "admin"
                } else if perms.permissions.push {
                    "push"
                } else if perms.permissions.pull {
                    "pull"
                } else {
                    "none"
                };
                Ok(Some(permission.to_string()))
            }
            Err(AppError::GitHubApi(e)) if e.contains("404") => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub async fn generate_config_from_org(&self) -> AppResult<Config> {
        let mut repos = Vec::new();
        let repo_url = format!("https://api.github.com/orgs/{}/repos?per_page=100", self.org);
        let repo_response = self.get(&repo_url).await?;
        let repo_json: Vec<serde_json::Value> = repo_response.json().await
            .map_err(|e| AppError::Http(e))?;

        for repo in repo_json {
            let name = repo["name"].as_str().ok_or_else(|| AppError::GitHubApi("Missing repo name".to_string()))?.to_string();
            let settings = self.get_repo_settings(&name).await?; // Use individual repo endpoint
            let visibility = if repo["private"].as_bool().unwrap_or(false) { Some("private".to_string()) } else { Some("public".to_string()) };
            let webhooks = self.get_webhooks(&name).await?;
            let webhook = webhooks.first().map(|wh| WebhookConfig {
                url: wh.config.url.clone(),
                content_type: wh.config.content_type.clone(),
                events: wh.events.clone(),
            });

            repos.push(Repo {
                name,
                settings,
                visibility,
                webhook,
                branch_protections: vec![],
                extra: std::collections::HashMap::new(),
            });
        }

        let mut teams = Vec::new();
        let team_url = format!("https://api.github.com/orgs/{}/teams?per_page=100", self.org);
        let team_response = self.get(&team_url).await?;
        let team_json: Vec<serde_json::Value> = team_response.json().await
            .map_err(|e| AppError::Http(e))?;

        for team in team_json {
            let name = team["name"].as_str().ok_or_else(|| AppError::GitHubApi("Missing team name".to_string()))?.to_string();
            let members_url = format!("https://api.github.com/orgs/{}/teams/{}/members?per_page=100", self.org, name);
            let members_response = self.get(&members_url).await?;
            let members_json: Vec<serde_json::Value> = members_response.json().await
                .map_err(|e| AppError::Http(e))?;
            let mut members = members_json.iter()
                .filter_map(|m| m["login"].as_str().map(String::from))
                .collect::<Vec<String>>();
            members.sort();

            teams.push(Team { name, members });
        }

        let mut users = Vec::new();
        let members_url = format!("https://api.github.com/orgs/{}/members?per_page=100", self.org);
        let members_response = self.get(&members_url).await?;
        let members_json: Vec<serde_json::Value> = members_response.json().await
            .map_err(|e| AppError::Http(e))?;

        for member in members_json {
            let login = member["login"].as_str().ok_or_else(|| AppError::GitHubApi("Missing member login".to_string()))?.to_string();
            let role_response = self.get_user_membership(&login).await?;
            let role = role_response.unwrap_or("member".to_string());
            users.push(User { login, role });
        }

        let mut assignments = Vec::new();
        for team in &teams {
            let team_repos = self.get_team_repos(&team.name).await?;
            for repo in team_repos {
                let permission = if repo.permissions.admin {
                    "admin"
                } else if repo.permissions.push {
                    "push"
                } else if repo.permissions.pull {
                    "pull"
                } else {
                    "none"
                };
                if permission != "none" {
                    assignments.push(Assignment {
                        repo: repo.name.clone(),
                        team: team.name.clone(),
                        permission: permission.to_string(),
                    });
                }
            }
        }

        let default_webhook = repos.first().and_then(|r| r.webhook.clone());

        Ok(Config {
            org: self.org.clone(),
            repos,
            teams,
            users,
            assignments,
            default_webhook,
            default_branch_protections: vec![],
            extra: std::collections::HashMap::new(),
        })
    }

    pub async fn generate_config_and_write(&self, config_path: &str, dry_run: bool) -> AppResult<()> {
        info!("Generating config from GitHub org: {}", self.org);
        let config = self.generate_config_from_org().await?;

        let mut yaml_content = String::new();
        yaml_content.push_str(&format!("org: {}\n\n", config.org));

        let mut assignments = config.assignments.clone();
        assignments.sort_by(|a, b| a.team.cmp(&b.team).then(a.repo.cmp(&b.repo)));
        if !assignments.is_empty() {
            yaml_content.push_str("assignments:\n");
            for assignment in &assignments {
                yaml_content.push_str(&format!(
                    "- repo: {}\n  team: {}\n  permission: {}\n",
                    assignment.repo, assignment.team, assignment.permission
                ));
            }
            yaml_content.push_str("\n");
        } else {
            yaml_content.push_str("assignments: []\n\n");
        }

        if let Some(default_webhook) = &config.default_webhook {
            yaml_content.push_str("default_webhook:\n");
            yaml_content.push_str(&format!("  url: {}\n", default_webhook.url));
            yaml_content.push_str(&format!("  content_type: {}\n", default_webhook.content_type));
            yaml_content.push_str("  events:\n");
            let mut events = default_webhook.events.clone();
            events.sort();
            for event in &events {
                yaml_content.push_str(&format!("  - {}\n", event));
            }
            yaml_content.push_str("\n");
        }

        let mut repos = config.repos.clone();
        repos.sort_by(|a, b| a.name.cmp(&b.name));
        if !repos.is_empty() {
            yaml_content.push_str("repos:\n");
            for repo in &repos {
                yaml_content.push_str(&format!("- name: {}\n", repo.name));
                yaml_content.push_str("  settings:\n");
                let amc = repo.settings.get("allow_merge_commit").map(|v| v.as_bool().unwrap_or(false)).unwrap_or(false);
                let asm = repo.settings.get("allow_squash_merge").map(|v| v.as_bool().unwrap_or(false)).unwrap_or(false);
                let arm = repo.settings.get("allow_rebase_merge").map(|v| v.as_bool().unwrap_or(false)).unwrap_or(false);
                yaml_content.push_str(&format!("    allow_merge_commit: {}\n", amc));
                yaml_content.push_str(&format!("    allow_squash_merge: {}\n", asm));
                yaml_content.push_str(&format!("    allow_rebase_merge: {}\n", arm));
                if let Some(visibility) = &repo.visibility {
                    yaml_content.push_str(&format!("  visibility: {}\n", visibility));
                }
                if let Some(webhook) = &repo.webhook {
                    if config.default_webhook.as_ref() != Some(webhook) {
                        yaml_content.push_str("  webhook:\n");
                        yaml_content.push_str(&format!("    url: {}\n", webhook.url));
                        yaml_content.push_str(&format!("    content_type: {}\n", webhook.content_type));
                        yaml_content.push_str("    events:\n");
                        let mut events = webhook.events.clone();
                        events.sort();
                        for event in &events {
                            yaml_content.push_str(&format!("    - {}\n", event));
                        }
                    }
                }
            }
            yaml_content.push_str("\n");
        }

        let mut teams = config.teams.clone();
        teams.sort_by(|a, b| a.name.cmp(&b.name));
        if !teams.is_empty() {
            yaml_content.push_str("teams:\n");
            for team in &teams {
                yaml_content.push_str(&format!("- name: {}\n", team.name));
                yaml_content.push_str("  members:\n");
                let mut members = team.members.clone();
                members.sort();
                for member in &members {
                    yaml_content.push_str(&format!("  - {}\n", member));
                }
            }
            yaml_content.push_str("\n");
        }

        let mut users = config.users.clone();
        users.sort_by(|a, b| a.login.cmp(&b.login));
        if !users.is_empty() {
            yaml_content.push_str("users:\n");
            for user in &users {
                yaml_content.push_str(&format!("- login: {}\n  role: {}\n", user.login, user.role));
            }
            yaml_content.push_str("\n");
        }

        if dry_run {
            println!("Dry run: Would write the following config to {}:\n{}", config_path, yaml_content);
        } else {
            println!("Writing generated config to {}", config_path);
            let mut file = File::create(config_path)
                .map_err(|e| AppError::Io(e))?;
            file.write_all(yaml_content.as_bytes())
                .map_err(|e| AppError::Io(e))?;
            println!("Config generation completed successfully.");
        }
        Ok(())
    }

    pub async fn sync(&mut self, config_path: &str, dry_run: bool) -> AppResult<()> {
        let config = crate::config::Config::from_file_with_defaults(config_path, None)?;
        self.org = config.org.clone();

        if dry_run {
            info!("Running in dry-run mode; validating changes without applying.");
        } else {
            info!("Running in apply mode; changes will be applied.");
        }

        let mut config = config;
        if let Some(default_webhook) = &config.default_webhook {
            for repo in &mut config.repos {
                if repo.webhook.is_none() {
                    repo.webhook = Some(default_webhook.clone());
                }
            }
        } else {
            return Err(AppError::GitHubApi(
                "No default_webhook specified in config, and not all repos have webhooks".to_string(),
            ));
        }

        for repo in &config.repos {
            if repo.webhook.is_none() {
                return Err(AppError::GitHubApi(format!(
                    "Repository {} has no webhook configuration",
                    repo.name
                )));
            }
            self.update_repo_settings(repo, dry_run).await?;
        }

        for team in &config.teams {
            self.create_team(team, dry_run).await?;
        }

        for user in &config.users {
            self.add_user_to_org(user, dry_run).await?;
        }

        for assignment in &config.assignments {
            self.assign_team_to_repo(assignment, dry_run).await?;
        }

        if dry_run {
            println!("Dry run completed successfully. No changes were applied.");
        } else {
            println!("Sync completed successfully. All changes applied.");
        }
        Ok(())
    }

    pub async fn diff(&self, config_path: &str) -> AppResult<bool> {
        info!("Generating diff between GitHub state and local config: {}", config_path);

        // Fetch GitHub's current state
        let mut github_config = self.generate_config_from_org().await?;

        // Load and consolidate local config (apply default_webhook to repos)
        let mut local_config = crate::config::Config::from_file_with_defaults(config_path, None)?;
        if let Some(default_webhook) = &local_config.default_webhook {
            for repo in &mut local_config.repos {
                if repo.webhook.is_none() {
                    repo.webhook = Some(default_webhook.clone());
                }
            }
        }

        // Sort repos alphabetically by name in both configs
        github_config.repos.sort_by(|a, b| a.name.cmp(&b.name));
        local_config.repos.sort_by(|a, b| a.name.cmp(&b.name));

        // Serialize both configs to YAML strings
        let github_yaml = serde_yaml::to_string(&github_config)?;
        let local_yaml = serde_yaml::to_string(&local_config)?;

        // Compute the unified diff with line numbers and context
        let diff = TextDiff::from_lines(&github_yaml, &local_yaml);
        let mut unified_diff = diff.unified_diff();
        let unified_diff = unified_diff.context_radius(3).header("GitHub", "Local");

        // Check if there are any differences and output them
        let has_diffs = unified_diff.iter_hunks().next().is_some();
        if !has_diffs {
            println!("No differences found between GitHub state and local config.");
        } else {
            println!("Differences between GitHub state and local config (with line numbers):");
            println!("--- GitHub");
            println!("+++ Local");
            for (idx, hunk) in unified_diff.iter_hunks().enumerate() {
                // Calculate old and new line ranges from changes
                let mut old_start = None;
                let mut old_count = 0;
                let mut new_start = None;
                let mut new_count = 0;

                for change in hunk.iter_changes() {
                    if let Some(old_line) = change.old_index() {
                        if old_start.is_none() {
                            old_start = Some(old_line + 1); // 1-based indexing
                        }
                        old_count += 1;
                    }
                    if let Some(new_line) = change.new_index() {
                        if new_start.is_none() {
                            new_start = Some(new_line + 1); // 1-based indexing
                        }
                        new_count += 1;
                    }
                }

                let old_start = old_start.unwrap_or(1); // Default to 1 if no old lines
                let new_start = new_start.unwrap_or(1); // Default to 1 if no new lines
                let old_count = if old_count == 0 { 1 } else { old_count }; // Ensure at least 1 line
                let new_count = if new_count == 0 { 1 } else { new_count }; // Ensure at least 1 line

                println!(
                    "@@ -{},{} +{},{} @@ Hunk {}",
                    old_start, old_count, new_start, new_count, idx + 1
                );
                for change in hunk.iter_changes() {
                    match change.tag() {
                        ChangeTag::Delete => println!("{}", format!("- {}", change.value().trim_end()).red()),
                        ChangeTag::Insert => println!("{}", format!("+ {}", change.value().trim_end()).green()),
                        ChangeTag::Equal => println!("  {}", change.value().trim_end()),
                    }
                }
            }
        }
        Ok(has_diffs)
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use mockito;
    use tokio;

    #[tokio::test]
    async fn test_github_client_new() {
        let client = GitHubClient::new("dummy_token", "dummy_org");
        assert_eq!(client.token, "dummy_token");
        assert_eq!(client.org, "dummy_org");
    }

    #[test]
    fn test_get_webhooks_parses_response() {
        let mut server = mockito::Server::new();

        let _m = server
            .mock("GET", "/repos/dummy_org/dummy_repo/hooks")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"
[
  {
    "id": 123,
    "url": "http://api.github.com/hook/123",
    "config": {
      "url": "http://example.com",
      "content_type": "json"
    },
    "events": ["push", "pull_request"]
  }
]
"#,
            )
            .create();

        let rt = tokio::runtime::Runtime::new().expect("create runtime");
        rt.block_on(async {
            let mut client = GitHubClient::new("dummy_token", "dummy_org");
            client.org = "dummy_org".to_string();

            let url = format!("{}/repos/dummy_org/dummy_repo/hooks", server.url());
            let response = client.get(&url).await.expect("HTTP GET failed");
            let text = response.text().await.expect("read response text");
            let hooks: Vec<WebhookResponse> = serde_json::from_str(&text).expect("parse JSON");

            assert_eq!(hooks.len(), 1);
            let hook = &hooks[0];
            assert_eq!(hook.id, Some(123));
            assert_eq!(hook.url, "http://api.github.com/hook/123");
            assert_eq!(hook.config.url, "http://example.com");
            assert_eq!(hook.config.content_type, "json");
            assert_eq!(hook.events, vec!["push", "pull_request"]);
        });
    }
}