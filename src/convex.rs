use std::collections::HashMap;
use std::path::PathBuf;

use oxc::allocator::Allocator;
use oxc::diagnostics::OxcDiagnostic;
use oxc::parser::{Parser, ParserReturn};
use oxc::semantic::SemanticBuilder;
use oxc::span::SourceType;
use serde::Serialize;
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

/// A wrapper around the `oxc::ast::ast::Program` to allow for serialization.
pub(crate) struct ProgramWrapper<'a>(pub(crate) &'a oxc::ast::ast::Program<'a>);

impl<'a> Serialize for ProgramWrapper<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{:?}", self.0))
    }
}

/// Create a convex schema AST from a schema file.
/// 
/// 
pub(crate) fn create_convex_schema_ast(path: PathBuf) -> Result<JsonValue, ConvexTypeGeneratorError>
{
    let allocator = Allocator::default();

    let source_type = SourceType::from_path(path.clone()).map_err(|_| ConvexTypeGeneratorError::ParsingFailed)?;
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

    let semantics = SemanticBuilder::new()
        .with_check_syntax_error(true)  // Enable extra syntax error checking
        .build(&ret.program);                                   // Produce the `Semantic`
    errors.extend(semantics.errors);

    if !errors.is_empty() {
        for error in &errors {
            eprintln!("{error:?}");
        }
        return Err(ConvexTypeGeneratorError::ParsingFailed);
    }

    if ret.program.is_empty() {
        return Err(ConvexTypeGeneratorError::EmptySchemaFile);
    }

    let ast_string = serde_json::to_string_pretty(&ProgramWrapper(&ret.program))
        .map_err(|e| ConvexTypeGeneratorError::SerializationFailed(e))?;

    let ast_value = serde_json::from_str(&ast_string)
        .map_err(|e| ConvexTypeGeneratorError::SerializationFailed(e))?;
    
    Ok(ast_value)
}