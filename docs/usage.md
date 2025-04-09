# gh-config-cli Usage Guide

This guide provides detailed examples of how to use `gh-config-cli` for managing your GitHub organization.

---

## Authentication

You can provide your GitHub Personal Access Token (PAT) in two ways:

- As a CLI argument: `--token <your-pat>`
- As an environment variable: `GITHUB_TOKEN=<your-pat>`

---

## Commands and Examples

### 1. Show Diff

Compare your local config file with the current GitHub org state.

- Using `--token`:

```bash
gh-config --token <your-pat> diff config.yaml
```

- Using environment variable:

```bash
GITHUB_TOKEN=<your-pat> gh-config diff config.yaml
```

---

### 2. Apply Changes (Sync)

Apply your local config to GitHub, creating/updating repos, teams, permissions, etc.

- Using `--token`:

```bash
gh-config --token <your-pat> sync config.yaml
```

- Using environment variable:

```bash
GITHUB_TOKEN=<your-pat> gh-config sync config.yaml
```

---

### 3. Dry Run (Validation)

Validate your config without making any changes.

- Using `--token`:

```bash
gh-config --token <your-pat> sync config.yaml --dry-run
```

- Using environment variable:

```bash
GITHUB_TOKEN=<your-pat> gh-config sync config.yaml --dry-run
```

---

### 4. Generate Config from GitHub Org

Export your current GitHub org state into a config file.

- Using `--token`:

```bash
gh-config --token <your-pat> sync-from-org config.yaml --org harmony-labs
```

- Using environment variable:

```bash
GITHUB_TOKEN=<your-pat> gh-config sync-from-org config.yaml --org harmony-labs
```

---

## Help

Run:

```bash
gh-config --help
```

to see all available options and commands.

---

## See also

- [Creating a PAT](./pat-setup.md)
- [GitHub Actions examples](./ci-examples.md)
- [Configuration schema](../README.md#configuration)