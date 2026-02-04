//! Real integration test - full wallet setup + MCP tool execution

use mcp_server::tools::ToolExecutor;
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::RwLock;
use wallet_core::storage::EncryptedFileStorage;
use wallet_core::Wallet;

#[tokio::main]
async fn main() {
    // Get API key from environment
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    println!("OpenAI API key found ({} chars)", api_key.len());

    // Create a temporary wallet directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    println!("Using temp wallet at: {:?}", temp_dir.path());

    // Create storage and wallet
    let storage = Arc::new(
        EncryptedFileStorage::with_dir(temp_dir.path().to_path_buf())
            .expect("Failed to create storage"),
    );
    let mut wallet = Wallet::with_storage(storage);

    // Initialize wallet with a test password
    println!("\n=== Initializing wallet ===");
    wallet
        .initialize("test-password")
        .await
        .expect("Failed to initialize wallet");
    println!("Wallet initialized and unlocked");

    // Add OpenAI integration from spec URL
    println!("\n=== Adding OpenAI integration ===");
    let spec_url = "https://app.stainless.com/api/spec/documented/openai/openapi.documented.yml";
    let integration = wallet
        .integrations
        .add_from_url("openai", spec_url)
        .await
        .expect("Failed to add OpenAI integration");

    println!(
        "Integration added: {} ({} operations)",
        integration.name, integration.operation_count
    );

    // Add API key credential
    println!("\n=== Adding API key credential ===");
    let credential = wallet
        .credentials
        .add_api_key("openai", "OpenAI API Key", &api_key)
        .await
        .expect("Failed to add credential");
    println!("Credential added: {}", credential.name);

    // Bind credential to integration
    wallet
        .integrations
        .set_credential("openai", credential.id)
        .await
        .expect("Failed to bind credential");
    println!("Credential bound to integration");

    // Wrap wallet in Arc<RwLock> for executor
    let wallet = Arc::new(RwLock::new(wallet));

    // Create executor
    let executor = ToolExecutor::new(wallet.clone());

    // Call the tool
    println!("\n=== Calling openai_create_chat_completion ===");
    let args = json!({
        "model": "gpt-4o-mini",
        "messages": [
            {"role": "user", "content": "Say 'Hello from MCP Wallet!' and nothing else."}
        ]
    });

    println!("Args: {}", serde_json::to_string_pretty(&args).unwrap());

    match executor
        .execute("openai_create_chat_completion", Some(args))
        .await
    {
        Ok(result) => {
            println!("\n=== SUCCESS ===");
            // Pretty print the full result
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }
        Err(e) => {
            println!("\n=== ERROR ===");
            println!("{:?}", e);
        }
    }
}
