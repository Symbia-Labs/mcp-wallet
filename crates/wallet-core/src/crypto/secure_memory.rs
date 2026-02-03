//! Secure memory handling with automatic zeroization

use zeroize::{Zeroize, ZeroizeOnDrop};

/// Master encryption key - automatically zeroed when dropped
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct MasterKey {
    key: [u8; 32],
}

impl MasterKey {
    /// Create a new master key from raw bytes
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    /// Get the key bytes (use carefully - avoid copying)
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key
    }

    /// Create from a slice (must be exactly 32 bytes)
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() != 32 {
            return None;
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(slice);
        Some(Self { key })
    }
}

impl Clone for MasterKey {
    fn clone(&self) -> Self {
        Self { key: self.key }
    }
}

impl std::fmt::Debug for MasterKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MasterKey")
            .field("key", &"[REDACTED]")
            .finish()
    }
}

/// Decrypted secret value - automatically zeroed when dropped
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecretString {
    value: String,
}

impl SecretString {
    /// Create a new secret string
    pub fn new(value: String) -> Self {
        Self { value }
    }

    /// Get the secret value (use carefully)
    pub fn expose(&self) -> &str {
        &self.value
    }

    /// Consume and return the inner value
    pub fn into_inner(mut self) -> String {
        std::mem::take(&mut self.value)
    }
}

impl std::fmt::Debug for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretString")
            .field("value", &"[REDACTED]")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_key_from_slice() {
        let bytes = [42u8; 32];
        let key = MasterKey::from_slice(&bytes).unwrap();
        assert_eq!(key.as_bytes(), &bytes);
    }

    #[test]
    fn test_master_key_from_invalid_slice() {
        let bytes = [42u8; 16];
        assert!(MasterKey::from_slice(&bytes).is_none());
    }

    #[test]
    fn test_secret_string_expose() {
        let secret = SecretString::new("my-secret".to_string());
        assert_eq!(secret.expose(), "my-secret");
    }

    #[test]
    fn test_debug_redacted() {
        let key = MasterKey::new([0u8; 32]);
        let debug = format!("{:?}", key);
        assert!(debug.contains("REDACTED"));
        assert!(!debug.contains("0"));
    }
}
