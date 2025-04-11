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
## Data-Driven API Mapping Layer

To support all possible configurable settings in the GitHub API, gh-config uses a data-driven API mapping table (see `src/api_mapping.rs`). This table describes, for each config field:

- The resource type (e.g., repo, org, team)
- The config key (e.g., `allow_merge_commit`)
- The corresponding GitHub API endpoint (with placeholders for variables)
- The HTTP method (PATCH, PUT, etc.)
- The JSON path for the field in the API payload

The CLI uses this mapping to dynamically build and send API requests for any supported field. This enables:

- Extensible support for new GitHub settings without hardcoding logic for each field
- Easy addition of new fields/endpoints by updating the mapping table
- The potential for automated mapping generation from the GitHub OpenAPI spec

**Extending the mapping:**
- To add support for a new field, add an entry to the mapping table with the correct endpoint, method, and JSON path.
- In the future, the mapping may be moved to an external YAML/JSON file for easier updates and automation.

This approach ensures gh-config can keep pace with changes in the GitHub API and declaratively manage all supported settings.

For more details, see the source code and inline documentation.