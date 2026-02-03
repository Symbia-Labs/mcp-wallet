import { useState, useEffect } from "react";
import {
  Key,
  Plus,
  Trash2,
  Eye,
  EyeOff,
  AlertCircle,
  Loader2,
  Shield,
  Clock,
} from "lucide-react";
import { Credential, Integration } from "../lib/types";
import {
  listCredentials,
  listIntegrations,
  addCredential,
  deleteCredential,
  bindCredential,
} from "../lib/api";
import { INTEGRATION_CATALOG } from "../lib/catalog";

export default function CredentialsPage() {
  const [credentials, setCredentials] = useState<Credential[]>([]);
  const [integrations, setIntegrations] = useState<Integration[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAddModal, setShowAddModal] = useState(false);
  const [deletingId, setDeletingId] = useState<string | null>(null);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      const [creds, ints] = await Promise.all([
        listCredentials(),
        listIntegrations(),
      ]);
      setCredentials(creds);
      setIntegrations(ints);
    } catch (error) {
      console.error("Failed to load credentials:", error);
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async (id: string) => {
    setDeletingId(id);
    try {
      await deleteCredential(id);
      await loadData();
    } catch (error) {
      console.error("Failed to delete credential:", error);
    } finally {
      setDeletingId(null);
    }
  };

  const handleAddComplete = async () => {
    setShowAddModal(false);
    await loadData();
  };

  // Group credentials by provider
  const credentialsByProvider = credentials.reduce((acc, cred) => {
    if (!acc[cred.provider]) acc[cred.provider] = [];
    acc[cred.provider].push(cred);
    return acc;
  }, {} as Record<string, Credential[]>);

  // Find integrations that need credentials
  const pendingIntegrations = integrations.filter((i) => i.status === "pending");

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="w-8 h-8 border-4 border-accent border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  return (
    <div className="space-y-6 animate-fadeIn">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Credentials</h1>
          <p className="text-gray-400 mt-1">
            Securely manage your API keys and tokens
          </p>
        </div>
        <button
          onClick={() => setShowAddModal(true)}
          className="flex items-center gap-2 px-4 py-2 bg-accent hover:bg-accent-hover text-white rounded-lg transition-colors"
        >
          <Plus className="w-4 h-4" />
          Add Credential
        </button>
      </div>

      {/* Pending Integrations Alert */}
      {pendingIntegrations.length > 0 && (
        <div className="bg-yellow-500/10 border border-yellow-500/20 rounded-xl p-4">
          <div className="flex items-start gap-3">
            <AlertCircle className="w-5 h-5 text-yellow-400 flex-shrink-0 mt-0.5" />
            <div>
              <h3 className="font-medium text-yellow-400">
                {pendingIntegrations.length} integration{pendingIntegrations.length > 1 ? "s" : ""} need credentials
              </h3>
              <p className="text-sm text-yellow-400/70 mt-1">
                Add API keys for: {pendingIntegrations.map((i) => i.name).join(", ")}
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Security Info */}
      <div className="bg-surface-elevated rounded-xl border border-gray-800 p-4">
        <div className="flex items-center gap-3">
          <Shield className="w-5 h-5 text-accent" />
          <p className="text-sm text-gray-400">
            All credentials are encrypted with AES-256-GCM using your master password.
            Only the prefix is shown for identification.
          </p>
        </div>
      </div>

      {/* Credentials List */}
      {credentials.length === 0 ? (
        <div className="text-center py-12 bg-surface-elevated rounded-xl border border-gray-800">
          <Key className="w-12 h-12 text-gray-600 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-white mb-2">
            No credentials yet
          </h3>
          <p className="text-gray-400 mb-4">
            Add API keys to enable your integrations
          </p>
          <button
            onClick={() => setShowAddModal(true)}
            className="inline-flex items-center gap-2 px-4 py-2 bg-accent hover:bg-accent-hover text-white rounded-lg transition-colors"
          >
            <Plus className="w-4 h-4" />
            Add Your First Credential
          </button>
        </div>
      ) : (
        <div className="space-y-4">
          {Object.entries(credentialsByProvider).map(([provider, creds]) => {
            const catalogEntry = INTEGRATION_CATALOG.find((i) => i.id === provider);
            const integration = integrations.find((i) => i.key === provider);

            return (
              <div
                key={provider}
                className="bg-surface-elevated rounded-xl border border-gray-800 overflow-hidden"
              >
                {/* Provider Header */}
                <div className="px-5 py-4 border-b border-gray-800 bg-gray-800/30">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <div className="w-8 h-8 rounded-lg bg-gray-700 flex items-center justify-center text-white font-bold">
                        {provider.charAt(0).toUpperCase()}
                      </div>
                      <div>
                        <h3 className="font-medium text-white">
                          {catalogEntry?.name || provider}
                        </h3>
                        <p className="text-xs text-gray-500">
                          {creds.length} credential{creds.length > 1 ? "s" : ""}
                        </p>
                      </div>
                    </div>
                    {integration && (
                      <div
                        className={`px-2 py-1 rounded-full text-xs font-medium ${
                          integration.status === "active"
                            ? "bg-green-500/10 text-green-400"
                            : integration.status === "pending"
                            ? "bg-yellow-500/10 text-yellow-400"
                            : "bg-red-500/10 text-red-400"
                        }`}
                      >
                        {integration.status === "active" ? "Connected" : integration.status}
                      </div>
                    )}
                  </div>
                </div>

                {/* Credentials */}
                <div className="divide-y divide-gray-800">
                  {creds.map((cred) => (
                    <div
                      key={cred.id}
                      className="px-5 py-4 flex items-center justify-between"
                    >
                      <div className="flex items-center gap-4">
                        <Key className="w-5 h-5 text-gray-500" />
                        <div>
                          <p className="font-medium text-white">{cred.name}</p>
                          <div className="flex items-center gap-3 mt-1">
                            <code className="text-sm text-gray-500 font-mono">
                              {cred.prefix}
                            </code>
                            <span className="text-xs text-gray-600">
                              {cred.credentialType.replace("_", " ")}
                            </span>
                          </div>
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        {cred.lastUsedAt && (
                          <span className="flex items-center gap-1 text-xs text-gray-500">
                            <Clock className="w-3 h-3" />
                            Used {new Date(cred.lastUsedAt).toLocaleDateString()}
                          </span>
                        )}
                        <button
                          onClick={() => handleDelete(cred.id)}
                          disabled={deletingId === cred.id}
                          className="p-2 text-gray-500 hover:text-red-400 hover:bg-red-400/10 rounded-lg transition-colors disabled:opacity-50"
                          title="Delete credential"
                        >
                          {deletingId === cred.id ? (
                            <Loader2 className="w-4 h-4 animate-spin" />
                          ) : (
                            <Trash2 className="w-4 h-4" />
                          )}
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Add Credential Modal */}
      {showAddModal && (
        <AddCredentialModal
          integrations={integrations}
          onClose={() => setShowAddModal(false)}
          onComplete={handleAddComplete}
        />
      )}
    </div>
  );
}

interface AddCredentialModalProps {
  integrations: Integration[];
  onClose: () => void;
  onComplete: () => void;
}

function AddCredentialModal({ integrations, onClose, onComplete }: AddCredentialModalProps) {
  const [provider, setProvider] = useState("");
  const [name, setName] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [showKey, setShowKey] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  // Get pending integrations for quick selection
  const pendingIntegrations = integrations.filter((i) => i.status === "pending");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setLoading(true);

    try {
      // Normalize provider for matching (lowercase, no spaces)
      const normalizedProvider = provider.toLowerCase().replace(/\s+/g, "");
      const credential = await addCredential(normalizedProvider, name || `${provider} API Key`, apiKey);

      // Auto-bind to integration if it exists (case-insensitive match)
      const integration = integrations.find(
        (i) => i.key.toLowerCase() === normalizedProvider ||
               i.name.toLowerCase().replace(/\s+/g, "") === normalizedProvider
      );
      if (integration) {
        await bindCredential(integration.key, credential.id);
      }

      onComplete();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to add credential");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center p-4 z-50">
      <div className="bg-surface-elevated rounded-2xl border border-gray-800 w-full max-w-md animate-slideIn">
        {/* Header */}
        <div className="px-6 py-4 border-b border-gray-800">
          <h2 className="text-lg font-semibold text-white">Add Credential</h2>
          <p className="text-sm text-gray-400 mt-1">
            Securely store an API key for your integrations
          </p>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="p-6 space-y-4">
          {/* Quick Select for Pending Integrations */}
          {pendingIntegrations.length > 0 && (
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Quick Select
              </label>
              <div className="flex flex-wrap gap-2">
                {pendingIntegrations.map((int) => (
                  <button
                    key={int.key}
                    type="button"
                    onClick={() => {
                      setProvider(int.key);
                      setName(`${int.name} API Key`);
                    }}
                    className={`px-3 py-1.5 rounded-lg text-sm transition-colors ${
                      provider === int.key
                        ? "bg-accent text-white"
                        : "bg-gray-800 text-gray-400 hover:text-white"
                    }`}
                  >
                    {int.name}
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* Provider */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Provider / Service
            </label>
            <input
              type="text"
              value={provider}
              onChange={(e) => setProvider(e.target.value)}
              placeholder="e.g., openai, stripe, github"
              className="w-full bg-gray-800 border border-gray-700 rounded-xl px-4 py-3 text-white placeholder-gray-500 focus:border-accent focus:ring-1 focus:ring-accent transition-colors"
              required
            />
          </div>

          {/* Name */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Display Name
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g., Production API Key"
              className="w-full bg-gray-800 border border-gray-700 rounded-xl px-4 py-3 text-white placeholder-gray-500 focus:border-accent focus:ring-1 focus:ring-accent transition-colors"
            />
          </div>

          {/* API Key */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              API Key
            </label>
            <div className="relative">
              <input
                type={showKey ? "text" : "password"}
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder="sk-..."
                className="w-full bg-gray-800 border border-gray-700 rounded-xl pl-4 pr-12 py-3 text-white placeholder-gray-500 focus:border-accent focus:ring-1 focus:ring-accent transition-colors font-mono"
                required
              />
              <button
                type="button"
                onClick={() => setShowKey(!showKey)}
                className="absolute right-4 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-300"
              >
                {showKey ? <EyeOff className="w-5 h-5" /> : <Eye className="w-5 h-5" />}
              </button>
            </div>
          </div>

          {/* Error */}
          {error && (
            <div className="bg-red-500/10 border border-red-500/20 rounded-lg px-4 py-3 text-red-400 text-sm">
              {error}
            </div>
          )}

          {/* Security Note */}
          <div className="flex items-start gap-2 text-xs text-gray-500">
            <Shield className="w-4 h-4 flex-shrink-0 mt-0.5" />
            <p>
              Your API key will be encrypted with AES-256-GCM before being stored.
              Only the first 8 characters will be visible for identification.
            </p>
          </div>

          {/* Actions */}
          <div className="flex gap-3 pt-2">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 px-4 py-3 rounded-xl border border-gray-700 text-gray-300 hover:bg-gray-800 transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={loading || !provider || !apiKey}
              className="flex-1 flex items-center justify-center gap-2 px-4 py-3 rounded-xl bg-accent hover:bg-accent-hover text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <>
                  <Key className="w-4 h-4" />
                  Add Credential
                </>
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
