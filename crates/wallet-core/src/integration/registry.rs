//! Integration registry for managing multiple integrations

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use super::types::{Integration, IntegrationStatus, StoredIntegration};
use crate::error::{Result, WalletError};
use crate::storage::SecureStorage;
use openapi_parser::{ApiOperation, OpenApiParser};

/// Storage key prefix for integrations
const INTEGRATION_PREFIX: &str = "integration:";

/// Registry for managing integrations
pub struct IntegrationRegistry {
    /// In-memory cache of integrations
    integrations: Arc<RwLock<HashMap<String, StoredIntegration>>>,
    /// Storage backend
    storage: Arc<dyn SecureStorage>,
}

impl IntegrationRegistry {
    /// Create a new integration registry
    pub fn new(storage: Arc<dyn SecureStorage>) -> Self {
        Self {
            integrations: Arc::new(RwLock::new(HashMap::new())),
            storage,
        }
    }

    /// Load all integrations from storage
    pub async fn load(&self) -> Result<()> {
        let keys = self.storage.list_keys(INTEGRATION_PREFIX).await?;

        let mut integrations = self.integrations.write().await;

        for key in keys {
            match self.storage.retrieve(&key).await? {
                Some(data) => {
                    let stored: StoredIntegration = serde_json::from_slice(&data)?;
                    let integration_key = key
                        .strip_prefix(INTEGRATION_PREFIX)
                        .unwrap_or(&key)
                        .to_string();
                    integrations.insert(integration_key, stored);
                }
                None => {
                    warn!("Integration key exists but no data: {}", key);
                }
            }
        }

        info!("Loaded {} integrations", integrations.len());
        Ok(())
    }

    /// Add an integration from an OpenAPI spec URL
    pub async fn add_from_url(&self, key: &str, spec_url: &str) -> Result<Integration> {
        info!("Adding integration from URL: {} -> {}", key, spec_url);

        let spec = OpenApiParser::fetch_and_parse(spec_url)
            .await
            .map_err(|e| WalletError::InvalidSpec(e.to_string()))?;

        let mut stored = StoredIntegration::from_spec(key.to_string(), spec, None);
        stored.integration.spec_url = Some(spec_url.to_string());

        self.save_integration(&stored).await?;

        let integration = stored.integration.clone();

        let mut integrations = self.integrations.write().await;
        integrations.insert(key.to_string(), stored);

        Ok(integration)
    }

    /// Add an integration from spec content
    pub async fn add_from_content(&self, key: &str, content: &str) -> Result<Integration> {
        info!("Adding integration from content: {}", key);

        let spec =
            OpenApiParser::parse(content).map_err(|e| WalletError::InvalidSpec(e.to_string()))?;

        let stored = StoredIntegration::from_spec(key.to_string(), spec, Some(content.to_string()));

        self.save_integration(&stored).await?;

        let integration = stored.integration.clone();

        let mut integrations = self.integrations.write().await;
        integrations.insert(key.to_string(), stored);

        Ok(integration)
    }

    /// Remove an integration
    pub async fn remove(&self, key: &str) -> Result<()> {
        info!("Removing integration: {}", key);

        let storage_key = format!("{}{}", INTEGRATION_PREFIX, key);
        self.storage.delete(&storage_key).await?;

        let mut integrations = self.integrations.write().await;
        integrations.remove(key);

        Ok(())
    }

    /// Get an integration by key
    pub async fn get(&self, key: &str) -> Option<Integration> {
        let integrations = self.integrations.read().await;
        integrations.get(key).map(|s| s.integration.clone())
    }

    /// Get a stored integration (with operations) by key
    pub async fn get_stored(&self, key: &str) -> Option<StoredIntegration> {
        let integrations = self.integrations.read().await;
        integrations.get(key).cloned()
    }

    /// List all integrations
    pub async fn list(&self) -> Vec<Integration> {
        let integrations = self.integrations.read().await;
        integrations
            .values()
            .map(|s| s.integration.clone())
            .collect()
    }

    /// Update integration status
    pub async fn set_status(&self, key: &str, status: IntegrationStatus) -> Result<()> {
        let mut integrations = self.integrations.write().await;

        if let Some(stored) = integrations.get_mut(key) {
            stored.integration.status = status;
            stored.integration.updated_at = chrono::Utc::now();

            // Persist
            drop(integrations);
            self.save_integration(self.integrations.read().await.get(key).unwrap())
                .await?;
        }

        Ok(())
    }

    /// Set credential for an integration
    pub async fn set_credential(&self, key: &str, credential_id: Uuid) -> Result<()> {
        let mut integrations = self.integrations.write().await;

        if let Some(stored) = integrations.get_mut(key) {
            stored.integration.credential_id = Some(credential_id);
            stored.integration.status = IntegrationStatus::Active;
            stored.integration.updated_at = chrono::Utc::now();

            // Persist
            let stored_clone = stored.clone();
            drop(integrations);
            self.save_integration(&stored_clone).await?;

            debug!("Set credential {} for integration {}", credential_id, key);
        }

        Ok(())
    }

    /// Look up an operation by integration key and operation path
    pub async fn lookup_operation(&self, key: &str, path: &str) -> Option<ApiOperation> {
        let integrations = self.integrations.read().await;
        integrations
            .get(key)
            .and_then(|s| s.lookup_operation(path))
            .cloned()
    }

    /// List all operations for an integration
    pub async fn list_operations(&self, key: &str) -> Vec<ApiOperation> {
        let integrations = self.integrations.read().await;
        integrations
            .get(key)
            .map(|s| s.operations.clone())
            .unwrap_or_default()
    }

    /// Get all operation paths across all integrations
    /// Returns tuples of (integration_key, operation_path)
    pub async fn all_operation_paths(&self) -> Vec<(String, String)> {
        let integrations = self.integrations.read().await;
        let mut paths = Vec::new();

        for (key, stored) in integrations.iter() {
            for path in stored.operation_paths() {
                paths.push((key.clone(), path));
            }
        }

        paths
    }

    /// Save an integration to storage
    async fn save_integration(&self, stored: &StoredIntegration) -> Result<()> {
        let key = format!("{}{}", INTEGRATION_PREFIX, stored.integration.key);
        let data = serde_json::to_vec(stored)?;
        self.storage.store(&key, &data).await?;
        Ok(())
    }

    /// Sync an integration (re-fetch and update spec)
    pub async fn sync(&self, key: &str) -> Result<()> {
        let spec_url = {
            let integrations = self.integrations.read().await;
            integrations
                .get(key)
                .and_then(|s| s.integration.spec_url.clone())
        };

        let spec_url = spec_url
            .ok_or_else(|| WalletError::IntegrationNotFound(format!("{} has no spec URL", key)))?;

        info!("Syncing integration: {} from {}", key, spec_url);

        let spec = OpenApiParser::fetch_and_parse(&spec_url)
            .await
            .map_err(|e| WalletError::InvalidSpec(e.to_string()))?;

        let mut integrations = self.integrations.write().await;

        if let Some(stored) = integrations.get_mut(key) {
            // Preserve credential_id and status
            let credential_id = stored.integration.credential_id;
            let status = stored.integration.status;

            // Update with new spec
            let mut new_stored = StoredIntegration::from_spec(key.to_string(), spec, None);
            new_stored.integration.spec_url = Some(spec_url);
            new_stored.integration.credential_id = credential_id;
            new_stored.integration.status = status;
            new_stored.integration.id = stored.integration.id;
            new_stored.integration.created_at = stored.integration.created_at;

            *stored = new_stored;

            // Persist
            let stored_clone = stored.clone();
            drop(integrations);
            self.save_integration(&stored_clone).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::key_derivation::{derive_key, generate_salt};
    use crate::storage::EncryptedFileStorage;
    use tempfile::TempDir;

    async fn test_registry() -> IntegrationRegistry {
        let temp_dir = TempDir::new().unwrap();
        let storage = EncryptedFileStorage::with_dir(temp_dir.path().to_path_buf()).unwrap();

        let salt = generate_salt();
        let key = derive_key("test", &salt, None).unwrap();
        storage.set_master_key(Some(key)).await;

        IntegrationRegistry::new(Arc::new(storage))
    }

    const TEST_SPEC: &str = r#"
openapi: "3.0.0"
info:
  title: Test API
  version: "1.0.0"
servers:
  - url: https://api.test.com
paths:
  /users:
    get:
      operationId: listUsers
      responses:
        '200':
          description: OK
"#;

    #[tokio::test]
    async fn test_add_from_content() {
        let registry = test_registry().await;

        let integration = registry.add_from_content("test", TEST_SPEC).await.unwrap();

        assert_eq!(integration.key, "test");
        assert_eq!(integration.name, "Test API");
        assert_eq!(integration.operation_count, 1);
    }

    #[tokio::test]
    async fn test_list_operations() {
        let registry = test_registry().await;

        registry.add_from_content("test", TEST_SPEC).await.unwrap();

        let operations = registry.list_operations("test").await;
        assert_eq!(operations.len(), 1);
        assert_eq!(operations[0].operation_id, "listUsers");
    }

    #[tokio::test]
    async fn test_remove() {
        let registry = test_registry().await;

        registry.add_from_content("test", TEST_SPEC).await.unwrap();
        assert!(registry.get("test").await.is_some());

        registry.remove("test").await.unwrap();
        assert!(registry.get("test").await.is_none());
    }
}
