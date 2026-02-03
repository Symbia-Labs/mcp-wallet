//! Credential manager for CRUD operations

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

use super::types::{Credential, CredentialType, DecryptedCredential, StoredCredential};
use crate::crypto::{encrypt_string, decrypt_string, MasterKey};
use crate::error::{Result, WalletError};
use crate::storage::SecureStorage;

/// Storage key prefix for credentials
const CREDENTIAL_PREFIX: &str = "credential:";

/// Credential manager
pub struct CredentialManager {
    /// Storage backend
    storage: Arc<dyn SecureStorage>,
    /// Master key for encryption
    master_key: Arc<RwLock<Option<MasterKey>>>,
}

impl CredentialManager {
    /// Create a new credential manager
    pub fn new(storage: Arc<dyn SecureStorage>) -> Self {
        Self {
            storage,
            master_key: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the master key for encryption/decryption
    pub async fn set_master_key(&self, key: Option<MasterKey>) {
        let mut master_key = self.master_key.write().await;
        *master_key = key;
    }

    /// Add a new API key credential
    pub async fn add_api_key(
        &self,
        provider: &str,
        name: &str,
        api_key: &str,
    ) -> Result<Credential> {
        let master_key = self.master_key.read().await;
        let key = master_key.as_ref().ok_or(WalletError::WalletLocked)?;

        let credential = Credential::new_api_key(provider, name, api_key);
        let encrypted_value = encrypt_string(api_key, key)?;

        let stored = StoredCredential {
            credential: credential.clone(),
            encrypted_value,
            encrypted_refresh_token: None,
            expires_at: None,
        };

        self.save_credential(&stored).await?;

        info!("Added credential: {} ({})", credential.name, credential.provider);
        Ok(credential)
    }

    /// Add an OAuth2 token credential
    pub async fn add_oauth2_token(
        &self,
        provider: &str,
        name: &str,
        access_token: &str,
        refresh_token: Option<&str>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Credential> {
        let master_key = self.master_key.read().await;
        let key = master_key.as_ref().ok_or(WalletError::WalletLocked)?;

        let credential = Credential::new_oauth2(provider, name);
        let encrypted_value = encrypt_string(access_token, key)?;
        let encrypted_refresh = match refresh_token {
            Some(rt) => Some(encrypt_string(rt, key)?),
            None => None,
        };

        let stored = StoredCredential {
            credential: credential.clone(),
            encrypted_value,
            encrypted_refresh_token: encrypted_refresh,
            expires_at,
        };

        self.save_credential(&stored).await?;

        info!("Added OAuth2 credential: {} ({})", credential.name, credential.provider);
        Ok(credential)
    }

    /// Get a credential by ID
    pub async fn get(&self, id: Uuid) -> Result<Option<Credential>> {
        let storage_key = format!("{}{}", CREDENTIAL_PREFIX, id);

        match self.storage.retrieve(&storage_key).await? {
            Some(data) => {
                let stored: StoredCredential = serde_json::from_slice(&data)?;
                Ok(Some(stored.credential))
            }
            None => Ok(None),
        }
    }

    /// Get and decrypt a credential value
    pub async fn get_decrypted(&self, id: Uuid) -> Result<DecryptedCredential> {
        let master_key = self.master_key.read().await;
        let key = master_key.as_ref().ok_or(WalletError::WalletLocked)?;

        let storage_key = format!("{}{}", CREDENTIAL_PREFIX, id);

        let data = self.storage.retrieve(&storage_key).await?
            .ok_or_else(|| WalletError::CredentialNotFound(id.to_string()))?;

        let stored: StoredCredential = serde_json::from_slice(&data)?;
        let decrypted = decrypt_string(&stored.encrypted_value, key)?;

        // Update last used timestamp
        self.update_last_used(id).await?;

        debug!("Decrypted credential: {}", id);
        Ok(DecryptedCredential::new(decrypted))
    }

    /// List all credentials
    pub async fn list(&self) -> Result<Vec<Credential>> {
        let keys = self.storage.list_keys(CREDENTIAL_PREFIX).await?;
        let mut credentials = Vec::new();

        for key in keys {
            if let Some(data) = self.storage.retrieve(&key).await? {
                let stored: StoredCredential = serde_json::from_slice(&data)?;
                credentials.push(stored.credential);
            }
        }

        Ok(credentials)
    }

    /// List credentials by provider
    pub async fn list_by_provider(&self, provider: &str) -> Result<Vec<Credential>> {
        let all = self.list().await?;
        Ok(all.into_iter().filter(|c| c.provider == provider).collect())
    }

    /// Delete a credential
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let storage_key = format!("{}{}", CREDENTIAL_PREFIX, id);
        self.storage.delete(&storage_key).await?;

        info!("Deleted credential: {}", id);
        Ok(())
    }

    /// Update a credential's value
    pub async fn update_value(&self, id: Uuid, new_value: &str) -> Result<()> {
        let master_key = self.master_key.read().await;
        let key = master_key.as_ref().ok_or(WalletError::WalletLocked)?;

        let storage_key = format!("{}{}", CREDENTIAL_PREFIX, id);

        let data = self.storage.retrieve(&storage_key).await?
            .ok_or_else(|| WalletError::CredentialNotFound(id.to_string()))?;

        let mut stored: StoredCredential = serde_json::from_slice(&data)?;

        // Update encrypted value
        stored.encrypted_value = encrypt_string(new_value, key)?;

        // Update prefix
        stored.credential.prefix = if new_value.len() >= 8 {
            Some(format!("{}...", &new_value[..8]))
        } else {
            Some(format!("{}...", new_value))
        };

        self.save_credential(&stored).await?;

        info!("Updated credential value: {}", id);
        Ok(())
    }

    /// Save a credential to storage
    async fn save_credential(&self, stored: &StoredCredential) -> Result<()> {
        let key = format!("{}{}", CREDENTIAL_PREFIX, stored.credential.id);
        let data = serde_json::to_vec(stored)?;
        self.storage.store(&key, &data).await?;
        Ok(())
    }

    /// Update last used timestamp
    async fn update_last_used(&self, id: Uuid) -> Result<()> {
        let storage_key = format!("{}{}", CREDENTIAL_PREFIX, id);

        if let Some(data) = self.storage.retrieve(&storage_key).await? {
            let mut stored: StoredCredential = serde_json::from_slice(&data)?;
            stored.credential.last_used_at = Some(chrono::Utc::now());
            self.save_credential(&stored).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::EncryptedFileStorage;
    use crate::crypto::key_derivation::{derive_key, generate_salt};
    use tempfile::TempDir;

    async fn test_manager() -> CredentialManager {
        let temp_dir = TempDir::new().unwrap();
        let storage = EncryptedFileStorage::with_dir(temp_dir.path().to_path_buf()).unwrap();

        let salt = generate_salt();
        let key = derive_key("test", &salt, None).unwrap();
        storage.set_master_key(Some(key.clone())).await;

        let manager = CredentialManager::new(Arc::new(storage));
        manager.set_master_key(Some(key)).await;

        manager
    }

    #[tokio::test]
    async fn test_add_and_get_api_key() {
        let manager = test_manager().await;

        let cred = manager
            .add_api_key("openai", "My OpenAI Key", "sk-test-12345678")
            .await
            .unwrap();

        assert_eq!(cred.provider, "openai");
        assert_eq!(cred.name, "My OpenAI Key");
        assert_eq!(cred.prefix, Some("sk-test-...".to_string()));

        let retrieved = manager.get(cred.id).await.unwrap().unwrap();
        assert_eq!(retrieved.id, cred.id);
    }

    #[tokio::test]
    async fn test_decrypt_credential() {
        let manager = test_manager().await;

        let cred = manager
            .add_api_key("stripe", "Stripe Key", "sk_live_abc123")
            .await
            .unwrap();

        let decrypted = manager.get_decrypted(cred.id).await.unwrap();
        assert_eq!(decrypted.expose(), "sk_live_abc123");
    }

    #[tokio::test]
    async fn test_list_credentials() {
        let manager = test_manager().await;

        manager.add_api_key("openai", "OpenAI", "key1").await.unwrap();
        manager.add_api_key("anthropic", "Anthropic", "key2").await.unwrap();

        let creds = manager.list().await.unwrap();
        assert_eq!(creds.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_credential() {
        let manager = test_manager().await;

        let cred = manager.add_api_key("test", "Test", "key").await.unwrap();
        assert!(manager.get(cred.id).await.unwrap().is_some());

        manager.delete(cred.id).await.unwrap();
        assert!(manager.get(cred.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_update_value() {
        let manager = test_manager().await;

        let cred = manager.add_api_key("test", "Test", "old-key").await.unwrap();

        manager.update_value(cred.id, "new-key-12345678").await.unwrap();

        let decrypted = manager.get_decrypted(cred.id).await.unwrap();
        assert_eq!(decrypted.expose(), "new-key-12345678");

        let updated = manager.get(cred.id).await.unwrap().unwrap();
        assert_eq!(updated.prefix, Some("new-key-...".to_string()));
    }
}
