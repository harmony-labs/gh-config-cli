mod config;
mod error;
mod github;

use clap::Parser;
use config::Config;  // Add this direct import
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

    /// Generate config from the specified GitHub org and write to file (e.g., "harmony-labs")
    #[arg(long)]
    sync_from_org: Option<String>,

    /// Show diff between GitHub state and local config file
    #[arg(long)]
    diff: bool,
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

    let mut client = if let Some(org) = &args.sync_from_org {
        GitHubClient::new(&args.token, org)
    } else {
        // Load the config file to get the org if not provided via --sync-from-org
        let config = Config::from_file(&args.config)?;
        GitHubClient::new(&args.token, &config.org)
    };

    if args.diff {
        client.diff(&args.config).await?;
    } else if args.sync_from_org.is_some() {
        client.generate_config_and_write(&args.config, args.dry_run).await?;
    } else {
        client.sync(&args.config, args.dry_run).await?;
    }
    Ok(())
}