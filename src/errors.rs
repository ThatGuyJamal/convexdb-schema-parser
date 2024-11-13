use std::fmt;

/// Errors that can occur during the type generation
#[derive(Debug)]
pub enum ConvexTypeGeneratorError
{
    /// The schema file was not found
    MissingSchemaFile,
    /// The schema file failed to parse
    ParsingFailed
    {
        file: String, details: String
    },
    /// The schema file is empty
    EmptySchemaFile
    {
        file: String
    },
    /// The path doesn't have a file name component
    InvalidPath(String),
    /// The file name contains invalid Unicode characters
    InvalidUnicode(String),
    /// The schema file failed to serialize
    SerializationFailed(serde_json::Error),
    /// An IO error occurred
    IOError
    {
        file: String, error: std::io::Error
    },
    /// The schema file is invalid
    InvalidSchema
    {
        context: String, details: String
    },
    /// Circular reference detected
    CircularReference
    {
        path: Vec<String>
    },
    /// Invalid type found
    InvalidType
    {
        found: String, valid_types: Vec<String>
    },
}

impl fmt::Display for ConvexTypeGeneratorError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self {
            Self::MissingSchemaFile => write!(f, "Schema file not found"),
            Self::ParsingFailed { file, details } => {
                write!(f, "Failed to parse file '{}': {}", file, details)
            }
            Self::EmptySchemaFile { file } => {
                write!(f, "Schema file '{}' is empty", file)
            }
            Self::InvalidPath(path) => {
                write!(f, "Invalid path: {}", path)
            }
            Self::InvalidUnicode(path) => {
                write!(f, "Path contains invalid Unicode: {}", path)
            }
            Self::SerializationFailed(err) => {
                write!(f, "Failed to serialize AST: {}", err)
            }
            Self::IOError { file, error } => {
                write!(f, "IO error while reading '{}': {}", file, error)
            }
            Self::InvalidSchema { context, details } => {
                write!(f, "Invalid schema at {}: {}", context, details)
            }
            Self::CircularReference { path } => {
                write!(f, "Circular type reference detected: {}", path.join(" -> "))
            }
            Self::InvalidType { found, valid_types } => {
                write!(f, "Invalid type '{}'. Valid types are: {}", found, valid_types.join(", "))
            }
        }
    }
}

// Add this implementation to convert std::io::Error to ConvexTypeGeneratorError
impl From<std::io::Error> for ConvexTypeGeneratorError
{
    fn from(error: std::io::Error) -> Self
    {
        ConvexTypeGeneratorError::IOError {
            file: String::new(),
            error,
        }
    }
}

// Implement std::error::Error for better error handling
impl std::error::Error for ConvexTypeGeneratorError {}
