use mcp_server::tools::ToolGenerator;
use openapi_parser::OpenApiParser;

const SPECS: &[(&str, &str)] = &[
    ("openai", "https://app.stainless.com/api/spec/documented/openai/openapi.documented.yml"),
    ("github", "https://raw.githubusercontent.com/github/rest-api-description/main/descriptions/api.github.com/api.github.com.json"),
    ("stripe", "https://raw.githubusercontent.com/stripe/openapi/master/openapi/spec3.json"),
];

#[tokio::main]
async fn main() {
    let generator = ToolGenerator::new();

    for (name, url) in SPECS {
        println!("\n{}", "=".repeat(60));
        println!("Testing: {}", name);
        println!("URL: {}", url);
        println!("{}", "=".repeat(60));

        match OpenApiParser::fetch_and_parse(url).await {
            Ok(spec) => {
                println!("✓ Parsed successfully");
                println!("  Title: {}", spec.title);
                println!("  Operations: {}", spec.operations.len());

                // Test tool generation for a POST operation
                let post_ops: Vec<_> = spec
                    .operations
                    .iter()
                    .filter(|op| op.method.to_string() == "POST")
                    .take(3)
                    .collect();

                println!("\n  Sample POST operations:");
                for op in post_ops {
                    let tool = generator.generate_tool(name, op);
                    let prop_count = tool
                        .input_schema
                        .properties
                        .as_ref()
                        .map(|p| p.len())
                        .unwrap_or(0);
                    let req_count = tool
                        .input_schema
                        .required
                        .as_ref()
                        .map(|r| r.len())
                        .unwrap_or(0);

                    println!(
                        "  - {} {} -> {} props, {} required",
                        op.method, op.path, prop_count, req_count
                    );

                    if prop_count == 0 && op.request_body.is_some() {
                        println!("    ⚠️  WARNING: Has request body but no properties extracted!");
                    }
                }
            }
            Err(e) => {
                println!("✗ Failed to parse: {:?}", e);
            }
        }
    }

    println!("\n\nDone!");
}
