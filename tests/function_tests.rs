use convex_typegen::errors::ConvexTypeGeneratorError;
use convex_typegen::{Configuration, generate};
use std::fs;
use tempdir::TempDir;

fn setup_test_dir() -> TempDir {
    TempDir::new("convex_typegen_test").expect("Failed to create temp directory")
}

#[test]
fn test_valid_function() {
    let temp_dir = setup_test_dir();
    
    // Create an empty schema file first
    let schema_path = temp_dir.path().join("schema.ts");
    fs::write(&schema_path, r#"
import { defineSchema, defineTable } from "convex/server";
import { v } from "convex/values";

export default defineSchema({
    test: defineTable({
        name: v.string(),
    }),
})
"#).unwrap();
    
    let function_path = temp_dir.path().join("valid_function.ts");
    fs::write(&function_path, r#"
import { query } from "./_generated/server";

export const testQuery = query({
    args: {},
    handler: async (ctx, args) => {},
});
    "#).unwrap();

    let config = Configuration {
        schema_path,
        function_paths: vec![function_path],
        out_file: temp_dir.path().join("types.rs").to_string_lossy().to_string(),
    };

    let result = generate(config);
    assert!(result.is_ok(), "Expected Ok result, got {:?}", result);
}

#[test]
fn test_invalid_function_args() {
    let temp_dir = setup_test_dir();
    
    // Create an empty schema file first
    let schema_path = temp_dir.path().join("schema.ts");
    fs::write(&schema_path, "export default {}").unwrap();
    
    let function_path = temp_dir.path().join("invalid_function.ts");
    fs::write(&function_path, r#"
        export default async function invalidFunction(
            _: { invalidType }  // Invalid type definition
        ) {
            // Function body
        }
    "#).unwrap();

    let config = Configuration {
        schema_path,
        function_paths: vec![function_path],
        out_file: temp_dir.path().join("types.rs").to_string_lossy().to_string(),
    };

    match generate(config) {
        Err(ConvexTypeGeneratorError::InvalidSchema { .. }) => (),
        other => panic!("Expected InvalidSchema error, got {:?}", other),
    }
} 