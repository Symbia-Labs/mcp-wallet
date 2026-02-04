//! Execute MCP tools by making HTTP requests

use openapi_parser::{ApiOperation, HttpMethod, ParameterLocation};
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::protocol::ToolCallResult;
use wallet_core::{Wallet, WalletError};

/// Executor for MCP tools
pub struct ToolExecutor {
    /// Wallet reference
    wallet: Arc<RwLock<Wallet>>,
    /// HTTP client
    client: Client,
}

impl ToolExecutor {
    /// Create a new tool executor
    pub fn new(wallet: Arc<RwLock<Wallet>>) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { wallet, client }
    }

    /// Execute a tool by name
    pub async fn execute(
        &self,
        tool_name: &str,
        arguments: Option<Value>,
    ) -> Result<ToolCallResult, WalletError> {
        // Parse tool name: {integration}_{operation_path}
        let (integration_key, operation_path) = self.parse_tool_name(tool_name)?;

        debug!(
            "Executing tool: {} -> {} / {}",
            tool_name, integration_key, operation_path
        );

        let wallet = self.wallet.read().await;

        if !wallet.is_unlocked() {
            return Err(WalletError::WalletLocked);
        }

        // Get integration
        let stored = wallet
            .integrations
            .get_stored(&integration_key)
            .await
            .ok_or_else(|| WalletError::IntegrationNotFound(integration_key.clone()))?;

        // Find operation
        let operation = stored
            .lookup_operation(&operation_path)
            .ok_or_else(|| WalletError::OperationNotFound(operation_path.clone()))?;

        // Get credential
        let credential_id = stored.integration.credential_id.ok_or_else(|| {
            WalletError::CredentialNotFound(format!(
                "No credential for integration {}",
                integration_key
            ))
        })?;

        let decrypted = wallet.credentials.get_decrypted(credential_id).await?;
        let api_key = decrypted.expose().to_string();

        // Drop wallet lock before making HTTP request
        drop(wallet);

        // Build and execute request
        let result = self
            .execute_operation(
                &stored.integration.server_url,
                operation,
                arguments,
                &api_key,
            )
            .await?;

        Ok(result)
    }

    /// Parse tool name into integration key and operation path
    fn parse_tool_name(&self, tool_name: &str) -> Result<(String, String), WalletError> {
        // Tool name format: {integration}_{operation_path_with_underscores}
        // e.g., "stripe_customers_create" -> ("stripe", "customers.create")

        let parts: Vec<&str> = tool_name.splitn(2, '_').collect();
        if parts.len() != 2 {
            return Err(WalletError::OperationNotFound(format!(
                "Invalid tool name format: {}",
                tool_name
            )));
        }

        let integration_key = parts[0].to_string();
        let operation_path = parts[1].replace('_', ".");

        Ok((integration_key, operation_path))
    }

    /// Execute an HTTP operation
    async fn execute_operation(
        &self,
        base_url: &str,
        operation: &ApiOperation,
        arguments: Option<Value>,
        api_key: &str,
    ) -> Result<ToolCallResult, WalletError> {
        let args = arguments.unwrap_or(Value::Object(serde_json::Map::new()));
        let args_map = args
            .as_object()
            .ok_or_else(|| WalletError::ParseError("Arguments must be an object".to_string()))?;

        // Build URL with path parameters substituted
        let mut url = format!("{}{}", base_url.trim_end_matches('/'), operation.path);
        for param in &operation.parameters {
            if param.location == ParameterLocation::Path {
                if let Some(value) = args_map.get(&param.name) {
                    let value_str = match value {
                        Value::String(s) => s.clone(),
                        other => other.to_string().trim_matches('"').to_string(),
                    };
                    url = url.replace(&format!("{{{}}}", param.name), &value_str);
                }
            }
        }

        // Build query parameters
        let mut query_params: Vec<(String, String)> = Vec::new();
        for param in &operation.parameters {
            if param.location == ParameterLocation::Query {
                if let Some(value) = args_map.get(&param.name) {
                    let value_str = match value {
                        Value::String(s) => s.clone(),
                        Value::Bool(b) => b.to_string(),
                        Value::Number(n) => n.to_string(),
                        other => other.to_string(),
                    };
                    query_params.push((param.name.clone(), value_str));
                }
            }
        }

        // Build request
        let method = match operation.method {
            HttpMethod::Get => reqwest::Method::GET,
            HttpMethod::Post => reqwest::Method::POST,
            HttpMethod::Put => reqwest::Method::PUT,
            HttpMethod::Patch => reqwest::Method::PATCH,
            HttpMethod::Delete => reqwest::Method::DELETE,
            HttpMethod::Head => reqwest::Method::HEAD,
            HttpMethod::Options => reqwest::Method::OPTIONS,
            HttpMethod::Trace => reqwest::Method::TRACE,
        };

        let mut request = self.client.request(method.clone(), &url);

        // Add query parameters
        if !query_params.is_empty() {
            request = request.query(&query_params);
        }

        // Add authentication header (assume bearer token for now)
        request = request.header("Authorization", format!("Bearer {}", api_key));

        // Add body for POST/PUT/PATCH
        let body_for_logging: Option<serde_json::Map<String, Value>>;
        if matches!(
            operation.method,
            HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch
        ) {
            // Collect body parameters (everything not in path/query/header)
            let mut body = serde_json::Map::new();
            let path_query_params: Vec<&str> = operation
                .parameters
                .iter()
                .filter(|p| {
                    matches!(
                        p.location,
                        ParameterLocation::Path
                            | ParameterLocation::Query
                            | ParameterLocation::Header
                    )
                })
                .map(|p| p.name.as_str())
                .collect();

            debug!(
                "Path/query/header params to exclude: {:?}",
                path_query_params
            );
            debug!(
                "Arguments received: {:?}",
                args_map.keys().collect::<Vec<_>>()
            );

            for (key, value) in args_map {
                if !path_query_params.contains(&key.as_str()) {
                    body.insert(key.clone(), value.clone());
                } else {
                    debug!(
                        "Excluding {} from body (it's a path/query/header param)",
                        key
                    );
                }
            }

            debug!("Final body keys: {:?}", body.keys().collect::<Vec<_>>());

            if !body.is_empty() {
                info!(
                    "Request body: {}",
                    serde_json::to_string(&body).unwrap_or_default()
                );
                request = request.json(&body);
                body_for_logging = Some(body);
            } else {
                info!("WARNING: Request body is EMPTY!");
                body_for_logging = None;
            }
        } else {
            body_for_logging = None;
        }

        info!(
            "Executing {} {} with body: {:?}",
            method,
            url,
            body_for_logging
                .as_ref()
                .map(|b| b.keys().collect::<Vec<_>>())
        );

        // Execute request
        let response = request
            .send()
            .await
            .map_err(|e| WalletError::StorageError(format!("HTTP request failed: {}", e)))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| WalletError::StorageError(format!("Failed to read response: {}", e)))?;

        debug!("Response status: {}", status);

        // Parse response
        if status.is_success() {
            // Try to format JSON response
            let formatted = match serde_json::from_str::<Value>(&response_text) {
                Ok(json) => serde_json::to_string_pretty(&json).unwrap_or(response_text),
                Err(_) => response_text,
            };
            Ok(ToolCallResult::text(formatted))
        } else {
            error!("Request failed with status {}: {}", status, response_text);
            Ok(ToolCallResult::error(format!(
                "HTTP {} - {}",
                status, response_text
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tool_name() {
        let wallet = Arc::new(RwLock::new(Wallet::new().expect("Failed to create wallet")));
        let executor = ToolExecutor::new(wallet);

        let (key, path) = executor.parse_tool_name("stripe_customers_create").unwrap();
        assert_eq!(key, "stripe");
        assert_eq!(path, "customers.create");

        let (key, path) = executor
            .parse_tool_name("openai_chat_completions_create")
            .unwrap();
        assert_eq!(key, "openai");
        assert_eq!(path, "chat.completions.create");
    }
}
