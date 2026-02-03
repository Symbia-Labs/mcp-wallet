//! Main wallet orchestration

use std::sync::Arc;
use tracing::{info, debug};

use crate::crypto::{derive_key, generate_salt, MasterKey, KeyDerivationParams};
use crate::credential::CredentialManager;
use crate::error::{Result, WalletError};
use crate::integration::IntegrationRegistry;
use crate::session::{Session, SessionManager};
use crate::settings::{SettingsManager, Settings, OtelSettings};
use crate::storage::EncryptedFileStorage;

/// Wallet state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalletState {
    /// Wallet has not been created yet
    NotInitialized,
    /// Wallet is locked (password required)
    Locked,
    /// Wallet is unlocked and ready
    Unlocked,
}

/// Main wallet struct that orchestrates all functionality
pub struct Wallet {
    /// Storage backend
    storage: Arc<EncryptedFileStorage>,
    /// Integration registry
    pub integrations: IntegrationRegistry,
    /// Credential manager
    pub credentials: CredentialManager,
    /// Session manager for CLI access
    session_manager: SessionManager,
    /// Settings manager (non-sensitive config)
    settings_manager: SettingsManager,
    /// Current master key (when unlocked)
    master_key: Option<MasterKey>,
    /// Current state
    state: WalletState,
}

impl Wallet {
    /// Create a new wallet instance
    pub fn new() -> Result<Self> {
        let storage = Arc::new(EncryptedFileStorage::new()?);

        let state = if storage.is_initialized() {
            WalletState::Locked
        } else {
            WalletState::NotInitialized
        };

        let session_manager = SessionManager::new(storage.storage_dir());
        let settings_manager = SettingsManager::new(storage.storage_dir());
        let integrations = IntegrationRegistry::new(storage.clone());
        let credentials = CredentialManager::new(storage.clone());

        Ok(Self {
            storage,
            integrations,
            credentials,
            session_manager,
            settings_manager,
            master_key: None,
            state,
        })
    }

    /// Create a new wallet with a custom storage directory (for testing)
    pub fn with_storage(storage: Arc<EncryptedFileStorage>) -> Self {
        let state = if storage.is_initialized() {
            WalletState::Locked
        } else {
            WalletState::NotInitialized
        };

        let session_manager = SessionManager::new(storage.storage_dir());
        let settings_manager = SettingsManager::new(storage.storage_dir());
        let integrations = IntegrationRegistry::new(storage.clone());
        let credentials = CredentialManager::new(storage.clone());

        Self {
            storage,
            integrations,
            credentials,
            session_manager,
            settings_manager,
            master_key: None,
            state,
        }
    }

    /// Get the current wallet state
    pub fn state(&self) -> WalletState {
        self.state
    }

    /// Check if the wallet is unlocked
    pub fn is_unlocked(&self) -> bool {
        self.state == WalletState::Unlocked
    }

    /// Initialize a new wallet with a password
    pub async fn initialize(&mut self, password: &str) -> Result<()> {
        if self.state != WalletState::NotInitialized {
            return Err(WalletError::StorageError(
                "Wallet already initialized".to_string(),
            ));
        }

        info!("Initializing new wallet");

        // Generate salt and derive key
        let salt = generate_salt();
        let master_key = derive_key(password, &salt, None)?;

        // Save salt
        self.storage.save_salt(&salt).await?;

        // Set master key and save verification
        self.storage.set_master_key(Some(master_key.clone())).await;
        self.storage.save_verification().await?;

        // Set master key for credentials and store locally
        self.credentials.set_master_key(Some(master_key.clone())).await;
        self.master_key = Some(master_key);

        self.state = WalletState::Unlocked;

        info!("Wallet initialized successfully");
        Ok(())
    }

    /// Unlock the wallet with a password
    pub async fn unlock(&mut self, password: &str) -> Result<()> {
        if self.state == WalletState::NotInitialized {
            return Err(WalletError::WalletNotInitialized);
        }

        if self.state == WalletState::Unlocked {
            debug!("Wallet already unlocked");
            return Ok(());
        }

        // Load salt and derive key
        let salt = self.storage.load_salt().await?
            .ok_or(WalletError::WalletNotInitialized)?;

        let master_key = derive_key(password, &salt, None)?;

        // Set key and verify
        self.storage.set_master_key(Some(master_key.clone())).await;

        if !self.storage.verify_key().await? {
            self.storage.set_master_key(None).await;
            return Err(WalletError::InvalidPassword);
        }

        // Load data
        self.storage.load().await?;

        // Set master key for credentials and store locally
        self.credentials.set_master_key(Some(master_key.clone())).await;
        self.master_key = Some(master_key);

        // Load integrations
        self.integrations.load().await?;

        self.state = WalletState::Unlocked;

        info!("Wallet unlocked successfully");
        Ok(())
    }

    /// Unlock the wallet using a session token (no password needed)
    pub async fn unlock_with_session(&mut self) -> Result<()> {
        if self.state == WalletState::NotInitialized {
            return Err(WalletError::WalletNotInitialized);
        }

        if self.state == WalletState::Unlocked {
            debug!("Wallet already unlocked");
            return Ok(());
        }

        // Load session
        let session = self.session_manager.load_session().await?
            .ok_or(WalletError::InvalidSession)?;

        // Get master key from session
        let master_key = session.get_master_key(&session.token)?;

        // Set key and verify
        self.storage.set_master_key(Some(master_key.clone())).await;

        if !self.storage.verify_key().await? {
            self.storage.set_master_key(None).await;
            return Err(WalletError::InvalidSession);
        }

        // Load data
        self.storage.load().await?;

        // Set master key for credentials and store locally
        self.credentials.set_master_key(Some(master_key.clone())).await;
        self.master_key = Some(master_key);

        // Load integrations
        self.integrations.load().await?;

        self.state = WalletState::Unlocked;

        info!("Wallet unlocked via session token");
        Ok(())
    }

    /// Create a session token for CLI access (requires wallet to be unlocked)
    pub async fn create_session(&self, duration_secs: Option<u64>) -> Result<String> {
        if self.state != WalletState::Unlocked {
            return Err(WalletError::WalletLocked);
        }

        let master_key = self.master_key.as_ref()
            .ok_or(WalletError::WalletLocked)?;

        let session = Session::create(master_key, duration_secs)?;
        let token = session.token.clone();

        self.session_manager.save_session(&session).await?;

        info!("Created session token (expires in {} seconds)", session.remaining_secs());
        Ok(token)
    }

    /// Check if a valid session exists
    pub async fn has_valid_session(&self) -> bool {
        match self.session_manager.load_session().await {
            Ok(Some(_)) => true,
            _ => false,
        }
    }

    /// Clear the current session
    pub async fn clear_session(&self) -> Result<()> {
        self.session_manager.clear_session().await
    }

    /// Get current settings
    pub fn get_settings(&self) -> &Settings {
        self.settings_manager.get()
    }

    /// Update settings
    pub async fn update_settings(&mut self, settings: Settings) -> Result<()> {
        self.settings_manager.update(settings).await
    }

    /// Get OpenTelemetry settings
    pub fn get_otel_settings(&self) -> &OtelSettings {
        self.settings_manager.get_otel()
    }

    /// Update OpenTelemetry settings
    pub async fn update_otel_settings(&mut self, otel: OtelSettings) -> Result<()> {
        self.settings_manager.update_otel(otel).await
    }

    /// Get auto-lock timeout in minutes
    pub fn get_auto_lock_timeout(&self) -> u32 {
        self.settings_manager.get_auto_lock_timeout()
    }

    /// Set auto-lock timeout in minutes
    pub async fn set_auto_lock_timeout(&mut self, minutes: u32) -> Result<()> {
        self.settings_manager.set_auto_lock_timeout(minutes).await
    }

    /// Lock the wallet (clear master key from memory)
    pub async fn lock(&mut self) -> Result<()> {
        self.storage.set_master_key(None).await;
        self.credentials.set_master_key(None).await;
        self.master_key = None;

        // Clear session when locking
        let _ = self.session_manager.clear_session().await;

        self.state = WalletState::Locked;

        info!("Wallet locked");
        Ok(())
    }

    /// Change the wallet password
    pub async fn change_password(&mut self, old_password: &str, new_password: &str) -> Result<()> {
        if self.state == WalletState::NotInitialized {
            return Err(WalletError::WalletNotInitialized);
        }

        // Verify old password
        let salt = self.storage.load_salt().await?
            .ok_or(WalletError::WalletNotInitialized)?;

        let old_key = derive_key(old_password, &salt, None)?;
        self.storage.set_master_key(Some(old_key)).await;

        if !self.storage.verify_key().await? {
            self.storage.set_master_key(None).await;
            return Err(WalletError::InvalidPassword);
        }

        // Generate new salt and key
        let new_salt = generate_salt();
        let new_key = derive_key(new_password, &new_salt, None)?;

        // Save new salt and verification
        self.storage.save_salt(&new_salt).await?;
        self.storage.set_master_key(Some(new_key.clone())).await;
        self.storage.save_verification().await?;

        // Update credentials manager
        self.credentials.set_master_key(Some(new_key)).await;

        // Re-encrypt all stored data with new key
        // Note: This is handled by the storage layer on save

        info!("Password changed successfully");
        Ok(())
    }

    /// Get the storage directory path
    pub fn storage_dir(&self) -> &std::path::PathBuf {
        self.storage.storage_dir()
    }

    /// Check if hardware-backed storage is available
    pub fn has_hardware_storage(&self) -> bool {
        // Check if keychain is available
        let keychain = crate::storage::KeychainStorage::new(None);
        keychain.is_available()
    }

    /// Reset the wallet completely - deletes ALL data including integrations, credentials, and settings
    /// WARNING: This is irreversible!
    pub async fn reset(&mut self) -> Result<()> {
        info!("Resetting wallet - deleting all data");

        // Clear storage
        self.storage.clear().await?;

        // Clear session
        let _ = self.session_manager.clear_session().await;

        // Reset settings
        self.settings_manager.reset().await?;

        // Clear in-memory state
        self.master_key = None;
        self.storage.set_master_key(None).await;
        self.credentials.set_master_key(None).await;

        self.state = WalletState::NotInitialized;

        info!("Wallet reset complete");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::path::PathBuf;

    async fn test_wallet() -> (Wallet, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(
            EncryptedFileStorage::with_dir(temp_dir.path().to_path_buf()).unwrap()
        );
        let wallet = Wallet::with_storage(storage);
        (wallet, temp_dir)
    }

    #[tokio::test]
    async fn test_initialize_and_unlock() {
        let (mut wallet, _temp) = test_wallet().await;

        assert_eq!(wallet.state(), WalletState::NotInitialized);

        wallet.initialize("test-password").await.unwrap();
        assert_eq!(wallet.state(), WalletState::Unlocked);

        wallet.lock().await.unwrap();
        assert_eq!(wallet.state(), WalletState::Locked);

        wallet.unlock("test-password").await.unwrap();
        assert_eq!(wallet.state(), WalletState::Unlocked);
    }

    #[tokio::test]
    async fn test_wrong_password() {
        let (mut wallet, _temp) = test_wallet().await;

        wallet.initialize("correct-password").await.unwrap();
        wallet.lock().await.unwrap();

        let result = wallet.unlock("wrong-password").await;
        assert!(matches!(result, Err(WalletError::InvalidPassword)));
    }

    #[tokio::test]
    async fn test_change_password() {
        let (mut wallet, _temp) = test_wallet().await;

        wallet.initialize("old-password").await.unwrap();
        wallet.lock().await.unwrap();

        wallet.unlock("old-password").await.unwrap();
        wallet.change_password("old-password", "new-password").await.unwrap();
        wallet.lock().await.unwrap();

        // Old password should fail
        let result = wallet.unlock("old-password").await;
        assert!(matches!(result, Err(WalletError::InvalidPassword)));

        // New password should work
        wallet.unlock("new-password").await.unwrap();
        assert_eq!(wallet.state(), WalletState::Unlocked);
    }
}
