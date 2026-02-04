//! Debug test - shows exactly what HTTP request would be sent

use mcp_server::tools::ToolExecutor;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use wallet_core::Wallet;
use wallet_core::storage::EncryptedFileStorage;
use tempfile::TempDir;

#[tokio::main]
async fn main() {
    // Get API key from environment
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    println!("API Key: {}...{}", &api_key[..7], &api_key[api_key.len()-4..]);
    println!("API Key length: {} chars", api_key.len());

    // Create a temporary wallet directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create storage and wallet
    let storage = Arc::new(
        EncryptedFileStorage::with_dir(temp_dir.path().to_path_buf())
            .expect("Failed to create storage")
    );
    let mut wallet = Wallet::with_storage(storage);

    // Initialize wallet
    wallet.initialize("test-password").await.expect("Failed to initialize wallet");

    // Add OpenAI integration
    let spec_url = "https://app.stainless.com/api/spec/documented/openai/openapi.documented.yml";
    let integration = wallet.integrations
        .add_from_url("openai", spec_url)
        .await
        .expect("Failed to add OpenAI integration");

    println!("\n=== Integration Info ===");
    println!("Server URL: {}", integration.server_url);
    println!("Operations: {}", integration.operation_count);

    // List a few operations
    let operations = wallet.integrations.list_operations("openai").await;
    println!("\nLooking for chat completion operation...");
    for op in &operations {
        if op.operation_id.contains("chat") || op.operation_id.contains("completion") {
            println!("  Found: {} {} - {}", op.method, op.path, op.operation_id);
        }
    }

    // Look up the specific operation
    if let Some(stored) = wallet.integrations.get_stored("openai").await {
        println!("\n=== Looking up create_chat_completion ===");

        // Try different lookup patterns
        let patterns = ["chat.completions.create", "create_chat_completion", "createChatCompletion"];
        for pattern in patterns {
            if let Some(op) = stored.lookup_operation(pattern) {
                println!("Found with pattern '{}': {} {}", pattern, op.method, op.path);
            } else {
                println!("NOT FOUND with pattern '{}'", pattern);
            }
        }

        // Show all operation paths
        println!("\n=== All Operation Paths ===");
        let paths = stored.operation_paths();
        for path in paths.iter().take(20) {
            println!("  {}", path);
        }
        if paths.len() > 20 {
            println!("  ... and {} more", paths.len() - 20);
        }
    }

    // Add credential
    let credential = wallet.credentials
        .add_api_key("openai", "OpenAI API Key", &api_key)
        .await
        .expect("Failed to add credential");

    wallet.integrations
        .set_credential("openai", credential.id)
        .await
        .expect("Failed to bind credential");

    // Now let's also make a direct test call to verify the API key works
    println!("\n=== Direct API test (bypassing MCP) ===");
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&json!({
            "model": "gpt-4o-mini",
            "messages": [{"role": "user", "content": "Say hello"}]
        }))
        .send()
        .await
        .expect("Direct request failed");

    println!("Direct API status: {}", response.status());
    let body = response.text().await.unwrap();
    println!("Direct API response: {}", &body[..body.len().min(500)]);

    // Now test via MCP
    let wallet = Arc::new(RwLock::new(wallet));
    let executor = ToolExecutor::new(wallet.clone());

    println!("\n=== MCP Tool execution ===");
    let args = json!({
        "model": "gpt-4o-mini",
        "messages": [
            {"role": "user", "content": "Say 'Hello from MCP Wallet!' and nothing else."}
        ]
    });

    match executor.execute("openai_create_chat_completion", Some(args)).await {
        Ok(result) => {
            println!("SUCCESS:");
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }
        Err(e) => {
            println!("ERROR: {:?}", e);
        }
    }
}
