//! Cryptographic primitives for secure credential storage
//!
//! This module provides:
//! - AES-256-GCM authenticated encryption
//! - Argon2id key derivation from passwords
//! - Secure memory handling with zeroize

mod encryption;
mod key_derivation;
mod secure_memory;

pub use encryption::{decrypt, decrypt_string, encrypt, encrypt_string, EncryptedData};
pub use key_derivation::{derive_key, generate_salt, KeyDerivationParams};
pub use secure_memory::{MasterKey, SecretString};
