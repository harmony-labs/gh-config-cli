# GitHub Actions Workflows for gh-config-cli

Automate validation, syncing, and PR creation for your GitHub org configuration.

---

## Prerequisites

- Store a **fine-grained PAT** with required permissions as a secret named `GH_PAT`.
- The built-in `GITHUB_TOKEN` is used for creating PRs.

---

## Basic Workflow: Validate on PR, Apply on Push

```yaml
name: Validate and Apply GitHub Org Config

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]

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
      - name: Validate Config (Dry Run)
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
      - name: Apply Config
        env:
          GITHUB_TOKEN: ${{ secrets.GH_PAT }}
        run: |
          $HOME/bin/gh-config sync config.yaml
```

---

## Advanced Workflow: Scheduled Diff and Auto-PR

```yaml
name: Check Diff and Create PR

on:
  schedule:
    - cron: '*/15 * * * *' # Every 15 minutes
  workflow_dispatch: # Allow manual trigger
permissions:
  contents: write
  pull-requests: write

jobs:
  diff-and-pr:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install UBI
        run: |
          mkdir -p "$HOME/bin"
          curl --silent --location https://raw.githubusercontent.com/houseabsolute/ubi/master/bootstrap/bootstrap-ubi.sh | sh
      - name: Install gh-config with UBI
        env:
          GITHUB_TOKEN: ${{ secrets.GH_PAT }}
        run: |
          "$HOME/bin/ubi" --project harmony-labs/gh-config-cli --exe gh-config --in "$HOME/bin"
      - name: Run Diff
        id: diff
        env:
          GITHUB_TOKEN: ${{ secrets.GH_PAT }}
          RUST_LOG: info
        run: |
          set +e
          $HOME/bin/gh-config diff config.yaml > diff_output.txt 2>&1
          EXIT_CODE=$?
          cat diff_output.txt
          echo "EXIT_CODE=$EXIT_CODE" >> $GITHUB_ENV
          DIFF_CONTENT="$(cat diff_output.txt)"
          echo "diff<<EOF" >> $GITHUB_OUTPUT
          echo "$DIFF_CONTENT" >> $GITHUB_OUTPUT
          echo "EOF" >> $GITHUB_OUTPUT
          if [ $EXIT_CODE -eq 0 ]; then
            echo "No differences found."
            exit 0
          elif [ $EXIT_CODE -eq 1 ]; then
            echo "Differences found."
            exit 0
          else
            echo "gh-config diff failed with exit code $EXIT_CODE"
            exit $EXIT_CODE
          fi
      - name: Generate New Config
        if: env.EXIT_CODE == 1
        env:
          GITHUB_TOKEN: ${{ secrets.GH_PAT }}
          RUST_LOG: info
        run: |
          $HOME/bin/gh-config sync-from-org config.yaml --org harmony-labs
      - name: Create PR
        if: env.EXIT_CODE == 1
        uses: peter-evans/create-pull-request@v7
        with:
          token: ${{ secrets.GH_PAT }}
          branch: auto-update-config
          title: "Auto-update config.yaml from GitHub state"
          body: |
            Differences found between local config and GitHub state. Updating config.yaml with latest changes.
            Diff output:
            ```
            ${{ steps.diff.outputs.diff }}
            ```
          commit-message: "gh-config sync-from-org detected config.yaml differences from GitHub Org state. Updating to match."
          delete-branch: true
          add-paths: config.yaml
          labels: auto-update
```

---

## Secrets Setup

- Go to **Settings > Secrets and variables > Actions**.
- Add a new secret named `GH_PAT` with your fine-grained PAT.
- The built-in `GITHUB_TOKEN` is used for PR creation.

---

## Notes

- Always review diffs before applying changes.
- Use branch protection rules to safeguard your main branch.
- Schedule the diff workflow as often as needed (daily, hourly, etc).

---

## See also

- [PAT Setup](./pat-setup.md)
- [Usage Guide](./usage.md)
- [Configuration Schema](../README.md#configuration)