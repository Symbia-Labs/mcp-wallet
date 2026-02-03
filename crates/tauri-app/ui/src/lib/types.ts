// Wallet state
export type WalletState = "loading" | "not_initialized" | "locked" | "unlocked";

// Integration category
export type IntegrationCategory =
  | "ai_models"
  | "chat"
  | "productivity"
  | "smart_home"
  | "tools"
  | "media"
  | "social"
  | "other";

// Integration definition (from catalog)
export interface IntegrationDef {
  id: string;
  name: string;
  description: string;
  category: IntegrationCategory;
  icon: string;
  specUrl?: string;
  authType: "bearer" | "api_key" | "oauth2" | "basic" | "none";
  authHeader?: string;
  docsUrl?: string;
  tags?: string[];
}

// Installed integration
export interface Integration {
  id: string;
  key: string;
  name: string;
  description?: string;
  specUrl?: string;
  serverUrl: string;
  status: "pending" | "active" | "error" | "disabled";
  credentialId?: string;
  operationCount: number;
  lastSyncedAt?: string;
  error?: string;
  createdAt: string;
  updatedAt: string;
}

// Credential
export interface Credential {
  id: string;
  provider: string;
  name: string;
  credentialType: "api_key" | "oauth2_token" | "basic_auth";
  prefix?: string;
  integrationId?: string;
  lastUsedAt?: string;
  createdAt: string;
}

// Server status
export interface ServerStatus {
  running: boolean;
  mode: "stdio" | "http";
  port?: number;
  connectedClients: number;
}

// MCP Operation/Tool
export interface Operation {
  id: string;
  name: string;
  description: string;
  method: "GET" | "POST" | "PUT" | "DELETE" | "PATCH";
  path: string;
  parameters: OperationParameter[];
}

export interface OperationParameter {
  name: string;
  type: string;
  required: boolean;
  description?: string;
  location: "path" | "query" | "header" | "body";
}

// Category metadata
export const CATEGORY_INFO: Record<IntegrationCategory, { label: string; icon: string; color: string }> = {
  ai_models: { label: "AI Models", icon: "Brain", color: "text-purple-400" },
  chat: { label: "Chat & Messaging", icon: "MessageSquare", color: "text-blue-400" },
  productivity: { label: "Productivity", icon: "Briefcase", color: "text-green-400" },
  smart_home: { label: "Smart Home", icon: "Home", color: "text-yellow-400" },
  tools: { label: "Tools & Automation", icon: "Wrench", color: "text-orange-400" },
  media: { label: "Media & Creative", icon: "Image", color: "text-pink-400" },
  social: { label: "Social", icon: "Users", color: "text-cyan-400" },
  other: { label: "Other", icon: "Layers", color: "text-gray-400" },
};
