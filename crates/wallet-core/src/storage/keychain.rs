//! OS Keychain storage backend
//!
//! Uses the system keychain for secure storage:
//! - macOS: Keychain
//! - Windows: Credential Manager (DPAPI)
//! - Linux: Secret Service (GNOME Keyring, KWallet)

use async_trait::async_trait;
use keyring::Entry;
use tracing::{debug, warn};

use super::SecureStorage;
use crate::error::{Result, WalletError};

/// Service name used for keychain entries
const SERVICE_NAME: &str = "mcp-wallet";

/// OS Keychain storage backend
pub struct KeychainStorage {
    /// Prefix for all keys (for namespacing)
    prefix: String,
    /// Whether keychain is available
    available: bool,
}

impl KeychainStorage {
    /// Create a new keychain storage with optional prefix
    pub fn new(prefix: Option<&str>) -> Self {
        let prefix = prefix.map(|p| format!("{}-", p)).unwrap_or_default();

        // Test if keychain is available
        let available = Self::test_availability();

        if available {
            debug!("Keychain storage is available");
        } else {
            warn!("Keychain storage is not available - will use fallback");
        }

        Self { prefix, available }
    }

    /// Test if the keychain is available
    fn test_availability() -> bool {
        let test_entry = Entry::new(SERVICE_NAME, "__test_availability__");
        match test_entry {
            Ok(entry) => {
                // Try to set and delete a test value
                let result = entry.set_password("test");
                if result.is_ok() {
                    let _ = entry.delete_password();
                    true
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }

    /// Get a keyring entry for a key
    fn get_entry(&self, key: &str) -> Result<Entry> {
        let full_key = format!("{}{}", self.prefix, key);
        Entry::new(SERVICE_NAME, &full_key).map_err(|e| WalletError::KeychainError(e.to_string()))
    }

    /// Check if keychain is available
    pub fn is_available(&self) -> bool {
        self.available
    }
}

#[async_trait]
impl SecureStorage for KeychainStorage {
    async fn store(&self, key: &str, value: &[u8]) -> Result<()> {
        if !self.available {
            return Err(WalletError::KeychainError(
                "Keychain not available".to_string(),
            ));
        }

        let entry = self.get_entry(key)?;

        // Store as base64-encoded string (keychain stores strings)
        let encoded = base64_encode(value);

        entry
            .set_password(&encoded)
            .map_err(|e| WalletError::KeychainError(e.to_string()))?;

        debug!("Stored key in keychain: {}", key);
        Ok(())
    }

    async fn retrieve(&self, key: &str) -> Result<Option<Vec<u8>>> {
        if !self.available {
            return Err(WalletError::KeychainError(
                "Keychain not available".to_string(),
            ));
        }

        let entry = self.get_entry(key)?;

        match entry.get_password() {
            Ok(encoded) => {
                let decoded = base64_decode(&encoded)?;
                debug!("Retrieved key from keychain: {}", key);
                Ok(Some(decoded))
            }
            Err(keyring::Error::NoEntry) => {
                debug!("Key not found in keychain: {}", key);
                Ok(None)
            }
            Err(e) => Err(WalletError::KeychainError(e.to_string())),
        }
    }

    async fn delete(&self, key: &str) -> Result<()> {
        if !self.available {
            return Err(WalletError::KeychainError(
                "Keychain not available".to_string(),
            ));
        }

        let entry = self.get_entry(key)?;

        match entry.delete_password() {
            Ok(()) => {
                debug!("Deleted key from keychain: {}", key);
                Ok(())
            }
            Err(keyring::Error::NoEntry) => {
                // Key doesn't exist, that's fine
                Ok(())
            }
            Err(e) => Err(WalletError::KeychainError(e.to_string())),
        }
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        if !self.available {
            return Err(WalletError::KeychainError(
                "Keychain not available".to_string(),
            ));
        }

        let entry = self.get_entry(key)?;

        match entry.get_password() {
            Ok(_) => Ok(true),
            Err(keyring::Error::NoEntry) => Ok(false),
            Err(e) => Err(WalletError::KeychainError(e.to_string())),
        }
    }

    async fn list_keys(&self, _prefix: &str) -> Result<Vec<String>> {
        // Note: Most keychain implementations don't support listing keys
        // This would require platform-specific implementations
        // For now, return an error indicating this isn't supported
        Err(WalletError::KeychainError(
            "Listing keys is not supported by keychain storage".to_string(),
        ))
    }

    async fn clear(&self) -> Result<()> {
        // Note: Clearing all keys requires knowing all keys first
        // Since listing isn't supported, we can't implement this generically
        Err(WalletError::KeychainError(
            "Clearing all keys is not supported by keychain storage".to_string(),
        ))
    }

    fn is_hardware_backed(&self) -> bool {
        // On macOS with Secure Enclave, this would be true
        // For now, we consider all OS keychains as "hardware-backed"
        // since they use OS-level protection (DPAPI on Windows, etc.)
        self.available
    }

    fn backend_name(&self) -> &'static str {
        #[cfg(target_os = "macos")]
        return "macOS Keychain";

        #[cfg(target_os = "windows")]
        return "Windows Credential Manager";

        #[cfg(target_os = "linux")]
        return "Linux Secret Service";

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        return "System Keychain";
    }
}

/// Base64 encode bytes
fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

/// Base64 decode string
fn base64_decode(encoded: &str) -> Result<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| WalletError::StorageError(format!("Base64 decode error: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_keychain_availability() {
        let storage = KeychainStorage::new(Some("test"));
        // Just check that we can query availability without panicking
        let _ = storage.is_available();
    }
}
