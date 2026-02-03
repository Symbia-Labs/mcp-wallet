use openapi_parser::{OpenApiParser, NamespaceTree};
use mcp_server::tools::ToolGenerator;
use wallet_core::{Integration, StoredIntegration};

#[tokio::main]
async fn main() {
    let url = "https://app.stainless.com/api/spec/documented/openai/openapi.documented.yml";

    println!("Fetching OpenAI spec...");
    let spec = OpenApiParser::fetch_and_parse(url).await.unwrap();

    println!("Found {} operations\n", spec.operations.len());

    // Find chat completions
    for op in &spec.operations {
        if op.path.contains("/chat/completions") && op.method.to_string() == "POST" {
            println!("=== {} {} ===", op.method, op.path);
            println!("Operation ID: {}", op.operation_id);

            // Generate the tool directly from the operation
            let generator = ToolGenerator::new();
            let tool = generator.generate_tool("openai", op);

            println!("\n=== Generated MCP Tool ===");
            println!("Tool name: {}", tool.name);

            if let Some(props) = &tool.input_schema.properties {
                println!("\nInput properties ({}):", props.len());
                for (key, _val) in props.iter().take(15) {
                    println!("  - {}", key);
                }
                if props.len() > 15 {
                    println!("  ... and {} more", props.len() - 15);
                }
            } else {
                println!("\nERROR: No input properties!");
            }

            if let Some(required) = &tool.input_schema.required {
                println!("\nRequired: {:?}", required);
            }

            break;
        }
    }
}
