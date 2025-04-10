# Development Guide

This guide will help you set up a local development environment for gh-config, build the project, run tests, and contribute changes.

## Prerequisites

- [Rust (stable)](https://rustup.rs/)
- Git
- Make (optional, for convenience)

## Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/harmony-labs/gh-config-cli.git
   cd gh-config-cli
   ```

2. **Install Rust (if not already installed):**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Build the project:**
   ```bash
   cargo build --release
   ```

4. **Run tests:**
   ```bash
   cargo test
   ```

5. **Lint and format:**
   ```bash
   cargo fmt -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   ```

## Makefile Shortcuts

The Makefile provides shortcuts for common tasks:

```bash
make build
make diff CONFIG_FILE=config.yaml
make sync CONFIG_FILE=config.yaml
make sync-from-github CONFIG_FILE=config.yaml GITHUB_ORG=harmony-labs
```

## Running the CLI

After building, run the CLI with:

```bash
./target/release/gh-config --help
```

## Submitting Changes

- Follow the [contribution guidelines](../CONTRIBUTING.md).
- Write tests for new features or bug fixes.
- Ensure all tests pass and code is formatted.

---
Happy hacking!