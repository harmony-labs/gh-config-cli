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

## Extensible Schema and Advanced Usage

gh-config now supports a fully extensible configuration schema. You can declaratively manage any setting supported by the GitHub API—just add the field to your config using the correct config key.

### Adding New Repo Settings

You can add any supported repo setting under `settings:` for each repo:

```yaml
repos:
  - name: my-repo
    settings:
      allow_merge_commit: false
      allow_squash_merge: true
      allow_rebase_merge: true
      delete_branch_on_merge: true   # New field, just add it!
      has_issues: true               # Another new field
    visibility: public
```

### Org-Level and Team-Level Settings

You can add org-level or team-level settings using the config key as defined in the GitHub API:

```yaml
org: harmony-labs
org_settings:
  billing_email: "admin@harmony.com"
  default_repository_permission: "read"
  members_can_create_repositories: false

teams:
  - name: core-team
    members:
      - alice
      - bob
    privacy: closed   # Example team setting
```

### How It Works

- The tool uses a mapping generated from the GitHub OpenAPI spec to translate config keys to API endpoints and payloads.
- To add a new setting, just add the field to your config. If it's supported by the GitHub API and present in the mapping, it will be managed automatically.
- To update the mapping, run `make update-github-api-mappings` to fetch the latest API spec and regenerate the mapping.

### Troubleshooting

- If a field is not applied, check that the config key matches the GitHub API field name.
- If a field is not present in the mapping, update the mapping as described above.
- For advanced troubleshooting, see [Architecture Overview](./architecture.md).

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
- [Architecture Overview](./architecture.md)