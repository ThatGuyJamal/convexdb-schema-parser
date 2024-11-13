#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod convex;
mod errors;

use std::path::PathBuf;
use std::time::Instant;

use convex::create_schema_ast;
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

    let ast = create_schema_ast(schema_path)?;

    // write this ast to a json file
    std::fs::write("./debug/ast.json", serde_json::to_string_pretty(&ast).unwrap())
        .map_err(ConvexTypeGeneratorError::IOError)?;

    let elapsed = start_time.elapsed();

    println!("Convex Types generated in {}ms", elapsed.as_millis());

    Ok(())
}

fn main()
{
    match generate(Configuration::default()) {
        Ok(_) => println!("Types generated successfully"),
        Err(e) => println!("Error generating types: {}", e),
    }
}
