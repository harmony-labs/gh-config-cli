# gh-config-cli

A Rust CLI tool to manage GitHub organization settings declaratively using YAML files. Define repositories, teams, users, and assignments in a config file, and apply, validate, or diff changes against the GitHub API.

## Features

- **Declarative Configuration**: Manage GitHub org settings with a single YAML file.
- **Dry Run Mode**: Validate changes without applying them, showing what would happen.
- **Diff Command**: Compare the local config with the current GitHub state, displaying differences in a line-numbered YAML format.
- **GitHub Actions Integration**: Automatically validate on PR pushes and apply on merge.
- **Supported Entities**:
  - Repositories (merge settings: squash, rebase, merge commits; visibility; webhooks).
  - Teams (creation and member assignment).
  - Users (org membership and roles).
  - Team-to-repo assignments (permissions).
  - Default webhook configuration applied to all repos unless overridden.

## Prerequisites

- Rust (stable) installed (see [rustup.rs](https://rustup.rs/)).
- A GitHub Personal Access Token (PAT) with specific fine-grained permissions (see below).

### Creating a GitHub Personal Access Token

`gh-config` requires a fine-grained PAT with permissions to manage your organization and repositories. Follow these steps to create one:

1. **Log in to GitHub** and go to **Settings > Developer settings > Personal access tokens > Fine-grained tokens**.
2. Click **Generate new token**.
3. **Token Name**: Enter a descriptive name (e.g., `gh-config-cli-token`).
4. **Expiration**: Choose a suitable expiration date (e.g., 90 days or no expiration, depending on your security policy).
5. **Repository Access**:
   - Select **Only select repositories** if you want to limit access, then choose all repositories listed in your `config.yaml` (e.g., `harmony-labs/harmony`, `harmony-labs/vnext`, etc.).
   - Alternatively, select **All repositories** under the organization `harmony-labs` for broader access.
6. **Permissions**:
   - **Repository Permissions**:
     - **Administration**: Read and Write (to update repo settings like merge options and visibility).
     - **Contents**: Read-only (to fetch repo details).
     - **Webhooks**: Read and Write (to set webhooks).
   - **Organization Permissions**:
     - **Administration**: Read and Write (to manage org settings and teams).
     - **Members**: Read and Write (to manage org memberships and roles).
     - **Webhooks**: Read and Write (to set webhooks).
7. Click **Generate token** and copy the token (e.g., `github_pat_XYZ...`).

**Note**: Store this token securely and never commit it to your repository. Use environment variables (e.g., `GITHUB_TOKEN`) or GitHub Secrets (see GitHub Actions setup).

## Installation

1. **Download the Binary**:
   - Go to the [Releases page](https://github.com/harmony-labs/gh-config-cli/releases).
   - Download the latest `gh-config` binary for your platform (e.g., `gh-config` for Linux/macOS).

2. **Make it Executable** (Linux/macOS):

   chmod +x gh-config
   mv gh-config /usr/local/bin/

3. **Clone and Build from Source (Optional)**:

   git clone https://github.com/harmony-labs/gh-config-cli.git
   cd gh-config-cli
   cargo build --release

The binary will be at `./target/release/gh-config`.

## Usage

### Examples

1. **Show Diff**:

   gh-config diff path/to/config.yaml --token <your-pat>

## Local Development

1. Clone the repository:
   ```
   git clone https://github.com/<your-username>/gh-config-cli.git
   cd gh-config-cli
   ```

2. Build the project:
   ```
   cargo build --release
   ```

The binary will be available at `./target/release/gh-config`.

## Configuration

Define your GitHub organization settings in a YAML file (e.g., `config.yaml`). Example:

```
org: harmony-labs
default_webhook:
  url: https://discord.com/api/webhooks/1234567890/...
  content_type: json
  events:
    - push
    - pull_request
repos:
  - name: harmony
    settings:
      allow_merge_commit: false
      allow_squash_merge: true
      allow_rebase_merge: true
teams:
  - name: core-team
    members:
      - adminuser
      - otheruser
users:
  - login: adminuser
    role: admin
assignments:
  - repo: harmony
    team: core-team
    permission: push
```

### Schema

- `org`: The GitHub organization name (e.g., `harmony-labs`).
- `default_webhook`: Optional default webhook configuration applied to all repos unless overridden.
  - `url`: Webhook URL (e.g., Discord webhook).
  - `content_type`: Format of webhook payload (e.g., `json`).
  - `events`: List of GitHub events to trigger the webhook (e.g., `push`, `pull_request`).
- `repos`: List of repositories.
  - `name`: Repository name (e.g., `harmony`).
  - `settings`: Merge options (`allow_merge_commit`, `allow_squash_merge`, `allow_rebase_merge`).
  - `visibility`: Optional, `public` or `private` (defaults to `private` if omitted).
  - `webhook`: Optional per-repo webhook overriding `default_webhook`.
- `teams`: List of teams.
  - `name`: Team name (e.g., `core-team`).
  - `members`: List of GitHub usernames.
- `users`: List of org members.
  - `login`: GitHub username (e.g., `adminuser`).
  - `role`: `member` or `admin`.
- `assignments`: Team-to-repo permissions.
  - `repo`: Repository name.
  - `team`: Team name.
  - `permission`: `pull`, `push`, or `admin`.

## Usage

### Command-Line Options

Run `cargo run -- --help` to see all options:

- `--config <PATH>`: Path to the YAML config file (default: `config.yaml`).
- `--token <TOKEN>`: GitHub PAT (or set via `GITHUB_TOKEN` env var).
- `--dry-run`: Validate changes without applying them.
- `--sync-from-org <ORG>`: Generate a config file from the specified GitHub org.
- `--diff`: Show a line-numbered diff between the local config and GitHub state.

### Examples

1. **Apply Changes**:
   ```
   cargo run -- --config config.yaml --token <your-pat>
   ```

2. **Dry Run (Validation)**:
   ```
   RUST_LOG=info cargo run -- --config config.yaml --token <your-pat> --dry-run
   ```
   Output example:
   ```
   INFO  gh_config_cli: Running in dry-run mode; validating changes without applying.
   INFO  gh_config_cli::github: [Dry Run] Would update harmony-labs/harmony settings: RepoSettings { allow_merge_commit: true, ... } -> RepoSettings { allow_merge_commit: false, ... }
   ```

3. **Generate Config from GitHub**:
   ```
   cargo run -- --config config.yaml --token <your-pat> --sync-from-org harmony-labs
   ```

4. **Show Diff**:
   ```
   cargo run -- --config config.yaml --token <your-pat> --diff
   ```
   Output example:
   ```
   --- GitHub
   +++ Local
   @@ -1,5 +1,5 @@ Hunk 1
    org: harmony-labs
    repos:
    - name: harmony
      settings:
   -    allow_merge_commit: true
   +    allow_merge_commit: false
        allow_squash_merge: true
        allow_rebase_merge: true
   ```

5. **Using Installed Binary**:
   After `cargo install --path .`:
   ```
   gh-config --config config.yaml --token <your-pat> --diff
   ```

### Makefile Commands

- `make build`: Build the project.
- `make diff`: Run the diff command.
- `make dry-run`: Run in dry-run mode.
- `make sync`: Apply the config.
- `make sync-from-github`: Generate config from GitHub.
- `make list-repos`: List org repos.
- `make help`: Show CLI help.

## GitHub Actions Integration

Automate validation and application with GitHub Actions. Add this workflow to `.github/workflows/validate-org.yml`:

```
name: Validate GitHub Org Config

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]

jobs:
  validate-config:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Set up Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Build CLI
        run: cargo build --release
      - name: Validate Config (Dry Run)
        env:
          GITHUB_TOKEN: ${{ secrets.GH_PAT }}
          RUST_LOG: info
        run: ./target/release/gh-config --config config.yaml --token $GITHUB_TOKEN --dry-run

  apply-config:
    if: github.event_name == 'pull_request' && github.event.action == 'closed' && github.event.pull_request.merged == true
    runs-on: ubuntu-latest
    needs: validate-config
    steps:
      - uses: actions/checkout@v4
      - name: Set up Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Build CLI
        run: cargo build --release
      - name: Apply Config
        env:
          GITHUB_TOKEN: ${{ secrets.GH_PAT }}
          RUST_LOG: info
        run: ./target/release/gh-config --config config.yaml --token $GITHUB_TOKEN
```

### Setup

1. **Store PAT in GitHub Secrets**:
   - Go to your repository’s **Settings > Secrets and variables > Actions**.
   - Click **New repository secret**.
   - Name it `GH_PAT` and paste your PAT value.
2. Push changes to a PR; the `validate-config` job runs on each commit.
3. Merge the PR; the `apply-config` job applies the changes.

## Contributing

- Report issues or suggest features via GitHub Issues.
- Submit pull requests with improvements.

## License

MIT License. See [LICENSE](LICENSE) for details.