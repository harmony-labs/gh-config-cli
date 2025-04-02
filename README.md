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

- A GitHub Personal Access Token (PAT) with specific fine-grained permissions (see below).

### Creating a GitHub Personal Access Token

`gh-config` requires a fine-grained PAT with permissions to manage your organization and repositories. Follow these steps to create one:

1. **Log in to GitHub** and go to **Settings > Developer settings > Personal access tokens > Fine-grained tokens**.
2. Click **Generate new token**.
3. **Token Name**: Enter a descriptive name (e.g., `gh-config-cli-token`).
4. **Expiration**: Choose a suitable expiration date (e.g., 90 days or no expiration, depending on your security policy).
5. **Repository Access**:
   - Select **Only select repositories** if you want to limit access, then choose all repositories listed in your `config.yaml`.
   - Alternatively, select **All repositories** under your organization for broader access.
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

## Downloading and Using Locally

### Using ubi

1. **Install ubi:**  
   Ensure you have ubi installed by running:
   ```bash
   mkdir -p ~/.ubi/bin
   echo '\nexport PATH="$HOME/.ubi/bin:$PATH"' >> ~/.zshrc  # or your preferred shell profile
   ```
2. **Install vnext with ubi:**  
   ```bash
   ubi --project harmony-labs/gh-config-cli --exe gh-config --in ~/.ubi/bin
   ```

### Installation from GitHub Releases

1. **Download the Binary**:
   - Visit the [Releases page](https://github.com/harmony-labs/gh-config-cli/releases).
   - Find the latest release (e.g., `v0.1.0`) and download the `gh-config` binary for your platform:
     - Linux/macOS: `gh-config` (e.g., `gh-config-v0.1.0-linux` or `gh-config-v0.1.0-macos`).
     - Windows: `gh-config.exe` (e.g., `gh-config-v0.1.0-windows.exe`).

2. **Make it Executable (Linux/macOS)**:
   ```bash
   chmod +x gh-config
   ```

3. **Move to a Directory in Your PATH** (Optional, for convenience):
   - Linux/macOS:
     ```bash
     sudo mv gh-config /usr/local/bin/
     ```
   - Windows: Move `gh-config.exe` to a directory in your PATH (e.g., `C:\Program Files\gh-config`), then add it to your system PATH via environment variables.

4. **Verify Installation**:
   ```bash
   gh-config --version
   ```
   Expected output: `gh-config-cli 0.1.0` (or the downloaded version).

### Usage Examples

1. **Show Diff**:
   - Using `--token`:
     ```bash
     gh-config --token <your-pat> diff config.yaml
     ```
   - Using `GITHUB_TOKEN` env var:
     ```bash
     GITHUB_TOKEN=<your-pat> gh-config diff config.yaml
     ```

2. **Apply Changes**:
   - Using `--token`:
     ```bash
     gh-config --token <your-pat> sync config.yaml
     ```
   - Using `GITHUB_TOKEN` env var:
     ```bash
     GITHUB_TOKEN=<your-pat> gh-config sync config.yaml
     ```

3. **Dry Run (Validation)**:
   - Using `--token`:
     ```bash
     gh-config --token <your-pat> sync config.yaml --dry-run
     ```
   - Using `GITHUB_TOKEN` env var:
     ```bash
     GITHUB_TOKEN=<your-pat> gh-config sync config.yaml --dry-run
     ```

4. **Generate Config from GitHub**:
   - Using `--token`:
     ```bash
     gh-config --token <your-pat> sync-from-org config.yaml --org harmony-labs
     ```
   - Using `GITHUB_TOKEN` env var:
     ```bash
     GITHUB_TOKEN=<your-pat> gh-config sync-from-org config.yaml --org harmony-labs
     ```

## Local Development

For contributors or those who prefer building from source:

### Prerequisites

- Rust (stable) installed (see [rustup.rs](https://rustup.rs/)).

### Setup

1. **Clone the Repository**:
   ```bash
   git clone https://github.com/harmony-labs/gh-config-cli.git
   cd gh-config-cli
   ```

2. **Build the Project**:
   ```bash
   cargo build --release
   ```
   The binary will be at `./target/release/gh-config`.

### Running Locally

Use `make` for convenience during development:

- **Build**:
   ```bash
   make build
   ```

- **Show Diff**:
   ```bash
   GITHUB_TOKEN=<your-pat> make diff CONFIG_FILE=config.yaml
   ```

- **Dry Run**:
   ```bash
   GITHUB_TOKEN=<your-pat> make dry-run CONFIG_FILE=config.yaml
   ```

- **Sync**:
   ```bash
   GITHUB_TOKEN=<your-pat> make sync CONFIG_FILE=config.yaml
   ```

- **Generate Config from GitHub**:
   ```bash
   GITHUB_TOKEN=<your-pat> make sync-from-github CONFIG_FILE=config.yaml GITHUB_ORG=harmony-labs
   ```

- **Help**:
   ```bash
   make help
   ```

**Note**: `make` is only for local development of `gh-config-cli`. For using the tool in your projects, use the released binary directly (see above).

## Using in GitHub Actions

Automate validation and application with GitHub Actions using the released binary.

### Basic Example: Validate and Apply Config

Add this workflow to `.github/workflows/validate-org.yml` to validate on PRs and apply on push to `main`:

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
      - name: Download gh-config Binary
        run: |
          curl -L -o gh-config https://github.com/harmony-labs/gh-config-cli/releases/latest/download/gh-config
          chmod +x gh-config
          sudo mv gh-config /usr/local/bin/
      - name: Validate Config (Dry Run)
        env:
          GITHUB_TOKEN: ${{ secrets.GH_PAT }}
          RUST_LOG: info
        run: gh-config --token $GITHUB_TOKEN sync config.yaml --dry-run

  apply-config:
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    needs: validate-config
    steps:
      - uses: actions/checkout@v4
      - name: Download gh-config Binary
        run: |
          curl -L -o gh-config https://github.com/harmony-labs/gh-config-cli/releases/latest/download/gh-config
          chmod +x gh-config
          sudo mv gh-config /usr/local/bin/
      - name: Apply Config
        env:
          GITHUB_TOKEN: ${{ secrets.GH_PAT }}
          RUST_LOG: info
        run: gh-config --token $GITHUB_TOKEN sync config.yaml
```

### Advanced Example: Scheduled Diff and PR Creation

Add this workflow to `.github/workflows/diff-and-pr.yml` to check for differences daily and create a PR if needed:

```yaml
name: Check Diff and Create PR

on:
  schedule:
    - cron: '0 0 * * *' # Daily at midnight UTC
  workflow_dispatch: # Allow manual trigger

jobs:
  diff-and-pr:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Download gh-config
        run: |
          curl -L -o gh-config https://github.com/harmony-labs/gh-config-cli/releases/latest/download/gh-config
          chmod +x gh-config
      - name: Run Diff
        id: diff
        env:
          GITHUB_TOKEN: ${{ secrets.GH_PAT }}
          RUST_LOG: info
        run: |
          ./gh-config diff config.yaml > diff_output.txt 2>&1
          echo "EXIT_CODE=$?" >> $GITHUB_ENV
      - name: Generate New Config
        if: env.EXIT_CODE == 1
        env:
          GITHUB_TOKEN: ${{ secrets.GH_PAT }}
          RUST_LOG: info
        run: |
          ./gh-config sync-from-org config.yaml.new --org harmony-labs
      - name: Create PR
        if: env.EXIT_CODE == 1
        uses: peter-evans/create-pull-request@v5
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
          branch: auto-update-config-${{ github.run_id }}
          title: "Auto-update config.yaml from GitHub state"
          body: |
            Differences found between local config and GitHub state. Updating config.yaml with latest changes.
            Diff output:
            ```
            $(cat diff_output.txt)
            ```
          commit-message: "Update config.yaml from GitHub state"
          delete-branch: true
          add-paths: config.yaml.new
          labels: auto-update
```

### Setup for Both Workflows

1. **Store PAT in GitHub Secrets**:
   - Go to your repository’s **Settings > Secrets and variables > Actions**.
   - Click **New repository secret**.
   - Name it `GH_PAT` and paste your PAT value.

2. **Workflow Behavior**:
   - **Basic Example**: Validates on PRs with `--dry-run` and applies on push to `main`.
   - **Advanced Example**: Runs daily (or manually), checks for differences, and creates a PR with the updated config if differences are found.

**Note**: Both workflows use the released binary from GitHub Releases, ensuring consistency. The advanced example requires the `peter-evans/create-pull-request` action.

## Configuration

Define your GitHub organization settings in a YAML file (e.g., `config.yaml`). Example:

```yaml
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

## Command-Line Options

Run `gh-config --help` to see all options:

- `--config <PATH>`: Path to the YAML config file (default: `config.yaml`).
- `--token <TOKEN>`: GitHub PAT (or set via `GITHUB_TOKEN` env var).
- `--dry-run`: Validate changes without applying them (used with `sync` or `sync-from-org`).
- `--sync-from-org <ORG>`: Generate a config file from the specified GitHub org.
- `--diff`: Show a line-numbered diff between the local config and GitHub state.

## Contributing

- Report issues or suggest features via GitHub Issues.
- Submit pull requests with improvements.

## License

MIT License. See [LICENSE](LICENSE) for details.