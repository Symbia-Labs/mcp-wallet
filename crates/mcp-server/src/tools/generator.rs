//! Generate MCP tools from OpenAPI operations

use openapi_parser::{ApiOperation, ParameterLocation, HttpMethod};
use wallet_core::StoredIntegration;
use crate::protocol::{McpTool, McpInputSchema};
use serde_json::{Value, Map};

/// Generator for MCP tools from OpenAPI specs
pub struct ToolGenerator;

/// Sanitize a property name to match Claude's pattern: ^[a-zA-Z0-9_.-]{1,64}$
fn sanitize_property_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-' {
                c
            } else {
                '_' // Replace invalid chars with underscore
            }
        })
        .collect();

    // Truncate to 64 chars max
    if sanitized.len() > 64 {
        sanitized[..64].to_string()
    } else if sanitized.is_empty() {
        "param".to_string()
    } else {
        sanitized
    }
}

impl ToolGenerator {
    /// Create a new tool generator
    pub fn new() -> Self {
        Self
    }

    /// Generate MCP tools from an integration
    pub fn generate_tools(&self, integration_key: &str, stored: &StoredIntegration) -> Vec<McpTool> {
        stored
            .operations
            .iter()
            .map(|op| self.generate_tool(integration_key, op))
            .collect()
    }

    /// Generate a single MCP tool from an operation
    pub fn generate_tool(&self, integration_key: &str, operation: &ApiOperation) -> McpTool {
        // Generate tool name: {integration}_{operation_id}
        // Replace dots and other special chars with underscores
        let tool_name = format!(
            "{}_{}",
            integration_key,
            operation.normalized_id.replace('.', "_")
        );

        // Build description
        let description = self.build_description(operation);

        // Build input schema
        let input_schema = self.build_input_schema(operation);

        McpTool {
            name: tool_name,
            description: Some(description),
            input_schema,
        }
    }

    /// Build tool description from operation
    fn build_description(&self, operation: &ApiOperation) -> String {
        let mut parts = Vec::new();

        // Add summary
        if let Some(summary) = &operation.summary {
            parts.push(summary.clone());
        }

        // Add description if different from summary
        if let Some(desc) = &operation.description {
            if operation.summary.as_ref() != Some(desc) {
                parts.push(desc.clone());
            }
        }

        // Add HTTP method and path
        parts.push(format!("[{} {}]", operation.method, operation.path));

        // Add deprecation warning
        if operation.deprecated {
            parts.push("(DEPRECATED)".to_string());
        }

        parts.join("\n\n")
    }

    /// Build JSON Schema for tool inputs
    fn build_input_schema(&self, operation: &ApiOperation) -> McpInputSchema {
        let mut properties = Map::new();
        let mut required = Vec::new();

        // Add path parameters
        for param in &operation.parameters {
            if param.location == ParameterLocation::Path {
                self.add_parameter(&mut properties, &mut required, param);
            }
        }

        // Add query parameters
        for param in &operation.parameters {
            if param.location == ParameterLocation::Query {
                self.add_parameter(&mut properties, &mut required, param);
            }
        }

        // Add header parameters (non-auth)
        let skip_headers = ["authorization", "x-api-key", "api-key"];
        for param in &operation.parameters {
            if param.location == ParameterLocation::Header {
                let name_lower = param.name.to_lowercase();
                if !skip_headers.contains(&name_lower.as_str()) {
                    self.add_parameter(&mut properties, &mut required, param);
                }
            }
        }

        // Add request body properties
        if let Some(body) = &operation.request_body {
            self.merge_body_schema(&mut properties, &mut required, body);
        }

        McpInputSchema {
            schema_type: "object".to_string(),
            properties: if properties.is_empty() { None } else { Some(properties) },
            required: if required.is_empty() { None } else { Some(required) },
        }
    }

    /// Add a parameter to the schema
    fn add_parameter(
        &self,
        properties: &mut Map<String, Value>,
        required: &mut Vec<String>,
        param: &openapi_parser::OperationParameter,
    ) {
        let mut prop = Map::new();

        // Use parameter schema if available
        if let Some(schema) = &param.schema {
            if let Some(obj) = schema.as_object() {
                prop = obj.clone();
            }
        }

        // Default to string type
        if !prop.contains_key("type") {
            prop.insert("type".to_string(), serde_json::json!("string"));
        }

        // Add description with location hint
        let location_hint = match param.location {
            ParameterLocation::Path => "(path parameter)",
            ParameterLocation::Query => "(query parameter)",
            ParameterLocation::Header => "(header)",
            ParameterLocation::Cookie => "(cookie)",
        };

        let desc = param.description.as_deref().unwrap_or("");
        prop.insert(
            "description".to_string(),
            serde_json::json!(format!("{} {}", desc, location_hint).trim()),
        );

        // Add example if available
        if let Some(example) = &param.example {
            prop.insert("example".to_string(), example.clone());
        }

        let sanitized_name = sanitize_property_name(&param.name);
        properties.insert(sanitized_name.clone(), Value::Object(prop));

        if param.required {
            required.push(sanitized_name);
        }
    }

    /// Merge request body schema into properties
    fn merge_body_schema(
        &self,
        properties: &mut Map<String, Value>,
        required: &mut Vec<String>,
        body: &openapi_parser::RequestBody,
    ) {
        if let Some(schema) = &body.schema {
            if let Some(obj) = schema.as_object() {
                // Merge properties from body schema
                if let Some(body_props) = obj.get("properties").and_then(|p| p.as_object()) {
                    for (key, value) in body_props {
                        let mut prop = value.clone();

                        // Add "(body)" hint to description
                        if let Some(obj) = prop.as_object_mut() {
                            let desc = obj
                                .get("description")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            obj.insert(
                                "description".to_string(),
                                serde_json::json!(format!("{} (body)", desc).trim()),
                            );
                        }

                        let sanitized_key = sanitize_property_name(key);
                        properties.insert(sanitized_key, prop);
                    }
                }

                // Add required fields from body
                if body.required {
                    if let Some(body_required) = obj.get("required").and_then(|r| r.as_array()) {
                        for r in body_required {
                            if let Some(s) = r.as_str() {
                                let sanitized = sanitize_property_name(s);
                                if !required.contains(&sanitized) {
                                    required.push(sanitized);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Default for ToolGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use openapi_parser::{OperationParameter, HttpMethod};

    fn test_operation() -> ApiOperation {
        ApiOperation {
            operation_id: "createCustomer".to_string(),
            normalized_id: "customers.create".to_string(),
            method: HttpMethod::Post,
            path: "/v1/customers".to_string(),
            summary: Some("Create a customer".to_string()),
            description: Some("Creates a new customer object.".to_string()),
            tags: vec!["Customers".to_string()],
            deprecated: false,
            parameters: vec![
                OperationParameter {
                    name: "email".to_string(),
                    location: ParameterLocation::Query,
                    required: true,
                    description: Some("Customer email".to_string()),
                    schema: Some(serde_json::json!({"type": "string", "format": "email"})),
                    example: Some(serde_json::json!("user@example.com")),
                    deprecated: false,
                },
            ],
            request_body: None,
            responses: vec![],
            security: vec![],
        }
    }

    #[test]
    fn test_generate_tool_name() {
        let generator = ToolGenerator::new();
        let operation = test_operation();

        let tool = generator.generate_tool("stripe", &operation);

        assert_eq!(tool.name, "stripe_customers_create");
    }

    #[test]
    fn test_generate_tool_description() {
        let generator = ToolGenerator::new();
        let operation = test_operation();

        let tool = generator.generate_tool("stripe", &operation);

        assert!(tool.description.as_ref().unwrap().contains("Create a customer"));
        assert!(tool.description.as_ref().unwrap().contains("[POST /v1/customers]"));
    }

    #[test]
    fn test_generate_tool_input_schema() {
        let generator = ToolGenerator::new();
        let operation = test_operation();

        let tool = generator.generate_tool("stripe", &operation);

        let props = tool.input_schema.properties.as_ref().unwrap();
        assert!(props.contains_key("email"));

        let required = tool.input_schema.required.as_ref().unwrap();
        assert!(required.contains(&"email".to_string()));
    }
}
