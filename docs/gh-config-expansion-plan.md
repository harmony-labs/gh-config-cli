# gh-config Expansion Technical Plan

## Objective

Expand `gh-config` to:
1. Support all settings manageable by the GitHub REST API.
2. Operate in a fully declarative, deterministic way—only update settings that differ from the source of truth.
3. Add support for a `defaults.config.yaml` file for org-wide/policy defaults, merged with clear precedence.

---

## 1. Schema Changes

### a. Extensible Config Schema

- Replace the current fixed Rust structs with a more flexible, data-driven schema.
- Use `serde_yaml::Value` or custom enums/maps for arbitrary key-value settings.
- For each resource (org, repo, team, etc.), allow a `settings` map/object that can include any supported field.
- Example:
  ```yaml
  repos:
    - name: my-repo
      settings:
        allow_merge_commit: true
        allow_auto_merge: true
        delete_branch_on_merge: true
        # ...any other GitHub repo setting
      custom_policies:
        my_policy: value
  ```

### b. Support for `defaults.config.yaml`

- New file: `defaults.config.yaml`, with the same schema as the main config, all fields optional.
- Add support for loading and merging defaults with the main config, with clear precedence (main config > defaults).
- Allow `defaults` to include custom fields for automation/policy (e.g., default webhooks, branch protections).

**Precedence Rule:**
- If a setting exists in both main config and defaults, main config wins.
- If a setting exists only in defaults, it is used unless overridden.

---

## 2. Data Flow and Merging Logic

### a. Loading and Merging

```
Load defaults.config.yaml
        ↓
Load main config.yaml
        ↓
Merge defaults into main config (main config takes precedence)
        ↓
Resulting effective config
```

- Implement a `merge_with_defaults` function in `config.rs` that recursively merges defaults into the main config.
- For each resource (repo, team, etc.), merge settings and custom fields.
- Expose the merged config to the rest of the tool.

### b. Handling Custom/Non-API Defaults

- For fields in defaults that do not map to GitHub, expose them for use in automation/policy enforcement (e.g., as part of a `custom_policies` map).

---

## 3. Diff/Apply Logic

### a. Generic Diffing

- Refactor diff logic in `github.rs` to:
  - Iterate over all settings in the effective config.
  - For each setting, fetch the current value from GitHub (using a generic API client).
  - Compare the config value to the cloud value.
  - Only apply changes where the values differ.

**Example Pseudocode:**
```rust
for resource in config.repos {
    for (setting, value) in resource.settings {
        let current = github_api.get_setting(resource.name, setting);
        if current != value {
            github_api.update_setting(resource.name, setting, value);
        }
    }
}
```

- For settings not supported by the API, skip or log as "custom/policy only".

### b. Deterministic, Declarative Operation

- Ensure that:
  - No setting is updated unless it differs from the source of truth (config or cloud).
  - Sync-from-GitHub-to-config works the same way: only update config if cloud differs.

---

## 4. Refactoring

### a. API Abstraction

- Refactor `github.rs` to:
  - Use a data-driven approach for API endpoints and payloads (e.g., endpoint templates, dynamic JSON construction).
  - Add a mapping layer for config keys to API endpoints/fields.
  - Support new settings/endpoints by updating a mapping table, not by writing new Rust code for each.

### b. Backward Compatibility

- Backward compatibility is not required.

---

## 5. Testing and Documentation

### a. Testing

- Add/expand unit tests for:
  - Schema parsing and merging (including edge cases for defaults).
  - Diff logic for arbitrary settings.
  - End-to-end tests for new settings and defaults.

### b. Documentation

- Update `README.md` and `docs/`:
  - Document the new schema, including extensibility and defaults.
  - Provide migration instructions and examples.
  - Document precedence and merging rules.

---

## Summary Table

| Area                | Change/Addition                                                                 |
|---------------------|--------------------------------------------------------------------------------|
| Config Schema       | Make extensible, support arbitrary settings, add settings maps                  |
| Defaults Support    | Add `defaults.config.yaml`, merge with main config, clear precedence            |
| Diff/Apply Logic    | Generic, data-driven diff/apply for all settings, deterministic operation       |
| API Abstraction     | Refactor to support dynamic endpoints/fields, mapping layer for config <-> API  |
| Testing             | Expand for new schema, merging, diff logic, end-to-end                         |
| Documentation       | Update for new schema, defaults, migration, usage                              |