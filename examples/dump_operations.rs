//! Dump all operations to see what's being parsed

use openapi_parser::OpenApiParser;

#[tokio::main]
async fn main() {
    let spec_url = "https://app.stainless.com/api/spec/documented/openai/openapi.documented.yml";

    println!("Fetching spec from: {}", spec_url);
    let spec = OpenApiParser::fetch_and_parse(spec_url)
        .await
        .expect("Failed to parse spec");

    println!("\n=== Spec Info ===");
    println!("Title: {}", spec.title);
    println!("Operations: {}", spec.operations.len());

    println!("\n=== Looking for chat completions ===");
    for op in &spec.operations {
        if op.operation_id.to_lowercase().contains("chat")
            || op.operation_id.to_lowercase().contains("completion")
            || op.path.contains("/chat")
        {
            println!(
                "Found: operation_id={}, normalized_id={}, path={}",
                op.operation_id, op.normalized_id, op.path
            );
        }
    }

    println!("\n=== First 30 operations ===");
    for (i, op) in spec.operations.iter().take(30).enumerate() {
        println!(
            "{:3}. {} {} - {} (normalized: {})",
            i + 1,
            op.method,
            op.path,
            op.operation_id,
            op.normalized_id
        );
    }
}
