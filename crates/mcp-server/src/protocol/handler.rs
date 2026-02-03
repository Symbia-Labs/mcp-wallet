//! MCP request handler

use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, error, info};

use super::types::*;
use super::capabilities::ServerCapabilities;
use crate::tools::{ToolGenerator, ToolExecutor};
use wallet_core::Wallet;

/// Handler for MCP requests
pub struct RequestHandler {
    /// Wallet reference
    wallet: Arc<tokio::sync::RwLock<Wallet>>,
    /// Tool generator
    tool_generator: ToolGenerator,
    /// Tool executor
    tool_executor: ToolExecutor,
    /// Server name
    server_name: String,
    /// Server version
    server_version: String,
    /// Whether the session is initialized
    initialized: bool,
}

impl RequestHandler {
    /// Create a new request handler
    pub fn new(wallet: Arc<tokio::sync::RwLock<Wallet>>) -> Self {
        Self {
            wallet: wallet.clone(),
            tool_generator: ToolGenerator::new(),
            tool_executor: ToolExecutor::new(wallet),
            server_name: "Symbia Labs MCP Wallet".to_string(),
            server_version: env!("CARGO_PKG_VERSION").to_string(),
            initialized: false,
        }
    }

    /// Handle an incoming message
    pub async fn handle(&mut self, message: McpMessage) -> Option<McpMessage> {
        if message.is_request() {
            let method = message.method.as_ref().unwrap();
            let id = message.id.clone().unwrap();

            debug!("Handling request: {}", method);

            let result = match method.as_str() {
                "initialize" => self.handle_initialize(message.params).await,
                "initialized" => {
                    // Notification, no response needed
                    return None;
                }
                "ping" => self.handle_ping().await,
                "tools/list" => self.handle_tools_list().await,
                "tools/call" => self.handle_tools_call(message.params).await,
                _ => Err(McpError::method_not_found()),
            };

            Some(match result {
                Ok(result) => McpMessage::response(id, result),
                Err(error) => McpMessage::error_response(Some(id), error),
            })
        } else if message.is_notification() {
            let method = message.method.as_ref().unwrap();
            debug!("Received notification: {}", method);

            match method.as_str() {
                "notifications/initialized" | "initialized" => {
                    info!("Client initialized");
                }
                "notifications/cancelled" => {
                    debug!("Request cancelled");
                }
                _ => {
                    debug!("Unknown notification: {}", method);
                }
            }

            None
        } else {
            // Response - we don't expect these in server mode
            debug!("Received unexpected response");
            None
        }
    }

    /// Handle initialize request
    async fn handle_initialize(&mut self, params: Option<Value>) -> Result<Value, McpError> {
        let params: InitializeParams = params
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| McpError::invalid_params(e.to_string()))?
            .ok_or_else(|| McpError::invalid_params("Missing params"))?;

        info!(
            "Initializing session with client: {} v{}",
            params.client_info.name, params.client_info.version
        );

        self.initialized = true;

        let result = InitializeResult {
            protocol_version: MCP_VERSION.to_string(),
            capabilities: ServerCapabilities::with_tools(),
            server_info: ServerInfo {
                name: self.server_name.clone(),
                version: self.server_version.clone(),
            },
        };

        serde_json::to_value(result).map_err(|e| McpError::internal_error(e.to_string()))
    }

    /// Handle ping request
    async fn handle_ping(&self) -> Result<Value, McpError> {
        Ok(serde_json::json!({}))
    }

    /// Handle tools/list request
    async fn handle_tools_list(&self) -> Result<Value, McpError> {
        let wallet = self.wallet.read().await;

        if !wallet.is_unlocked() {
            return Err(McpError::internal_error("Wallet is locked"));
        }

        let integrations = wallet.integrations.list().await;
        let mut tools = Vec::new();

        for integration in integrations {
            if let Some(stored) = wallet.integrations.get_stored(&integration.key).await {
                let integration_tools = self.tool_generator.generate_tools(&integration.key, &stored);
                tools.extend(integration_tools);
            }
        }

        let result = ToolsListResult { tools };
        serde_json::to_value(result).map_err(|e| McpError::internal_error(e.to_string()))
    }

    /// Handle tools/call request
    async fn handle_tools_call(&self, params: Option<Value>) -> Result<Value, McpError> {
        let params: ToolCallParams = params
            .map(serde_json::from_value)
            .transpose()
            .map_err(|e| McpError::invalid_params(e.to_string()))?
            .ok_or_else(|| McpError::invalid_params("Missing params"))?;

        debug!("Calling tool: {}", params.name);

        let result = self
            .tool_executor
            .execute(&params.name, params.arguments)
            .await;

        match result {
            Ok(tool_result) => {
                serde_json::to_value(tool_result).map_err(|e| McpError::internal_error(e.to_string()))
            }
            Err(e) => {
                error!("Tool execution failed: {}", e);
                let error_result = ToolCallResult::error(e.to_string());
                serde_json::to_value(error_result).map_err(|e| McpError::internal_error(e.to_string()))
            }
        }
    }
}
