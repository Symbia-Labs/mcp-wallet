//! Check the model property schema specifically

use mcp_server::tools::ToolGenerator;
use openapi_parser::OpenApiParser;

#[tokio::main]
async fn main() {
    let spec_url = "https://app.stainless.com/api/spec/documented/openai/openapi.documented.yml";
    let spec = OpenApiParser::fetch_and_parse(spec_url)
        .await
        .expect("Failed to parse spec");

    let generator = ToolGenerator::new();

    for op in &spec.operations {
        if op.operation_id == "createChatCompletion" {
            let tool = generator.generate_tool("openai", op);

            println!("=== Tool: {} ===", tool.name);

            let properties = tool.input_schema.properties.as_ref().unwrap();

            // Check model property
            println!("\n=== 'model' property ===");
            if let Some(model_prop) = properties.get("model") {
                println!("{}", serde_json::to_string_pretty(model_prop).unwrap());
            } else {
                println!("NOT FOUND!");
            }

            // Check messages property
            println!("\n=== 'messages' property (first 50 lines) ===");
            if let Some(messages_prop) = properties.get("messages") {
                let json = serde_json::to_string_pretty(messages_prop).unwrap();
                for (i, line) in json.lines().enumerate() {
                    if i < 50 {
                        println!("{}", line);
                    } else {
                        println!("... ({} total lines)", json.lines().count());
                        break;
                    }
                }
            } else {
                println!("NOT FOUND!");
            }

            break;
        }
    }
}
