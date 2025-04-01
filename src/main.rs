mod config;
mod error;
mod github;
mod ssh_keys;

use clap::{Parser, Subcommand};
use config::Config;
use error::AppResult;
use github::GitHubClient;
use log::{error, info};
use std::process;

#[derive(Parser, Debug)]
#[command(version, about = "Manage GitHub org settings declaratively")]
struct Args {
    /// GitHub Personal Access Token
    #[arg(short, long, env = "GITHUB_TOKEN")]
    token: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Show a line-numbered diff between GitHub state and local config file
    Diff {
        /// Path to the config file
        config: String,
    },
    /// Sync local config to GitHub
    Sync {
        /// Path to the config file
        config: String,
        /// Dry run mode (no changes applied, only validation)
        #[arg(long)]
        dry_run: bool,
    },
    /// Generate config from a GitHub org and write to file
    SyncFromOrg {
        /// Path to the config file
        config: String,
        /// Dry run mode (no changes applied, only validation)
        #[arg(long)]
        dry_run: bool,
        /// GitHub organization name to sync from
        #[arg(long)]
        org: String,
    },
}

#[tokio::main]
async fn main() {
    env_logger::init();
    match run().await {
        Ok(false) => process::exit(0),
        Ok(true) => {
            info!("Exiting with code 1 due to differences found.");
            process::exit(1)
        }
        Err(e) => {
            error!("Application error: {}", e);
            process::exit(1)
        }
    }
}

async fn run() -> AppResult<bool> {
    let args = Args::parse();

    let (command, config_path, dry_run, org) = match &args.command {
        Command::Diff { config } => ("diff", config, false, None),
        Command::Sync { config, dry_run } => ("sync", config, *dry_run, None),
        Command::SyncFromOrg { config, dry_run, org } => ("sync-from-org", config, *dry_run, Some(org)),
    };

    info!("Starting gh-config-cli with command: {}, config: {}", command, config_path);

    let mut client = match &args.command {
        Command::SyncFromOrg { .. } => GitHubClient::new(&args.token, org.unwrap()),
        _ => {
            let cfg = Config::from_file(config_path)?;
            GitHubClient::new(&args.token, &cfg.org)
        }
    };

    match args.command {
        Command::Diff { .. } => client.diff(config_path).await,
        Command::Sync { .. } => {
            client.sync(config_path, dry_run).await?;
            Ok(false)
        }
        Command::SyncFromOrg { .. } => {
            client.generate_config_and_write(config_path, dry_run).await?;
            Ok(false)
        }
    }
}
