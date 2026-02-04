//! Storage backends for secure credential persistence
//!
//! This module provides two storage backends:
//! 1. OS Keychain (hardware-backed where available)
//! 2. Encrypted file (fallback)

mod encrypted_file;
mod keychain;
mod traits;

pub use encrypted_file::EncryptedFileStorage;
pub use keychain::KeychainStorage;
pub use traits::SecureStorage;
