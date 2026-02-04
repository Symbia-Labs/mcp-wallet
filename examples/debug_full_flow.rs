//! Full flow debug - trace every step

use mcp_server::tools::ToolExecutor;
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::RwLock;
use wallet_core::storage::EncryptedFileStorage;
use wallet_core::Wallet;

#[tokio::main]
async fn main() {
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "test-key".to_string());

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

    println!("=== Adding integration ===");
    let spec_url = "https://app.stainless.com/api/spec/documented/openai/openapi.documented.yml";
    let _integration = wallet
        .integrations
        .add_from_url("openai", spec_url)
        .await
        .expect("Failed to add OpenAI integration");

    println!("\n=== Getting stored integration ===");
    let stored = wallet
        .integrations
        .get_stored("openai")
        .await
        .expect("Failed to get stored integration");

    println!("Operations count: {}", stored.operations.len());

    // Find the createChatCompletion operation
    println!("\n=== Finding createChatCompletion in operations list ===");
    for (i, op) in stored.operations.iter().enumerate() {
        if op.operation_id == "createChatCompletion" {
            println!(
                "Found at index {}: operation_id={}, normalized_id={}, path={}",
                i, op.operation_id, op.normalized_id, op.path
            );
        }
    }

    // Test namespace lookup
    println!("\n=== Testing namespace lookup ===");
    let test_paths = [
        "create.chat.completion",
        "create.chat.completions",
        "chat.completion.create",
        "chat.completions.create",
    ];

    for path in &test_paths {
        match stored.lookup_operation(path) {
            Some(op) => println!(
                "  lookup('{}') -> Found: {} {}",
                path, op.operation_id, op.path
            ),
            None => println!("  lookup('{}') -> NOT FOUND", path),
        }
    }

    // Show some paths from namespace
    println!("\n=== Namespace paths (first 20) ===");
    let paths = stored.operation_paths();
    for path in paths.iter().take(20) {
        println!("  {}", path);
    }

    // Now test tool name parsing and lookup
    println!("\n=== Testing tool name parsing ===");
    let credential = wallet
        .credentials
        .add_api_key("openai", "OpenAI API Key", &api_key)
        .await
        .expect("Failed to add credential");
    wallet
        .integrations
        .set_credential("openai", credential.id)
        .await
        .expect("Failed to bind credential");

    let wallet = Arc::new(RwLock::new(wallet));
    let executor = ToolExecutor::new(wallet.clone());

    // Test the tool execution
    let args = json!({
        "model": "gpt-4o-mini",
        "messages": [{"role": "user", "content": "Hello"}]
    });

    println!("Calling: openai_create_chat_completion");
    match executor
        .execute("openai_create_chat_completion", Some(args))
        .await
    {
        Ok(result) => println!(
            "SUCCESS: {}",
            serde_json::to_string_pretty(&result).unwrap()
        ),
        Err(e) => println!("ERROR: {:?}", e),
    }
}
