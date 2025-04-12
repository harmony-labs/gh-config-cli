/*
    Script: generate_api_mapping.rs

    This script parses the GitHub OpenAPI (Swagger) spec and generates a Rust mapping table
    for all configurable fields, endpoints, and methods. The output can be used in src/api_mapping.rs
    to enable dynamic, data-driven API coverage for gh-config.

    Usage:
        1. Download the GitHub OpenAPI spec (YAML or JSON) from:
           https://github.com/github/rest-api-description
        2. Run this script with the path to the spec file.
        3. The script outputs Rust code for the mapping table.
*/

use std::fs;
use serde_json::Value;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: generate_api_mapping <path_to_openapi.json>");
        std::process::exit(1);
    }
    let spec_path = &args[1];
    let spec_content = fs::read_to_string(spec_path).expect("Failed to read OpenAPI spec");
    let spec: Value = serde_json::from_str(&spec_content).expect("Failed to parse OpenAPI spec");

    println!("// AUTO-GENERATED FILE. DO NOT EDIT BY HAND.");
    println!("use std::collections::HashMap;");
    println!("use crate::api_mapping::ApiFieldMapping;");
    println!("pub fn get_github_api_mapping() -> HashMap<&'static str, ApiFieldMapping> {{");
    println!("    let mut map = HashMap::new();");

    if let Some(paths) = spec.get("paths") {
        for (endpoint, methods) in paths.as_object().unwrap() {
            for method in ["patch", "put", "post"] {
                if let Some(method_obj) = methods.get(method) {
                    // Only process endpoints with a requestBody schema
                    if let Some(request_body) = method_obj.get("requestBody") {
                        if let Some(content) = request_body.get("content") {
                            if let Some(app_json) = content.get("application/json") {
                                if let Some(schema) = app_json.get("schema") {
                                    // Handle $ref indirection
                                    let mut schema_obj = schema;
                                    if let Some(ref_path) = schema.get("$ref").and_then(|v| v.as_str()) {
                                        schema_obj = resolve_ref(&spec, ref_path);
                                    }
                                    if let Some(props) = schema_obj.get("properties") {
                                        for (field, _field_schema) in props.as_object().unwrap() {
                                            // Output a mapping entry
                                            println!(
                                                "    map.insert(\"{field}\", ApiFieldMapping {{ resource_type: \"auto\", config_key: \"{field}\", endpoint: \"{endpoint}\", method: \"{}\", json_path: \"{field}\" }});",
                                                method.to_uppercase()
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    println!("    map");
    println!("}}");

    // Helper to resolve $ref pointers in the OpenAPI spec
    fn resolve_ref<'a>(spec: &'a Value, ref_path: &str) -> &'a Value {
        let mut parts = ref_path.trim_start_matches("#/").split('/');
        let mut current = spec;
        while let Some(part) = parts.next() {
            current = &current[part];
        }
        current
    }
}