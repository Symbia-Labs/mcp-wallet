//! Type definitions for parsed OpenAPI specs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use indexmap::IndexMap;

/// HTTP methods supported by OpenAPI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
    Trace,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Trace => "TRACE",
        }
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Parameter location in HTTP request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterLocation {
    Path,
    Query,
    Header,
    Cookie,
}

/// A parameter for an API operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationParameter {
    /// Parameter name
    pub name: String,
    /// Where the parameter is located
    pub location: ParameterLocation,
    /// Whether the parameter is required
    pub required: bool,
    /// Parameter description
    pub description: Option<String>,
    /// JSON Schema for the parameter
    pub schema: Option<serde_json::Value>,
    /// Example value
    pub example: Option<serde_json::Value>,
    /// Whether the parameter is deprecated
    pub deprecated: bool,
}

/// Request body schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    /// Whether the body is required
    pub required: bool,
    /// Content type (e.g., "application/json")
    pub content_type: String,
    /// JSON Schema for the body
    pub schema: Option<serde_json::Value>,
    /// Description
    pub description: Option<String>,
}

/// Response schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseSchema {
    /// HTTP status code
    pub status_code: String,
    /// Content type
    pub content_type: Option<String>,
    /// JSON Schema for the response
    pub schema: Option<serde_json::Value>,
    /// Description
    pub description: Option<String>,
}

/// A single API operation extracted from the spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiOperation {
    /// Unique operation ID (from spec or generated)
    pub operation_id: String,
    /// Normalized operation ID for namespacing (dots replaced with underscores)
    pub normalized_id: String,
    /// HTTP method
    pub method: HttpMethod,
    /// URL path (e.g., "/v1/customers/{id}")
    pub path: String,
    /// Short summary
    pub summary: Option<String>,
    /// Full description
    pub description: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Whether the operation is deprecated
    pub deprecated: bool,
    /// Parameters (path, query, header, cookie)
    pub parameters: Vec<OperationParameter>,
    /// Request body schema
    pub request_body: Option<RequestBody>,
    /// Response schemas keyed by status code
    pub responses: Vec<ResponseSchema>,
    /// Security requirements for this operation
    pub security: Vec<SecurityRequirement>,
}

/// Security requirement for an operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirement {
    /// Name of the security scheme
    pub scheme_name: String,
    /// Required scopes (for OAuth2)
    pub scopes: Vec<String>,
}

/// Parsed OpenAPI specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSpec {
    /// API title
    pub title: String,
    /// API description
    pub description: Option<String>,
    /// API version
    pub version: String,
    /// Server URLs
    pub servers: Vec<ServerInfo>,
    /// All extracted operations
    pub operations: Vec<ApiOperation>,
    /// Security schemes defined in the spec
    pub security_schemes: HashMap<String, SecurityScheme>,
    /// Global security requirements
    pub global_security: Vec<SecurityRequirement>,
}

/// Server information from the spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server URL
    pub url: String,
    /// Server description
    pub description: Option<String>,
}

/// Security scheme definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SecurityScheme {
    /// API key authentication
    ApiKey {
        name: String,
        #[serde(rename = "in")]
        location: ApiKeyLocation,
    },
    /// HTTP authentication (bearer, basic)
    Http {
        scheme: String,
        bearer_format: Option<String>,
    },
    /// OAuth2 authentication
    OAuth2 {
        flows: OAuth2Flows,
    },
    /// OpenID Connect
    OpenIdConnect {
        openid_connect_url: String,
    },
}

/// API key location
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiKeyLocation {
    Header,
    Query,
    Cookie,
}

/// OAuth2 flows
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OAuth2Flows {
    pub authorization_code: Option<OAuth2Flow>,
    pub implicit: Option<OAuth2Flow>,
    pub password: Option<OAuth2Flow>,
    pub client_credentials: Option<OAuth2Flow>,
}

/// OAuth2 flow details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Flow {
    pub authorization_url: Option<String>,
    pub token_url: Option<String>,
    pub refresh_url: Option<String>,
    pub scopes: HashMap<String, String>,
}

// --- Raw OpenAPI 3.x structures for parsing ---

/// Raw OpenAPI document structure
#[derive(Debug, Clone, Deserialize)]
pub struct RawOpenApiSpec {
    pub openapi: String,
    pub info: RawInfo,
    #[serde(default)]
    pub servers: Vec<RawServer>,
    #[serde(default)]
    pub paths: IndexMap<String, RawPathItem>,
    #[serde(default)]
    pub components: Option<RawComponents>,
    #[serde(default)]
    pub security: Vec<IndexMap<String, Vec<String>>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawInfo {
    pub title: String,
    pub description: Option<String>,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawServer {
    pub url: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawPathItem {
    pub get: Option<RawOperation>,
    pub post: Option<RawOperation>,
    pub put: Option<RawOperation>,
    pub patch: Option<RawOperation>,
    pub delete: Option<RawOperation>,
    pub head: Option<RawOperation>,
    pub options: Option<RawOperation>,
    pub trace: Option<RawOperation>,
    #[serde(default)]
    pub parameters: Vec<RawParameter>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawOperation {
    pub operation_id: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub deprecated: bool,
    #[serde(default)]
    pub parameters: Vec<RawParameter>,
    pub request_body: Option<RawRequestBody>,
    #[serde(default)]
    pub responses: IndexMap<String, RawResponse>,
    #[serde(default)]
    pub security: Option<Vec<IndexMap<String, Vec<String>>>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawParameter {
    /// Parameter name (optional when $ref is used)
    #[serde(default)]
    pub name: String,
    /// Parameter location (optional when $ref is used)
    #[serde(rename = "in", default)]
    pub location: String,
    #[serde(default)]
    pub required: bool,
    pub description: Option<String>,
    pub schema: Option<serde_json::Value>,
    pub example: Option<serde_json::Value>,
    #[serde(default)]
    pub deprecated: bool,
    /// Reference to a parameter in components/parameters
    #[serde(rename = "$ref")]
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawRequestBody {
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub content: IndexMap<String, RawMediaType>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawMediaType {
    pub schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawResponse {
    pub description: Option<String>,
    #[serde(default)]
    pub content: Option<IndexMap<String, RawMediaType>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawComponents {
    #[serde(default)]
    pub security_schemes: IndexMap<String, RawSecurityScheme>,
    #[serde(default)]
    pub schemas: IndexMap<String, serde_json::Value>,
    #[serde(default)]
    pub parameters: IndexMap<String, RawParameter>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawSecurityScheme {
    #[serde(rename = "type")]
    pub scheme_type: String,
    pub name: Option<String>,
    #[serde(rename = "in")]
    pub location: Option<String>,
    pub scheme: Option<String>,
    pub bearer_format: Option<String>,
    pub flows: Option<RawOAuth2Flows>,
    pub openid_connect_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawOAuth2Flows {
    pub authorization_code: Option<RawOAuth2Flow>,
    pub implicit: Option<RawOAuth2Flow>,
    pub password: Option<RawOAuth2Flow>,
    pub client_credentials: Option<RawOAuth2Flow>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawOAuth2Flow {
    pub authorization_url: Option<String>,
    pub token_url: Option<String>,
    pub refresh_url: Option<String>,
    #[serde(default)]
    pub scopes: IndexMap<String, String>,
}
