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
    /// The schema file failed to serialize
    SerializationFailed(serde_json::Error),
    /// An IO error occurred
    IOError(std::io::Error),
}

impl std::fmt::Display for ConvexTypeGeneratorError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{:?}", self)
    }
}
