//! Session management for MCP clients
//!
//! Allows the GUI app to create a session token that CLI processes can use
//! to access the wallet without needing the master password.

use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;

use crate::crypto::{decrypt_string, encrypt_string, MasterKey};
use crate::error::{Result, WalletError};

/// Session token for CLI access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Random session token (used as encryption key for the master key)
    pub token: String,
    /// Master key encrypted with the session token
    pub encrypted_master_key: String,
    /// When this session expires (Unix timestamp)
    pub expires_at: u64,
    /// Session ID for logging/revocation
    pub session_id: String,
}

impl Session {
    /// Default session duration: 24 hours
    const DEFAULT_DURATION_SECS: u64 = 24 * 60 * 60;

    /// Create a new session from an unlocked master key
    pub fn create(master_key: &MasterKey, duration_secs: Option<u64>) -> Result<Self> {
        let duration = duration_secs.unwrap_or(Self::DEFAULT_DURATION_SECS);

        // Generate random 32-byte session token
        let mut token_bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut token_bytes);
        let token = hex::encode(token_bytes); // 64 hex chars

        // Generate session ID
        let session_id = uuid::Uuid::new_v4().to_string();

        // Use the raw bytes as the encryption key
        let token_key = MasterKey::new(token_bytes);

        // Serialize and encrypt the master key
        let master_key_bytes = master_key.as_bytes();
        let master_key_hex = hex::encode(master_key_bytes);
        let encrypted_master_key = encrypt_string(&master_key_hex, &token_key)?;

        // Calculate expiration
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expires_at = now + duration;

        debug!("Created session {} expiring at {}", session_id, expires_at);

        Ok(Self {
            token,
            encrypted_master_key,
            expires_at,
            session_id,
        })
    }

    /// Decrypt and retrieve the master key using the session token
    pub fn get_master_key(&self, token: &str) -> Result<MasterKey> {
        // Check expiration
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if now > self.expires_at {
            return Err(WalletError::SessionExpired);
        }

        // Verify token matches
        if token != self.token {
            return Err(WalletError::InvalidSession);
        }

        // Decrypt master key - decode hex token back to bytes
        let token_vec = hex::decode(token)
            .map_err(|e| WalletError::CryptoError(format!("Invalid token format: {}", e)))?;
        let token_bytes: [u8; 32] = token_vec
            .as_slice()
            .try_into()
            .map_err(|_| WalletError::CryptoError("Invalid token length".to_string()))?;
        let token_key = MasterKey::new(token_bytes);

        let master_key_hex = decrypt_string(&self.encrypted_master_key, &token_key)?;
        let master_key_bytes =
            hex::decode(&master_key_hex).map_err(|e| WalletError::CryptoError(e.to_string()))?;

        let master_key_arr: [u8; 32] = master_key_bytes
            .as_slice()
            .try_into()
            .map_err(|_| WalletError::CryptoError("Invalid master key length".to_string()))?;
        let master_key = MasterKey::new(master_key_arr);

        Ok(master_key)
    }

    /// Check if the session is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.expires_at
    }

    /// Get remaining time in seconds
    pub fn remaining_secs(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.expires_at.saturating_sub(now)
    }
}

/// Session file manager
pub struct SessionManager {
    session_file: PathBuf,
}

impl SessionManager {
    /// Create a new session manager for the given wallet directory
    pub fn new(wallet_dir: &Path) -> Self {
        Self {
            session_file: wallet_dir.join("session.json"),
        }
    }

    /// Save a session to disk (called by GUI app)
    pub async fn save_session(&self, session: &Session) -> Result<()> {
        let json = serde_json::to_string_pretty(session)
            .map_err(|e| WalletError::StorageError(e.to_string()))?;

        tokio::fs::write(&self.session_file, &json).await?;

        debug!("Saved session to {:?}", self.session_file);
        Ok(())
    }

    /// Load a session from disk (called by CLI)
    pub async fn load_session(&self) -> Result<Option<Session>> {
        if !self.session_file.exists() {
            return Ok(None);
        }

        let json = tokio::fs::read_to_string(&self.session_file).await?;
        let session: Session =
            serde_json::from_str(&json).map_err(|e| WalletError::StorageError(e.to_string()))?;

        // Check if expired
        if session.is_expired() {
            debug!("Session expired, removing file");
            self.clear_session().await?;
            return Ok(None);
        }

        Ok(Some(session))
    }

    /// Clear the session (called on logout/lock or expiration)
    pub async fn clear_session(&self) -> Result<()> {
        if self.session_file.exists() {
            tokio::fs::remove_file(&self.session_file).await?;
            debug!("Cleared session file");
        }
        Ok(())
    }

    /// Get the session token if a valid session exists
    pub async fn get_token(&self) -> Result<Option<String>> {
        match self.load_session().await? {
            Some(session) => Ok(Some(session.token.clone())),
            None => Ok(None),
        }
    }
}
