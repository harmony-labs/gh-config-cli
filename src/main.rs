mod config;
mod error;
mod github;

use clap::Parser;
use config::Config;
use error::{AppError, AppResult};
use github::GitHubClient;
use log::{error, info};
use std::process;

#[derive(Parser, Debug)]
#[command(version, about = "Manage GitHub org settings declaratively")]
struct Args {
    /// Path to the config file
    #[arg(short, long, default_value = "config.yaml")]
    config: String,

    /// GitHub Personal Access Token
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: String,

    /// Dry run mode (no changes applied, only validation)
    #[arg(long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    if let Err(e) = run().await {
        error!("Application error: {}", e);
        process::exit(1);
    }
}

async fn run() -> AppResult<()> {
    let args = Args::parse();
    info!("Starting gh-config-cli with config: {}", args.config);

    let mut config = Config::from_file(&args.config)?;
    let client = GitHubClient::new(&args.token, &config.org);

    if args.dry_run {
        info!("Running in dry-run mode; validating changes without applying.");
    } else {
        info!("Running in apply mode; changes will be applied.");
    }

    // Ensure every repo has a webhook by applying default_webhook if none exists
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

    // Verify all repos have a webhook
    for repo in &config.repos {
        if repo.webhook.is_none() {
            return Err(AppError::GitHubApi(format!(
                "Repository {} has no webhook configuration",
                repo.name
            )));
        }
    }

    for repo in &config.repos {
        client.update_repo_settings(repo, args.dry_run).await?;
    }

    for team in &config.teams {
        client.create_team(team, args.dry_run).await?;
    }

    for user in &config.users {
        client.add_user_to_org(user, args.dry_run).await?;
    }

    for assignment in &config.assignments {
        client.assign_team_to_repo(assignment, args.dry_run).await?;
    }

    if args.dry_run {
        info!("Dry run completed successfully. No changes were applied.");
    } else {
        info!("Sync completed successfully. All changes applied.");
    }
    Ok(())
}