//! Storage backends for secure credential persistence
//!
//! This module provides two storage backends:
//! 1. OS Keychain (hardware-backed where available)
//! 2. Encrypted file (fallback)

mod traits;
mod keychain;
mod encrypted_file;

pub use traits::SecureStorage;
pub use keychain::KeychainStorage;
pub use encrypted_file::EncryptedFileStorage;
