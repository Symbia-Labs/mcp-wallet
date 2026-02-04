//! Application settings management
//!
//! Stores non-sensitive configuration in a plain JSON file.
//! Settings are accessible even when the wallet is locked.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::debug;

use crate::error::Result;

/// OpenTelemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OtelSettings {
    /// Whether OpenTelemetry is enabled
    pub enabled: bool,
    /// OTLP endpoint URL (e.g., "http://localhost:4317" or "https://otel.example.com:4317")
    pub endpoint: Option<String>,
    /// Service name for traces/metrics (defaults to "symbia-mcp-wallet")
    pub service_name: Option<String>,
    /// Optional authorization header value (e.g., "Bearer <token>")
    pub auth_header: Option<String>,
    /// Whether to export traces
    pub export_traces: bool,
    /// Whether to export metrics
    pub export_metrics: bool,
}

impl OtelSettings {
    /// Get the effective service name
    pub fn effective_service_name(&self) -> &str {
        self.service_name.as_deref().unwrap_or("symbia-mcp-wallet")
    }
}

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// Settings file version
    pub version: u32,
    /// Auto-lock timeout in minutes (0 = never)
    pub auto_lock_timeout_minutes: u32,
    /// OpenTelemetry configuration
    pub otel: OtelSettings,
}

impl Settings {
    /// Create default settings
    pub fn new() -> Self {
        Self {
            version: 1,
            auto_lock_timeout_minutes: 15,
            otel: OtelSettings {
                enabled: false,
                endpoint: None,
                service_name: Some("symbia-mcp-wallet".to_string()),
                auth_header: None,
                export_traces: true,
                export_metrics: true,
            },
        }
    }
}

/// Settings manager
pub struct SettingsManager {
    settings_file: PathBuf,
    settings: Settings,
}

impl SettingsManager {
    /// Create a new settings manager
    pub fn new(storage_dir: &Path) -> Self {
        let settings_file = storage_dir.join("settings.json");
        let settings = Self::load_from_file(&settings_file).unwrap_or_default();

        Self {
            settings_file,
            settings,
        }
    }

    /// Load settings from file
    fn load_from_file(path: &Path) -> Result<Settings> {
        if !path.exists() {
            debug!("No settings file found, using defaults");
            return Ok(Settings::new());
        }

        let contents = std::fs::read_to_string(path)?;
        let settings: Settings = serde_json::from_str(&contents)?;
        debug!("Loaded settings from {:?}", path);
        Ok(settings)
    }

    /// Save settings to file
    pub async fn save(&self) -> Result<()> {
        let contents = serde_json::to_string_pretty(&self.settings)?;

        // Write atomically using temp file
        let temp_path = self.settings_file.with_extension("tmp");
        tokio::fs::write(&temp_path, &contents).await?;
        tokio::fs::rename(&temp_path, &self.settings_file).await?;

        debug!("Saved settings to {:?}", self.settings_file);
        Ok(())
    }

    /// Get current settings
    pub fn get(&self) -> &Settings {
        &self.settings
    }

    /// Get mutable settings
    pub fn get_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }

    /// Update settings and save
    pub async fn update(&mut self, settings: Settings) -> Result<()> {
        self.settings = settings;
        self.save().await
    }

    /// Get OTEL settings
    pub fn get_otel(&self) -> &OtelSettings {
        &self.settings.otel
    }

    /// Update OTEL settings and save
    pub async fn update_otel(&mut self, otel: OtelSettings) -> Result<()> {
        self.settings.otel = otel;
        self.save().await
    }

    /// Get auto-lock timeout
    pub fn get_auto_lock_timeout(&self) -> u32 {
        self.settings.auto_lock_timeout_minutes
    }

    /// Set auto-lock timeout and save
    pub async fn set_auto_lock_timeout(&mut self, minutes: u32) -> Result<()> {
        self.settings.auto_lock_timeout_minutes = minutes;
        self.save().await
    }

    /// Reset settings to defaults and delete settings file
    pub async fn reset(&mut self) -> Result<()> {
        self.settings = Settings::default();

        // Delete settings file if it exists
        if self.settings_file.exists() {
            tokio::fs::remove_file(&self.settings_file)
                .await
                .map_err(|e| crate::error::WalletError::StorageError(e.to_string()))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_settings_default() {
        let temp_dir = TempDir::new().unwrap();
        let manager = SettingsManager::new(temp_dir.path());

        let settings = manager.get();
        assert_eq!(settings.auto_lock_timeout_minutes, 15);
        assert!(!settings.otel.enabled);
    }

    #[tokio::test]
    async fn test_settings_persistence() {
        let temp_dir = TempDir::new().unwrap();

        // Create and modify settings
        {
            let mut manager = SettingsManager::new(temp_dir.path());
            manager.get_mut().auto_lock_timeout_minutes = 30;
            manager.get_mut().otel.enabled = true;
            manager.get_mut().otel.endpoint = Some("http://localhost:4317".to_string());
            manager.save().await.unwrap();
        }

        // Load and verify
        {
            let manager = SettingsManager::new(temp_dir.path());
            assert_eq!(manager.get().auto_lock_timeout_minutes, 30);
            assert!(manager.get().otel.enabled);
            assert_eq!(
                manager.get().otel.endpoint,
                Some("http://localhost:4317".to_string())
            );
        }
    }

    #[tokio::test]
    async fn test_update_otel() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = SettingsManager::new(temp_dir.path());

        let otel = OtelSettings {
            enabled: true,
            endpoint: Some("https://otel.example.com:4317".to_string()),
            service_name: Some("my-service".to_string()),
            auth_header: Some("Bearer token123".to_string()),
            export_traces: true,
            export_metrics: false,
        };

        manager.update_otel(otel.clone()).await.unwrap();

        assert!(manager.get_otel().enabled);
        assert_eq!(
            manager.get_otel().endpoint,
            Some("https://otel.example.com:4317".to_string())
        );
        assert_eq!(manager.get_otel().effective_service_name(), "my-service");
    }
}
