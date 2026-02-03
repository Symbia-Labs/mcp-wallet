//! stdio transport for MCP (used by Claude Desktop)

use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::protocol::{McpMessage, McpError, RequestHandler};
use wallet_core::Wallet;

/// stdio transport for MCP protocol
pub struct StdioTransport {
    handler: RequestHandler,
}

impl StdioTransport {
    /// Create a new stdio transport
    pub fn new(wallet: Arc<RwLock<Wallet>>) -> Self {
        Self {
            handler: RequestHandler::new(wallet),
        }
    }

    /// Run the stdio transport (blocking)
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting MCP server on stdio");

        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();

            // Read a line from stdin
            let bytes_read = reader.read_line(&mut line).await?;
            if bytes_read == 0 {
                // EOF
                info!("EOF received, shutting down");
                break;
            }

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            debug!("Received: {}", line);

            // Parse the message
            let message: McpMessage = match serde_json::from_str(line) {
                Ok(msg) => msg,
                Err(e) => {
                    error!("Failed to parse message: {}", e);
                    let error_response = McpMessage::error_response(None, McpError::parse_error());
                    let response_line = serde_json::to_string(&error_response)?;
                    stdout.write_all(response_line.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                    continue;
                }
            };

            // Handle the message
            if let Some(response) = self.handler.handle(message).await {
                let response_line = serde_json::to_string(&response)?;
                debug!("Sending: {}", response_line);
                stdout.write_all(response_line.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
            }
        }

        Ok(())
    }
}
