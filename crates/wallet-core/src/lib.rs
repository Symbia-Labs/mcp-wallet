//! # wallet-core
//!
//! Core wallet functionality for MCP Wallet including:
//! - AES-256-GCM encryption with secure key derivation
//! - OS keychain integration with encrypted file fallback
//! - Integration registry for OpenAPI-based services
//! - Credential management with zeroize-on-drop security

pub mod crypto;
pub mod storage;
pub mod integration;
pub mod credential;
pub mod error;
pub mod session;
pub mod settings;
mod wallet;

pub use error::{WalletError, Result};
pub use wallet::{Wallet, WalletState};
pub use crypto::{MasterKey, encrypt, decrypt, encrypt_string, decrypt_string, generate_salt};
pub use storage::{SecureStorage, KeychainStorage, EncryptedFileStorage};
pub use integration::{Integration, IntegrationRegistry, IntegrationOperation, IntegrationStatus, StoredIntegration};
pub use credential::{Credential, CredentialManager, DecryptedCredential, CredentialType};
pub use session::{Session, SessionManager};
pub use settings::{Settings, SettingsManager, OtelSettings};
