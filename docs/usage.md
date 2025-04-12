# gh-config-cli Usage Guide

This guide provides detailed instructions and examples for using `gh-config-cli` to manage your GitHub organization.

---

## Available Commands

| Command                                 | Description                                                                                  |
|------------------------------------------|----------------------------------------------------------------------------------------------|
| `diff <config.yaml>`                     | Compare your local config file with the current GitHub org state and show differences.       |
| `sync <config.yaml>`                     | Apply your local config to GitHub, creating/updating repos, teams, users, permissions, etc.  |
| `sync <config.yaml> --dry-run`           | Validate your config without making any changes (dry run/preview mode).                      |
| `sync-from-org <config.yaml> [--org <org>]` | Export your current GitHub org state into a config file.                                 |
| `--help`                                 | Show all available options and commands.                                                     |

All commands accept `--token <your-pat>` or the `GITHUB_TOKEN` environment variable for authentication.

---

## Authentication

You can provide your GitHub Personal Access Token (PAT) in two ways:

- As a CLI argument: `--token <your-pat>`
- As an environment variable: `GITHUB_TOKEN=<your-pat>`

---

## Configuration Schema

The configuration file is written in YAML and describes your organization, repositories, teams, users, assignments, and default settings. Below is the schema with all top-level keys and their meanings.

```yaml
org: harmony-labs                # (string) Name of the GitHub organization

repos:                            # (list) Repository configurations
  - name: my-repo                 # (string) Repository name
    settings:                     # (map) Arbitrary repo settings (see below)
      allow_merge_commit: false   # (bool) Example setting
      allow_squash_merge: true
      allow_rebase_merge: true
      # ...any supported GitHub repo setting
    visibility: public            # (string, optional) "public" or "private"
    webhook:                      # (object, optional) Webhook configuration
      url: "http://example.com"   # (string) Webhook endpoint URL
      content_type: "json"        # (string) Payload content type
      events:                     # (list) Events that trigger the webhook
        - push
        - pull_request
    branch_protections:           # (list, optional) Branch protection rules
      - pattern: main             # (string) Branch name or glob pattern
        enforce_admins: true      # (bool)
        allow_deletions: false    # (bool)
        allow_force_pushes: false # (bool)
    # extra:                      # (map, optional) Arbitrary extra fields

teams:                            # (list) Team configurations
  - name: core-team               # (string) Team name
    members:                      # (list) Usernames belonging to the team
      - alice
      - bob
    # privacy: closed             # (string, optional) Team privacy setting

users:                            # (list) User configurations
  - login: alice                  # (string) GitHub username
    role: admin                   # (string) Role in the org ("admin", "member", etc.)

assignments:                      # (list) Team-to-repo permission assignments
  - repo: my-repo                 # (string) Repository name
    team: core-team               # (string) Team name
    permission: admin             # (string) Permission ("admin", "write", "read")

default_webhook:                  # (object, optional) Default webhook for all repos
  url: "http://default.com"
  content_type: "json"
  events:
    - push

default_branch_protections:       # (list, optional) Default branch protection rules
  - pattern: main
    enforce_admins: true
    allow_deletions: false
    allow_force_pushes: false

# extra:                          # (map, optional) Arbitrary extra fields for extensibility
```

### Notes

- All fields marked as "optional" can be omitted.
- The `settings` map under each repo supports any field present in the GitHub API (see [Extensible Schema](#extensible-schema-and-advanced-usage)).
- Extra fields are supported at the top level and within objects for future extensibility.

---

## Commands and Examples

### Show Diff

Compare your local config file with the current GitHub org state.

```bash
gh-config --token <your-pat> diff config.yaml
# or
GITHUB_TOKEN=<your-pat> gh-config diff config.yaml
```

---

### Apply Changes (Sync)

Apply your local config to GitHub, creating/updating repos, teams, permissions, etc.

```bash
gh-config --token <your-pat> sync config.yaml
# or
GITHUB_TOKEN=<your-pat> gh-config sync config.yaml
```

---

### Dry Run (Validation)

Validate your config without making any changes.

```bash
gh-config --token <your-pat> sync config.yaml --dry-run
# or
GITHUB_TOKEN=<your-pat> gh-config sync config.yaml --dry-run
```

---

### Generate Config from GitHub Org

Export your current GitHub org state into a config file.

```bash
gh-config --token <your-pat> sync-from-org config.yaml --org harmony-labs
# or
GITHUB_TOKEN=<your-pat> gh-config sync-from-org config.yaml --org harmony-labs
```

---

## Extensible Schema and Advanced Usage

gh-config supports a fully extensible configuration schema. You can declaratively manage any setting supported by the GitHub API—just add the field to your config using the correct config key.

- The tool uses a mapping generated from the GitHub OpenAPI spec to translate config keys to API endpoints and payloads.
- To add a new setting, just add the field to your config. If it's supported by the GitHub API and present in the mapping, it will be managed automatically.
- To update the mapping, run `make update-github-api-mappings` to fetch the latest API spec and regenerate the mapping.

### Troubleshooting

- If a field is not applied, check that the config key matches the GitHub API field name.
- If a field is not present in the mapping, update the mapping as described above.
- For advanced troubleshooting, see [Architecture Overview](./architecture.md).

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