//! AES-256-GCM authenticated encryption
//!
//! Encryption format: `{iv_hex}:{auth_tag_hex}:{ciphertext_hex}`
//! - IV: 12 bytes (96 bits) - standard for GCM
//! - Auth tag: 16 bytes (128 bits)
//! - Ciphertext: variable length

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;

use super::MasterKey;
use crate::error::{Result, WalletError};

/// Encrypted data with IV and auth tag
#[derive(Debug, Clone)]
pub struct EncryptedData {
    /// Initialization vector (12 bytes for GCM)
    pub iv: [u8; 12],
    /// Authentication tag (16 bytes)
    pub auth_tag: [u8; 16],
    /// Encrypted ciphertext
    pub ciphertext: Vec<u8>,
}

impl std::fmt::Display for EncryptedData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            hex::encode(self.iv),
            hex::encode(self.auth_tag),
            hex::encode(&self.ciphertext)
        )
    }
}

impl EncryptedData {
    /// Parse from the format: `{iv_hex}:{auth_tag_hex}:{ciphertext_hex}`
    pub fn from_string(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 3 {
            return Err(WalletError::DecryptionError(
                "Invalid encrypted data format: expected iv:tag:ciphertext".to_string(),
            ));
        }

        let iv_bytes = hex::decode(parts[0])
            .map_err(|e| WalletError::DecryptionError(format!("Invalid IV hex: {}", e)))?;
        let auth_tag_bytes = hex::decode(parts[1])
            .map_err(|e| WalletError::DecryptionError(format!("Invalid auth tag hex: {}", e)))?;
        let ciphertext = hex::decode(parts[2])
            .map_err(|e| WalletError::DecryptionError(format!("Invalid ciphertext hex: {}", e)))?;

        if iv_bytes.len() != 12 {
            return Err(WalletError::DecryptionError(format!(
                "Invalid IV length: expected 12, got {}",
                iv_bytes.len()
            )));
        }
        if auth_tag_bytes.len() != 16 {
            return Err(WalletError::DecryptionError(format!(
                "Invalid auth tag length: expected 16, got {}",
                auth_tag_bytes.len()
            )));
        }

        let mut iv = [0u8; 12];
        iv.copy_from_slice(&iv_bytes);

        let mut auth_tag = [0u8; 16];
        auth_tag.copy_from_slice(&auth_tag_bytes);

        Ok(Self {
            iv,
            auth_tag,
            ciphertext,
        })
    }
}

/// Encrypt plaintext using AES-256-GCM
///
/// # Arguments
/// * `plaintext` - The data to encrypt
/// * `key` - The 256-bit encryption key
///
/// # Returns
/// Encrypted data containing IV, auth tag, and ciphertext
pub fn encrypt(plaintext: &[u8], key: &MasterKey) -> Result<EncryptedData> {
    let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
        .map_err(|e| WalletError::EncryptionError(e.to_string()))?;

    // Generate random IV (12 bytes for GCM)
    let mut iv = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut iv);
    let nonce = Nonce::from_slice(&iv);

    // Encrypt - aes-gcm appends the auth tag to the ciphertext
    let ciphertext_with_tag = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| WalletError::EncryptionError(e.to_string()))?;

    // Split ciphertext and auth tag (last 16 bytes)
    if ciphertext_with_tag.len() < 16 {
        return Err(WalletError::EncryptionError(
            "Ciphertext too short".to_string(),
        ));
    }

    let tag_start = ciphertext_with_tag.len() - 16;
    let ciphertext = ciphertext_with_tag[..tag_start].to_vec();
    let mut auth_tag = [0u8; 16];
    auth_tag.copy_from_slice(&ciphertext_with_tag[tag_start..]);

    Ok(EncryptedData {
        iv,
        auth_tag,
        ciphertext,
    })
}

/// Encrypt a string and return the serialized format
pub fn encrypt_string(plaintext: &str, key: &MasterKey) -> Result<String> {
    let encrypted = encrypt(plaintext.as_bytes(), key)?;
    Ok(encrypted.to_string())
}

/// Decrypt ciphertext using AES-256-GCM
///
/// # Arguments
/// * `encrypted` - The encrypted data containing IV, auth tag, and ciphertext
/// * `key` - The 256-bit decryption key
///
/// # Returns
/// The decrypted plaintext
pub fn decrypt(encrypted: &EncryptedData, key: &MasterKey) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
        .map_err(|e| WalletError::DecryptionError(e.to_string()))?;

    let nonce = Nonce::from_slice(&encrypted.iv);

    // Reconstruct ciphertext with tag appended (as expected by aes-gcm)
    let mut ciphertext_with_tag = encrypted.ciphertext.clone();
    ciphertext_with_tag.extend_from_slice(&encrypted.auth_tag);

    cipher
        .decrypt(nonce, ciphertext_with_tag.as_slice())
        .map_err(|e| WalletError::DecryptionError(e.to_string()))
}

/// Decrypt from serialized format and return as string
pub fn decrypt_string(encrypted_str: &str, key: &MasterKey) -> Result<String> {
    let encrypted = EncryptedData::from_string(encrypted_str)?;
    let plaintext = decrypt(&encrypted, key)?;
    String::from_utf8(plaintext)
        .map_err(|e| WalletError::DecryptionError(format!("Invalid UTF-8: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::key_derivation::{derive_key, generate_salt};

    fn test_key() -> MasterKey {
        let salt = generate_salt();
        derive_key("test-password", &salt, None).unwrap()
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = test_key();
        let plaintext = b"Hello, World!";

        let encrypted = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_string_decrypt_string_roundtrip() {
        let key = test_key();
        let plaintext = "sk-proj-abc123xyz789";

        let encrypted = encrypt_string(plaintext, &key).unwrap();
        let decrypted = decrypt_string(&encrypted, &key).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypted_data_serialization() {
        let key = test_key();
        let plaintext = b"test data";

        let encrypted = encrypt(plaintext, &key).unwrap();
        let serialized = encrypted.to_string();
        let parsed = EncryptedData::from_string(&serialized).unwrap();

        assert_eq!(encrypted.iv, parsed.iv);
        assert_eq!(encrypted.auth_tag, parsed.auth_tag);
        assert_eq!(encrypted.ciphertext, parsed.ciphertext);
    }

    #[test]
    fn test_different_ivs_produce_different_ciphertext() {
        let key = test_key();
        let plaintext = b"same plaintext";

        let encrypted1 = encrypt(plaintext, &key).unwrap();
        let encrypted2 = encrypt(plaintext, &key).unwrap();

        // IVs should be different (random)
        assert_ne!(encrypted1.iv, encrypted2.iv);
        // Ciphertexts should be different due to different IVs
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
    }

    #[test]
    fn test_wrong_key_fails_decryption() {
        let key1 = test_key();
        let key2 = test_key(); // Different key due to different salt
        let plaintext = b"secret data";

        let encrypted = encrypt(plaintext, &key1).unwrap();
        let result = decrypt(&encrypted, &key2);

        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_ciphertext_fails_decryption() {
        let key = test_key();
        let plaintext = b"secret data";

        let mut encrypted = encrypt(plaintext, &key).unwrap();
        // Tamper with ciphertext
        if !encrypted.ciphertext.is_empty() {
            encrypted.ciphertext[0] ^= 0xFF;
        }

        let result = decrypt(&encrypted, &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_auth_tag_fails_decryption() {
        let key = test_key();
        let plaintext = b"secret data";

        let mut encrypted = encrypt(plaintext, &key).unwrap();
        // Tamper with auth tag
        encrypted.auth_tag[0] ^= 0xFF;

        let result = decrypt(&encrypted, &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_format_parsing() {
        assert!(EncryptedData::from_string("invalid").is_err());
        assert!(EncryptedData::from_string("a:b").is_err());
        assert!(EncryptedData::from_string("a:b:c:d").is_err());
        assert!(EncryptedData::from_string("not_hex:not_hex:not_hex").is_err());
    }
}
