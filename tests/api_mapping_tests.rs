use gh_config_cli::api_mapping_generated::get_github_api_mapping;

#[test]
fn test_mapping_contains_known_repo_fields() {
    let map = get_github_api_mapping();
    let amc = map.get("allow_merge_commit").expect("mapping for allow_merge_commit");
    assert!(amc.endpoint.contains("/repos/") || amc.endpoint.contains("{repo}"));
    assert_eq!(amc.method, "PATCH");
    assert_eq!(amc.json_path, "allow_merge_commit");
}

#[test]
fn test_mapping_contains_org_field() {
    let map = get_github_api_mapping();
    // Example: check for a common org field, e.g., "billing_email"
    if let Some(org_field) = map.get("billing_email") {
        assert!(org_field.endpoint.contains("/orgs/") || org_field.endpoint.contains("{org}"));
    }
}

#[test]
fn test_unmapped_field_returns_none() {
    let map = get_github_api_mapping();
    assert!(map.get("this_field_does_not_exist").is_none());
}

use gh_config_cli::api_mapping::get_repo_settings_mapping;

#[test]
fn test_repo_settings_mapping_contains_expected_keys() {
    let map = get_repo_settings_mapping();
    let keys = ["allow_merge_commit", "allow_squash_merge", "allow_rebase_merge"];
    for key in &keys {
        assert!(map.contains_key(key), "Missing key: {}", key);
    }
}

#[test]
fn test_repo_settings_mapping_fields_are_correct() {
    let map = get_repo_settings_mapping();

    let amc = map.get("allow_merge_commit").expect("mapping for allow_merge_commit");
    assert_eq!(amc.resource_type, "repo");
    assert_eq!(amc.config_key, "allow_merge_commit");
    assert_eq!(amc.endpoint, "/repos/{org}/{repo}");
    assert_eq!(amc.method, "PATCH");
    assert_eq!(amc.json_path, "allow_merge_commit");

    let asm = map.get("allow_squash_merge").expect("mapping for allow_squash_merge");
    assert_eq!(asm.resource_type, "repo");
    assert_eq!(asm.config_key, "allow_squash_merge");
    assert_eq!(asm.endpoint, "/repos/{org}/{repo}");
    assert_eq!(asm.method, "PATCH");
    assert_eq!(asm.json_path, "allow_squash_merge");

    let arm = map.get("allow_rebase_merge").expect("mapping for allow_rebase_merge");
    assert_eq!(arm.resource_type, "repo");
    assert_eq!(arm.config_key, "allow_rebase_merge");
    assert_eq!(arm.endpoint, "/repos/{org}/{repo}");
    assert_eq!(arm.method, "PATCH");
    assert_eq!(arm.json_path, "allow_rebase_merge");
}

#[test]
fn test_repo_settings_mapping_unmapped_key_returns_none() {
    let map = get_repo_settings_mapping();
    assert!(map.get("this_key_does_not_exist").is_none());
}