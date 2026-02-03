//! Main OpenAPI parser

use crate::error::{ParseError, ParseResult};
use crate::operations::OperationExtractor;
use crate::types::*;
use regex::Regex;
use std::collections::HashMap;
use tracing::{debug, info};

/// OpenAPI 3.x parser
pub struct OpenApiParser;

impl OpenApiParser {
    /// Parse an OpenAPI spec from a string (auto-detects JSON/YAML)
    pub fn parse(content: &str) -> ParseResult<ParsedSpec> {
        // Sanitize content to handle problematic large numbers
        let content = Self::sanitize_large_numbers(content);

        // Try JSON first, then YAML
        let raw_spec: RawOpenApiSpec = if content.trim().starts_with('{') {
            serde_json::from_str(&content)?
        } else {
            serde_yaml::from_str(&content)?
        };

        Self::convert_spec(raw_spec)
    }

    /// Parse an OpenAPI spec from JSON
    pub fn parse_json(content: &str) -> ParseResult<ParsedSpec> {
        let content = Self::sanitize_large_numbers(content);
        let raw_spec: RawOpenApiSpec = serde_json::from_str(&content)?;
        Self::convert_spec(raw_spec)
    }

    /// Parse an OpenAPI spec from YAML
    pub fn parse_yaml(content: &str) -> ParseResult<ParsedSpec> {
        let content = Self::sanitize_large_numbers(content);
        let raw_spec: RawOpenApiSpec = serde_yaml::from_str(&content)?;
        Self::convert_spec(raw_spec)
    }

    /// Sanitize large numbers that may cause parsing issues
    /// Some OpenAPI specs (like OpenAI) use very large numbers for min/max values
    /// which can cause serde_yaml to fail with "JSON number out of range"
    fn sanitize_large_numbers(content: &str) -> String {
        // Replace any integer that's too large for safe JSON parsing (> 15 digits)
        // These are typically used for min/max constraints and the exact value doesn't matter
        let re_large = Regex::new(r"(?m)^(\s*(?:minimum|maximum|exclusiveMinimum|exclusiveMaximum):\s*)(-?\d{16,})").unwrap();
        let content = re_large.replace_all(content, |caps: &regex::Captures| {
            let prefix = &caps[1];
            let num_str = &caps[2];
            if num_str.starts_with('-') {
                format!("{}-2147483648", prefix)
            } else {
                format!("{}2147483647", prefix)
            }
        });

        content.into_owned()
    }

    /// Fetch and parse an OpenAPI spec from a URL
    pub async fn fetch_and_parse(url: &str) -> ParseResult<ParsedSpec> {
        info!("Fetching OpenAPI spec from: {}", url);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| ParseError::HttpError(e.to_string()))?;

        let response = client
            .get(url)
            .header("Accept", "application/json, application/yaml, text/yaml")
            .send()
            .await
            .map_err(|e| ParseError::FetchError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ParseError::FetchError(format!(
                "HTTP {} from {}",
                response.status(),
                url
            )));
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_default();

        let content = response
            .text()
            .await
            .map_err(|e| ParseError::FetchError(e.to_string()))?;

        // Parse based on content type or file extension
        if content_type.contains("yaml") || url.ends_with(".yaml") || url.ends_with(".yml") {
            Self::parse_yaml(&content)
        } else {
            Self::parse(&content)
        }
    }

    /// Convert a raw OpenAPI spec to our internal format
    fn convert_spec(raw: RawOpenApiSpec) -> ParseResult<ParsedSpec> {
        // Validate OpenAPI version
        if !raw.openapi.starts_with("3.") {
            return Err(ParseError::UnsupportedVersion(raw.openapi));
        }

        debug!("Parsing OpenAPI {} spec: {}", raw.openapi, raw.info.title);

        // Extract operations
        let operations = OperationExtractor::extract(&raw)?;

        debug!("Extracted {} operations", operations.len());

        // Convert security schemes
        let security_schemes = raw
            .components
            .as_ref()
            .map(|c| Self::convert_security_schemes(&c.security_schemes))
            .unwrap_or_default();

        // Convert global security requirements
        let global_security = raw
            .security
            .iter()
            .flat_map(|req| {
                req.iter().map(|(name, scopes)| SecurityRequirement {
                    scheme_name: name.clone(),
                    scopes: scopes.clone(),
                })
            })
            .collect();

        // Convert servers
        let servers = raw
            .servers
            .iter()
            .map(|s| ServerInfo {
                url: s.url.clone(),
                description: s.description.clone(),
            })
            .collect();

        Ok(ParsedSpec {
            title: raw.info.title,
            description: raw.info.description,
            version: raw.info.version,
            servers,
            operations,
            security_schemes,
            global_security,
        })
    }

    /// Convert raw security schemes to our format
    fn convert_security_schemes(
        raw: &indexmap::IndexMap<String, RawSecurityScheme>,
    ) -> HashMap<String, SecurityScheme> {
        raw.iter()
            .filter_map(|(name, scheme)| {
                Self::convert_security_scheme(scheme).map(|s| (name.clone(), s))
            })
            .collect()
    }

    fn convert_security_scheme(raw: &RawSecurityScheme) -> Option<SecurityScheme> {
        match raw.scheme_type.as_str() {
            "apiKey" => Some(SecurityScheme::ApiKey {
                name: raw.name.clone().unwrap_or_default(),
                location: match raw.location.as_deref() {
                    Some("header") => ApiKeyLocation::Header,
                    Some("query") => ApiKeyLocation::Query,
                    Some("cookie") => ApiKeyLocation::Cookie,
                    _ => ApiKeyLocation::Header,
                },
            }),
            "http" => Some(SecurityScheme::Http {
                scheme: raw.scheme.clone().unwrap_or_else(|| "bearer".to_string()),
                bearer_format: raw.bearer_format.clone(),
            }),
            "oauth2" => {
                let flows = raw.flows.as_ref().map(Self::convert_oauth2_flows)
                    .unwrap_or_default();
                Some(SecurityScheme::OAuth2 { flows })
            }
            "openIdConnect" => Some(SecurityScheme::OpenIdConnect {
                openid_connect_url: raw.openid_connect_url.clone().unwrap_or_default(),
            }),
            _ => None,
        }
    }

    fn convert_oauth2_flows(raw: &RawOAuth2Flows) -> OAuth2Flows {
        OAuth2Flows {
            authorization_code: raw.authorization_code.as_ref().map(Self::convert_oauth2_flow),
            implicit: raw.implicit.as_ref().map(Self::convert_oauth2_flow),
            password: raw.password.as_ref().map(Self::convert_oauth2_flow),
            client_credentials: raw.client_credentials.as_ref().map(Self::convert_oauth2_flow),
        }
    }

    fn convert_oauth2_flow(raw: &RawOAuth2Flow) -> OAuth2Flow {
        OAuth2Flow {
            authorization_url: raw.authorization_url.clone(),
            token_url: raw.token_url.clone(),
            refresh_url: raw.refresh_url.clone(),
            scopes: raw.scopes.clone().into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_SPEC: &str = r#"
openapi: "3.0.0"
info:
  title: Test API
  version: "1.0.0"
servers:
  - url: https://api.example.com/v1
paths:
  /users:
    get:
      operationId: listUsers
      summary: List all users
      responses:
        '200':
          description: A list of users
    post:
      operationId: createUser
      summary: Create a user
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                name:
                  type: string
      responses:
        '201':
          description: User created
  /users/{id}:
    get:
      operationId: getUser
      summary: Get a user by ID
      parameters:
        - name: id
          in: path
          required: true
          schema:
            type: string
      responses:
        '200':
          description: A user
components:
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
security:
  - bearerAuth: []
"#;

    #[test]
    fn test_parse_yaml() {
        let spec = OpenApiParser::parse_yaml(SAMPLE_SPEC).unwrap();

        assert_eq!(spec.title, "Test API");
        assert_eq!(spec.version, "1.0.0");
        assert_eq!(spec.operations.len(), 3);
        assert_eq!(spec.servers.len(), 1);
        assert_eq!(spec.servers[0].url, "https://api.example.com/v1");
    }

    #[test]
    fn test_parse_extracts_operations() {
        let spec = OpenApiParser::parse_yaml(SAMPLE_SPEC).unwrap();

        let list_users = spec.operations.iter()
            .find(|op| op.operation_id == "listUsers")
            .unwrap();
        assert_eq!(list_users.method, HttpMethod::Get);
        assert_eq!(list_users.path, "/users");

        let create_user = spec.operations.iter()
            .find(|op| op.operation_id == "createUser")
            .unwrap();
        assert_eq!(create_user.method, HttpMethod::Post);
        assert!(create_user.request_body.is_some());

        let get_user = spec.operations.iter()
            .find(|op| op.operation_id == "getUser")
            .unwrap();
        assert_eq!(get_user.parameters.len(), 1);
        assert_eq!(get_user.parameters[0].name, "id");
    }

    #[test]
    fn test_parse_extracts_security() {
        let spec = OpenApiParser::parse_yaml(SAMPLE_SPEC).unwrap();

        assert!(spec.security_schemes.contains_key("bearerAuth"));
        assert_eq!(spec.global_security.len(), 1);
        assert_eq!(spec.global_security[0].scheme_name, "bearerAuth");
    }

    #[test]
    fn test_sanitize_large_numbers() {
        let yaml_with_large_nums = r#"
openapi: "3.0.0"
info:
  title: Test API
  version: "1.0.0"
servers:
  - url: https://api.example.com
paths: {}
components:
  schemas:
    TestSchema:
      type: object
      properties:
        seed:
          type: integer
          minimum: -9223372036854776000
          maximum: 9223372036854776000
"#;

        // This should not panic or error
        let result = OpenApiParser::parse_yaml(yaml_with_large_nums);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_fetch_openai_spec() {
        let url = "https://raw.githubusercontent.com/openai/openai-openapi/refs/heads/manual_spec/openapi.yaml";
        let result = OpenApiParser::fetch_and_parse(url).await;

        match &result {
            Ok(spec) => {
                assert_eq!(spec.title, "OpenAI API");
                assert!(!spec.operations.is_empty(), "Should have operations");
                println!("Successfully parsed OpenAI spec with {} operations", spec.operations.len());
            }
            Err(e) => {
                panic!("Failed to parse OpenAI spec: {:?}", e);
            }
        }
    }
}
