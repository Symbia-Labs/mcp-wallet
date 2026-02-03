//! Error types for wallet-core

use thiserror::Error;

/// Result type alias for wallet operations
pub type Result<T> = std::result::Result<T, WalletError>;

/// Wallet error types
#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Wallet is locked - unlock with password first")]
    WalletLocked,

    #[error("Wallet is not initialized - create a new wallet first")]
    WalletNotInitialized,

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Encryption failed: {0}")]
    EncryptionError(String),

    #[error("Decryption failed: {0}")]
    DecryptionError(String),

    #[error("Key derivation failed: {0}")]
    KeyDerivationError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Keychain error: {0}")]
    KeychainError(String),

    #[error("Integration not found: {0}")]
    IntegrationNotFound(String),

    #[error("Credential not found: {0}")]
    CredentialNotFound(String),

    #[error("Operation not found: {0}")]
    OperationNotFound(String),

    #[error("Invalid OpenAPI spec: {0}")]
    InvalidSpec(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Session expired - please unlock wallet in the app")]
    SessionExpired,

    #[error("Invalid session token")]
    InvalidSession,

    #[error("Crypto error: {0}")]
    CryptoError(String),
}
