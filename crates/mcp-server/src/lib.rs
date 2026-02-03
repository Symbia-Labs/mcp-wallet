//! # mcp-server
//!
//! MCP (Model Context Protocol) server implementation for MCP Wallet.
//! Supports both stdio and HTTP/SSE transports.

pub mod protocol;
pub mod transport;
pub mod tools;
mod server;

pub use server::{McpServer, ServerMode};
pub use protocol::{McpMessage, McpError, ServerCapabilities};
pub use transport::{StdioTransport, HttpTransport};
pub use tools::{ToolGenerator, ToolExecutor};
