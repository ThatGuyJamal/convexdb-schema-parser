#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod convex;
mod errors;

use std::path::PathBuf;
use std::time::Instant;
use std::vec;

use convex::{create_functions_ast, create_schema_ast, parse_function_ast, parse_schema_ast};
use errors::ConvexTypeGeneratorError;

/// Configuration for the type generator
pub struct Configuration
{
    /// The path to the schema.ts file
    pub schema_path: PathBuf,
    /// The output file for the generated types
    pub out_file: String,
    /// The paths to the function files
    pub function_paths: Vec<PathBuf>,
}

impl Default for Configuration
{
    fn default() -> Self
    {
        Self {
            schema_path: PathBuf::from("./convex/schema.ts"),
            out_file: "schema.rs".to_string(),
            function_paths: Vec::new(),
        }
    }
}

/// Main entrypoint for the type generator
pub fn generate(config: Configuration) -> Result<(), ConvexTypeGeneratorError>
{
    let start_time = Instant::now();

    let schema_path = match config.schema_path.canonicalize() {
        Ok(path) => path,
        Err(_) => return Err(ConvexTypeGeneratorError::MissingSchemaFile),
    };

    let schema_ast = create_schema_ast(schema_path)?;
    let functions_ast = create_functions_ast(config.function_paths)?;

    // std::fs::write("./debug/schema_ast.json", serde_json::to_string_pretty(&schema_ast).unwrap())
    //     .map_err(ConvexTypeGeneratorError::IOError)?;

    // std::fs::write(
    //     "./debug/functions_ast.json",
    //     serde_json::to_string_pretty(&functions_ast).unwrap(),
    // )
    // .map_err(ConvexTypeGeneratorError::IOError)?;

    let parsed_schema = parse_schema_ast(schema_ast)?;
    let parsed_functions = parse_function_ast(functions_ast)?;

    println!("--------------------------------");
    println!("{:?}", parsed_schema);
    println!("--------------------------------");
    println!("{:?}", parsed_functions);
    println!("--------------------------------");

    let elapsed = start_time.elapsed();

    println!("Convex Types generated in {}ms", elapsed.as_millis());

    Ok(())
}

fn main()
{
    let config = Configuration {
        schema_path: PathBuf::from("./convex/schema.ts"),
        out_file: "schema.rs".to_string(),
        function_paths: vec![PathBuf::from("./convex/test.ts"), PathBuf::from("./convex/test2.ts")],
    };

    match generate(config) {
        Ok(_) => println!("Types generated successfully"),
        Err(e) => println!("Error generating types: {}", e),
    }
}
