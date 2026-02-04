//! Test actual execution of /responses endpoint

use mcp_server::tools::ToolExecutor;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use wallet_core::Wallet;
use wallet_core::storage::EncryptedFileStorage;
use tempfile::TempDir;

#[tokio::main]
async fn main() {
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "test-key".to_string());

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage = Arc::new(
        EncryptedFileStorage::with_dir(temp_dir.path().to_path_buf())
            .expect("Failed to create storage")
    );
    let mut wallet = Wallet::with_storage(storage);
    wallet.initialize("test-password").await.expect("Failed to initialize wallet");

    let spec_url = "https://app.stainless.com/api/spec/documented/openai/openapi.documented.yml";
    wallet.integrations
        .add_from_url("openai", spec_url)
        .await
        .expect("Failed to add integration");

    let credential = wallet.credentials
        .add_api_key("openai", "OpenAI API Key", &api_key)
        .await
        .expect("Failed to add credential");
    wallet.integrations
        .set_credential("openai", credential.id)
        .await
        .expect("Failed to bind credential");

    // Look up the operation
    let stored = wallet.integrations.get_stored("openai").await.unwrap();
    let op = stored.operations.iter().find(|o| o.operation_id == "createResponse").unwrap();

    println!("=== Operation Details ===");
    println!("operation_id: {}", op.operation_id);
    println!("normalized_id: {}", op.normalized_id);
    println!("method: {:?}", op.method);
    println!("path: {}", op.path);

    println!("\n=== Parameters ===");
    for p in &op.parameters {
        println!("  {} ({:?}): required={}", p.name, p.location, p.required);
    }

    println!("\n=== Tool name parsing ===");
    // The tool name would be: openai_create_response
    // parse_tool_name splits at first underscore:
    // - integration: "openai"
    // - operation_path: "create_response" -> "create.response"
    let tool_name = "openai_create_response";
    let parts: Vec<&str> = tool_name.splitn(2, '_').collect();
    println!("Tool name: {}", tool_name);
    println!("Integration key: {}", parts[0]);
    println!("Operation path (raw): {}", parts[1]);
    println!("Operation path (normalized): {}", parts[1].replace('_', "."));

    // Check if this matches the normalized_id
    let expected_path = parts[1].replace('_', ".");
    println!("\nNormalized ID in spec: {}", op.normalized_id);
    println!("Expected path from tool name: {}", expected_path);
    println!("Match: {}", op.normalized_id == expected_path);

    let wallet = Arc::new(RwLock::new(wallet));
    let executor = ToolExecutor::new(wallet.clone());

    // Test with exact parameters QA used
    let args = json!({
        "model": "gpt-4o",
        "input": "How's it going today?"
    });

    println!("\n=== Executing Tool ===");
    println!("Arguments: {}", serde_json::to_string_pretty(&args).unwrap());

    match executor.execute("openai_create_response", Some(args)).await {
        Ok(result) => {
            println!("\n=== Result ===");
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }
        Err(e) => {
            println!("\n=== Error ===");
            println!("{:?}", e);
        }
    }
}
