//! Debug the raw createResponse schema structure

use openapi_parser::OpenApiParser;

#[tokio::main]
async fn main() {
    let spec_url = "https://app.stainless.com/api/spec/documented/openai/openapi.documented.yml";
    let spec = OpenApiParser::fetch_and_parse(spec_url)
        .await
        .expect("Failed to parse spec");

    for op in &spec.operations {
        if op.operation_id == "createResponse" {
            println!("=== createResponse raw schema ===\n");

            if let Some(body) = &op.request_body {
                if let Some(schema) = &body.schema {
                    // Print full schema structure (limited)
                    let json = serde_json::to_string_pretty(schema).unwrap();

                    // Look for where required fields are defined
                    println!("Looking for 'required' in schema...\n");

                    // Check top level
                    if let Some(req) = schema.get("required") {
                        println!("TOP LEVEL required: {:?}", req);
                    }

                    // Check allOf items
                    if let Some(all_of) = schema.get("allOf").and_then(|a| a.as_array()) {
                        println!("\nallOf has {} items:\n", all_of.len());
                        for (i, item) in all_of.iter().enumerate() {
                            println!("--- allOf[{}] ---", i);

                            // Check for required in this item
                            if let Some(req) = item.get("required") {
                                println!("  required: {:?}", req);
                            } else {
                                println!("  required: NOT PRESENT");
                            }

                            // Check for properties in this item
                            if let Some(props) = item.get("properties").and_then(|p| p.as_object()) {
                                println!("  properties: {} fields", props.len());
                                for key in props.keys().take(5) {
                                    println!("    - {}", key);
                                }
                                if props.len() > 5 {
                                    println!("    ... and {} more", props.len() - 5);
                                }
                            } else {
                                println!("  properties: NOT PRESENT");
                            }

                            // Check for discriminator / oneOf / anyOf
                            for key in ["oneOf", "anyOf", "discriminator", "$ref"] {
                                if item.get(key).is_some() {
                                    println!("  has {}", key);
                                }
                            }

                            println!();
                        }
                    }

                    // Print first 100 lines of full schema
                    println!("\n=== First 100 lines of schema ===\n");
                    for (i, line) in json.lines().enumerate() {
                        if i < 100 {
                            println!("{}", line);
                        } else {
                            println!("... ({} total lines)", json.lines().count());
                            break;
                        }
                    }
                }
            }
            break;
        }
    }
}
