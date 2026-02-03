//! Operation extraction from OpenAPI specs

use crate::types::*;
use crate::error::ParseResult;
use crate::resolver::SchemaResolver;
use indexmap::IndexMap;

/// Extracts operations from raw OpenAPI spec structures
pub struct OperationExtractor;

impl OperationExtractor {
    /// Extract all operations from a raw OpenAPI spec
    pub fn extract(spec: &RawOpenApiSpec) -> ParseResult<Vec<ApiOperation>> {
        let mut operations = Vec::new();

        // Get component schemas for resolving $ref
        let empty_schemas = IndexMap::new();
        let schemas = spec
            .components
            .as_ref()
            .map(|c| &c.schemas)
            .unwrap_or(&empty_schemas);
        let resolver = SchemaResolver::new(schemas);

        for (path, path_item) in &spec.paths {
            // Extract path-level parameters
            let path_params: Vec<OperationParameter> = path_item
                .parameters
                .iter()
                .filter_map(|p| Self::convert_parameter(p, &resolver))
                .collect();

            // Process each HTTP method
            let methods = [
                (HttpMethod::Get, &path_item.get),
                (HttpMethod::Post, &path_item.post),
                (HttpMethod::Put, &path_item.put),
                (HttpMethod::Patch, &path_item.patch),
                (HttpMethod::Delete, &path_item.delete),
                (HttpMethod::Head, &path_item.head),
                (HttpMethod::Options, &path_item.options),
                (HttpMethod::Trace, &path_item.trace),
            ];

            for (method, operation) in methods {
                if let Some(op) = operation {
                    let api_op = Self::extract_operation(path, method, op, &path_params, spec, &resolver)?;
                    operations.push(api_op);
                }
            }
        }

        Ok(operations)
    }

    /// Extract a single operation
    fn extract_operation(
        path: &str,
        method: HttpMethod,
        operation: &RawOperation,
        path_params: &[OperationParameter],
        spec: &RawOpenApiSpec,
        resolver: &SchemaResolver,
    ) -> ParseResult<ApiOperation> {
        // Generate operation ID if not provided
        let operation_id = operation.operation_id.clone().unwrap_or_else(|| {
            Self::generate_operation_id(path, method)
        });

        // Normalize operation ID (convert to dot notation)
        let normalized_id = Self::normalize_operation_id(&operation_id);

        // Combine path-level and operation-level parameters
        let mut parameters = path_params.to_vec();
        for param in &operation.parameters {
            if let Some(p) = Self::convert_parameter(param, resolver) {
                // Remove any path-level param with the same name
                parameters.retain(|existing| existing.name != p.name);
                parameters.push(p);
            }
        }

        // Extract request body with schema resolution
        let request_body = operation.request_body.as_ref().and_then(|body| {
            Self::extract_request_body(body, resolver)
        });

        // Extract responses
        let responses = Self::extract_responses(&operation.responses);

        // Extract security requirements
        let security = Self::extract_security(
            operation.security.as_ref(),
            &spec.security,
        );

        Ok(ApiOperation {
            operation_id,
            normalized_id,
            method,
            path: path.to_string(),
            summary: operation.summary.clone(),
            description: operation.description.clone(),
            tags: operation.tags.clone(),
            deprecated: operation.deprecated,
            parameters,
            request_body,
            responses,
            security,
        })
    }

    /// Generate an operation ID from path and method
    fn generate_operation_id(path: &str, method: HttpMethod) -> String {
        // Convert path to operation ID: /users/{id}/posts -> users_id_posts
        let path_part = path
            .trim_start_matches('/')
            .replace('/', "_")
            .replace('{', "")
            .replace('}', "");

        format!("{}_{}", method.as_str().to_lowercase(), path_part)
    }

    /// Normalize operation ID to dot notation for namespacing
    fn normalize_operation_id(operation_id: &str) -> String {
        // Convert camelCase and snake_case to dot notation
        // createCustomer -> create.customer
        // create_customer -> create.customer
        // customerCreate -> customer.create

        let mut result = String::new();
        let mut prev_was_lowercase = false;

        for c in operation_id.chars() {
            if c == '_' || c == '-' {
                result.push('.');
                prev_was_lowercase = false;
            } else if c.is_uppercase() && prev_was_lowercase {
                result.push('.');
                result.push(c.to_lowercase().next().unwrap());
                prev_was_lowercase = true;
            } else {
                result.push(c.to_lowercase().next().unwrap());
                prev_was_lowercase = c.is_lowercase();
            }
        }

        result
    }

    /// Convert a raw parameter to our parameter type
    fn convert_parameter(param: &RawParameter, resolver: &SchemaResolver) -> Option<OperationParameter> {
        // Skip references for now (would need to resolve them)
        if param.reference.is_some() {
            return None;
        }

        let location = match param.location.as_str() {
            "path" => ParameterLocation::Path,
            "query" => ParameterLocation::Query,
            "header" => ParameterLocation::Header,
            "cookie" => ParameterLocation::Cookie,
            _ => return None,
        };

        // Resolve schema if it contains $ref
        let schema = param.schema.as_ref().map(|s| resolver.resolve(s));

        Some(OperationParameter {
            name: param.name.clone(),
            location,
            required: param.required || location == ParameterLocation::Path,
            description: param.description.clone(),
            schema,
            example: param.example.clone(),
            deprecated: param.deprecated,
        })
    }

    /// Extract request body information
    fn extract_request_body(body: &RawRequestBody, resolver: &SchemaResolver) -> Option<RequestBody> {
        // Prefer JSON content type
        let (content_type, media) = body.content.iter()
            .find(|(ct, _)| ct.contains("json"))
            .or_else(|| body.content.first())?;

        // Resolve the schema, following any $ref references
        let schema = media.schema.as_ref().map(|s| resolver.resolve(s));

        Some(RequestBody {
            required: body.required,
            content_type: content_type.clone(),
            schema,
            description: body.description.clone(),
        })
    }

    /// Extract response information
    fn extract_responses(responses: &IndexMap<String, RawResponse>) -> Vec<ResponseSchema> {
        responses
            .iter()
            .map(|(status, response)| {
                let (content_type, schema) = response.content.as_ref()
                    .and_then(|content| {
                        content.iter()
                            .find(|(ct, _)| ct.contains("json"))
                            .or_else(|| content.first())
                            .map(|(ct, media)| (Some(ct.clone()), media.schema.clone()))
                    })
                    .unwrap_or((None, None));

                ResponseSchema {
                    status_code: status.clone(),
                    content_type,
                    schema,
                    description: response.description.clone(),
                }
            })
            .collect()
    }

    /// Extract security requirements
    fn extract_security(
        operation_security: Option<&Vec<IndexMap<String, Vec<String>>>>,
        global_security: &[IndexMap<String, Vec<String>>],
    ) -> Vec<SecurityRequirement> {
        let security: &[IndexMap<String, Vec<String>>] = operation_security
            .map(|v| v.as_slice())
            .unwrap_or(global_security);

        security
            .iter()
            .flat_map(|req| {
                req.iter().map(|(name, scopes)| SecurityRequirement {
                    scheme_name: name.clone(),
                    scopes: scopes.clone(),
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_operation_id_camel_case() {
        assert_eq!(
            OperationExtractor::normalize_operation_id("createCustomer"),
            "create.customer"
        );
    }

    #[test]
    fn test_normalize_operation_id_snake_case() {
        assert_eq!(
            OperationExtractor::normalize_operation_id("create_customer"),
            "create.customer"
        );
    }

    #[test]
    fn test_generate_operation_id() {
        assert_eq!(
            OperationExtractor::generate_operation_id("/users/{id}/posts", HttpMethod::Get),
            "get_users_id_posts"
        );
    }
}
