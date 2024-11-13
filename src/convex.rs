use std::collections::HashMap;
use std::path::PathBuf;

use oxc::allocator::Allocator;
use oxc::diagnostics::OxcDiagnostic;
use oxc::parser::Parser;
use oxc::semantic::SemanticBuilder;
use oxc::span::SourceType;
use serde_json::Value as JsonValue;

use crate::errors::ConvexTypeGeneratorError;

/// The convex schema.
///
/// A schema can contain many tables. https://docs.convex.dev/database/schemas
pub(crate) struct ConvexSchema
{
    pub(crate) tables: Vec<ConvexTable>,
}

/// A table in the convex schema.
///
/// A table can contain many columns.
pub(crate) struct ConvexTable
{
    /// The name of the table.
    pub(crate) name: String,
    /// The columns in the table.
    pub(crate) columns: Vec<ConvexColumn>,
}

/// A column in the convex schema.
pub(crate) struct ConvexColumn
{
    /// The name of the column.
    pub(crate) name: String,
    /// The data type of the column.
    /// https://docs.rs/convex/latest/convex/enum.Value.html
    pub(crate) data_type: JsonValue,
}

/// A map of all convex functions.
///
/// key: function name
/// value: function definition
pub(crate) type ConvexFunctions = HashMap<String, ConvexFunction>;

/// Convex functions (Queries, Mutations, and Actions)
///
/// https://docs.convex.dev/functions
pub(crate) struct ConvexFunction
{
    pub(crate) name: String,
    pub(crate) params: Vec<ConvexFunctionParam>,
}

/// A parameter in a convex function.
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
