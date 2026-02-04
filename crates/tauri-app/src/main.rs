//! MCP Wallet - Desktop Application
//!
//! A secure desktop wallet for managing API integrations and exposing them via MCP protocol.

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;
use uuid::Uuid;

use mcp_server::ServerMode;
use wallet_core::credential::{Credential, CredentialType};
use wallet_core::integration::{Integration, IntegrationStatus};
use wallet_core::settings::OtelSettings;
use wallet_core::{Wallet, WalletState as CoreWalletState};

/// Application state managed by Tauri
pub struct AppState {
    pub wallet: Arc<RwLock<Wallet>>,
    pub server: Arc<RwLock<Option<ServerHandle>>>,
}

/// Handle to a running MCP server
pub struct ServerHandle {
    pub mode: ServerMode,
    pub running: bool,
}

/// Wallet state for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WalletStateResponse {
    NotInitialized,
    Locked,
    Unlocked,
}

impl From<CoreWalletState> for WalletStateResponse {
    fn from(state: CoreWalletState) -> Self {
        match state {
            CoreWalletState::NotInitialized => WalletStateResponse::NotInitialized,
            CoreWalletState::Locked => WalletStateResponse::Locked,
            CoreWalletState::Unlocked => WalletStateResponse::Unlocked,
        }
    }
}

/// Integration response for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegrationResponse {
    pub id: String,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub spec_url: Option<String>,
    pub server_url: String,
    pub status: String,
    pub credential_id: Option<String>,
    pub operation_count: usize,
    pub last_synced_at: Option<String>,
    pub error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Integration> for IntegrationResponse {
    fn from(i: Integration) -> Self {
        let status = match i.status {
            IntegrationStatus::Pending => "pending",
            IntegrationStatus::Active => "active",
            IntegrationStatus::Error => "error",
            IntegrationStatus::Disabled => "disabled",
        };

        Self {
            id: i.id.to_string(),
            key: i.key.clone(),
            name: i.name.clone(),
            description: i.description.clone(),
            spec_url: i.spec_url.clone(),
            server_url: i.server_url.clone(),
            status: status.to_string(),
            credential_id: i.credential_id.map(|id| id.to_string()),
            operation_count: i.operation_count,
            last_synced_at: i.last_synced_at.map(|dt| dt.to_rfc3339()),
            error: i.error.clone(),
            created_at: i.created_at.to_rfc3339(),
            updated_at: i.updated_at.to_rfc3339(),
        }
    }
}

/// Credential response for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialResponse {
    pub id: String,
    pub provider: String,
    pub name: String,
    pub credential_type: String,
    pub prefix: Option<String>,
    pub integration_id: Option<String>,
    pub last_used_at: Option<String>,
    pub created_at: String,
}

impl From<Credential> for CredentialResponse {
    fn from(c: Credential) -> Self {
        let cred_type = match c.credential_type {
            CredentialType::ApiKey => "api_key",
            CredentialType::OAuth2Token => "oauth2_token",
            CredentialType::BasicAuth => "basic_auth",
        };

        Self {
            id: c.id.to_string(),
            provider: c.provider.clone(),
            name: c.name.clone(),
            credential_type: cred_type.to_string(),
            prefix: c.prefix.clone(),
            integration_id: c.integration_id.map(|id| id.to_string()),
            last_used_at: c.last_used_at.map(|dt| dt.to_rfc3339()),
            created_at: c.created_at.to_rfc3339(),
        }
    }
}

/// Server status response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerStatusResponse {
    pub running: bool,
    pub mode: String,
    pub port: Option<u16>,
    pub connected_clients: usize,
}

/// OpenTelemetry settings response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OtelSettingsResponse {
    pub enabled: bool,
    pub endpoint: Option<String>,
    pub service_name: Option<String>,
    pub auth_header: Option<String>,
    pub export_traces: bool,
    pub export_metrics: bool,
}

impl From<OtelSettings> for OtelSettingsResponse {
    fn from(s: OtelSettings) -> Self {
        Self {
            enabled: s.enabled,
            endpoint: s.endpoint,
            service_name: s.service_name,
            auth_header: s.auth_header,
            export_traces: s.export_traces,
            export_metrics: s.export_metrics,
        }
    }
}

impl From<OtelSettingsResponse> for OtelSettings {
    fn from(s: OtelSettingsResponse) -> Self {
        Self {
            enabled: s.enabled,
            endpoint: s.endpoint,
            service_name: s.service_name,
            auth_header: s.auth_header,
            export_traces: s.export_traces,
            export_metrics: s.export_metrics,
        }
    }
}

/// Operation response for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub method: String,
    pub path: String,
    pub parameters: Vec<OperationParameterResponse>,
}

/// Operation parameter response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationParameterResponse {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub required: bool,
    pub description: Option<String>,
    pub location: String,
}

// ============================================================================
// Wallet Commands
// ============================================================================

#[tauri::command]
async fn get_wallet_state(state: State<'_, AppState>) -> Result<WalletStateResponse, String> {
    let wallet = state.wallet.read().await;
    Ok(wallet.state().into())
}

#[tauri::command]
async fn initialize_wallet(password: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut wallet = state.wallet.write().await;
    wallet
        .initialize(&password)
        .await
        .map_err(|e| e.to_string())?;

    // Create a session for CLI access (24 hour default)
    wallet
        .create_session(None)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn unlock_wallet(password: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut wallet = state.wallet.write().await;
    wallet.unlock(&password).await.map_err(|e| e.to_string())?;

    // Create a session for CLI access (24 hour default)
    wallet
        .create_session(None)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn lock_wallet(state: State<'_, AppState>) -> Result<(), String> {
    // Stop the server first to prevent any in-flight requests
    {
        let mut server = state.server.write().await;
        *server = None;
    }

    // Lock the wallet (clears master key from memory and revokes session token)
    let mut wallet = state.wallet.write().await;
    wallet.lock().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn reset_wallet(state: State<'_, AppState>) -> Result<(), String> {
    let mut wallet = state.wallet.write().await;
    wallet.reset().await.map_err(|e| e.to_string())
}

// ============================================================================
// Integration Commands
// ============================================================================

#[tauri::command]
async fn list_integrations(state: State<'_, AppState>) -> Result<Vec<IntegrationResponse>, String> {
    let wallet = state.wallet.read().await;
    let integrations = wallet.integrations.list().await;
    Ok(integrations
        .into_iter()
        .map(IntegrationResponse::from)
        .collect())
}

#[tauri::command]
async fn add_integration(
    key: String,
    spec_url: String,
    state: State<'_, AppState>,
) -> Result<IntegrationResponse, String> {
    let wallet = state.wallet.read().await;

    if spec_url.is_empty() {
        return Err("Spec URL is required to add an integration".to_string());
    }

    let integration = wallet
        .integrations
        .add_from_url(&key, &spec_url)
        .await
        .map_err(|e| e.to_string())?;

    Ok(IntegrationResponse::from(integration))
}

#[tauri::command]
async fn remove_integration(key: String, state: State<'_, AppState>) -> Result<(), String> {
    let wallet = state.wallet.read().await;
    wallet
        .integrations
        .remove(&key)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn sync_integration(
    key: String,
    state: State<'_, AppState>,
) -> Result<IntegrationResponse, String> {
    let wallet = state.wallet.read().await;
    wallet
        .integrations
        .sync(&key)
        .await
        .map_err(|e| e.to_string())?;

    let integration = wallet
        .integrations
        .get(&key)
        .await
        .ok_or_else(|| format!("Integration '{}' not found after sync", key))?;

    Ok(IntegrationResponse::from(integration))
}

#[tauri::command]
async fn get_operations(
    integration_key: String,
    state: State<'_, AppState>,
) -> Result<Vec<OperationResponse>, String> {
    let wallet = state.wallet.read().await;
    let operations = wallet.integrations.list_operations(&integration_key).await;

    Ok(operations
        .into_iter()
        .map(|op| {
            let method_str = match op.method {
                openapi_parser::HttpMethod::Get => "GET",
                openapi_parser::HttpMethod::Post => "POST",
                openapi_parser::HttpMethod::Put => "PUT",
                openapi_parser::HttpMethod::Patch => "PATCH",
                openapi_parser::HttpMethod::Delete => "DELETE",
                openapi_parser::HttpMethod::Head => "HEAD",
                openapi_parser::HttpMethod::Options => "OPTIONS",
                openapi_parser::HttpMethod::Trace => "TRACE",
            };

            OperationResponse {
                id: op.operation_id.clone(),
                name: format!("{}_{}", integration_key, op.normalized_id),
                description: op.description.or(op.summary).unwrap_or_default(),
                method: method_str.to_string(),
                path: op.path,
                parameters: op
                    .parameters
                    .into_iter()
                    .map(|p| {
                        let location_str = match p.location {
                            openapi_parser::ParameterLocation::Path => "path",
                            openapi_parser::ParameterLocation::Query => "query",
                            openapi_parser::ParameterLocation::Header => "header",
                            openapi_parser::ParameterLocation::Cookie => "cookie",
                        };

                        // Extract type from schema if available
                        let param_type = p
                            .schema
                            .as_ref()
                            .and_then(|s| s.get("type"))
                            .and_then(|t| t.as_str())
                            .unwrap_or("string")
                            .to_string();

                        OperationParameterResponse {
                            name: p.name,
                            param_type,
                            required: p.required,
                            description: p.description,
                            location: location_str.to_string(),
                        }
                    })
                    .collect(),
            }
        })
        .collect())
}

// ============================================================================
// Credential Commands
// ============================================================================

#[tauri::command]
async fn list_credentials(state: State<'_, AppState>) -> Result<Vec<CredentialResponse>, String> {
    let wallet = state.wallet.read().await;
    let credentials = wallet.credentials.list().await.map_err(|e| e.to_string())?;
    Ok(credentials
        .into_iter()
        .map(CredentialResponse::from)
        .collect())
}

#[tauri::command]
async fn add_credential(
    provider: String,
    name: String,
    api_key: String,
    state: State<'_, AppState>,
) -> Result<CredentialResponse, String> {
    let wallet = state.wallet.read().await;
    let credential = wallet
        .credentials
        .add_api_key(&provider, &name, &api_key)
        .await
        .map_err(|e| e.to_string())?;
    Ok(CredentialResponse::from(credential))
}

#[tauri::command]
async fn delete_credential(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let wallet = state.wallet.read().await;
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    wallet
        .credentials
        .delete(uuid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn bind_credential(
    integration_key: String,
    credential_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let wallet = state.wallet.read().await;
    let cred_uuid = Uuid::parse_str(&credential_id).map_err(|e| e.to_string())?;
    wallet
        .integrations
        .set_credential(&integration_key, cred_uuid)
        .await
        .map_err(|e| e.to_string())
}

// ============================================================================
// Server Commands
// ============================================================================

#[tauri::command]
async fn get_server_status(state: State<'_, AppState>) -> Result<ServerStatusResponse, String> {
    let server = state.server.read().await;
    match server.as_ref() {
        Some(handle) => Ok(ServerStatusResponse {
            running: handle.running,
            mode: match handle.mode {
                ServerMode::Stdio => "stdio".to_string(),
                ServerMode::Http { .. } => "http".to_string(),
            },
            port: match handle.mode {
                ServerMode::Http { port } => Some(port),
                _ => None,
            },
            connected_clients: 0,
        }),
        None => Ok(ServerStatusResponse {
            running: false,
            mode: "stdio".to_string(),
            port: None,
            connected_clients: 0,
        }),
    }
}

#[tauri::command]
async fn start_server(
    mode: String,
    port: Option<u16>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut server = state.server.write().await;

    let server_mode = if mode == "http" {
        ServerMode::Http {
            port: port.unwrap_or(3000),
        }
    } else {
        ServerMode::Stdio
    };

    *server = Some(ServerHandle {
        mode: server_mode,
        running: true,
    });

    Ok(())
}

#[tauri::command]
async fn stop_server(state: State<'_, AppState>) -> Result<(), String> {
    let mut server = state.server.write().await;
    *server = None;
    Ok(())
}

// ============================================================================
// Utility Commands
// ============================================================================

#[tauri::command]
fn get_executable_path() -> Result<String, String> {
    // Return the path to the MCP server binary (bundled as sidecar)
    let exe_dir = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .ok_or("Failed to get exe directory")?
        .to_path_buf();

    // Try different possible names for the sidecar binary
    // Tauri may add the target triple suffix
    let possible_names = [
        "mcp-wallet-server",
        "mcp-wallet-server-aarch64-apple-darwin",
        "mcp-wallet-server-x86_64-apple-darwin",
        "mcp-wallet-server-x86_64-unknown-linux-gnu",
        "mcp-wallet-server-x86_64-pc-windows-msvc.exe",
        "mcp-wallet-server.exe",
    ];

    for name in possible_names {
        let path = exe_dir.join(name);
        if path.exists() {
            return Ok(path.to_string_lossy().to_string());
        }
    }

    // Fallback to default name even if it doesn't exist
    Ok(exe_dir
        .join("mcp-wallet-server")
        .to_string_lossy()
        .to_string())
}

// ============================================================================
// Settings Commands
// ============================================================================

#[tauri::command]
async fn get_otel_settings(state: State<'_, AppState>) -> Result<OtelSettingsResponse, String> {
    let wallet = state.wallet.read().await;
    Ok(OtelSettingsResponse::from(
        wallet.get_otel_settings().clone(),
    ))
}

#[tauri::command]
async fn update_otel_settings(
    settings: OtelSettingsResponse,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut wallet = state.wallet.write().await;
    wallet
        .update_otel_settings(OtelSettings::from(settings))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_auto_lock_timeout(state: State<'_, AppState>) -> Result<u32, String> {
    let wallet = state.wallet.read().await;
    Ok(wallet.get_auto_lock_timeout())
}

#[tauri::command]
async fn set_auto_lock_timeout(minutes: u32, state: State<'_, AppState>) -> Result<(), String> {
    let mut wallet = state.wallet.write().await;
    wallet
        .set_auto_lock_timeout(minutes)
        .await
        .map_err(|e| e.to_string())
}

// ============================================================================
// Main Entry Point
// ============================================================================

fn main() {
    let wallet = Wallet::new().expect("Failed to create wallet");
    let app_state = AppState {
        wallet: Arc::new(RwLock::new(wallet)),
        server: Arc::new(RwLock::new(None)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_wallet_state,
            initialize_wallet,
            unlock_wallet,
            lock_wallet,
            reset_wallet,
            list_integrations,
            add_integration,
            remove_integration,
            sync_integration,
            get_operations,
            list_credentials,
            add_credential,
            delete_credential,
            bind_credential,
            get_server_status,
            start_server,
            stop_server,
            get_executable_path,
            get_otel_settings,
            update_otel_settings,
            get_auto_lock_timeout,
            set_auto_lock_timeout,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
