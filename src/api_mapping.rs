use std::collections::HashMap;

/// Describes how to map a config field to a GitHub API call.
#[derive(Debug, Clone)]
pub struct ApiFieldMapping {
    #[allow(dead_code)]
    pub resource_type: &'static str, // e.g., "repo", "org", "team"
    #[allow(dead_code)]
    pub config_key: &'static str,    // e.g., "allow_merge_commit"
    pub endpoint: &'static str,      // e.g., "/repos/{org}/{repo}"
    pub method: &'static str,        // "PATCH", "PUT", etc.
    pub json_path: &'static str,     // e.g., "allow_merge_commit"
    // Add more fields as needed for transformation, etc.
}

/// Returns a mapping table for repo settings as a proof of concept.
/// In a full implementation, this would be generated from the GitHub OpenAPI spec or maintained as a comprehensive table.
#[allow(dead_code)]
pub fn get_repo_settings_mapping() -> HashMap<&'static str, ApiFieldMapping> {
    let mut map = HashMap::new();
    map.insert("allow_merge_commit", ApiFieldMapping {
        resource_type: "repo",
        config_key: "allow_merge_commit",
        endpoint: "/repos/{org}/{repo}",
        method: "PATCH",
        json_path: "allow_merge_commit",
    });
    map.insert("allow_squash_merge", ApiFieldMapping {
        resource_type: "repo",
        config_key: "allow_squash_merge",
        endpoint: "/repos/{org}/{repo}",
        method: "PATCH",
        json_path: "allow_squash_merge",
    });
    map.insert("allow_rebase_merge", ApiFieldMapping {
        resource_type: "repo",
        config_key: "allow_rebase_merge",
        endpoint: "/repos/{org}/{repo}",
        method: "PATCH",
        json_path: "allow_rebase_merge",
    });
    // Add more fields as needed...
    map
}