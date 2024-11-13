#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod convex;
mod errors;
mod code_generator;
mod schema;

use std::path::PathBuf;
use std::time::Instant;
use std::vec;

use convex::{create_functions_ast, create_schema_ast, parse_function_ast, parse_schema_ast};
use errors::ConvexTypeGeneratorError;
use code_generator::CodeGenerator;

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

    let parsed_schema = parse_schema_ast(schema_ast)?;
    let parsed_functions = parse_function_ast(functions_ast)?;

    let generator = CodeGenerator::new(parsed_schema, parsed_functions);
    generator.generate(&config.out_file)?;

    let elapsed = start_time.elapsed();
    println!("Convex Types generated in {}ms", elapsed.as_millis());

    Ok(())
}

fn main()
{
    let config = Configuration {
        schema_path: PathBuf::from("./convex/schema.ts"),
        out_file: "./src/schema.rs".to_string(),
        function_paths: vec![PathBuf::from("./convex/test.ts"), PathBuf::from("./convex/test2.ts")],
    };

    match generate(config) {
        Ok(_) => println!("Types generated successfully"),
        Err(e) => println!("Error generating types: {}", e),
    }
}
