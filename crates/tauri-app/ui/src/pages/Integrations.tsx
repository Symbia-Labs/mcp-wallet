import { useState, useEffect, useMemo } from "react";
import {
  Search,
  Plus,
  ExternalLink,
  Check,
  Loader2,
  Brain,
  MessageSquare,
  Briefcase,
  Home,
  Wrench,
  Image,
  Users,
  Layers,
  ChevronRight,
  AlertCircle,
  Trash2,
  Key,
  Eye,
  EyeOff,
  Shield,
  RefreshCw,
} from "lucide-react";
import {
  IntegrationDef,
  IntegrationCategory,
  Integration,
  Operation,
  CATEGORY_INFO,
} from "../lib/types";
import {
  INTEGRATION_CATALOG,
  searchIntegrations,
  getIntegrationsByCategory,
} from "../lib/catalog";
import { listIntegrations, addIntegration, removeIntegration, syncIntegration, addCredential, bindCredential, getOperations } from "../lib/api";

const CATEGORY_ICONS: Record<IntegrationCategory, React.ComponentType<{ className?: string }>> = {
  ai_models: Brain,
  chat: MessageSquare,
  productivity: Briefcase,
  smart_home: Home,
  tools: Wrench,
  media: Image,
  social: Users,
  other: Layers,
};

const ALL_CATEGORIES: IntegrationCategory[] = [
  "ai_models",
  "chat",
  "productivity",
  "smart_home",
  "tools",
  "media",
  "social",
];

export default function IntegrationsPage() {
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedCategory, setSelectedCategory] = useState<IntegrationCategory | "all">("all");
  const [installedIntegrations, setInstalledIntegrations] = useState<Integration[]>([]);
  const [loading, setLoading] = useState(true);
  const [removingId, setRemovingId] = useState<string | null>(null);
  const [syncingId, setSyncingId] = useState<string | null>(null);
  const [addModalDef, setAddModalDef] = useState<IntegrationDef | null>(null);
  const [showCustomModal, setShowCustomModal] = useState(false);
  const [opsModalIntegration, setOpsModalIntegration] = useState<Integration | null>(null);

  useEffect(() => {
    loadInstalledIntegrations();
  }, []);

  const loadInstalledIntegrations = async () => {
    try {
      const integrations = await listIntegrations();
      setInstalledIntegrations(integrations);
    } catch (error) {
      console.error("Failed to load integrations:", error);
    } finally {
      setLoading(false);
    }
  };

  const filteredIntegrations = useMemo(() => {
    let results = INTEGRATION_CATALOG;

    if (searchQuery) {
      results = searchIntegrations(searchQuery);
    } else if (selectedCategory !== "all") {
      results = getIntegrationsByCategory(selectedCategory);
    }

    return results;
  }, [searchQuery, selectedCategory]);

  const handleAdd = (def: IntegrationDef) => {
    setAddModalDef(def);
  };

  const handleAddComplete = async () => {
    setAddModalDef(null);
    await loadInstalledIntegrations();
  };

  const handleCustomAddComplete = async () => {
    setShowCustomModal(false);
    await loadInstalledIntegrations();
  };

  const handleRemove = async (key: string) => {
    setRemovingId(key);
    try {
      await removeIntegration(key);
      await loadInstalledIntegrations();
    } catch (error) {
      console.error("Failed to remove integration:", error);
    } finally {
      setRemovingId(null);
    }
  };

  const handleSync = async (key: string) => {
    setSyncingId(key);
    try {
      await syncIntegration(key);
      await loadInstalledIntegrations();
    } catch (error) {
      console.error("Failed to sync integration:", error);
    } finally {
      setSyncingId(null);
    }
  };

  const getInstallStatus = (def: IntegrationDef): Integration | undefined => {
    return installedIntegrations.find((i) => i.key === def.id);
  };

  return (
    <div className="space-y-6 animate-fadeIn">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Integrations</h1>
        <p className="text-gray-400 mt-1">
          Browse and add API integrations to your wallet
        </p>
      </div>

      {/* Search and Filters */}
      <div className="flex flex-col lg:flex-row gap-4">
        {/* Search */}
        <div className="relative flex-1">
          <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-500" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => {
              setSearchQuery(e.target.value);
              if (e.target.value) setSelectedCategory("all");
            }}
            placeholder="Search integrations..."
            className="w-full bg-surface-elevated border border-gray-800 rounded-xl pl-12 pr-4 py-3 text-white placeholder-gray-500 focus:border-accent focus:ring-1 focus:ring-accent transition-colors"
          />
        </div>

        {/* Installed count */}
        <div className="flex items-center gap-2 px-4 py-2 bg-surface-elevated rounded-xl border border-gray-800">
          <Check className="w-4 h-4 text-green-400" />
          <span className="text-sm text-gray-300">
            {installedIntegrations.length} installed
          </span>
        </div>

        {/* Add Custom Button */}
        <button
          onClick={() => setShowCustomModal(true)}
          className="flex items-center gap-2 px-4 py-2 bg-accent hover:bg-accent-hover text-white rounded-xl transition-colors"
        >
          <Plus className="w-4 h-4" />
          <span className="text-sm font-medium">Add Custom</span>
        </button>
      </div>

      {/* Category Pills */}
      <div className="flex flex-wrap gap-2">
        <button
          onClick={() => {
            setSelectedCategory("all");
            setSearchQuery("");
          }}
          className={`px-4 py-2 rounded-full text-sm font-medium transition-colors ${
            selectedCategory === "all" && !searchQuery
              ? "bg-accent text-white"
              : "bg-surface-elevated text-gray-400 hover:text-white border border-gray-800"
          }`}
        >
          All ({INTEGRATION_CATALOG.length})
        </button>
        {ALL_CATEGORIES.map((cat) => {
          const info = CATEGORY_INFO[cat];
          const count = getIntegrationsByCategory(cat).length;
          const Icon = CATEGORY_ICONS[cat];
          return (
            <button
              key={cat}
              onClick={() => {
                setSelectedCategory(cat);
                setSearchQuery("");
              }}
              className={`flex items-center gap-2 px-4 py-2 rounded-full text-sm font-medium transition-colors ${
                selectedCategory === cat && !searchQuery
                  ? "bg-accent text-white"
                  : "bg-surface-elevated text-gray-400 hover:text-white border border-gray-800"
              }`}
            >
              <Icon className="w-4 h-4" />
              {info.label} ({count})
            </button>
          );
        })}
      </div>

      {/* Integration Grid */}
      {loading ? (
        <div className="flex items-center justify-center h-64">
          <div className="w-8 h-8 border-4 border-accent border-t-transparent rounded-full animate-spin" />
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {filteredIntegrations.map((def) => {
            const installed = getInstallStatus(def);
            const isRemoving = removingId === def.id;
            const isSyncing = syncingId === def.id;
            const info = CATEGORY_INFO[def.category];
            const Icon = CATEGORY_ICONS[def.category];

            return (
              <div
                key={def.id}
                className={`bg-surface-elevated rounded-xl border p-5 transition-all ${
                  installed
                    ? "border-green-500/30 bg-green-500/5"
                    : "border-gray-800 hover:border-gray-700"
                }`}
              >
                {/* Header */}
                <div className="flex items-start justify-between mb-4">
                  <div className="flex items-center gap-3">
                    <div
                      className={`w-10 h-10 rounded-lg flex items-center justify-center ${
                        installed ? "bg-green-500/10" : "bg-gray-800"
                      }`}
                    >
                      <IntegrationIcon name={def.icon} className="w-5 h-5" />
                    </div>
                    <div>
                      <h3 className="font-semibold text-white">{def.name}</h3>
                      <div className="flex items-center gap-1 text-xs">
                        <Icon className={`w-3 h-3 ${info.color}`} />
                        <span className="text-gray-500">{info.label}</span>
                      </div>
                    </div>
                  </div>

                  {/* Status Badge */}
                  {installed && (
                    <div
                      className={`px-2 py-1 rounded-full text-xs font-medium ${
                        installed.status === "active"
                          ? "bg-green-500/10 text-green-400"
                          : installed.status === "error"
                          ? "bg-red-500/10 text-red-400"
                          : "bg-yellow-500/10 text-yellow-400"
                      }`}
                    >
                      {installed.status === "active" ? (
                        <span className="flex items-center gap-1">
                          <Check className="w-3 h-3" />
                          Active
                        </span>
                      ) : installed.status === "error" ? (
                        <span className="flex items-center gap-1">
                          <AlertCircle className="w-3 h-3" />
                          Error
                        </span>
                      ) : (
                        "Pending"
                      )}
                    </div>
                  )}
                </div>

                {/* Description */}
                <p className="text-sm text-gray-400 mb-4 line-clamp-2">
                  {def.description}
                </p>

                {/* Tags */}
                {def.tags && def.tags.length > 0 && (
                  <div className="flex flex-wrap gap-1 mb-4">
                    {def.tags.slice(0, 3).map((tag) => (
                      <span
                        key={tag}
                        className="px-2 py-0.5 bg-gray-800 rounded text-xs text-gray-400"
                      >
                        {tag}
                      </span>
                    ))}
                  </div>
                )}

                {/* Actions */}
                <div className="flex items-center gap-2">
                  {installed ? (
                    <>
                      {/* Sync button - re-fetches OpenAPI spec */}
                      <button
                        onClick={() => handleSync(installed.key)}
                        disabled={isSyncing || isRemoving}
                        className="flex items-center justify-center gap-2 px-3 py-2 rounded-lg bg-blue-500/10 text-blue-400 hover:bg-blue-500/20 transition-colors disabled:opacity-50"
                        title="Sync integration (re-fetch OpenAPI spec)"
                      >
                        {isSyncing ? (
                          <Loader2 className="w-4 h-4 animate-spin" />
                        ) : (
                          <RefreshCw className="w-4 h-4" />
                        )}
                      </button>
                      <button
                        onClick={() => handleRemove(installed.key)}
                        disabled={isRemoving || isSyncing}
                        className="flex-1 flex items-center justify-center gap-2 px-4 py-2 rounded-lg bg-red-500/10 text-red-400 hover:bg-red-500/20 transition-colors disabled:opacity-50"
                      >
                        {isRemoving ? (
                          <Loader2 className="w-4 h-4 animate-spin" />
                        ) : (
                          <>
                            <Trash2 className="w-4 h-4" />
                            Remove
                          </>
                        )}
                      </button>
                      {installed.status === "pending" && (
                        <span className="text-xs text-yellow-400">
                          Needs credentials
                        </span>
                      )}
                    </>
                  ) : (
                    <button
                      onClick={() => handleAdd(def)}
                      className="flex-1 flex items-center justify-center gap-2 px-4 py-2 rounded-lg bg-accent/10 text-accent hover:bg-accent/20 transition-colors"
                    >
                      <Plus className="w-4 h-4" />
                      Add
                    </button>
                  )}

                  {def.docsUrl && (
                    <a
                      href={def.docsUrl}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="p-2 rounded-lg text-gray-500 hover:text-white hover:bg-gray-800 transition-colors"
                      title="View documentation"
                    >
                      <ExternalLink className="w-4 h-4" />
                    </a>
                  )}
                </div>

                {/* Operation count if installed */}
                {installed && installed.status === "active" && (
                  <button
                    onClick={() => setOpsModalIntegration(installed)}
                    className="mt-3 pt-3 border-t border-gray-800 flex items-center justify-between text-xs text-gray-500 hover:text-accent w-full transition-colors"
                  >
                    <span>{installed.operationCount} MCP tools available</span>
                    <ChevronRight className="w-4 h-4" />
                  </button>
                )}
              </div>
            );
          })}
        </div>
      )}

      {/* Empty State */}
      {!loading && filteredIntegrations.length === 0 && (
        <div className="text-center py-12">
          <Search className="w-12 h-12 text-gray-600 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-white mb-2">
            No integrations found
          </h3>
          <p className="text-gray-400">
            Try a different search term or category
          </p>
        </div>
      )}

      {/* Add Integration Modal */}
      {addModalDef && (
        <AddIntegrationModal
          def={addModalDef}
          onClose={() => setAddModalDef(null)}
          onComplete={handleAddComplete}
        />
      )}

      {/* Add Custom Integration Modal */}
      {showCustomModal && (
        <AddCustomIntegrationModal
          onClose={() => setShowCustomModal(false)}
          onComplete={handleCustomAddComplete}
        />
      )}

      {/* Operations Modal */}
      {opsModalIntegration && (
        <OperationsModal
          integration={opsModalIntegration}
          onClose={() => setOpsModalIntegration(null)}
        />
      )}
    </div>
  );
}

// Modal for adding an integration with API key
interface AddIntegrationModalProps {
  def: IntegrationDef;
  onClose: () => void;
  onComplete: () => void;
}

function AddIntegrationModal({ def, onClose, onComplete }: AddIntegrationModalProps) {
  const [apiKey, setApiKey] = useState("");
  const [specUrl, setSpecUrl] = useState(def.specUrl || "");
  const [showKey, setShowKey] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");

    if (!specUrl.trim()) {
      setError("OpenAPI spec URL is required to add this integration.");
      return;
    }

    setLoading(true);

    try {
      // First add the integration (fetches and parses OpenAPI spec)
      await addIntegration(def.id, specUrl.trim());

      // Then add and bind the credential
      const credential = await addCredential(def.id, `${def.name} API Key`, apiKey);
      await bindCredential(def.id, credential.id);

      onComplete();
    } catch (err: unknown) {
      // Handle both Error objects and string errors from Tauri
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center p-4 z-50">
      <div className="bg-surface-elevated rounded-2xl border border-gray-800 w-full max-w-md animate-slideIn">
        {/* Header */}
        <div className="px-6 py-4 border-b border-gray-800">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-lg bg-gray-800 flex items-center justify-center">
              <IntegrationIcon name={def.icon} className="w-5 h-5" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-white">Add {def.name}</h2>
              <p className="text-sm text-gray-400">{def.description}</p>
            </div>
          </div>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="p-6 space-y-4">
          {/* API Key Input */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              API Key
            </label>
            <div className="relative">
              <Key className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-500" />
              <input
                type={showKey ? "text" : "password"}
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder={`Enter your ${def.name} API key`}
                className="w-full bg-gray-800 border border-gray-700 rounded-xl pl-12 pr-12 py-3 text-white placeholder-gray-500 focus:border-accent focus:ring-1 focus:ring-accent transition-colors font-mono"
                required
                autoFocus
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

          {/* OpenAPI Spec URL */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              OpenAPI Spec URL
            </label>
            <div className="relative">
              <ExternalLink className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-500" />
              <input
                type="url"
                value={specUrl}
                onChange={(e) => setSpecUrl(e.target.value)}
                placeholder="https://api.example.com/openapi.yaml"
                className="w-full bg-gray-800 border border-gray-700 rounded-xl pl-12 pr-4 py-3 text-white placeholder-gray-500 focus:border-accent focus:ring-1 focus:ring-accent transition-colors font-mono text-sm"
                required
              />
            </div>
            <p className="text-xs text-gray-500 mt-1">
              Enter the URL to the OpenAPI/Swagger spec (YAML or JSON)
            </p>
          </div>

          {/* Docs Link */}
          {def.docsUrl && (
            <a
              href={def.docsUrl}
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-2 text-sm text-accent hover:text-accent-hover transition-colors"
            >
              <ExternalLink className="w-4 h-4" />
              Get your API key from {def.name}
            </a>
          )}

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
              Your API key will be encrypted with AES-256-GCM and stored securely.
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
              disabled={loading || !apiKey || !specUrl.trim()}
              className="flex-1 flex items-center justify-center gap-2 px-4 py-3 rounded-xl bg-accent hover:bg-accent-hover text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <>
                  <Plus className="w-4 h-4" />
                  Add Integration
                </>
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

// Modal for adding a custom integration (not from catalog)
interface AddCustomIntegrationModalProps {
  onClose: () => void;
  onComplete: () => void;
}

function AddCustomIntegrationModal({ onClose, onComplete }: AddCustomIntegrationModalProps) {
  const [integrationKey, setIntegrationKey] = useState("");
  const [integrationName, setIntegrationName] = useState("");
  const [specUrl, setSpecUrl] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [showKey, setShowKey] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  // Auto-generate key from name
  const handleNameChange = (name: string) => {
    setIntegrationName(name);
    // Only auto-generate if user hasn't manually edited the key
    if (!integrationKey || integrationKey === integrationName.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/(^-|-$)/g, "")) {
      const newKey = name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/(^-|-$)/g, "");
      setIntegrationKey(newKey);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");

    if (!integrationKey.trim()) {
      setError("Integration key is required.");
      return;
    }

    if (!specUrl.trim()) {
      setError("OpenAPI spec URL is required.");
      return;
    }

    setLoading(true);

    try {
      // Add the integration (fetches and parses OpenAPI spec)
      await addIntegration(integrationKey.trim(), specUrl.trim());

      // If API key provided, add and bind the credential
      if (apiKey.trim()) {
        const credential = await addCredential(
          integrationKey.trim(),
          `${integrationName || integrationKey} API Key`,
          apiKey
        );
        await bindCredential(integrationKey.trim(), credential.id);
      }

      onComplete();
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center p-4 z-50">
      <div className="bg-surface-elevated rounded-2xl border border-gray-800 w-full max-w-md animate-slideIn">
        {/* Header */}
        <div className="px-6 py-4 border-b border-gray-800">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-lg bg-accent/10 flex items-center justify-center">
              <Plus className="w-5 h-5 text-accent" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-white">Add Custom Integration</h2>
              <p className="text-sm text-gray-400">Connect any OpenAPI-compatible API</p>
            </div>
          </div>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="p-6 space-y-4">
          {/* Integration Name */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Integration Name
            </label>
            <input
              type="text"
              value={integrationName}
              onChange={(e) => handleNameChange(e.target.value)}
              placeholder="My Custom API"
              className="w-full bg-gray-800 border border-gray-700 rounded-xl px-4 py-3 text-white placeholder-gray-500 focus:border-accent focus:ring-1 focus:ring-accent transition-colors"
              autoFocus
            />
          </div>

          {/* Integration Key */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Integration Key
            </label>
            <input
              type="text"
              value={integrationKey}
              onChange={(e) => setIntegrationKey(e.target.value.toLowerCase().replace(/[^a-z0-9-]/g, ""))}
              placeholder="my-custom-api"
              className="w-full bg-gray-800 border border-gray-700 rounded-xl px-4 py-3 text-white placeholder-gray-500 focus:border-accent focus:ring-1 focus:ring-accent transition-colors font-mono"
              required
            />
            <p className="text-xs text-gray-500 mt-1">
              Unique identifier (lowercase, hyphens only). Used for MCP tool names.
            </p>
          </div>

          {/* OpenAPI Spec URL */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              OpenAPI Spec URL
            </label>
            <div className="relative">
              <ExternalLink className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-500" />
              <input
                type="url"
                value={specUrl}
                onChange={(e) => setSpecUrl(e.target.value)}
                placeholder="https://api.example.com/openapi.yaml"
                className="w-full bg-gray-800 border border-gray-700 rounded-xl pl-12 pr-4 py-3 text-white placeholder-gray-500 focus:border-accent focus:ring-1 focus:ring-accent transition-colors font-mono text-sm"
                required
              />
            </div>
            <p className="text-xs text-gray-500 mt-1">
              URL to the OpenAPI/Swagger specification (YAML or JSON)
            </p>
          </div>

          {/* API Key Input (Optional) */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              API Key <span className="text-gray-500 font-normal">(optional)</span>
            </label>
            <div className="relative">
              <Key className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-500" />
              <input
                type={showKey ? "text" : "password"}
                value={apiKey}
                onChange={(e) => setApiKey(e.target.value)}
                placeholder="Enter your API key"
                className="w-full bg-gray-800 border border-gray-700 rounded-xl pl-12 pr-12 py-3 text-white placeholder-gray-500 focus:border-accent focus:ring-1 focus:ring-accent transition-colors font-mono"
              />
              <button
                type="button"
                onClick={() => setShowKey(!showKey)}
                className="absolute right-4 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-300"
              >
                {showKey ? <EyeOff className="w-5 h-5" /> : <Eye className="w-5 h-5" />}
              </button>
            </div>
            <p className="text-xs text-gray-500 mt-1">
              You can add credentials later from the Credentials page
            </p>
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
              Your API key will be encrypted with AES-256-GCM and stored securely.
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
              disabled={loading || !integrationKey || !specUrl.trim()}
              className="flex-1 flex items-center justify-center gap-2 px-4 py-3 rounded-xl bg-accent hover:bg-accent-hover text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <>
                  <Plus className="w-4 h-4" />
                  Add Integration
                </>
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

// Simple icon component that maps integration names to placeholder icons
function IntegrationIcon({ name, className }: { name: string; className?: string }) {
  // In a real app, you'd have actual brand icons here
  // For now, use colored circles or the first letter
  const colors: Record<string, string> = {
    openai: "text-green-400",
    anthropic: "text-orange-400",
    google: "text-blue-400",
    mistral: "text-purple-400",
    groq: "text-red-400",
    slack: "text-purple-400",
    discord: "text-indigo-400",
    github: "text-white",
    stripe: "text-purple-400",
    notion: "text-white",
    linear: "text-indigo-400",
    spotify: "text-green-400",
    default: "text-gray-400",
  };

  const color = colors[name] || colors.default;

  return (
    <span className={`font-bold text-lg ${color} ${className}`}>
      {name.charAt(0).toUpperCase()}
    </span>
  );
}

// Modal for viewing available operations
interface OperationsModalProps {
  integration: Integration;
  onClose: () => void;
}

function OperationsModal({ integration, onClose }: OperationsModalProps) {
  const [operations, setOperations] = useState<Operation[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState("");
  const [expandedOp, setExpandedOp] = useState<string | null>(null);

  useEffect(() => {
    loadOperations();
  }, [integration.key]);

  const loadOperations = async () => {
    try {
      const ops = await getOperations(integration.key);
      setOperations(ops);
    } catch (error) {
      console.error("Failed to load operations:", error);
    } finally {
      setLoading(false);
    }
  };

  const filteredOps = useMemo(() => {
    if (!searchQuery) return operations;
    const query = searchQuery.toLowerCase();
    return operations.filter(
      (op) =>
        op.name.toLowerCase().includes(query) ||
        op.description.toLowerCase().includes(query) ||
        op.path.toLowerCase().includes(query)
    );
  }, [operations, searchQuery]);

  const methodColors: Record<string, string> = {
    GET: "bg-green-500/10 text-green-400 border-green-500/30",
    POST: "bg-blue-500/10 text-blue-400 border-blue-500/30",
    PUT: "bg-yellow-500/10 text-yellow-400 border-yellow-500/30",
    DELETE: "bg-red-500/10 text-red-400 border-red-500/30",
    PATCH: "bg-purple-500/10 text-purple-400 border-purple-500/30",
  };

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center p-4 z-50">
      <div className="bg-surface-elevated rounded-2xl border border-gray-800 w-full max-w-2xl max-h-[80vh] flex flex-col animate-slideIn">
        {/* Header */}
        <div className="px-6 py-4 border-b border-gray-800 flex-shrink-0">
          <div className="flex items-center justify-between mb-3">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-lg bg-green-500/10 flex items-center justify-center">
                <Wrench className="w-5 h-5 text-green-400" />
              </div>
              <div>
                <h2 className="text-lg font-semibold text-white">
                  {integration.name} Operations
                </h2>
                <p className="text-sm text-gray-400">
                  {operations.length} MCP tools available
                </p>
              </div>
            </div>
            <button
              onClick={onClose}
              className="p-2 rounded-lg text-gray-500 hover:text-white hover:bg-gray-800 transition-colors"
            >
              <span className="sr-only">Close</span>
              ×
            </button>
          </div>

          {/* Search */}
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search operations..."
              className="w-full bg-gray-800 border border-gray-700 rounded-lg pl-10 pr-4 py-2 text-sm text-white placeholder-gray-500 focus:border-accent focus:ring-1 focus:ring-accent transition-colors"
            />
          </div>
        </div>

        {/* Operations List */}
        <div className="flex-1 overflow-y-auto p-4 space-y-2">
          {loading ? (
            <div className="flex items-center justify-center h-32">
              <Loader2 className="w-6 h-6 text-accent animate-spin" />
            </div>
          ) : filteredOps.length === 0 ? (
            <div className="text-center py-8 text-gray-400">
              {searchQuery ? "No operations match your search" : "No operations available"}
            </div>
          ) : (
            filteredOps.map((op) => {
              const isExpanded = expandedOp === op.id;
              return (
                <div
                  key={op.id}
                  className="bg-gray-800/50 rounded-lg border border-gray-700 overflow-hidden"
                >
                  {/* Operation Header */}
                  <button
                    onClick={() => setExpandedOp(isExpanded ? null : op.id)}
                    className="w-full px-4 py-3 flex items-center gap-3 text-left hover:bg-gray-700/30 transition-colors"
                  >
                    <span
                      className={`px-2 py-0.5 rounded text-xs font-mono font-semibold border ${
                        methodColors[op.method] || methodColors.GET
                      }`}
                    >
                      {op.method}
                    </span>
                    <div className="flex-1 min-w-0">
                      <div className="font-medium text-white truncate">
                        {op.name}
                      </div>
                      <div className="text-xs text-gray-500 font-mono truncate">
                        {op.path}
                      </div>
                    </div>
                    <ChevronRight
                      className={`w-4 h-4 text-gray-500 transition-transform ${
                        isExpanded ? "rotate-90" : ""
                      }`}
                    />
                  </button>

                  {/* Expanded Details */}
                  {isExpanded && (
                    <div className="px-4 pb-4 border-t border-gray-700 pt-3 space-y-3">
                      {/* Description */}
                      <p className="text-sm text-gray-300">{op.description}</p>

                      {/* Parameters */}
                      {op.parameters.length > 0 && (
                        <div>
                          <h4 className="text-xs font-semibold text-gray-400 uppercase tracking-wide mb-2">
                            Parameters
                          </h4>
                          <div className="space-y-2">
                            {op.parameters.map((param) => (
                              <div
                                key={param.name}
                                className="bg-gray-900/50 rounded px-3 py-2"
                              >
                                <div className="flex items-center gap-2 mb-1">
                                  <code className="text-sm text-accent font-mono">
                                    {param.name}
                                  </code>
                                  <span className="text-xs text-gray-500">
                                    {param.type}
                                  </span>
                                  <span
                                    className={`text-xs px-1.5 py-0.5 rounded ${
                                      param.required
                                        ? "bg-red-500/10 text-red-400"
                                        : "bg-gray-700 text-gray-400"
                                    }`}
                                  >
                                    {param.required ? "required" : "optional"}
                                  </span>
                                  <span className="text-xs text-gray-600">
                                    ({param.location})
                                  </span>
                                </div>
                                {param.description && (
                                  <p className="text-xs text-gray-400">
                                    {param.description}
                                  </p>
                                )}
                              </div>
                            ))}
                          </div>
                        </div>
                      )}

                      {op.parameters.length === 0 && (
                        <p className="text-xs text-gray-500 italic">
                          No parameters required
                        </p>
                      )}
                    </div>
                  )}
                </div>
              );
            })
          )}
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-gray-800 flex-shrink-0">
          <button
            onClick={onClose}
            className="w-full px-4 py-2 rounded-lg border border-gray-700 text-gray-300 hover:bg-gray-800 transition-colors"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
