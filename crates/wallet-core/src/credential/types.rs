//! Credential type definitions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Type of credential
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialType {
    /// API key (static token)
    ApiKey,
    /// OAuth2 access token
    OAuth2Token,
    /// Basic authentication (username:password)
    BasicAuth,
}

impl Default for CredentialType {
    fn default() -> Self {
        Self::ApiKey
    }
}

/// Credential metadata (safe to display)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    /// Unique identifier
    pub id: Uuid,

    /// Provider/service name (e.g., "openai", "stripe")
    pub provider: String,

    /// User-friendly name
    pub name: String,

    /// Type of credential
    pub credential_type: CredentialType,

    /// First 8 characters for display (e.g., "sk-proj-...")
    pub prefix: Option<String>,

    /// Associated integration ID (if any)
    pub integration_id: Option<Uuid>,

    /// Last time this credential was used
    pub last_used_at: Option<DateTime<Utc>>,

    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

impl Credential {
    /// Create a new API key credential
    pub fn new_api_key(provider: &str, name: &str, api_key: &str) -> Self {
        let prefix = if api_key.len() >= 8 {
            Some(format!("{}...", &api_key[..8]))
        } else {
            Some(format!("{}...", api_key))
        };

        Self {
            id: Uuid::new_v4(),
            provider: provider.to_string(),
            name: name.to_string(),
            credential_type: CredentialType::ApiKey,
            prefix,
            integration_id: None,
            last_used_at: None,
            created_at: Utc::now(),
        }
    }

    /// Create a new OAuth2 token credential
    pub fn new_oauth2(provider: &str, name: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            provider: provider.to_string(),
            name: name.to_string(),
            credential_type: CredentialType::OAuth2Token,
            prefix: None,
            integration_id: None,
            last_used_at: None,
            created_at: Utc::now(),
        }
    }
}

/// Decrypted credential value - automatically zeroed when dropped
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct DecryptedCredential {
    /// The actual secret value
    value: String,
}

impl DecryptedCredential {
    /// Create a new decrypted credential
    pub fn new(value: String) -> Self {
        Self { value }
    }

    /// Get the secret value (use carefully)
    pub fn expose(&self) -> &str {
        &self.value
    }

    /// Consume and return the inner value
    pub fn into_inner(mut self) -> String {
        std::mem::take(&mut self.value)
    }
}

impl std::fmt::Debug for DecryptedCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DecryptedCredential")
            .field("value", &"[REDACTED]")
            .finish()
    }
}

/// Stored credential (encrypted)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCredential {
    /// Credential metadata
    pub credential: Credential,

    /// Encrypted value (iv:tag:ciphertext format)
    pub encrypted_value: String,

    /// Encrypted refresh token (for OAuth2)
    pub encrypted_refresh_token: Option<String>,

    /// Token expiration (for OAuth2)
    pub expires_at: Option<DateTime<Utc>>,
}
