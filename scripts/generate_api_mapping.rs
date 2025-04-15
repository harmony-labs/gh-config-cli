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

// Helper to resolve $ref pointers in the OpenAPI spec (keep as is)
fn resolve_ref<'a>(spec: &'a Value, ref_path: &str) -> &'a Value {
    let mut parts = ref_path.trim_start_matches("#/").split('/');
    let mut current = spec;
    while let Some(part) = parts.next() {
        // Handle potential URL encoding in parts (e.g., "~1" for "/")
        let decoded_part = part.replace("~1", "/").replace("~0", "~");
        current = current.get(&decoded_part).unwrap_or_else(|| panic!("Failed to resolve ref part: '{}' in path '{}'", decoded_part, ref_path));
    }
    current
}

// Basic inference of resource type from path
fn infer_resource_type(endpoint_path: &str) -> &'static str {
    if endpoint_path.starts_with("/repos/{") {
        "repo"
    } else if endpoint_path.starts_with("/orgs/{") {
        "org"
    } else if endpoint_path.starts_with("/user") {
        "user"
    } else if endpoint_path.starts_with("/teams") || endpoint_path.contains("/teams/") { // Simplified team check
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

    // Use a HashMap to store the latest mapping found for each config key.
    // This implicitly handles overwriting based on processing order, which isn't ideal
    // but avoids the overly aggressive repo-PATCH prioritization.
    let mut generated_map: HashMap<String, ApiFieldMapping> = HashMap::new();

    if let Some(paths) = spec.get("paths").and_then(|p| p.as_object()) {
        for (endpoint_path, methods) in paths {
            // Convert endpoint_path string literal early and infer resource type
            let endpoint_str: &'static str = Box::leak(endpoint_path.clone().into_boxed_str());
            let resource_type_str = infer_resource_type(endpoint_path);


            for (method, method_obj) in methods.as_object().unwrap() {
                 // Only consider methods relevant for *applying* config state
                 if !["patch", "put", "post"].contains(&method.as_str()) {
                     continue;
                 }
                 // Convert method string literal early
                 let method_upper: &'static str = Box::leak(method.to_uppercase().into_boxed_str());


                if let Some(request_body) = method_obj.get("requestBody") {
                    if let Some(content) = request_body.get("content") {
                        if let Some(app_json) = content.get("application/json") {
                            if let Some(schema) = app_json.get("schema") {
                                let mut schema_obj = schema;
                                // Handle $ref resolution for the main schema
                                if let Some(ref_path) = schema.get("$ref").and_then(|v| v.as_str()) {
                                    schema_obj = resolve_ref(&spec, ref_path);
                                }

                                // Process properties if they exist
                                if let Some(props) = schema_obj.get("properties").and_then(|p| p.as_object()) {
                                    for (field_name, _field_schema) in props {
                                        // Convert field string literal early
                                        let config_key_str: &'static str = Box::leak(field_name.clone().into_boxed_str());
                                        // Assume json_path is same as config_key for now (might need refinement for nested structures)
                                        let json_path_str: &'static str = config_key_str;

                                        let current_mapping = ApiFieldMapping {
                                            resource_type: resource_type_str,
                                            config_key: config_key_str,
                                            endpoint: endpoint_str,
                                            method: method_upper,
                                            json_path: json_path_str,
                                        };

                                        // Simple Overwrite Strategy: Keep the last mapping found for a key.
                                        // This might prioritize org hooks if they appear later in the spec
                                        // than repo hooks for the same key, or vice-versa. It's arbitrary.
                                        // A better strategy needs more context or manual rules.
                                        generated_map.insert(field_name.clone(), current_mapping);

                                    }
                                }
                                // Handle cases where the schema might be a direct $ref to something without properties (less common for request bodies)
                                // or an `allOf` structure, etc. This part might need more sophisticated schema parsing.
                            }
                        }
                    }
                }
            }
        }
    }


    // --- Output Generation ---
    println!("// AUTO-GENERATED FILE. DO NOT EDIT BY HAND.");
    println!("// Please run `make generate-api-mappings` to regenerate.");
    println!("// WARNING: This mapping might contain collisions where multiple API endpoints");
    println!("// use the same configuration key (e.g., 'url' for repo hooks and org hooks).");
    println!("// The generator currently keeps the *last processed* mapping for such keys.");
    println!("// A robust solution requires changes to the mapping structure and application logic.");
    println!("use std::collections::HashMap;");
    println!("use crate::api_mapping::ApiFieldMapping;"); // Adjust path if needed
    println!("pub fn get_github_api_mapping() -> HashMap<&'static str, ApiFieldMapping> {{");
    println!("    let mut map = HashMap::new();");

    // Sort by key for consistent output order
    let mut sorted_keys: Vec<&String> = generated_map.keys().collect();
    sorted_keys.sort();

    for key in sorted_keys {
        if let Some(mapping) = generated_map.get(key) {
             println!(
                "    map.insert(\"{}\", ApiFieldMapping {{ resource_type: \"{}\", config_key: \"{}\", endpoint: \"{}\", method: \"{}\", json_path: \"{}\" }});",
                mapping.config_key.escape_default(), // Use config_key from mapping struct
                mapping.resource_type.escape_default(),
                mapping.config_key.escape_default(),
                mapping.endpoint.escape_default(),
                mapping.method.escape_default(),
                mapping.json_path.escape_default() // Use json_path from mapping struct
            );
        }
    }

    println!("    map");
    println!("}}");
}

