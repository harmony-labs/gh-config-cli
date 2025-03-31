# gh-config-cli

A Rust CLI tool to manage GitHub organization settings declaratively using YAML files. Define repositories, teams, users, and assignments in a config file, and apply or validate changes via the GitHub API.

## Features

- **Declarative Configuration**: Manage GitHub org settings with a single YAML file.
- **Dry Run Mode**: Validate changes without applying them, showing what would happen.
- **GitHub Actions Integration**: Automatically validate on PR pushes and apply on merge.
- **Supported Entities**:
  - Repositories (merge settings: squash, rebase, merge commits).
  - Teams (creation and member assignment).
  - Users (org membership and roles).
  - Team-to-repo assignments (permissions).

## Prerequisites

- Rust (stable) installed (see [rustup.rs](https://rustup.rs/)).
- A GitHub Personal Access Token (PAT) with specific fine-grained permissions (see below).

### Creating a GitHub Personal Access Token

`gh-config-cli` requires a fine-grained PAT with permissions to manage your organization and repositories. Follow these steps to create one:

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
   - **Organization Permissions**:
     - **Administration**: Read and Write (to manage org settings and teams).
     - **Members**: Read and Write (to manage org memberships and roles).
7. Click **Generate token** and copy the token (e.g., `github_pat_11AAEHOXY0I5in52IE2hcX_...`).

**Note**: Store this token securely and never commit it to your repository. Use environment variables or GitHub Secrets (see GitHub Actions setup).

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/<your-username>/gh-config-cli.git
   cd gh-config-cli
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

The binary will be available at `./target/release/gh-config-cli`.

## Configuration

Define your GitHub organization settings in a YAML file (e.g., `config.yaml`). Example:

```yaml
org: harmony-labs
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
- `repos`: List of repositories with merge settings.
  - `name`: Repository name (e.g., `harmony`).
  - `settings`: Merge options (`allow_merge_commit`, `allow_squash_merge`, `allow_rebase_merge`).
  - `visibility`: Optional, `public` or `private` (defaults to `private` if omitted).
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

```bash
cargo run -- --help
```

- `--config <PATH>`: Path to the YAML config file (default: `config.yaml`).
- `--token <TOKEN>`: GitHub PAT (or set via `GITHUB_TOKEN` env var).
- `--dry-run`: Validate changes without applying them.

### Examples

1. **Apply Changes**:
   ```bash
   cargo run -- --config config.yaml --token <your-pat>
   ```

2. **Dry Run (Validation)**:
   ```bash
   RUST_LOG=info cargo run -- --config config.yaml --token <your-pat> --dry-run
   ```
   Output example:
    ```
    INFO  gh_org_manager: Running in dry-run mode; validating changes without applying.
    INFO  gh_org_manager::github: [Dry Run] Would update harmony-labs/harmony settings: RepoSettings { allow_merge_commit: true, ... } -> RepoSettings { allow_merge_commit: false, ... }
    ```

3. **Using Installed Binary**:
After `cargo install --path .`:
```bash
gh-config-cli --config config.yaml --token <your-pat>
```

## GitHub Actions Integration

Automate validation and application with GitHub Actions. Add this workflow to `.github/workflows/validate-org.yml`:

```yaml
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
     run: ./target/release/gh-config-cli --config config.yaml --token $GITHUB_TOKEN --dry-run

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
     run: ./target/release/gh-config-cli --config config.yaml --token $GITHUB_TOKEN
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