# gh-config-cli

[![CI](https://github.com/harmony-labs/gh-config-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/harmony-labs/gh-config-cli/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Releases](https://img.shields.io/github/v/release/harmony-labs/gh-config-cli)](https://github.com/harmony-labs/gh-config-cli/releases)

A fast, declarative CLI tool to manage GitHub organization configuration as code, written in Rust.

---

## Table of Contents

- [Features](#features)
- [Why use gh-config-cli?](#why-use-gh-config-cli)
- [Supported Platforms](#supported-platforms)
- [Installation](#installation)
- [Usage](#usage)
- [Configuration](#configuration)
- [GitHub Actions Integration](#github-actions-integration)
- [Local Development](#local-development)
- [Security Considerations](#security-considerations)
- [Contributing](#contributing)
- [License](#license)
- [Documentation](#documentation)
- [Getting Help](#getting-help)
---

## Features

- **Declarative YAML config** for your entire GitHub org: repos, teams, members, permissions, webhooks, branch protections.
- **Diff mode**: See what would change before applying.
- **Dry run**: Validate changes without modifying anything.
- **Sync from org**: Generate a config file from your current GitHub org state.
- **Cross-platform**: Linux, macOS, Windows.
- **Fast**: Built in Rust.
- **Easy CI integration**: Automate validation and updates.
- **No vendor lock-in**: Open source under MIT license.

---

## Why use gh-config-cli?

Managing GitHub orgs manually or with scattered scripts is error-prone. `gh-config-cli` lets you:

- Version control your org structure.
- Review changes via diffs and pull requests.
- Enforce consistent settings across repos.
- Automate org management in CI/CD.
- Avoid surprises from manual changes.

---

## Supported Platforms

- Linux (x86_64, ARM64)
- macOS (Intel & Apple Silicon)
- Windows

---

## Installation

### Using [UBI](https://github.com/houseabsolute/ubi)

```bash
mkdir -p "$HOME/bin"
curl --silent --location https://raw.githubusercontent.com/houseabsolute/ubi/master/bootstrap/bootstrap-ubi.sh | sh
"$HOME/bin/ubi" --project harmony-labs/gh-config-cli --exe gh-config --in "$HOME/bin"
```

### From GitHub Releases

Download the latest binary for your platform from [Releases](https://github.com/harmony-labs/gh-config-cli/releases), make it executable, and put it in your PATH.

### From Source

Requires Rust stable:

```bash
git clone https://github.com/harmony-labs/gh-config-cli.git
cd gh-config-cli
cargo build --release
./target/release/gh-config --help
```

---

## Usage

### Authentication

- Use a **GitHub Personal Access Token (PAT)** with appropriate permissions (repo admin, org admin).
- Pass via `--token` flag or `GITHUB_TOKEN` environment variable.

### Commands

- **Diff local config with GitHub:**

```bash
gh-config --token <your-pat> diff config.yaml
```

- **Apply config (sync):**

```bash
gh-config --token <your-pat> sync config.yaml
```

- **Dry run (validate):**

```bash
gh-config --token <your-pat> sync config.yaml --dry-run
```

- **Generate config from org:**

```bash
gh-config --token <your-pat> sync-from-org config.yaml --org harmony-labs
```

### Example diff output

```yaml
- allow_merge_commit: true
+ allow_merge_commit: false
```

---

## Configuration

gh-config uses a fully extensible, declarative YAML schema. You can specify any settings supported by the GitHub API, as well as custom fields for automation and policy.

**Example `config.yaml`:**
```yaml
org: harmony-labs
repos:
  - name: harmony
    settings:
      allow_merge_commit: false
      allow_squash_merge: true
      allow_rebase_merge: true
      # Any other GitHub repo setting or custom field
      custom_policy: "enforced"
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

### Defaults and Merging

You can provide a `defaults.config.yaml` file to specify org-wide or policy defaults. All fields are optional. The main config takes precedence over defaults.

**Example `defaults.config.yaml`:**
```yaml
repos:
  - settings:
      allow_merge_commit: true
      custom_policy: "default"
default_webhook:
  url: https://discord.com/api/webhooks/...
  content_type: json
  events: [push, pull_request]
default_branch_protections:
  - pattern: main
    enforce_admins: true
    allow_deletions: false
    allow_force_pushes: false
```

**Merging rules:**
- If a setting exists in both main config and defaults, main config wins.
- If a setting exists only in defaults, it is used unless overridden.
- Custom fields are supported for automation/policy.

**Supports:**
- All GitHub API-manageable settings (see [GitHub REST API docs](https://docs.github.com/en/rest?apiVersion=2022-11-28))
- Org-wide default webhooks
- Repo settings (merge options, visibility, webhooks, etc.)
- Teams and members
- User roles
- Team-repo permissions
- Branch protection rules
- Custom org-wide or policy fields for automation

---

## GitHub Actions Integration

Automate validation and updates.

### Validate on PRs, apply on main branch push

```yaml
permissions:
  contents: write
  pull-requests: write

jobs:
  validate-config:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install UBI
        run: |
          mkdir -p "$HOME/bin"
          curl --silent --location https://raw.githubusercontent.com/houseabsolute/ubi/master/bootstrap/bootstrap-ubi.sh | sh
      - name: Install gh-config
        run: |
          "$HOME/bin/ubi" --project harmony-labs/gh-config-cli --exe gh-config --in "$HOME/bin"
      - name: Validate config
        env:
          GITHUB_TOKEN: ${{ secrets.GH_PAT }}
        run: |
          $HOME/bin/gh-config sync config.yaml --dry-run

  apply-config:
    if: github.ref == 'refs/heads/main' && github.event_name == 'push'
    runs-on: ubuntu-latest
    needs: validate-config
    steps:
      - uses: actions/checkout@v4
      - name: Install UBI
        run: |
          mkdir -p "$HOME/bin"
          curl --silent --location https://raw.githubusercontent.com/houseabsolute/ubi/master/bootstrap/bootstrap-ubi.sh | sh
      - name: Install gh-config
        run: |
          "$HOME/bin/ubi" --project harmony-labs/gh-config-cli --exe gh-config --in "$HOME/bin"
      - name: Apply config
        env:
          GITHUB_TOKEN: ${{ secrets.GH_PAT }}
        run: |
          $HOME/bin/gh-config sync config.yaml
```

---

## Local Development

- Requires Rust stable (`rustup.rs`)
- Clone repo, then:

```bash
cargo build --release
./target/release/gh-config --help
```

- Use `make` targets for common tasks:

```bash
make build
make diff CONFIG_FILE=config.yaml
make sync CONFIG_FILE=config.yaml
make sync-from-github CONFIG_FILE=config.yaml GITHUB_ORG=harmony-labs
```

---

## Security Considerations

- Use fine-grained PATs with least privilege.
- Store tokens in environment variables or GitHub Secrets.
- Never commit secrets to version control.
- Review diffs before applying changes.

---

## Contributing
- Issues and PRs welcome!
- Please file bugs, suggest features, or contribute code.
- See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.
- Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).


---

## License

MIT License. See [LICENSE](LICENSE) for details.

## Documentation

- [Creating a GitHub Personal Access Token (PAT)](docs/pat-setup.md)
- [Usage Guide](docs/usage.md)
- [GitHub Actions CI Examples](docs/ci-examples.md)
- [Development Guide](docs/development.md)
- [Architecture Overview](docs/architecture.md)
- [FAQ](docs/faq.md)
- [Changelog](docs/changelog.md)

More detailed guides and examples can be found in the `docs/` directory.

## Getting Help

- See the [FAQ](docs/faq.md) for common questions.
- Open an issue on [GitHub](https://github.com/harmony-labs/gh-config-cli/issues).
