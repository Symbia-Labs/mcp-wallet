//! MCP Protocol Test - Tests the MCP server with simulated messages
//!
//! Run with: cargo run --example mcp_test

use mcp_server::protocol::{McpMessage, RequestHandler};
use std::sync::Arc;
use tokio::sync::RwLock;
use wallet_core::{EncryptedFileStorage, Wallet};

const TEST_SPEC: &str = r#"
openapi: "3.0.0"
info:
  title: Weather API
  version: "1.0.0"
servers:
  - url: https://api.weather.com/v1
paths:
  /forecast:
    get:
      operationId: getForecast
      summary: Get weather forecast
      parameters:
        - name: city
          in: query
          required: true
          schema:
            type: string
        - name: days
          in: query
          schema:
            type: integer
            default: 7
      responses:
        '200':
          description: Weather forecast
components:
  securitySchemes:
    apiKey:
      type: apiKey
      in: header
      name: X-API-Key
security:
  - apiKey: []
"#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MCP Protocol Test ===\n");

    // Setup wallet with test data
    let temp_dir = tempfile::TempDir::new()?;
    let storage = Arc::new(EncryptedFileStorage::with_dir(
        temp_dir.path().to_path_buf(),
    )?);
    let mut wallet = Wallet::with_storage(storage);

    println!("Setting up test wallet...");
    wallet.initialize("test-password").await?;

    // Add integration
    let integration = wallet
        .integrations
        .add_from_content("weather", TEST_SPEC)
        .await?;
    println!(
        "  Added integration: {} ({} operations)",
        integration.name, integration.operation_count
    );

    // Add credential
    let cred = wallet
        .credentials
        .add_api_key("weather", "Weather API Key", "wk_live_abc123")
        .await?;
    wallet
        .integrations
        .set_credential("weather", cred.id)
        .await?;
    println!("  Added and bound credential\n");

    // Create MCP handler
    let wallet = Arc::new(RwLock::new(wallet));
    let mut handler = RequestHandler::new(wallet);

    // Test 1: Initialize
    println!("1. Testing 'initialize' method:");
    let init_request = McpMessage::request(
        1,
        "initialize",
        Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        })),
    );
    println!("   Request:  {}", serde_json::to_string(&init_request)?);

    if let Some(response) = handler.handle(init_request).await {
        println!("   Response: {}", serde_json::to_string_pretty(&response)?);
    }
    println!();

    // Test 2: tools/list
    println!("2. Testing 'tools/list' method:");
    let list_request = McpMessage::request(2, "tools/list", None);
    println!("   Request:  {}", serde_json::to_string(&list_request)?);

    if let Some(response) = handler.handle(list_request).await {
        println!("   Response: {}", serde_json::to_string_pretty(&response)?);
    }
    println!();

    // Test 3: ping
    println!("3. Testing 'ping' method:");
    let ping_request = McpMessage::request(3, "ping", None);
    println!("   Request:  {}", serde_json::to_string(&ping_request)?);

    if let Some(response) = handler.handle(ping_request).await {
        println!("   Response: {}", serde_json::to_string_pretty(&response)?);
    }
    println!();

    // Test 4: Unknown method
    println!("4. Testing unknown method (should return error):");
    let unknown_request = McpMessage::request(4, "unknown/method", None);
    println!("   Request:  {}", serde_json::to_string(&unknown_request)?);

    if let Some(response) = handler.handle(unknown_request).await {
        println!("   Response: {}", serde_json::to_string_pretty(&response)?);
    }
    println!();

    // Test 5: tools/call (will fail because it's a mock API, but shows the flow)
    println!("5. Testing 'tools/call' method:");
    let call_request = McpMessage::request(
        5,
        "tools/call",
        Some(serde_json::json!({
            "name": "weather_get_forecast",
            "arguments": {
                "city": "San Francisco",
                "days": 3
            }
        })),
    );
    println!("   Request:  {}", serde_json::to_string(&call_request)?);

    if let Some(response) = handler.handle(call_request).await {
        // Note: This will show an HTTP error because the API doesn't exist
        println!("   Response: {}", serde_json::to_string_pretty(&response)?);
    }

    println!("\n=== Test Complete ===");

    Ok(())
}
