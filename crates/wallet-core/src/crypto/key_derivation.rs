//! Password-based key derivation using Argon2id

use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2, Algorithm, Params, Version,
};
use rand::rngs::OsRng;

use super::MasterKey;
use crate::error::{Result, WalletError};

/// Parameters for Argon2id key derivation
#[derive(Debug, Clone)]
pub struct KeyDerivationParams {
    /// Memory cost in KiB (default: 65536 = 64MB)
    pub memory_cost: u32,
    /// Time cost / iterations (default: 3)
    pub time_cost: u32,
    /// Parallelism (default: 4)
    pub parallelism: u32,
}

impl Default for KeyDerivationParams {
    fn default() -> Self {
        Self {
            memory_cost: 65536, // 64 MB
            time_cost: 3,
            parallelism: 4,
        }
    }
}

/// Generate a cryptographically secure random salt
pub fn generate_salt() -> String {
    SaltString::generate(&mut OsRng).to_string()
}

/// Derive a 256-bit master key from a password using Argon2id
///
/// # Arguments
/// * `password` - The user's password
/// * `salt` - A salt string (use `generate_salt()` to create one)
/// * `params` - Optional key derivation parameters
///
/// # Returns
/// A 32-byte master key suitable for AES-256 encryption
pub fn derive_key(
    password: &str,
    salt: &str,
    params: Option<KeyDerivationParams>,
) -> Result<MasterKey> {
    let params = params.unwrap_or_default();

    let argon2_params = Params::new(
        params.memory_cost,
        params.time_cost,
        params.parallelism,
        Some(32), // Output length: 32 bytes = 256 bits
    )
    .map_err(|e| WalletError::KeyDerivationError(e.to_string()))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon2_params);

    let salt = SaltString::from_b64(salt)
        .map_err(|e| WalletError::KeyDerivationError(format!("Invalid salt: {}", e)))?;

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| WalletError::KeyDerivationError(e.to_string()))?;

    let hash = password_hash
        .hash
        .ok_or_else(|| WalletError::KeyDerivationError("No hash output".to_string()))?;

    let hash_bytes = hash.as_bytes();
    if hash_bytes.len() < 32 {
        return Err(WalletError::KeyDerivationError(
            "Hash output too short".to_string(),
        ));
    }

    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&hash_bytes[..32]);

    Ok(MasterKey::new(key_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_salt() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();

        // Salts should be different
        assert_ne!(salt1, salt2);

        // Salt should be valid base64
        assert!(!salt1.is_empty());
    }

    #[test]
    fn test_derive_key() {
        let password = "test-password-123";
        let salt = generate_salt();

        let key = derive_key(password, &salt, None).unwrap();

        // Key should be 32 bytes
        assert_eq!(key.as_bytes().len(), 32);
    }

    #[test]
    fn test_derive_key_deterministic() {
        let password = "test-password-123";
        let salt = generate_salt();

        let key1 = derive_key(password, &salt, None).unwrap();
        let key2 = derive_key(password, &salt, None).unwrap();

        // Same password + salt should produce same key
        assert_eq!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_derive_key_different_passwords() {
        let salt = generate_salt();

        let key1 = derive_key("password1", &salt, None).unwrap();
        let key2 = derive_key("password2", &salt, None).unwrap();

        // Different passwords should produce different keys
        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_derive_key_different_salts() {
        let password = "test-password";

        let key1 = derive_key(password, &generate_salt(), None).unwrap();
        let key2 = derive_key(password, &generate_salt(), None).unwrap();

        // Different salts should produce different keys
        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_derive_key_with_custom_params() {
        let password = "test-password";
        let salt = generate_salt();

        let params = KeyDerivationParams {
            memory_cost: 8192, // 8 MB (faster for testing)
            time_cost: 1,
            parallelism: 1,
        };

        let key = derive_key(password, &salt, Some(params)).unwrap();
        assert_eq!(key.as_bytes().len(), 32);
    }
}
