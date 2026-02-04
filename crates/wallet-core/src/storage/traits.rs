//! Storage trait definitions

use crate::error::Result;
use async_trait::async_trait;

/// Trait for secure storage backends
#[async_trait]
pub trait SecureStorage: Send + Sync {
    /// Store a value with the given key
    async fn store(&self, key: &str, value: &[u8]) -> Result<()>;

    /// Retrieve a value by key
    async fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Delete a value by key
    async fn delete(&self, key: &str) -> Result<()>;

    /// Check if a key exists
    async fn exists(&self, key: &str) -> Result<bool>;

    /// List all keys with a given prefix
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>>;

    /// Clear all stored data
    async fn clear(&self) -> Result<()>;

    /// Check if this storage backend is hardware-backed
    fn is_hardware_backed(&self) -> bool;

    /// Get a human-readable name for this storage backend
    fn backend_name(&self) -> &'static str;
}
