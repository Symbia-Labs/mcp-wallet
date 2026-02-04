//! # mcp-server
//!
//! MCP (Model Context Protocol) server implementation for MCP Wallet.
//! Supports both stdio and HTTP/SSE transports.

pub mod protocol;
mod server;
pub mod tools;
pub mod transport;

pub use protocol::{McpError, McpMessage, ServerCapabilities};
pub use server::{McpServer, ServerMode};
pub use tools::{ToolExecutor, ToolGenerator};
pub use transport::{HttpTransport, StdioTransport};
