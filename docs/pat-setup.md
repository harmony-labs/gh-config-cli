# Creating a GitHub Personal Access Token (PAT) for gh-config-cli

`gh-config-cli` requires a fine-grained PAT with permissions to manage your organization and repositories.

---

## How to create a fine-grained PAT

1. **Log in to GitHub** and go to  
   **Settings > Developer settings > Personal access tokens > Fine-grained tokens**.

2. Click **Generate new token**.

3. **Token Name**: Enter a descriptive name, e.g., `gh-config-cli-token`.

4. **Expiration**: Choose a suitable expiration date (e.g., 90 days or no expiration).

5. **Repository Access**:
   - Select **Only select repositories** and choose all repos managed by your config,  
     or select **All repositories** under your organization.

6. **Permissions**:

### Repository Permissions

| Permission       | Access Level | Why                                         |
|------------------|--------------|----------------------------------------------|
| Administration   | Read & Write | Update repo settings, merge options, visibility |
| Contents         | Read-only    | Fetch repo details                          |
| Webhooks         | Read & Write | Manage webhooks                             |

### Organization Permissions

| Permission       | Access Level | Why                                         |
|------------------|--------------|----------------------------------------------|
| Administration   | Read & Write | Manage org settings, teams                  |
| Members          | Read & Write | Manage org memberships and roles            |
| Webhooks         | Read & Write | Manage org-level webhooks                   |

7. Click **Generate token** and **copy** the token (e.g., `github_pat_...`).

---

## Usage

- **Never commit your PAT** to version control.
- Store it securely, e.g., in environment variables or GitHub Secrets.
- Pass it to `gh-config` via:
  - `--token <your-pat>` argument, or
  - `GITHUB_TOKEN=<your-pat>` environment variable.

---

## Example usage

```bash
gh-config --token <your-pat> diff config.yaml
# or
GITHUB_TOKEN=<your-pat> gh-config diff config.yaml
```

---

## More info

- [GitHub fine-grained PAT docs](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token)