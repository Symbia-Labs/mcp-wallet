//! # openapi-parser
//!
//! OpenAPI 3.x parser for MCP Wallet.
//! Extracts operations from OpenAPI specs and builds namespace trees for fast lookup.

mod auth;
mod error;
mod namespace;
mod operations;
mod parser;
mod resolver;
mod types;

pub use auth::AuthScheme;
pub use error::{ParseError, ParseResult};
pub use namespace::NamespaceTree;
pub use operations::OperationExtractor;
pub use parser::OpenApiParser;
pub use types::*;
