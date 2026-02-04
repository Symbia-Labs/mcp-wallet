//! # wallet-core
//!
//! Core wallet functionality for MCP Wallet including:
//! - AES-256-GCM encryption with secure key derivation
//! - OS keychain integration with encrypted file fallback
//! - Integration registry for OpenAPI-based services
//! - Credential management with zeroize-on-drop security

pub mod credential;
pub mod crypto;
pub mod error;
pub mod integration;
pub mod session;
pub mod settings;
pub mod storage;
mod wallet;

pub use credential::{Credential, CredentialManager, CredentialType, DecryptedCredential};
pub use crypto::{decrypt, decrypt_string, encrypt, encrypt_string, generate_salt, MasterKey};
pub use error::{Result, WalletError};
pub use integration::{
    Integration, IntegrationOperation, IntegrationRegistry, IntegrationStatus, StoredIntegration,
};
pub use session::{Session, SessionManager};
pub use settings::{OtelSettings, Settings, SettingsManager};
pub use storage::{EncryptedFileStorage, KeychainStorage, SecureStorage};
pub use wallet::{Wallet, WalletState};
