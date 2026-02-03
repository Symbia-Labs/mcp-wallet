//! # openapi-parser
//!
//! OpenAPI 3.x parser for MCP Wallet.
//! Extracts operations from OpenAPI specs and builds namespace trees for fast lookup.

mod types;
mod parser;
mod auth;
mod operations;
mod namespace;
mod error;

pub use types::*;
pub use parser::OpenApiParser;
pub use auth::AuthScheme;
pub use operations::OperationExtractor;
pub use namespace::NamespaceTree;
pub use error::{ParseError, ParseResult};
