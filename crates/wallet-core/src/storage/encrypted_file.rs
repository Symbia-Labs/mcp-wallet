//! Encrypted file storage backend
//!
//! Stores data in encrypted JSON files in the user's data directory.
//! Each entry is individually encrypted with AES-256-GCM.

use async_trait::async_trait;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error};

use super::SecureStorage;
use crate::crypto::{encrypt_string, decrypt_string, MasterKey};
use crate::error::{Result, WalletError};

/// Encrypted file storage backend
pub struct EncryptedFileStorage {
    /// Directory for storage files
    storage_dir: PathBuf,
    /// In-memory cache of the storage
    cache: Arc<RwLock<StorageCache>>,
    /// Master key for encryption (if wallet is unlocked)
    master_key: Arc<RwLock<Option<MasterKey>>>,
}

/// In-memory representation of stored data
#[derive(Debug, Default, Serialize, Deserialize)]
struct StorageCache {
    /// Map of key -> encrypted value
    entries: HashMap<String, String>,
    /// Whether the cache has been modified since last save
    #[serde(skip)]
    dirty: bool,
}

/// File format for persistent storage
#[derive(Debug, Serialize, Deserialize)]
struct StorageFile {
    version: u32,
    entries: HashMap<String, String>,
}

impl EncryptedFileStorage {
    /// Create a new encrypted file storage
    pub fn new() -> Result<Self> {
        let storage_dir = Self::get_storage_dir()?;

        // Ensure storage directory exists
        std::fs::create_dir_all(&storage_dir)?;

        debug!("Encrypted file storage initialized at: {:?}", storage_dir);

        Ok(Self {
            storage_dir,
            cache: Arc::new(RwLock::new(StorageCache::default())),
            master_key: Arc::new(RwLock::new(None)),
        })
    }

    /// Create with a custom storage directory (for testing)
    pub fn with_dir(storage_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&storage_dir)?;

        Ok(Self {
            storage_dir,
            cache: Arc::new(RwLock::new(StorageCache::default())),
            master_key: Arc::new(RwLock::new(None)),
        })
    }

    /// Get the default storage directory
    fn get_storage_dir() -> Result<PathBuf> {
        ProjectDirs::from("com", "symbia-labs", "mcp-wallet")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .ok_or_else(|| WalletError::StorageError("Could not determine data directory".to_string()))
    }

    /// Set the master key for encryption/decryption
    pub async fn set_master_key(&self, key: Option<MasterKey>) {
        let mut master_key = self.master_key.write().await;
        *master_key = key;
    }

    /// Check if a master key is set
    pub async fn has_master_key(&self) -> bool {
        self.master_key.read().await.is_some()
    }

    /// Get the path to the storage file
    fn storage_file_path(&self) -> PathBuf {
        self.storage_dir.join("wallet.json")
    }

    /// Get the path to the salt file
    pub fn salt_file_path(&self) -> PathBuf {
        self.storage_dir.join("salt")
    }

    /// Get the path to the verification file (used to verify password)
    fn verification_file_path(&self) -> PathBuf {
        self.storage_dir.join("verify")
    }

    /// Load storage from disk
    pub async fn load(&self) -> Result<()> {
        let path = self.storage_file_path();

        if !path.exists() {
            debug!("No existing storage file found");
            return Ok(());
        }

        let contents = tokio::fs::read_to_string(&path).await?;
        let file: StorageFile = serde_json::from_str(&contents)?;

        let mut cache = self.cache.write().await;
        cache.entries = file.entries;
        cache.dirty = false;

        debug!("Loaded {} entries from storage", cache.entries.len());
        Ok(())
    }

    /// Save storage to disk
    pub async fn save(&self) -> Result<()> {
        let cache = self.cache.read().await;

        if !cache.dirty {
            return Ok(());
        }

        let file = StorageFile {
            version: 1,
            entries: cache.entries.clone(),
        };

        let contents = serde_json::to_string_pretty(&file)?;
        let path = self.storage_file_path();

        // Write atomically using a temp file
        let temp_path = path.with_extension("tmp");
        tokio::fs::write(&temp_path, &contents).await?;
        tokio::fs::rename(&temp_path, &path).await?;

        debug!("Saved {} entries to storage", cache.entries.len());
        Ok(())
    }

    /// Save salt to disk
    pub async fn save_salt(&self, salt: &str) -> Result<()> {
        let path = self.salt_file_path();
        tokio::fs::write(&path, salt).await?;
        debug!("Saved salt to {:?}", path);
        Ok(())
    }

    /// Load salt from disk
    pub async fn load_salt(&self) -> Result<Option<String>> {
        let path = self.salt_file_path();

        if !path.exists() {
            return Ok(None);
        }

        let salt = tokio::fs::read_to_string(&path).await?;
        Ok(Some(salt.trim().to_string()))
    }

    /// Save verification data (encrypted known plaintext)
    pub async fn save_verification(&self) -> Result<()> {
        let master_key = self.master_key.read().await;
        let key = master_key.as_ref()
            .ok_or(WalletError::WalletLocked)?;

        // Encrypt a known plaintext
        let verification = encrypt_string("mcp-wallet-verification", key)?;

        let path = self.verification_file_path();
        tokio::fs::write(&path, &verification).await?;

        debug!("Saved verification data");
        Ok(())
    }

    /// Verify the master key is correct
    pub async fn verify_key(&self) -> Result<bool> {
        let path = self.verification_file_path();

        if !path.exists() {
            // No verification file - wallet not initialized
            return Ok(false);
        }

        let master_key = self.master_key.read().await;
        let key = master_key.as_ref()
            .ok_or(WalletError::WalletLocked)?;

        let encrypted = tokio::fs::read_to_string(&path).await?;

        match decrypt_string(&encrypted, key) {
            Ok(decrypted) => {
                if decrypted == "mcp-wallet-verification" {
                    debug!("Master key verified successfully");
                    Ok(true)
                } else {
                    debug!("Verification plaintext mismatch");
                    Ok(false)
                }
            }
            Err(_) => {
                debug!("Master key verification failed");
                Ok(false)
            }
        }
    }

    /// Check if the wallet has been initialized
    pub fn is_initialized(&self) -> bool {
        self.salt_file_path().exists() && self.verification_file_path().exists()
    }

    /// Get the storage directory path
    pub fn storage_dir(&self) -> &PathBuf {
        &self.storage_dir
    }
}

#[async_trait]
impl SecureStorage for EncryptedFileStorage {
    async fn store(&self, key: &str, value: &[u8]) -> Result<()> {
        let master_key_guard = self.master_key.read().await;
        let master_key = master_key_guard.as_ref()
            .ok_or(WalletError::WalletLocked)?;

        // Encrypt the value
        let value_str = String::from_utf8_lossy(value);
        let encrypted = encrypt_string(&value_str, master_key)?;

        // Store in cache
        let mut cache = self.cache.write().await;
        cache.entries.insert(key.to_string(), encrypted);
        cache.dirty = true;

        // Release locks before saving
        drop(master_key_guard);
        drop(cache);

        // Persist to disk
        self.save().await?;

        debug!("Stored key: {}", key);
        Ok(())
    }

    async fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let master_key_guard = self.master_key.read().await;
        let master_key = master_key_guard.as_ref()
            .ok_or(WalletError::WalletLocked)?;

        let cache = self.cache.read().await;

        match cache.entries.get(key) {
            Some(encrypted) => {
                let decrypted = decrypt_string(encrypted, master_key)?;
                debug!("Retrieved key: {}", key);
                Ok(Some(decrypted.into_bytes()))
            }
            None => {
                debug!("Key not found: {}", key);
                Ok(None)
            }
        }
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let mut cache = self.cache.write().await;

        if cache.entries.remove(key).is_some() {
            cache.dirty = true;
            drop(cache);
            self.save().await?;
            debug!("Deleted key: {}", key);
        }

        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let cache = self.cache.read().await;
        Ok(cache.entries.contains_key(key))
    }

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>> {
        let cache = self.cache.read().await;

        let keys: Vec<String> = cache
            .entries
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();

        Ok(keys)
    }

    async fn clear(&self) -> Result<()> {
        let mut cache = self.cache.write().await;
        cache.entries.clear();
        cache.dirty = true;
        drop(cache);

        self.save().await?;
        debug!("Cleared all entries");
        Ok(())
    }

    fn is_hardware_backed(&self) -> bool {
        false
    }

    fn backend_name(&self) -> &'static str {
        "Encrypted File Storage"
    }
}

impl Default for EncryptedFileStorage {
    fn default() -> Self {
        Self::new().expect("Failed to create encrypted file storage")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::key_derivation::{derive_key, generate_salt};
    use tempfile::TempDir;

    async fn test_storage() -> (EncryptedFileStorage, MasterKey) {
        let temp_dir = TempDir::new().unwrap();
        let storage = EncryptedFileStorage::with_dir(temp_dir.path().to_path_buf()).unwrap();

        let salt = generate_salt();
        let key = derive_key("test-password", &salt, None).unwrap();

        storage.set_master_key(Some(key.clone())).await;

        (storage, key)
    }

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let (storage, _) = test_storage().await;

        storage.store("test-key", b"test-value").await.unwrap();

        let retrieved = storage.retrieve("test-key").await.unwrap();
        assert_eq!(retrieved, Some(b"test-value".to_vec()));
    }

    #[tokio::test]
    async fn test_retrieve_nonexistent() {
        let (storage, _) = test_storage().await;

        let retrieved = storage.retrieve("nonexistent").await.unwrap();
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_delete() {
        let (storage, _) = test_storage().await;

        storage.store("test-key", b"test-value").await.unwrap();
        storage.delete("test-key").await.unwrap();

        let retrieved = storage.retrieve("test-key").await.unwrap();
        assert_eq!(retrieved, None);
    }

    #[tokio::test]
    async fn test_exists() {
        let (storage, _) = test_storage().await;

        assert!(!storage.exists("test-key").await.unwrap());

        storage.store("test-key", b"test-value").await.unwrap();

        assert!(storage.exists("test-key").await.unwrap());
    }

    #[tokio::test]
    async fn test_list_keys() {
        let (storage, _) = test_storage().await;

        storage.store("cred:openai", b"key1").await.unwrap();
        storage.store("cred:anthropic", b"key2").await.unwrap();
        storage.store("other:foo", b"bar").await.unwrap();

        let cred_keys = storage.list_keys("cred:").await.unwrap();
        assert_eq!(cred_keys.len(), 2);
        assert!(cred_keys.contains(&"cred:openai".to_string()));
        assert!(cred_keys.contains(&"cred:anthropic".to_string()));
    }

    #[tokio::test]
    async fn test_clear() {
        let (storage, _) = test_storage().await;

        storage.store("key1", b"value1").await.unwrap();
        storage.store("key2", b"value2").await.unwrap();

        storage.clear().await.unwrap();

        let keys = storage.list_keys("").await.unwrap();
        assert!(keys.is_empty());
    }

    #[tokio::test]
    async fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let salt = generate_salt();
        let key = derive_key("test-password", &salt, None).unwrap();

        // Create storage and store data
        {
            let storage = EncryptedFileStorage::with_dir(temp_dir.path().to_path_buf()).unwrap();
            storage.set_master_key(Some(key.clone())).await;
            storage.store("persistent-key", b"persistent-value").await.unwrap();
        }

        // Create new storage instance and verify data persists
        {
            let storage = EncryptedFileStorage::with_dir(temp_dir.path().to_path_buf()).unwrap();
            storage.set_master_key(Some(key)).await;
            storage.load().await.unwrap();

            let retrieved = storage.retrieve("persistent-key").await.unwrap();
            assert_eq!(retrieved, Some(b"persistent-value".to_vec()));
        }
    }

    #[tokio::test]
    async fn test_verification() {
        let temp_dir = TempDir::new().unwrap();
        let salt = generate_salt();
        let correct_key = derive_key("correct-password", &salt, None).unwrap();
        let wrong_key = derive_key("wrong-password", &salt, None).unwrap();

        let storage = EncryptedFileStorage::with_dir(temp_dir.path().to_path_buf()).unwrap();

        // Save verification with correct key
        storage.set_master_key(Some(correct_key.clone())).await;
        storage.save_verification().await.unwrap();

        // Verify with correct key
        assert!(storage.verify_key().await.unwrap());

        // Verify with wrong key
        storage.set_master_key(Some(wrong_key)).await;
        assert!(!storage.verify_key().await.unwrap());
    }
}
