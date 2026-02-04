//! Main MCP server orchestration

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::transport::{HttpTransport, StdioTransport};
use wallet_core::Wallet;

/// Server mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ServerMode {
    /// stdio transport (for Claude Desktop)
    #[default]
    Stdio,
    /// HTTP/SSE transport
    Http { port: u16 },
}

/// MCP server
pub struct McpServer {
    wallet: Arc<RwLock<Wallet>>,
    mode: ServerMode,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new(wallet: Arc<RwLock<Wallet>>) -> Self {
        Self {
            wallet,
            mode: ServerMode::default(),
        }
    }

    /// Set the server mode
    pub fn with_mode(mut self, mode: ServerMode) -> Self {
        self.mode = mode;
        self
    }

    /// Run the server
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self.mode {
            ServerMode::Stdio => {
                info!("Starting MCP server in stdio mode");
                let mut transport = StdioTransport::new(self.wallet.clone());
                transport.run().await
            }
            ServerMode::Http { port } => {
                info!("Starting MCP server in HTTP mode on port {}", port);
                let transport = HttpTransport::new(self.wallet.clone(), port);
                transport.run().await
            }
        }
    }
}
