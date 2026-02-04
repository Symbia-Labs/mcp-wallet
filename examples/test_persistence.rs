//! Test that schema survives persistence (save/load cycle)

use mcp_server::tools::ToolGenerator;
use std::sync::Arc;
use wallet_core::Wallet;
use wallet_core::storage::EncryptedFileStorage;
use tempfile::TempDir;

#[tokio::main]
async fn main() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage_path = temp_dir.path().to_path_buf();

    println!("=== Phase 1: Create wallet and add integration ===");
    {
        let storage = Arc::new(
            EncryptedFileStorage::with_dir(storage_path.clone())
                .expect("Failed to create storage")
        );
        let mut wallet = Wallet::with_storage(storage);
        wallet.initialize("test-password").await.expect("Failed to initialize wallet");

        let spec_url = "https://app.stainless.com/api/spec/documented/openai/openapi.documented.yml";
        wallet.integrations
            .add_from_url("openai", spec_url)
            .await
            .expect("Failed to add integration");

        // Check schema before save
        let stored = wallet.integrations.get_stored("openai").await.unwrap();
        let generator = ToolGenerator::new();
        let tools = generator.generate_tools("openai", &stored);

        let chat_tool = tools.iter().find(|t| t.name == "openai_create_chat_completion").unwrap();
        let props = chat_tool.input_schema.properties.as_ref();
        println!("BEFORE SAVE - properties count: {}", props.map(|p| p.len()).unwrap_or(0));
        println!("BEFORE SAVE - has model: {}", props.map(|p| p.contains_key("model")).unwrap_or(false));
        println!("BEFORE SAVE - has messages: {}", props.map(|p| p.contains_key("messages")).unwrap_or(false));

        // Wallet will be dropped here, saving to storage
        wallet.lock().await.expect("Failed to lock");
    }

    println!("\n=== Phase 2: Load wallet from storage ===");
    {
        let storage = Arc::new(
            EncryptedFileStorage::with_dir(storage_path.clone())
                .expect("Failed to create storage")
        );
        let mut wallet = Wallet::with_storage(storage);

        // Unlock to load integrations
        wallet.unlock("test-password").await.expect("Failed to unlock");

        // Check schema after load
        let stored = wallet.integrations.get_stored("openai").await;

        match stored {
            Some(s) => {
                println!("Integration loaded successfully");
                println!("Operations count: {}", s.operations.len());

                // Find createChatCompletion operation
                let op = s.operations.iter().find(|o| o.operation_id == "createChatCompletion");
                match op {
                    Some(op) => {
                        println!("createChatCompletion found");
                        println!("  has request_body: {}", op.request_body.is_some());
                        if let Some(body) = &op.request_body {
                            println!("  request_body.required: {}", body.required);
                            println!("  request_body.schema is_some: {}", body.schema.is_some());
                            if let Some(schema) = &body.schema {
                                let keys: Vec<_> = schema.as_object().map(|o| o.keys().collect()).unwrap_or_default();
                                println!("  request_body.schema keys: {:?}", keys);
                            }
                        }
                    }
                    None => println!("createChatCompletion NOT FOUND!"),
                }

                // Generate tools
                let generator = ToolGenerator::new();
                let tools = generator.generate_tools("openai", &s);

                let chat_tool = tools.iter().find(|t| t.name == "openai_create_chat_completion");
                match chat_tool {
                    Some(tool) => {
                        let props = tool.input_schema.properties.as_ref();
                        println!("\nAFTER LOAD - properties count: {}", props.map(|p| p.len()).unwrap_or(0));
                        println!("AFTER LOAD - has model: {}", props.map(|p| p.contains_key("model")).unwrap_or(false));
                        println!("AFTER LOAD - has messages: {}", props.map(|p| p.contains_key("messages")).unwrap_or(false));

                        if props.map(|p| p.len()).unwrap_or(0) == 0 {
                            println!("\n!!! SCHEMA LOST DURING PERSISTENCE !!!");
                        } else {
                            println!("\n✓ Schema preserved correctly!");
                        }
                    }
                    None => println!("Tool not found after load!"),
                }
            }
            None => println!("Integration NOT FOUND after load!"),
        }
    }
}
