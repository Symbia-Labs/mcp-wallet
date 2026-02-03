//! MCP Wallet Demo - Demonstrates the full workflow
//!
//! Run with: cargo run --example demo

use std::sync::Arc;
use tokio::sync::RwLock;
use wallet_core::{Wallet, WalletState};
use mcp_server::protocol::{McpMessage, McpTool};
use mcp_server::tools::ToolGenerator;

const TEST_SPEC: &str = r#"
openapi: "3.0.0"
info:
  title: Pet Store API
  version: "1.0.0"
  description: A sample Pet Store API for demonstration
servers:
  - url: https://petstore.example.com/api/v1
paths:
  /pets:
    get:
      operationId: listPets
      summary: List all pets
      description: Returns a list of all pets in the store
      parameters:
        - name: limit
          in: query
          description: Maximum number of pets to return
          required: false
          schema:
            type: integer
            default: 10
        - name: species
          in: query
          description: Filter by species
          schema:
            type: string
            enum: [dog, cat, bird, fish]
      responses:
        '200':
          description: A list of pets
          content:
            application/json:
              schema:
                type: array
                items:
                  $ref: '#/components/schemas/Pet'
    post:
      operationId: createPet
      summary: Create a new pet
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required: [name, species]
              properties:
                name:
                  type: string
                  description: The pet's name
                species:
                  type: string
                  description: The species of the pet
                age:
                  type: integer
                  description: Age in years
      responses:
        '201':
          description: Pet created
  /pets/{petId}:
    get:
      operationId: getPet
      summary: Get a pet by ID
      parameters:
        - name: petId
          in: path
          required: true
          description: The ID of the pet
          schema:
            type: string
      responses:
        '200':
          description: The pet
    delete:
      operationId: deletePet
      summary: Delete a pet
      parameters:
        - name: petId
          in: path
          required: true
          schema:
            type: string
      responses:
        '204':
          description: Pet deleted
components:
  schemas:
    Pet:
      type: object
      properties:
        id:
          type: string
        name:
          type: string
        species:
          type: string
        age:
          type: integer
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
security:
  - bearerAuth: []
"#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MCP Wallet Demo ===\n");

    // Use a temporary directory for this demo
    let temp_dir = tempfile::TempDir::new()?;
    let storage = Arc::new(
        wallet_core::EncryptedFileStorage::with_dir(temp_dir.path().to_path_buf())?
    );
    let mut wallet = Wallet::with_storage(storage);

    // Step 1: Initialize the wallet
    println!("1. Initializing wallet with password...");
    wallet.initialize("demo-password-123").await?;
    println!("   ✓ Wallet initialized and unlocked\n");

    // Step 2: Add an integration from OpenAPI spec
    println!("2. Adding integration from OpenAPI spec...");
    let integration = wallet.integrations
        .add_from_content("petstore", TEST_SPEC)
        .await?;
    println!("   ✓ Integration added: {} ({})", integration.name, integration.key);
    println!("   ✓ Operations discovered: {}\n", integration.operation_count);

    // Step 3: Add a credential
    println!("3. Adding API credential...");
    let credential = wallet.credentials
        .add_api_key("petstore", "Demo API Key", "pk_demo_abc123xyz789")
        .await?;
    println!("   ✓ Credential added: {} (prefix: {})\n",
        credential.name,
        credential.prefix.as_deref().unwrap_or("N/A")
    );

    // Step 4: Bind credential to integration
    println!("4. Binding credential to integration...");
    wallet.integrations
        .set_credential("petstore", credential.id)
        .await?;
    println!("   ✓ Credential bound to integration\n");

    // Step 5: List operations
    println!("5. Listing available operations:");
    let operations = wallet.integrations.list_operations("petstore").await;
    for op in &operations {
        println!("   - {} {} -> {}", op.method, op.path, op.operation_id);
    }
    println!();

    // Step 6: Generate MCP tools
    println!("6. Generating MCP tools:");
    let stored = wallet.integrations.get_stored("petstore").await.unwrap();
    let generator = ToolGenerator::new();
    let tools = generator.generate_tools("petstore", &stored);

    for tool in &tools {
        println!("\n   Tool: {}", tool.name);
        if let Some(desc) = &tool.description {
            let first_line = desc.lines().next().unwrap_or("");
            println!("   Description: {}", first_line);
        }
        if let Some(props) = &tool.input_schema.properties {
            println!("   Parameters: {:?}", props.keys().collect::<Vec<_>>());
        }
        if let Some(required) = &tool.input_schema.required {
            println!("   Required: {:?}", required);
        }
    }
    println!();

    // Step 7: Simulate MCP protocol messages
    println!("7. Simulating MCP protocol:");

    // Initialize message
    let init_msg = McpMessage::request(
        1,
        "initialize",
        Some(serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "demo-client",
                "version": "1.0.0"
            }
        })),
    );
    println!("   → initialize request: {}", serde_json::to_string(&init_msg)?);

    // Tools list request
    let tools_list_msg = McpMessage::request(2, "tools/list", None);
    println!("   → tools/list request: {}", serde_json::to_string(&tools_list_msg)?);

    // Tools call request
    let tools_call_msg = McpMessage::request(
        3,
        "tools/call",
        Some(serde_json::json!({
            "name": "petstore_list.pets",
            "arguments": {
                "limit": 5,
                "species": "dog"
            }
        })),
    );
    println!("   → tools/call request: {}", serde_json::to_string(&tools_call_msg)?);
    println!();

    // Step 8: Lock and unlock wallet
    println!("8. Testing lock/unlock:");
    wallet.lock().await?;
    println!("   ✓ Wallet locked (state: {:?})", wallet.state());

    wallet.unlock("demo-password-123").await?;
    println!("   ✓ Wallet unlocked (state: {:?})\n", wallet.state());

    // Step 9: Verify credential decryption
    println!("9. Verifying credential encryption/decryption:");
    let decrypted = wallet.credentials.get_decrypted(credential.id).await?;
    let exposed = decrypted.expose();
    println!("   ✓ Decrypted credential: {}...{}",
        &exposed[..8.min(exposed.len())],
        &exposed[exposed.len().saturating_sub(4)..]
    );
    println!();

    println!("=== Demo Complete ===");
    println!("\nThe MCP Wallet successfully:");
    println!("  • Initialized with password-derived encryption");
    println!("  • Parsed OpenAPI spec and extracted {} operations", operations.len());
    println!("  • Stored encrypted credential");
    println!("  • Generated {} MCP tools", tools.len());
    println!("  • Verified lock/unlock cycle");

    Ok(())
}
