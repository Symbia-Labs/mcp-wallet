//! Deep debug to find where required fields are defined

use openapi_parser::OpenApiParser;
use serde_json::Value;

fn find_required(value: &Value, path: &str) {
    match value {
        Value::Object(obj) => {
            // Check if this object has a required field
            if let Some(req) = obj.get("required") {
                if req.is_array() && !req.as_array().unwrap().is_empty() {
                    println!("FOUND required at {}: {:?}", path, req);
                }
            }

            // Recurse into all fields
            for (key, val) in obj {
                let new_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };
                find_required(val, &new_path);
            }
        }
        Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                let new_path = format!("{}[{}]", path, i);
                find_required(val, &new_path);
            }
        }
        _ => {}
    }
}

#[tokio::main]
async fn main() {
    let spec_url = "https://app.stainless.com/api/spec/documented/openai/openapi.documented.yml";
    let spec = OpenApiParser::fetch_and_parse(spec_url)
        .await
        .expect("Failed to parse spec");

    for op in &spec.operations {
        if op.operation_id == "createResponse" {
            println!("=== Searching for 'required' in createResponse schema ===\n");

            if let Some(body) = &op.request_body {
                if let Some(schema) = &body.schema {
                    find_required(schema, "");
                }
            }
            break;
        }
    }
}
