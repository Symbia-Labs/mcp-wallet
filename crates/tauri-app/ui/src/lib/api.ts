import { invoke } from "@tauri-apps/api/core";
import { WalletState, Integration, Credential, ServerStatus, Operation } from "./types";

// Wallet state
export async function getWalletState(): Promise<WalletState> {
  return await invoke<WalletState>("get_wallet_state");
}

export async function initializeWallet(password: string): Promise<void> {
  await invoke("initialize_wallet", { password });
}

export async function unlockWallet(password: string): Promise<void> {
  await invoke("unlock_wallet", { password });
}

export async function lockWallet(): Promise<void> {
  await invoke("lock_wallet");
}

// Integrations
export async function listIntegrations(): Promise<Integration[]> {
  return await invoke<Integration[]>("list_integrations");
}

export async function addIntegration(key: string, specUrl: string): Promise<Integration> {
  return await invoke<Integration>("add_integration", { key, specUrl });
}

export async function removeIntegration(key: string): Promise<void> {
  await invoke("remove_integration", { key });
}

// Credentials
export async function listCredentials(): Promise<Credential[]> {
  return await invoke<Credential[]>("list_credentials");
}

export async function addCredential(
  provider: string,
  name: string,
  apiKey: string
): Promise<Credential> {
  return await invoke<Credential>("add_credential", { provider, name, apiKey });
}

export async function deleteCredential(id: string): Promise<void> {
  await invoke("delete_credential", { id });
}

export async function bindCredential(integrationKey: string, credentialId: string): Promise<void> {
  await invoke("bind_credential", { integrationKey, credentialId });
}

// Server
export async function getServerStatus(): Promise<ServerStatus> {
  return await invoke<ServerStatus>("get_server_status");
}

export async function startServer(mode: "stdio" | "http", port?: number): Promise<void> {
  await invoke("start_server", { mode, port });
}

export async function stopServer(): Promise<void> {
  await invoke("stop_server");
}

// Operations
export async function getOperations(integrationKey: string): Promise<Operation[]> {
  return await invoke<Operation[]>("get_operations", { integrationKey });
}

// Utilities
export async function getExecutablePath(): Promise<string> {
  return await invoke<string>("get_executable_path");
}

// Settings
export interface OtelSettings {
  enabled: boolean;
  endpoint: string | null;
  serviceName: string | null;
  authHeader: string | null;
  exportTraces: boolean;
  exportMetrics: boolean;
}

export async function getOtelSettings(): Promise<OtelSettings> {
  return await invoke<OtelSettings>("get_otel_settings");
}

export async function updateOtelSettings(settings: OtelSettings): Promise<void> {
  await invoke("update_otel_settings", { settings });
}

export async function getAutoLockTimeout(): Promise<number> {
  return await invoke<number>("get_auto_lock_timeout");
}

export async function setAutoLockTimeout(minutes: number): Promise<void> {
  await invoke("set_auto_lock_timeout", { minutes });
}
