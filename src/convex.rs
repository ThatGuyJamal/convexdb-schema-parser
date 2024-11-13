use std::collections::HashMap;
use std::path::PathBuf;

use oxc::allocator::Allocator;
use oxc::diagnostics::OxcDiagnostic;
use oxc::parser::Parser;
use oxc::semantic::SemanticBuilder;
use oxc::span::SourceType;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

use crate::errors::ConvexTypeGeneratorError;

/// The convex schema.
///
/// A schema can contain many tables. https://docs.convex.dev/database/schemas
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ConvexSchema
{
    pub(crate) tables: Vec<ConvexTable>,
}

/// A table in the convex schema.
///
/// A table can contain many columns.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ConvexTable
{
    /// The name of the table.
    pub(crate) name: String,
    /// The columns in the table.
    pub(crate) columns: Vec<ConvexColumn>,
}

/// A column in the convex schema.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ConvexColumn
{
    /// The name of the column.
    pub(crate) name: String,
    /// The data type of the column.
    /// https://docs.rs/convex/latest/convex/enum.Value.html
    pub(crate) data_type: JsonValue,
}

/// A collection of all convex functions.
pub(crate) type ConvexFunctions = Vec<ConvexFunction>;

/// Convex functions (Queries, Mutations, and Actions)
///
/// https://docs.convex.dev/functions
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ConvexFunction
{
    pub(crate) name: String,
    pub(crate) params: Vec<ConvexFunctionParam>,
}

/// A parameter in a convex function.
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ConvexFunctionParam
{
    pub(crate) name: String,
    pub(crate) data_type: JsonValue,
}

/// Creates a schema AST from a schema path.
pub(crate) fn create_schema_ast(path: PathBuf) -> Result<JsonValue, ConvexTypeGeneratorError>
{
    Ok(generate_ast(&path)?)
}

/// Creates a map of all convex functions from a list of function paths.
pub(crate) fn create_functions_ast(paths: Vec<PathBuf>) -> Result<HashMap<String, JsonValue>, ConvexTypeGeneratorError>
{
    let mut functions = HashMap::new();

    for path in paths {
        let function_ast = generate_ast(&path)?;
        let file_name = path
            .file_name()
            .ok_or(ConvexTypeGeneratorError::InvalidPath)?
            .to_str()
            .ok_or(ConvexTypeGeneratorError::InvalidUnicode)?;

        functions.insert(file_name.to_string(), function_ast);
    }

    Ok(functions)
}

pub(crate) fn parse_schema_ast(ast: JsonValue) -> Result<ConvexSchema, ConvexTypeGeneratorError>
{
    // Get the body array
    let body = ast["body"]
        .as_array()
        .ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema("Missing body array".into()))?;

    // Find the defineSchema call
    let define_schema = find_define_schema(body)
        .ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema("Could not find defineSchema call".into()))?;

    // Get the arguments array of defineSchema
    let schema_args = define_schema["arguments"]
        .as_array()
        .ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema("Missing schema arguments".into()))?;

    // Get the first argument which is an object containing table definitions
    let tables_obj = schema_args
        .first()
        .and_then(|arg| arg["properties"].as_array())
        .ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema("Missing table definitions".into()))?;

    let mut tables = Vec::new();

    // Iterate through each table definition
    for table_prop in tables_obj {
        // Get the table name
        let table_name = table_prop["key"]["name"]
            .as_str()
            .ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema("Invalid table name".into()))?;

        // Get the defineTable call arguments
        let define_table_args = table_prop["value"]["arguments"]
            .as_array()
            .ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema("Invalid table definition".into()))?;

        // Get the first argument which contains column definitions
        let columns_obj = define_table_args
            .first()
            .and_then(|arg| arg["properties"].as_array())
            .ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema("Missing column definitions".into()))?;

        let mut columns = Vec::new();

        // Iterate through each column definition
        for column_prop in columns_obj {
            // Get column name
            let column_name = column_prop["key"]["name"]
                .as_str()
                .ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema("Invalid column name".into()))?;

            // Get column type by looking at the property chain
            let column_type = extract_column_type(column_prop)?;

            columns.push(ConvexColumn {
                name: column_name.to_string(),
                data_type: column_type,
            });
        }

        tables.push(ConvexTable {
            name: table_name.to_string(),
            columns,
        });
    }

    Ok(ConvexSchema { tables })
}

/// Helper function to find the defineSchema call in the AST
fn find_define_schema(body: &[JsonValue]) -> Option<&JsonValue>
{
    for node in body {
        // Check if this is an export default declaration
        if let Some(declaration) = node.get("declaration") {
            // Check if this is a call expression
            if declaration["type"].as_str() == Some("CallExpression") {
                // Check if the callee is defineSchema
                if let Some(callee) = declaration.get("callee") {
                    if callee["type"].as_str() == Some("Identifier") && callee["name"].as_str() == Some("defineSchema") {
                        return Some(declaration);
                    }
                }
            }
        }

        // Could also be a regular variable declaration or expression
        // that calls defineSchema
        if node["type"].as_str() == Some("CallExpression") {
            if let Some(callee) = node.get("callee") {
                if callee["type"].as_str() == Some("Identifier") && callee["name"].as_str() == Some("defineSchema") {
                    return Some(node);
                }
            }
        }
    }
    None
}

/// Helper function to extract the column type from a column property
fn extract_column_type(column_prop: &JsonValue) -> Result<JsonValue, ConvexTypeGeneratorError> {
    // Get the value which contains the type call expression
    let value = &column_prop["value"];

    // Get the callee which contains the type information
    let callee = &value["callee"];

    // The type is in the property name of the StaticMemberExpression
    let type_name = callee["property"]["name"]
        .as_str()
        .ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema("Invalid column type".into()))?;

    // Get any arguments passed to the type function
    let binding = Vec::new();
    let args = value["arguments"].as_array().unwrap_or(&binding);

    // Create a JSON object representing the type
    let mut type_obj = serde_json::Map::new();
    type_obj.insert("type".to_string(), JsonValue::String(type_name.to_string()));

    // Handle nested types
    match type_name {
        "array" => {
            // For arrays, recursively parse the element type
            if let Some(element_type) = args.first() {
                // The element type will be another v.type() expression
                let element_type_prop = json!({
                    "key": { "name": "element" },
                    "value": element_type
                });
                let parsed_element_type = extract_column_type(&element_type_prop)?;
                type_obj.insert("elements".to_string(), parsed_element_type);
            }
        }
        "object" => {
            // For objects, parse each property type
            if let Some(obj_def) = args.first() {
                if let Some(properties) = obj_def["properties"].as_array() {
                    let mut prop_types = serde_json::Map::new();

                    for prop in properties {
                        let prop_name = prop["key"]["name"]
                            .as_str()
                            .ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema("Invalid object property name".into()))?;

                        let prop_type = extract_column_type(prop)?;
                        prop_types.insert(prop_name.to_string(), prop_type);
                    }

                    type_obj.insert("properties".to_string(), JsonValue::Object(prop_types));
                }
            }
        }
        "record" => {
            // For records, parse both key and value types
            if args.len() >= 2 {
                // First argument is the key type
                let key_type_prop = json!({
                    "key": { "name": "key" },
                    "value": args[0]
                });
                let key_type = extract_column_type(&key_type_prop)?;
                type_obj.insert("keyType".to_string(), key_type);

                // Second argument is the value type
                let value_type_prop = json!({
                    "key": { "name": "value" },
                    "value": args[1]
                });
                let value_type = extract_column_type(&value_type_prop)?;
                type_obj.insert("valueType".to_string(), value_type);
            }
        }
        "union" => {
            // For unions, parse all variant types
            let mut variants = Vec::new();
            for variant in args {
                let variant_prop = json!({
                    "key": { "name": "variant" },
                    "value": variant
                });
                let variant_type = extract_column_type(&variant_prop)?;
                variants.push(variant_type);
            }
            type_obj.insert("variants".to_string(), JsonValue::Array(variants));
        }
        "literal" => {
            // For literals, store the literal value
            if let Some(literal_value) = args.first() {
                type_obj.insert("value".to_string(), literal_value.clone());
            }
        }
        // For other types, just include their arguments if any
        _ => {
            if !args.is_empty() {
                type_obj.insert("arguments".to_string(), JsonValue::Array(args.to_vec()));
            }
        }
    }

    Ok(JsonValue::Object(type_obj))
}

pub(crate) fn parse_function_ast(ast_map: HashMap<String, JsonValue>) -> Result<ConvexFunctions, ConvexTypeGeneratorError> {
    let mut functions = Vec::new();

    for (file_name, ast) in ast_map {
        // Get the body array
        let body = ast["body"]
            .as_array()
            .ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema("Missing body array".into()))?;

        for node in body {
            // Look for export declarations
            if node["type"].as_str() == Some("ExportNamedDeclaration") {
                if let Some(declaration) = node.get("declaration") {
                    // Handle variable declarations (const testQuery = query({...}))
                    if declaration["type"].as_str() == Some("VariableDeclaration") {
                        if let Some(declarators) = declaration["declarations"].as_array() {
                            for declarator in declarators {
                                // Get function name
                                let name = declarator["id"]["name"]
                                    .as_str()
                                    .ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema("Missing function name".into()))?;

                                // Get the function call (query/mutation/action)
                                let init = &declarator["init"];
                                if init["type"].as_str() == Some("CallExpression") {
                                    // Get the first argument which contains the function config
                                    if let Some(args) = init["arguments"].as_array() {
                                        if let Some(config) = args.first() {
                                            // Extract function parameters from the args property
                                            let params = extract_function_params(config)?;

                                            functions.push(ConvexFunction {
                                                name: name.to_string(),
                                                params,
                                            });
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

    Ok(functions)
}

/// Helper function to extract function parameters from the function configuration
fn extract_function_params(config: &JsonValue) -> Result<Vec<ConvexFunctionParam>, ConvexTypeGeneratorError> {
    let mut params = Vec::new();

    // Get the args object from the function config
    if let Some(properties) = config["properties"].as_array() {
        for prop in properties {
            if prop["key"]["name"].as_str() == Some("args") {
                // Ensure args is an object
                if prop["value"]["type"].as_str() != Some("ObjectExpression") {
                    return Err(ConvexTypeGeneratorError::InvalidSchema(
                        "Function args must be an object".into()
                    ));
                }

                // Get the args object value
                if let Some(args_props) = prop["value"]["properties"].as_array() {
                    for arg_prop in args_props {
                        // Validate argument property structure
                        if !arg_prop["type"].as_str().map_or(false, |t| t == "ObjectProperty") {
                            return Err(ConvexTypeGeneratorError::InvalidSchema(
                                "Invalid argument property structure".into()
                            ));
                        }

                        // Get parameter name
                        let param_name = arg_prop["key"]["name"]
                            .as_str()
                            .ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema("Invalid parameter name".into()))?;

                        // Get parameter type using the same extraction logic as schema
                        let param_type = extract_column_type(arg_prop)?;

                        params.push(ConvexFunctionParam {
                            name: param_name.to_string(),
                            data_type: param_type,
                        });
                    }
                }
                break; // Found args object, no need to continue
            }
        }
    }

    Ok(params)
}

/// Internal helper function to generate an AST from a source file
fn generate_ast(path: &PathBuf) -> Result<JsonValue, ConvexTypeGeneratorError>
{
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).map_err(|_| ConvexTypeGeneratorError::ParsingFailed)?;
    let source_text = std::fs::read_to_string(path).map_err(ConvexTypeGeneratorError::IOError)?;

    let mut errors: Vec<OxcDiagnostic> = Vec::new();

    let ret = Parser::new(&allocator, &source_text, source_type).parse();
    errors.extend(ret.errors);

    if ret.panicked {
        for error in &errors {
            eprintln!("{error:?}");
        }
        return Err(ConvexTypeGeneratorError::ParsingFailed);
    }

    if ret.program.is_empty() {
        return Err(ConvexTypeGeneratorError::EmptySchemaFile);
    }

    let semantics = SemanticBuilder::new().with_check_syntax_error(true).build(&ret.program);
    errors.extend(semantics.errors);

    if !errors.is_empty() {
        for error in &errors {
            eprintln!("{error:?}");
        }
        return Err(ConvexTypeGeneratorError::ParsingFailed);
    }

    serde_json::to_value(&ret.program).map_err(ConvexTypeGeneratorError::SerializationFailed)
}
