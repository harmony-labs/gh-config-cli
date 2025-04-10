# Frequently Asked Questions (FAQ)

## What is gh-config?

A fast, declarative CLI tool to manage GitHub organization configuration as code, written in Rust.

## How do I install gh-config?

See the [Installation section in the README](../README.md#installation) for instructions using UBI, GitHub Releases, or building from source.

## What permissions does my GitHub PAT need?

Your Personal Access Token (PAT) should have:
- `admin:org`
- `repo`
- `read:org`
- `write:org`
- Any other permissions required for your org's specific needs

See [docs/pat-setup.md](pat-setup.md) for details.

## How do I validate my config before applying changes?

Use the `--dry-run` flag:
```bash
gh-config --token <your-pat> sync config.yaml --dry-run
```

## How do I generate a config file from my current GitHub org?

Use:
```bash
gh-config --token <your-pat> sync-from-org config.yaml --org <your-org>
```

## Can I use gh-config in CI/CD?

Yes! See [docs/ci-examples.md](ci-examples.md) for GitHub Actions integration examples.

## Where can I find the full config schema?

See [docs/usage.md](usage.md) and [docs/architecture.md](architecture.md) for schema and code structure.

## How do I report a bug or request a feature?

Open an issue on [GitHub](https://github.com/harmony-labs/gh-config-cli/issues).

## How do I contribute?

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## Where can I get help?

- Check the documentation in the `docs/` directory.
- Open an issue on GitHub.

---
If your question isn't answered here, please open an issue!