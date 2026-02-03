//! JSON Schema $ref resolver for OpenAPI specs

use indexmap::IndexMap;
use serde_json::Value;

/// Resolves $ref references in JSON schemas
pub struct SchemaResolver<'a> {
    /// Component schemas from the OpenAPI spec
    schemas: &'a IndexMap<String, Value>,
    /// Maximum recursion depth to prevent infinite loops
    max_depth: usize,
}

impl<'a> SchemaResolver<'a> {
    /// Create a new resolver with the given component schemas
    pub fn new(schemas: &'a IndexMap<String, Value>) -> Self {
        Self {
            schemas,
            max_depth: 10,
        }
    }

    /// Resolve a schema, following $ref references
    pub fn resolve(&self, schema: &Value) -> Value {
        self.resolve_with_depth(schema, 0)
    }

    fn resolve_with_depth(&self, schema: &Value, depth: usize) -> Value {
        if depth > self.max_depth {
            return schema.clone();
        }

        match schema {
            Value::Object(obj) => {
                // Check for $ref
                if let Some(ref_value) = obj.get("$ref") {
                    if let Some(ref_str) = ref_value.as_str() {
                        if let Some(resolved) = self.resolve_ref(ref_str) {
                            // Recursively resolve the referenced schema
                            return self.resolve_with_depth(&resolved, depth + 1);
                        }
                    }
                }

                // Not a $ref, recursively resolve nested schemas
                let mut result = serde_json::Map::new();
                for (key, value) in obj {
                    let resolved = match key.as_str() {
                        // Resolve schema-containing fields
                        "properties" => self.resolve_properties(value, depth),
                        "items" => self.resolve_with_depth(value, depth + 1),
                        "additionalProperties" if value.is_object() => {
                            self.resolve_with_depth(value, depth + 1)
                        }
                        "allOf" | "oneOf" | "anyOf" => self.resolve_array(value, depth),
                        _ => value.clone(),
                    };
                    result.insert(key.clone(), resolved);
                }
                Value::Object(result)
            }
            _ => schema.clone(),
        }
    }

    fn resolve_ref(&self, ref_str: &str) -> Option<Value> {
        // Parse refs like "#/components/schemas/CreateChatCompletionRequest"
        const PREFIX: &str = "#/components/schemas/";
        if ref_str.starts_with(PREFIX) {
            let schema_name = &ref_str[PREFIX.len()..];
            self.schemas.get(schema_name).cloned()
        } else {
            None
        }
    }

    fn resolve_properties(&self, value: &Value, depth: usize) -> Value {
        if let Some(obj) = value.as_object() {
            let mut result = serde_json::Map::new();
            for (key, prop_schema) in obj {
                result.insert(key.clone(), self.resolve_with_depth(prop_schema, depth + 1));
            }
            Value::Object(result)
        } else {
            value.clone()
        }
    }

    fn resolve_array(&self, value: &Value, depth: usize) -> Value {
        if let Some(arr) = value.as_array() {
            Value::Array(
                arr.iter()
                    .map(|item| self.resolve_with_depth(item, depth + 1))
                    .collect(),
            )
        } else {
            value.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_resolve_simple_ref() {
        let mut schemas = IndexMap::new();
        schemas.insert(
            "User".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "email": {"type": "string"}
                },
                "required": ["name", "email"]
            }),
        );

        let resolver = SchemaResolver::new(&schemas);
        let schema = json!({"$ref": "#/components/schemas/User"});
        let resolved = resolver.resolve(&schema);

        assert_eq!(resolved["type"], "object");
        assert!(resolved["properties"]["name"].is_object());
    }

    #[test]
    fn test_resolve_nested_ref() {
        let mut schemas = IndexMap::new();
        schemas.insert(
            "Address".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "street": {"type": "string"}
                }
            }),
        );
        schemas.insert(
            "User".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "address": {"$ref": "#/components/schemas/Address"}
                }
            }),
        );

        let resolver = SchemaResolver::new(&schemas);
        let schema = json!({"$ref": "#/components/schemas/User"});
        let resolved = resolver.resolve(&schema);

        assert_eq!(resolved["properties"]["address"]["type"], "object");
        assert!(resolved["properties"]["address"]["properties"]["street"].is_object());
    }

    #[test]
    fn test_resolve_allof() {
        let mut schemas = IndexMap::new();
        schemas.insert(
            "Base".to_string(),
            json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"}
                }
            }),
        );

        let resolver = SchemaResolver::new(&schemas);
        let schema = json!({
            "allOf": [
                {"$ref": "#/components/schemas/Base"},
                {"properties": {"name": {"type": "string"}}}
            ]
        });
        let resolved = resolver.resolve(&schema);

        let all_of = resolved["allOf"].as_array().unwrap();
        assert_eq!(all_of[0]["properties"]["id"]["type"], "string");
    }
}
