use std::fmt;

/// Errors that can occur during the type generation
#[derive(Debug)]
pub enum ConvexTypeGeneratorError
{
    /// The schema file was not found
    MissingSchemaFile,
    /// The schema file failed to parse
    ParsingFailed,
    /// The schema file is empty
    EmptySchemaFile,
    /// The path doesn't have a file name component
    InvalidPath,
    /// The file name contains invalid Unicode characters
    InvalidUnicode,
    /// The schema file failed to serialize
    SerializationFailed(serde_json::Error),
    /// An IO error occurred
    IOError(std::io::Error),
    /// The schema file is invalid
    InvalidSchema(String),
}

impl fmt::Display for ConvexTypeGeneratorError
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "{:?}", self)
    }
}

// Add this implementation to convert std::io::Error to ConvexTypeGeneratorError
impl From<std::io::Error> for ConvexTypeGeneratorError
{
    fn from(error: std::io::Error) -> Self
    {
        ConvexTypeGeneratorError::IOError(error)
    }
}

// Implement std::error::Error for better error handling
impl std::error::Error for ConvexTypeGeneratorError {}
