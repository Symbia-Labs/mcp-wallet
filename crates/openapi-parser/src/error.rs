//! Error types for the OpenAPI parser

use thiserror::Error;

/// Result type alias for parser operations
pub type ParseResult<T> = std::result::Result<T, ParseError>;

/// Parser error types
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Failed to fetch OpenAPI spec: {0}")]
    FetchError(String),

    #[error("Invalid OpenAPI spec format: {0}")]
    InvalidFormat(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("YAML parse error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Unsupported OpenAPI version: {0}")]
    UnsupportedVersion(String),
}
