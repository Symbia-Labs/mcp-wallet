//! Simulate MCP protocol to see exactly what Claude receives

use mcp_server::protocol::RequestHandler;
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::RwLock;
use wallet_core::storage::EncryptedFileStorage;
use wallet_core::Wallet;

#[tokio::main]
async fn main() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage = Arc::new(
        EncryptedFileStorage::with_dir(temp_dir.path().to_path_buf())
            .expect("Failed to create storage"),
    );
    let mut wallet = Wallet::with_storage(storage);
    wallet
        .initialize("test-password")
        .await
        .expect("Failed to initialize wallet");

    // Add OpenAI integration
    let spec_url = "https://app.stainless.com/api/spec/documented/openai/openapi.documented.yml";
    wallet
        .integrations
        .add_from_url("openai", spec_url)
        .await
        .expect("Failed to add OpenAI integration");

    // Add dummy credential
    let credential = wallet
        .credentials
        .add_api_key("openai", "OpenAI API Key", "sk-test")
        .await
        .expect("Failed to add credential");
    wallet
        .integrations
        .set_credential("openai", credential.id)
        .await
        .expect("Failed to bind credential");

    let wallet = Arc::new(RwLock::new(wallet));
    let mut handler = RequestHandler::new(wallet);

    // Initialize
    let init_msg = mcp_server::protocol::McpMessage::request(
        json!(1),
        "initialize",
        Some(json!({
            "protocolVersion": "2024-11-05",
            "clientInfo": {"name": "test", "version": "1.0"},
            "capabilities": {}
        })),
    );
    handler.handle(init_msg).await;

    // List tools
    let list_msg = mcp_server::protocol::McpMessage::request(json!(2), "tools/list", None);

    let response = handler.handle(list_msg).await.expect("No response");
    let result = response.result.expect("No result");

    // Find the chat completion tool
    let tools = result
        .get("tools")
        .and_then(|t| t.as_array())
        .expect("No tools");

    for tool in tools {
        let name = tool.get("name").and_then(|n| n.as_str()).unwrap_or("");
        if name == "openai_create_chat_completion" {
            println!("=== Found: {} ===", name);

            let input_schema = tool.get("inputSchema").expect("No inputSchema");
            println!("\ninputSchema.type: {:?}", input_schema.get("type"));

            let properties = input_schema.get("properties");
            match properties {
                Some(props) if props.is_object() => {
                    let obj = props.as_object().unwrap();
                    println!("\nproperties count: {}", obj.len());

                    // Show some key properties
                    for key in ["model", "messages", "temperature", "stream"] {
                        if let Some(prop) = obj.get(key) {
                            let prop_type = prop
                                .get("type")
                                .map(|t| t.to_string())
                                .unwrap_or("?".to_string());
                            let desc = prop
                                .get("description")
                                .and_then(|d| d.as_str())
                                .map(|s| &s[..s.len().min(60)])
                                .unwrap_or("?");
                            println!("  {}: type={}, desc=\"{}...\"", key, prop_type, desc);
                        } else {
                            println!("  {}: NOT FOUND!", key);
                        }
                    }
                }
                Some(props) => {
                    println!("\nproperties is not an object: {:?}", props);
                }
                None => {
                    println!("\nproperties is MISSING!");
                }
            }

            let required = input_schema.get("required");
            match required {
                Some(req) if req.is_array() => {
                    println!("\nrequired: {:?}", req);
                }
                _ => {
                    println!("\nrequired is MISSING or not an array!");
                }
            }

            // Print the full schema for analysis
            println!("\n=== Full inputSchema (first 100 lines) ===");
            let schema_json = serde_json::to_string_pretty(input_schema).unwrap();
            for (i, line) in schema_json.lines().enumerate() {
                if i < 100 {
                    println!("{}", line);
                } else {
                    println!("... ({} total lines)", schema_json.lines().count());
                    break;
                }
            }

            break;
        }
    }
}
