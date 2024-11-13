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
    pub(crate) type_: String,
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
        let path_str = path.to_string_lossy().to_string();
        let file_name = path
            .file_name()
            .ok_or_else(|| ConvexTypeGeneratorError::InvalidPath(path_str.clone()))?
            .to_str()
            .ok_or_else(|| ConvexTypeGeneratorError::InvalidUnicode(path_str))?;

        functions.insert(file_name.to_string(), function_ast);
    }

    Ok(functions)
}

pub(crate) fn parse_schema_ast(ast: JsonValue) -> Result<ConvexSchema, ConvexTypeGeneratorError>
{
    let context = "root";
    // Get the body array
    let body = ast["body"]
        .as_array()
        .ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema {
            context: context.to_string(),
            details: "Missing body array".to_string(),
        })?;

    // Find the defineSchema call
    let define_schema = find_define_schema(body).ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema {
        context: context.to_string(),
        details: "Could not find defineSchema call".to_string(),
    })?;

    // Get the arguments array of defineSchema
    let schema_args = define_schema["arguments"]
        .as_array()
        .ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema {
            context: context.to_string(),
            details: "Missing schema arguments".to_string(),
        })?;

    // Get the first argument which is an object containing table definitions
    let tables_obj = schema_args
        .first()
        .and_then(|arg| arg["properties"].as_array())
        .ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema {
            context: context.to_string(),
            details: "Missing table definitions".to_string(),
        })?;

    let mut tables = Vec::new();

    // Iterate through each table definition
    for table_prop in tables_obj {
        // Get the table name
        let table_name = table_prop["key"]["name"]
            .as_str()
            .ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema {
                context: context.to_string(),
                details: "Invalid table name".to_string(),
            })?;

        // Get the defineTable call arguments
        let define_table_args =
            table_prop["value"]["arguments"]
                .as_array()
                .ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema {
                    context: context.to_string(),
                    details: "Invalid table definition".to_string(),
                })?;

        // Get the first argument which contains column definitions
        let columns_obj = define_table_args
            .first()
            .and_then(|arg| arg["properties"].as_array())
            .ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema {
                context: context.to_string(),
                details: "Missing column definitions".to_string(),
            })?;

        let mut columns = Vec::new();

        // Iterate through each column definition
        for column_prop in columns_obj {
            // Get column name
            let column_name =
                column_prop["key"]["name"]
                    .as_str()
                    .ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema {
                        context: context.to_string(),
                        details: "Invalid column name".to_string(),
                    })?;

            // Get column type by looking at the property chain
            let mut context = TypeContext::new("schema".to_string());
            let column_type = extract_column_type(column_prop, &mut context)?;

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
fn extract_column_type(column_prop: &JsonValue, context: &mut TypeContext) -> Result<JsonValue, ConvexTypeGeneratorError>
{
    let value = &column_prop["value"];
    let callee = &value["callee"];

    let type_name = callee["property"]["name"]
        .as_str()
        .ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema {
            context: context.type_path.join("."),
            details: "Invalid column type".to_string(),
        })?;

    // Validate the type name
    validate_type_name(type_name)?;

    let binding = Vec::new();
    let args = value["arguments"].as_array().unwrap_or(&binding);

    let mut type_obj = serde_json::Map::new();
    type_obj.insert("type".to_string(), JsonValue::String(type_name.to_string()));

    // Handle nested types
    match type_name {
        "optional" => {
            // For optional types, recursively parse the inner type
            if let Some(inner_type) = args.first() {
                let inner_type_prop = json!({
                    "key": { "name": "inner" },
                    "value": inner_type
                });
                context.type_path.push("inner".to_string());
                let parsed_inner_type = extract_column_type(&inner_type_prop, context)?;
                context.type_path.pop();
                type_obj.insert("inner".to_string(), parsed_inner_type);
            } else {
                return Err(ConvexTypeGeneratorError::InvalidSchema {
                    context: context.type_path.join("."),
                    details: "Optional type must have an inner type".to_string(),
                });
            }
        }
        "array" => {
            // For arrays, recursively parse the element type
            if let Some(element_type) = args.first() {
                let element_type_prop = json!({
                    "key": { "name": "element" },
                    "value": element_type
                });
                context.type_path.push("elements".to_string());
                let parsed_element_type = extract_column_type(&element_type_prop, context)?;
                context.type_path.pop();
                type_obj.insert("elements".to_string(), parsed_element_type);
            }
        }
        "object" => {
            // For objects, parse each property type
            if let Some(obj_def) = args.first() {
                if let Some(properties) = obj_def["properties"].as_array() {
                    let mut prop_types = serde_json::Map::new();

                    for prop in properties {
                        let prop_name =
                            prop["key"]["name"]
                                .as_str()
                                .ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema {
                                    context: context.type_path.join("."),
                                    details: "Invalid object property name".to_string(),
                                })?;

                        let prop_type = extract_column_type(prop, context)?;
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
                let key_type = extract_column_type(&key_type_prop, context)?;
                type_obj.insert("keyType".to_string(), key_type);

                // Second argument is the value type
                let value_type_prop = json!({
                    "key": { "name": "value" },
                    "value": args[1]
                });
                let value_type = extract_column_type(&value_type_prop, context)?;
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
                let variant_type = extract_column_type(&variant_prop, context)?;
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

    // Build the type object as before...
    let type_value = JsonValue::Object(type_obj);

    // Check for circular references
    check_circular_references(&type_value, context)?;

    Ok(type_value)
}

pub(crate) fn parse_function_ast(ast_map: HashMap<String, JsonValue>) -> Result<ConvexFunctions, ConvexTypeGeneratorError>
{
    let mut functions = Vec::new();

    for (file_name, ast) in ast_map {
        // Get the body array
        let body = ast["body"]
            .as_array()
            .ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema {
                context: format!("file_{}", file_name),
                details: "Missing body array".to_string(),
            })?;

        for node in body {
            // Look for export declarations
            if node["type"].as_str() == Some("ExportNamedDeclaration") {
                if let Some(declaration) = node.get("declaration") {
                    // Handle variable declarations (const testQuery = query({...}))
                    if declaration["type"].as_str() == Some("VariableDeclaration") {
                        if let Some(declarators) = declaration["declarations"].as_array() {
                            for declarator in declarators {
                                // Get function name
                                let name = declarator["id"]["name"].as_str().ok_or_else(|| {
                                    ConvexTypeGeneratorError::InvalidSchema {
                                        context: format!("file_{}", file_name),
                                        details: "Missing function name".to_string(),
                                    }
                                })?;

                                // Get the function call (query/mutation/action)
                                let init = &declarator["init"];
                                if init["type"].as_str() == Some("CallExpression") {
                                    // Get the callee to determine function type
                                    let fn_type = init["callee"]["name"].as_str().ok_or_else(|| {
                                        ConvexTypeGeneratorError::InvalidSchema {
                                            context: format!("function_{}", name),
                                            details: "Missing function type".to_string(),
                                        }
                                    })?;

                                    // Get the first argument which contains the function config
                                    if let Some(args) = init["arguments"].as_array() {
                                        if let Some(config) = args.first() {
                                            // Extract function parameters from the args property
                                            let params = extract_function_params(config, &file_name)?;

                                            functions.push(ConvexFunction {
                                                name: name.to_string(),
                                                params,
                                                type_: fn_type.to_string(),
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
fn extract_function_params(config: &JsonValue, file_name: &str)
    -> Result<Vec<ConvexFunctionParam>, ConvexTypeGeneratorError>
{
    let mut params = Vec::new();

    // Get the args object from the function config
    if let Some(properties) = config["properties"].as_array() {
        for prop in properties {
            if prop["key"]["name"].as_str() == Some("args") {
                // Ensure args is an object
                if prop["value"]["type"].as_str() != Some("ObjectExpression") {
                    return Err(ConvexTypeGeneratorError::InvalidSchema {
                        context: format!("file_{}", file_name),
                        details: "Function args must be an object".to_string(),
                    });
                }

                // Get the args object value
                if let Some(args_props) = prop["value"]["properties"].as_array() {
                    for arg_prop in args_props {
                        // Validate argument property structure
                        if !arg_prop["type"].as_str().map_or(false, |t| t == "ObjectProperty") {
                            return Err(ConvexTypeGeneratorError::InvalidSchema {
                                context: format!("file_{}", file_name),
                                details: "Invalid argument property structure".to_string(),
                            });
                        }

                        // Get parameter name
                        let param_name =
                            arg_prop["key"]["name"]
                                .as_str()
                                .ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema {
                                    context: format!("file_{}", file_name),
                                    details: "Invalid parameter name".to_string(),
                                })?;

                        // Get parameter type using the same extraction logic as schema
                        let mut context = TypeContext::new(format!("function_{}", param_name));
                        let param_type = extract_column_type(arg_prop, &mut context)?;

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
    let path_str = path.to_string_lossy().to_string();
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).map_err(|_| ConvexTypeGeneratorError::ParsingFailed {
        file: path_str.clone(),
        details: "Failed to determine source type".to_string(),
    })?;
    let source_text = std::fs::read_to_string(path).map_err(|error| ConvexTypeGeneratorError::IOError {
        file: path_str.clone(),
        error,
    })?;

    let mut errors: Vec<OxcDiagnostic> = Vec::new();

    let ret = Parser::new(&allocator, &source_text, source_type).parse();
    errors.extend(ret.errors);

    if ret.panicked {
        for error in &errors {
            eprintln!("{error:?}");
        }
        return Err(ConvexTypeGeneratorError::ParsingFailed {
            file: path_str.clone(),
            details: "Parser panicked".to_string(),
        });
    }

    if ret.program.is_empty() {
        return Err(ConvexTypeGeneratorError::EmptySchemaFile { file: path_str });
    }

    let semantics = SemanticBuilder::new().with_check_syntax_error(true).build(&ret.program);
    errors.extend(semantics.errors);

    if !errors.is_empty() {
        for error in &errors {
            eprintln!("{error:?}");
        }
        return Err(ConvexTypeGeneratorError::ParsingFailed {
            file: path_str,
            details: "Semantic analysis failed".to_string(),
        });
    }

    serde_json::to_value(&ret.program).map_err(ConvexTypeGeneratorError::SerializationFailed)
}

const VALID_TYPES: &[&str] = &[
    "id",
    "null",
    "int64",
    "number",
    "boolean",
    "string",
    "bytes",
    "array",
    "object",
    "record",
    "union",
    "literal",
    "optional",
    "any"
];

fn validate_type_name(type_name: &str) -> Result<(), ConvexTypeGeneratorError>
{
    if !VALID_TYPES.contains(&type_name) {
        return Err(ConvexTypeGeneratorError::InvalidType {
            found: type_name.to_string(),
            valid_types: VALID_TYPES.iter().map(|&s| s.to_string()).collect(),
        });
    }
    Ok(())
}

#[derive(Debug)]
pub struct ParseContext
{
    pub file_name: String,
    pub type_path: Vec<String>,
}

impl ParseContext
{
    fn push(&mut self, segment: &str)
    {
        self.type_path.push(segment.to_string());
    }

    fn pop(&mut self)
    {
        self.type_path.pop();
    }

    fn current_path(&self) -> String
    {
        self.type_path.join(".")
    }
}

#[derive(Debug, Default)]
struct TypeContext
{
    /// Stack of type paths being processed (includes type name and path)
    type_stack: Vec<(String, String)>,  // (type_name, full_path)
    /// Current file being processed
    file_name: String,
    /// Current path in the type structure
    type_path: Vec<String>,
}

impl TypeContext
{
    fn new(file_name: String) -> Self
    {
        Self {
            file_name,
            type_stack: Vec::new(),
            type_path: Vec::new(),
        }
    }

    fn push_type(&mut self, type_name: &str) -> Result<(), ConvexTypeGeneratorError>
    {
        let current_path = self.type_path.join(".");
        
        // Only check for circular references in object types
        // Arrays and other container types can be nested
        if type_name == "object" {
            let full_path = if current_path.is_empty() {
                type_name.to_string()
            } else {
                format!("{}.{}", current_path, type_name)
            };

            // Check if this exact path has been seen before
            if self.type_stack.iter().any(|(_, path)| path == &full_path) {
                return Err(ConvexTypeGeneratorError::CircularReference {
                    path: self.type_stack.iter()
                        .map(|(_, path)| path.clone())
                        .collect(),
                });
            }
            self.type_stack.push((type_name.to_string(), full_path));
        }
        Ok(())
    }

    fn pop_type(&mut self)
    {
        self.type_stack.pop();
    }
}

fn check_circular_references(type_obj: &JsonValue, context: &mut TypeContext) -> Result<(), ConvexTypeGeneratorError>
{
    let type_name = type_obj["type"]
        .as_str()
        .ok_or_else(|| ConvexTypeGeneratorError::InvalidSchema {
            context: context.type_path.join("."),
            details: "Missing type name".to_string(),
        })?;

    context.push_type(type_name)?;

    match type_name {
        "optional" => {
            if let Some(inner) = type_obj.get("inner") {
                context.type_path.push("inner".to_string());
                check_circular_references(inner, context)?;
                context.type_path.pop();
            }
        }
        "array" => {
            if let Some(elements) = type_obj.get("elements") {
                context.type_path.push("elements".to_string());
                check_circular_references(elements, context)?;
                context.type_path.pop();
            }
        }
        "object" => {
            if let Some(properties) = type_obj.get("properties") {
                if let Some(props) = properties.as_object() {
                    for (prop_name, prop_type) in props {
                        context.type_path.push(prop_name.to_string());
                        check_circular_references(prop_type, context)?;
                        context.type_path.pop();
                    }
                }
            }
        }
        "record" => {
            // Check key type
            if let Some(key_type) = type_obj.get("keyType") {
                context.type_path.push("keyType".to_string());
                check_circular_references(key_type, context)?;
                context.type_path.pop();
            }
            // Check value type
            if let Some(value_type) = type_obj.get("valueType") {
                context.type_path.push("valueType".to_string());
                check_circular_references(value_type, context)?;
                context.type_path.pop();
            }
        }
        "union" | "intersection" => {
            if let Some(variants) = type_obj["variants"].as_array() {
                for (i, variant) in variants.iter().enumerate() {
                    context.type_path.push(format!("variant_{}", i));
                    check_circular_references(variant, context)?;
                    context.type_path.pop();
                }
            }
        }
        _ => {} // Other types don't have nested types
    }

    context.pop_type();
    Ok(())
}
