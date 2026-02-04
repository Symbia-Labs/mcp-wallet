//! Trace exactly what HTTP request is being built

use openapi_parser::OpenApiParser;
use serde_json::{json, Map, Value};

#[tokio::main]
async fn main() {
    let spec_url = "https://app.stainless.com/api/spec/documented/openai/openapi.documented.yml";
    let spec = OpenApiParser::fetch_and_parse(spec_url)
        .await
        .expect("Failed to parse spec");

    for op in &spec.operations {
        if op.operation_id == "createChatCompletion" {
            println!("=== Operation: {} ===", op.operation_id);
            println!("Method: {:?}", op.method);
            println!("Path: {}", op.path);

            // Simulated arguments from Claude
            let args = json!({
                "model": "gpt-4o-mini",
                "messages": [
                    {"role": "user", "content": "Hello"}
                ]
            });
            let args_map = args.as_object().unwrap();

            println!("\n=== Input arguments ===");
            println!("{}", serde_json::to_string_pretty(&args).unwrap());

            // Simulate body building (same logic as executor.rs)
            let path_query_params: Vec<&str> = op
                .parameters
                .iter()
                .filter(|p| {
                    matches!(
                        p.location,
                        openapi_parser::ParameterLocation::Path
                            | openapi_parser::ParameterLocation::Query
                            | openapi_parser::ParameterLocation::Header
                    )
                })
                .map(|p| p.name.as_str())
                .collect();

            println!("\n=== Path/Query/Header params to exclude from body ===");
            println!("{:?}", path_query_params);

            let mut body = Map::new();
            for (key, value) in args_map {
                if !path_query_params.contains(&key.as_str()) {
                    body.insert(key.clone(), value.clone());
                }
            }

            println!("\n=== Final request body ===");
            if body.is_empty() {
                println!("BODY IS EMPTY!");
            } else {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&Value::Object(body)).unwrap()
                );
            }

            // Check if operation has request_body
            println!("\n=== Operation request_body ===");
            if let Some(rb) = &op.request_body {
                println!("required: {}", rb.required);
                println!("content_type: {}", rb.content_type);
                if let Some(schema) = &rb.schema {
                    println!(
                        "schema keys: {:?}",
                        schema.as_object().map(|o| o.keys().collect::<Vec<_>>())
                    );
                } else {
                    println!("schema: None");
                }
            } else {
                println!("No request_body defined!");
            }

            break;
        }
    }
}
