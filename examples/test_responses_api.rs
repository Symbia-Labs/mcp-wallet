//! Test the /responses endpoint specifically (what QA is testing)

use mcp_server::tools::ToolGenerator;
use openapi_parser::OpenApiParser;

#[tokio::main]
async fn main() {
    let spec_url = "https://app.stainless.com/api/spec/documented/openai/openapi.documented.yml";
    let spec = OpenApiParser::fetch_and_parse(spec_url)
        .await
        .expect("Failed to parse spec");

    println!("Looking for /responses operations...\n");

    let generator = ToolGenerator::new();

    for op in &spec.operations {
        // Look for the responses endpoint
        if op.path.contains("/responses") && !op.path.contains("usage") {
            println!("=== {} {} ===", op.method, op.path);
            println!("operation_id: {}", op.operation_id);
            println!("normalized_id: {}", op.normalized_id);

            // Check request body
            println!("\nrequest_body:");
            if let Some(body) = &op.request_body {
                println!("  required: {}", body.required);
                println!("  content_type: {}", body.content_type);
                if let Some(schema) = &body.schema {
                    let keys: Vec<_> = schema.as_object().map(|o| o.keys().collect()).unwrap_or_default();
                    println!("  schema top-level keys: {:?}", keys);

                    // Check if it has allOf
                    if let Some(all_of) = schema.get("allOf") {
                        println!("  has allOf with {} items", all_of.as_array().map(|a| a.len()).unwrap_or(0));
                    }

                    // Check if it has direct properties
                    if let Some(props) = schema.get("properties") {
                        println!("  has direct properties: {}", props.as_object().map(|o| o.len()).unwrap_or(0));
                    }
                } else {
                    println!("  schema: None!");
                }
            } else {
                println!("  NO REQUEST BODY!");
            }

            // Generate tool and check schema
            let tool = generator.generate_tool("openai", op);
            println!("\nGenerated MCP tool: {}", tool.name);

            let props = tool.input_schema.properties.as_ref();
            let prop_count = props.map(|p| p.len()).unwrap_or(0);
            println!("  properties count: {}", prop_count);

            if prop_count > 0 {
                let props = props.unwrap();
                println!("  has 'model': {}", props.contains_key("model"));
                println!("  has 'input': {}", props.contains_key("input"));

                // Show model property
                if let Some(model) = props.get("model") {
                    println!("\n  'model' property:");
                    let json = serde_json::to_string_pretty(model).unwrap();
                    for line in json.lines().take(10) {
                        println!("    {}", line);
                    }
                }
            } else {
                println!("\n  !!! NO PROPERTIES - THIS IS THE BUG !!!");
            }

            let required = tool.input_schema.required.as_ref();
            println!("\n  required: {:?}", required);

            println!("\n---\n");
        }
    }
}
