# Architecture Overview

This document provides a high-level overview of the gh-config CLI codebase and its main components.

## Project Structure

```
gh-config-cli/
├── src/
│   ├── main.rs         # CLI entry point and argument parsing
│   ├── config.rs       # Configuration file parsing and schema
│   ├── github.rs       # GitHub API integration and logic
│   ├── error.rs        # Error types and handling
├── docs/               # Documentation
├── Makefile            # Build and test shortcuts
├── Cargo.toml          # Rust package manifest
├── README.md           # Project overview
```

## Main Modules

- **main.rs**
  - Handles CLI argument parsing (using `clap` or similar).
  - Dispatches commands (diff, sync, sync-from-org, etc.).
  - Initializes logging and error handling.

- **config.rs**
  - Defines the schema for the YAML configuration file.
  - Handles loading, parsing, and validating config files.
  - Provides helpers for diffing and applying config.

- **github.rs**
  - Contains logic for interacting with the GitHub API.
  - Handles authentication, org/repo/team/user management.
  - Implements diff, sync, and sync-from-org operations.

- **error.rs**
  - Defines custom error types.
  - Implements error conversions and reporting.

## Data Flow

1. **User runs a CLI command** (e.g., `gh-config diff config.yaml`).
2. **main.rs** parses arguments and loads the config file via **config.rs**.
3. The appropriate operation is performed by calling functions in **github.rs**.
4. Results (diffs, errors, etc.) are printed to the terminal.

## Extending the CLI

- Add new commands or flags in `main.rs`.
- Extend the config schema in `config.rs`.
- Add new GitHub operations in `github.rs`.

## Error Handling

- All errors are handled via custom types in `error.rs` and surfaced to the user with clear messages.

## Testing

- Unit and integration tests are located alongside source files.
- Run all tests with `cargo test`.

---
For more details, see the source code and inline documentation.