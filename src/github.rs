use crate::config::{Assignment, Repo, RepoSettings, Team, User, WebhookConfig};
use crate::error::{AppError, AppResult};
use log::{debug, info, error};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
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
struct TeamRepoPermission {
    permissions: PermissionDetails,
}

#[derive(Debug, Deserialize)]
struct PermissionDetails {
    pull: bool,
    push: bool,
    admin: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct WebhookResponse {
    id: Option<i64>,
    url: String,
    config: WebhookConfigResponse,
    events: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct WebhookConfigResponse {
    url: String,
    content_type: String,
}

pub struct GitHubClient {
    client: Client,
    token: String,
    org: String,
}

impl GitHubClient {
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
            .header("User-Agent", "gh-config-cli")
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
            .header("User-Agent", "gh-config-cli")
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
            .header("User-Agent", "gh-config-cli");
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
            .header("User-Agent", "gh-config-cli")
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

    async fn get_repo_settings(&self, repo_name: &str) -> AppResult<RepoSettings> {
        let url = format!("https://api.github.com/repos/{}/{}", self.org, repo_name);
        let response = self.get(&url).await?;
        let text = response.text().await?;
        if text.is_empty() {
            error!("Empty response body from GET {}", url);
            return Err(AppError::GitHubApi("Empty response body".to_string()));
        }
        let repo: RepoResponse = serde_json::from_str(&text)
            .map_err(|e| AppError::GitHubApi(format!("Failed to parse response from {}: {}", url, e)))?;
        Ok(RepoSettings {
            allow_merge_commit: repo.allow_merge_commit,
            allow_squash_merge: repo.allow_squash_merge,
            allow_rebase_merge: repo.allow_rebase_merge,
        })
    }

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

    async fn get_team_repo_permission(&self, team: &str, repo: &str) -> AppResult<Option<String>> {
        let url = format!(
            "https://api.github.com/orgs/{}/teams/{}/repos/{}/{}",
            self.org, team, self.org, repo
        );
        match self.get(&url).await {
            Ok(response) => {
                let text = response.text().await?;
                if text.is_empty() {
                    debug!("Empty response body from GET {}, assuming no permission", url);
                    return Ok(None);
                }
                let perms: TeamRepoPermission = serde_json::from_str(&text)
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

    async fn get_webhooks(&self, repo_name: &str) -> AppResult<Vec<WebhookResponse>> {
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
        debug!("Webhook create payload: {}", serde_json::to_string(&body)?); // Add this line
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
        debug!("Webhook update payload: {}", serde_json::to_string(&body)?); // Add this line
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

    pub async fn update_repo_settings(&self, repo: &Repo, dry_run: bool) -> AppResult<()> {
        // Handle repository settings
        let current = self.get_repo_settings(&repo.name).await?;
        let desired = &repo.settings;
        let current_visibility = self.get_repo_visibility(&repo.name).await?;
        let desired_visibility = repo.visibility.as_deref().unwrap_or("private");

        if dry_run {
            if current.allow_merge_commit != desired.allow_merge_commit
                || current.allow_squash_merge != desired.allow_squash_merge
                || current.allow_rebase_merge != desired.allow_rebase_merge
            {
                info!(
                    "[Dry Run] Would update {}/{} settings: {:?} -> {:?}",
                    self.org, repo.name, current, desired
                );
            }
            if current_visibility != desired_visibility {
                info!(
                    "[Dry Run] Would update {}/{} visibility: {} -> {}",
                    self.org, repo.name, current_visibility, desired_visibility
                );
            }
        } else {
            if current.allow_merge_commit != desired.allow_merge_commit
                || current.allow_squash_merge != desired.allow_squash_merge
                || current.allow_rebase_merge != desired.allow_rebase_merge
                || current_visibility != desired_visibility
            {
                let url = format!("https://api.github.com/repos/{}/{}", self.org, repo.name);
                let body = json!({
                    "allow_merge_commit": repo.settings.allow_merge_commit,
                    "allow_squash_merge": repo.settings.allow_squash_merge,
                    "allow_rebase_merge": repo.settings.allow_rebase_merge,
                    "private": repo.visibility.as_deref() != Some("public")
                });
                info!("Updating settings for {}/{}", self.org, repo.name);
                self.send_patch(&url, body).await?;
            }
        }

        // Handle webhook (always called since repo.webhook is guaranteed to be Some)
        self.manage_webhooks(&repo.name, repo.webhook.as_ref().unwrap(), dry_run).await?;

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
}