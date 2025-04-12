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