mod config;
mod error;
mod github;
mod ssh_keys;

use clap::Parser;
use config::Config;
use error::AppResult;
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

    let config = Config::from_file(&args.config)?;
    let client = GitHubClient::new(&args.token, &config.org);

    if args.dry_run {
        info!("Running in dry-run mode; validating changes without applying.");
    } else {
        info!("Running in apply mode; changes will be applied.");
    }

    for repo in &config.repos {
        client.update_repo_settings(repo, args.dry_run).await?;

        // Check if deploy key is enabled for this repo
    if let Some(deploy_config) = &repo.deploy_key {
        if args.dry_run {
            info!("[Dry Run] Would check and generate SSH key pair for deploy key and secret for repo {}", repo.name);
        } else {
            // Check if the deploy key and secret already exist
            let key_exists = client.deploy_key_exists(&repo.name, &deploy_config.name).await?;
            let secret_exists = client.repo_secret_exists(&repo.name, &deploy_config.name).await?;

            if key_exists && secret_exists {
                info!("Deploy key and secret already exist for repo {}", repo.name);
            } else {
                // Only generate a new key pair if either key or secret is missing.
                let (private_key, public_key) = ssh_keys::generate_key_pair(&repo.name)
                    .map_err(|e| {
                        error!("Failed to generate key pair for {}: {}", repo.name, e);
                        e
                    })?;
                if !key_exists {
                    client.create_deploy_key(&repo.name, &deploy_config.name, &public_key, true).await?;
                }
                if !secret_exists {
                    client.create_repo_secret(&repo.name, &deploy_config.name, &private_key).await?;
                }
            }
        }
    }
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
