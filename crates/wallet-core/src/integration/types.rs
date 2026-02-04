//! Integration type definitions

use chrono::{DateTime, Utc};
use openapi_parser::{ApiOperation, AuthScheme, NamespaceTree, ParsedSpec};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Status of an integration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationStatus {
    /// Integration is pending setup (no credential)
    #[default]
    Pending,
    /// Integration is active and ready to use
    Active,
    /// Integration encountered an error
    Error,
    /// Integration is disabled
    Disabled,
}

/// An integration represents a configured OpenAPI service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Integration {
    /// Unique identifier
    pub id: Uuid,

    /// Short key for the integration (e.g., "stripe", "github")
    pub key: String,

    /// Human-readable name
    pub name: String,

    /// Description
    pub description: Option<String>,

    /// Source OpenAPI spec URL (if fetched remotely)
    pub spec_url: Option<String>,

    /// Base server URL
    pub server_url: String,

    /// Current status
    pub status: IntegrationStatus,

    /// Associated credential ID (if any)
    pub credential_id: Option<Uuid>,

    /// Detected authentication scheme
    #[serde(skip)]
    pub auth_scheme: Option<AuthScheme>,

    /// Number of operations
    pub operation_count: usize,

    /// Last sync timestamp
    pub last_synced_at: Option<DateTime<Utc>>,

    /// Error message (if status is Error)
    pub error: Option<String>,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

impl Integration {
    /// Create a new integration from a parsed spec
    pub fn from_spec(key: String, spec: &ParsedSpec) -> Self {
        let server_url = spec
            .servers
            .first()
            .map(|s| s.url.clone())
            .unwrap_or_default();

        Self {
            id: Uuid::new_v4(),
            key,
            name: spec.title.clone(),
            description: spec.description.clone(),
            spec_url: None,
            server_url,
            status: IntegrationStatus::Pending,
            credential_id: None,
            auth_scheme: None,
            operation_count: spec.operations.len(),
            last_synced_at: Some(Utc::now()),
            error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

/// An operation within an integration (reference to parsed operation)
pub type IntegrationOperation = ApiOperation;

/// Stored integration data (serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredIntegration {
    /// Integration metadata
    pub integration: Integration,

    /// Parsed operations
    pub operations: Vec<ApiOperation>,

    /// Namespace tree for lookup
    pub namespace: NamespaceTree,

    /// Raw spec content (for re-parsing)
    pub spec_content: Option<String>,
}

impl StoredIntegration {
    /// Create from a parsed spec
    pub fn from_spec(key: String, spec: ParsedSpec, spec_content: Option<String>) -> Self {
        let integration = Integration::from_spec(key, &spec);
        let namespace = NamespaceTree::build(&spec.operations);

        Self {
            integration,
            operations: spec.operations,
            namespace,
            spec_content,
        }
    }

    /// Look up an operation by namespace path
    pub fn lookup_operation(&self, path: &str) -> Option<&ApiOperation> {
        self.namespace
            .lookup(path)
            .map(|op_ref| &self.operations[op_ref.index])
    }

    /// List all operations under a namespace prefix
    pub fn list_operations(&self, prefix: &str) -> Vec<&ApiOperation> {
        self.namespace
            .list(prefix)
            .iter()
            .map(|op_ref| &self.operations[op_ref.index])
            .collect()
    }

    /// Get all operation paths
    pub fn operation_paths(&self) -> Vec<String> {
        self.namespace.paths()
    }
}
