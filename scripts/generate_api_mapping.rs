// scripts/generate_api_mapping.rs

use std::collections::HashMap;
use std::fs;
use serde_json::Value;

// Ensure this definition matches the one in src/api_mapping.rs
#[derive(Debug, Clone)]
pub struct ApiFieldMapping {
    pub resource_type: &'static str,
    pub config_key: &'static str,
    pub endpoint: &'static str,
    pub method: &'static str,
    pub json_path: &'static str,
}

// Helper to resolve $ref pointers (keep as is)
fn resolve_ref<'a>(spec: &'a Value, ref_path: &str) -> &'a Value {
    // ... (keep existing implementation) ...
    let mut parts = ref_path.trim_start_matches("#/").split('/');
    let mut current = spec;
    while let Some(part) = parts.next() {
        // Handle potential URL encoding in parts (e.g., "~1" for "/")
        let decoded_part = part.replace("~1", "/").replace("~0", "~");
        current = current.get(&decoded_part).unwrap_or_else(|| panic!("Failed to resolve ref part: '{}' in path '{}'", decoded_part, ref_path));
    }
    current
}

// Basic inference of resource type from path (keep as is)
fn infer_resource_type(endpoint_path: &str) -> &'static str {
     // Handle {owner} as well as {org}
    if endpoint_path.starts_with("/repos/{owner}/{repo}") || endpoint_path.starts_with("/repos/{org}/{repo}") {
        "repo"
    } else if endpoint_path.starts_with("/orgs/{org}") {
        "org"
    } else if endpoint_path.starts_with("/user") { // Be careful, /user/repos is problematic
        // Distinguish /user operations vs /user/repos create
        if endpoint_path == "/user/repos" {
            "repo_create_user" // Use a distinct type for creation
        } else {
            "user"
        }
    } else if endpoint_path.starts_with("/orgs/{org}/repos") { // Org repo creation
         "repo_create_org"
    } else if endpoint_path.starts_with("/teams") || endpoint_path.contains("/teams/") {
        "team"
    } else if endpoint_path.starts_with("/projects") {
        "project"
    } else if endpoint_path.starts_with("/gists") {
        "gist"
    } else if endpoint_path.starts_with("/enterprises/{") {
        "enterprise"
    } else {
        "other" // Default/unknown
    }
}


fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: generate_api_mapping <path_to_openapi.json>");
        std::process::exit(1);
    }
    let spec_path = &args[1];
    let spec_content = fs::read_to_string(spec_path).expect("Failed to read OpenAPI spec");
    let spec: Value = serde_json::from_str(&spec_content).expect("Failed to parse OpenAPI spec");

    let mut generated_map: HashMap<String, ApiFieldMapping> = HashMap::new();

    // Define the preferred repo update endpoint pattern
    // Note: Using starts_with allows for potential trailing path elements if any exist
    let preferred_repo_update_path_start = "/repos/{owner}/{repo}";
    let preferred_repo_update_method = "PATCH";


    if let Some(paths) = spec.get("paths").and_then(|p| p.as_object()) {
        for (endpoint_path, methods) in paths {
            let endpoint_str: &'static str = Box::leak(endpoint_path.clone().into_boxed_str());
            let resource_type_str = infer_resource_type(endpoint_path);


            for (method, method_obj) in methods.as_object().unwrap() {
                // Only consider methods relevant for *applying* state
                let method_lower = method.to_lowercase();
                 if !["patch", "put", "post"].contains(&method_lower.as_str()) {
                     continue;
                 }
                 let method_upper: &'static str = Box::leak(method.to_uppercase().into_boxed_str());


                if let Some(request_body) = method_obj.get("requestBody") {
                    if let Some(content) = request_body.get("content") {
                        if let Some(app_json) = content.get("application/json") {
                            if let Some(schema) = app_json.get("schema") {
                                let mut schema_obj = schema;
                                if let Some(ref_path) = schema.get("$ref").and_then(|v| v.as_str()) {
                                    schema_obj = resolve_ref(&spec, ref_path);
                                }

                                if let Some(props) = schema_obj.get("properties").and_then(|p| p.as_object()) {
                                    for (field_name, _field_schema) in props {
                                        let config_key_str: &'static str = Box::leak(field_name.clone().into_boxed_str());
                                        let json_path_str: &'static str = config_key_str;

                                        let current_mapping = ApiFieldMapping {
                                             resource_type: resource_type_str, // Use inferred type
                                             config_key: config_key_str,
                                            endpoint: endpoint_str,
                                            method: method_upper,
                                            json_path: json_path_str,
                                        };

                                        // --- Prioritization Logic ---
                                        let should_insert = match generated_map.get(field_name) {
                                            Some(existing_mapping) => {
                                                // Check if the current one is the preferred repo update type
                                                let is_current_preferred = endpoint_path.starts_with(preferred_repo_update_path_start)
                                                                             && method_upper == preferred_repo_update_method;
                                                // Check if the existing one is the preferred repo update type
                                                let is_existing_preferred = existing_mapping.endpoint.starts_with(preferred_repo_update_path_start)
                                                                              && existing_mapping.method == preferred_repo_update_method;

                                                // Prefer the current one ONLY if it's the preferred type AND the existing one is NOT
                                                is_current_preferred && !is_existing_preferred
                                                // OR if neither is preferred, fall back to overwriting (last seen wins among non-preferred)
                                                 || (!is_current_preferred && !is_existing_preferred)
                                            }
                                            None => true, // No existing mapping, always insert
                                        };

                                        if should_insert {
                                             generated_map.insert(field_name.clone(), current_mapping);
                                         }
                                        // --- End Prioritization Logic ---
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }


    // --- Output Generation (Keep as is) ---
    println!("// AUTO-GENERATED FILE. DO NOT EDIT BY HAND.");
    println!("// Please run `make generate-api-mappings` to regenerate.");
    println!("// This generator prioritizes PATCH /repos/{{owner}}/{{repo}} for repo settings."); // Added note
    println!("// WARNING: Collisions for other keys might still use the *last processed* mapping.");
    println!("use std::collections::HashMap;");
    println!("use crate::api_mapping::ApiFieldMapping;");
    println!("pub fn get_github_api_mapping() -> HashMap<&'static str, ApiFieldMapping> {{");
    println!("    let mut map = HashMap::new();");

    let mut sorted_keys: Vec<&String> = generated_map.keys().collect();
    sorted_keys.sort();

    for key in sorted_keys {
        if let Some(mapping) = generated_map.get(key) {
             println!(
                 // Use mapping.endpoint etc. correctly
                "    map.insert(\"{}\", ApiFieldMapping {{ resource_type: \"{}\", config_key: \"{}\", endpoint: \"{}\", method: \"{}\", json_path: \"{}\" }});",
                mapping.config_key.escape_default(),
                mapping.resource_type.escape_default(), // Use mapping's resource_type
                mapping.config_key.escape_default(),
                mapping.endpoint.escape_default(),
                mapping.method.escape_default(),
                mapping.json_path.escape_default()
            );
        }
    }

    println!("    map");
    println!("}}");
}

