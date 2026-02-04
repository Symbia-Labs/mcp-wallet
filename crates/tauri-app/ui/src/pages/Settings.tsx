import { useState, useEffect } from "react";
import {
  Server,
  Shield,
  Key,
  RefreshCw,
  Download,
  Upload,
  Trash2,
  AlertTriangle,
  Check,
  Copy,
  Terminal,
  Activity,
  Eye,
  EyeOff,
  Loader2,
  Save,
} from "lucide-react";
import { ServerStatus } from "../lib/types";
import {
  getServerStatus,
  startServer,
  stopServer,
  getExecutablePath,
  getOtelSettings,
  updateOtelSettings,
  OtelSettings,
  getAutoLockTimeout,
  setAutoLockTimeout,
  resetWallet,
} from "../lib/api";

export default function SettingsPage() {
  const [serverStatus, setServerStatus] = useState<ServerStatus | null>(null);
  const [serverMode, setServerMode] = useState<"stdio" | "http">("stdio");
  const [httpPort, setHttpPort] = useState(3000);
  const [loading, setLoading] = useState(false);
  const [copied, setCopied] = useState(false);
  const [copiedPath, setCopiedPath] = useState(false);
  const [executablePath, setExecutablePath] = useState<string>("/path/to/symbia-mcp-wallet");

  // OTEL settings state
  const [otelSettings, setOtelSettings] = useState<OtelSettings>({
    enabled: false,
    endpoint: null,
    serviceName: null,
    authHeader: null,
    exportTraces: true,
    exportMetrics: true,
  });
  const [showAuthHeader, setShowAuthHeader] = useState(false);
  const [otelSaving, setOtelSaving] = useState(false);
  const [otelSaved, setOtelSaved] = useState(false);
  const [autoLockTimeout, setAutoLockTimeoutState] = useState(15);
  const [showResetConfirm, setShowResetConfirm] = useState(false);
  const [resetting, setResetting] = useState(false);

  useEffect(() => {
    loadServerStatus();
    loadExecutablePath();
    loadOtelSettings();
    loadAutoLockTimeout();
  }, []);

  const loadServerStatus = async () => {
    try {
      const status = await getServerStatus();
      setServerStatus(status);
      if (status.mode) setServerMode(status.mode);
      if (status.port) setHttpPort(status.port);
    } catch (error) {
      console.error("Failed to get server status:", error);
    }
  };

  const loadExecutablePath = async () => {
    try {
      const path = await getExecutablePath();
      setExecutablePath(path);
    } catch (error) {
      console.error("Failed to get executable path:", error);
    }
  };

  const loadOtelSettings = async () => {
    try {
      const settings = await getOtelSettings();
      setOtelSettings(settings);
    } catch (error) {
      console.error("Failed to load OTEL settings:", error);
    }
  };

  const loadAutoLockTimeout = async () => {
    try {
      const timeout = await getAutoLockTimeout();
      setAutoLockTimeoutState(timeout);
    } catch (error) {
      console.error("Failed to load auto-lock timeout:", error);
    }
  };

  const handleOtelSave = async () => {
    setOtelSaving(true);
    try {
      await updateOtelSettings(otelSettings);
      setOtelSaved(true);
      setTimeout(() => setOtelSaved(false), 2000);
    } catch (error) {
      console.error("Failed to save OTEL settings:", error);
    } finally {
      setOtelSaving(false);
    }
  };

  const handleAutoLockChange = async (minutes: number) => {
    setAutoLockTimeoutState(minutes);
    try {
      await setAutoLockTimeout(minutes);
    } catch (error) {
      console.error("Failed to save auto-lock timeout:", error);
    }
  };

  const handleServerToggle = async () => {
    setLoading(true);
    try {
      if (serverStatus?.running) {
        await stopServer();
      } else {
        await startServer(serverMode, serverMode === "http" ? httpPort : undefined);
      }
      await loadServerStatus();
    } catch (error) {
      console.error("Failed to toggle server:", error);
    } finally {
      setLoading(false);
    }
  };

  const copyConfig = () => {
    const config = serverMode === "stdio"
      ? JSON.stringify({
          mcpServers: {
            "symbia-mcp-wallet": {
              command: executablePath,
              args: ["--stdio"]
            }
          }
        }, null, 2)
      : JSON.stringify({
          mcpServers: {
            "symbia-mcp-wallet": {
              url: `http://localhost:${httpPort}/sse`
            }
          }
        }, null, 2);
    navigator.clipboard.writeText(config);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const copyPath = () => {
    navigator.clipboard.writeText(executablePath);
    setCopiedPath(true);
    setTimeout(() => setCopiedPath(false), 2000);
  };

  const handleReset = async () => {
    setResetting(true);
    try {
      await resetWallet();
      // Reload the page to show the "not initialized" state
      window.location.reload();
    } catch (error) {
      console.error("Failed to reset wallet:", error);
      setResetting(false);
      setShowResetConfirm(false);
    }
  };

  return (
    <div className="space-y-6 animate-fadeIn max-w-3xl">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Settings</h1>
        <p className="text-gray-400 mt-1">
          Configure your MCP Wallet server and security options
        </p>
      </div>

      {/* Server Settings */}
      <section className="bg-surface-elevated rounded-xl border border-gray-800 overflow-hidden">
        <div className="px-6 py-4 border-b border-gray-800 bg-gray-800/30">
          <div className="flex items-center gap-3">
            <Server className="w-5 h-5 text-accent" />
            <h2 className="font-semibold text-white">MCP Server</h2>
          </div>
        </div>

        <div className="p-6 space-y-6">
          {/* Server Mode */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-3">
              Transport Mode
            </label>
            <div className="grid grid-cols-2 gap-3">
              <button
                onClick={() => setServerMode("stdio")}
                disabled={serverStatus?.running}
                className={`p-4 rounded-xl border-2 transition-colors text-left ${
                  serverMode === "stdio"
                    ? "border-accent bg-accent/10"
                    : "border-gray-700 hover:border-gray-600"
                } disabled:opacity-50`}
              >
                <p className="font-medium text-white">stdio</p>
                <p className="text-sm text-gray-400 mt-1">
                  Direct process communication
                </p>
              </button>
              <button
                onClick={() => setServerMode("http")}
                disabled={serverStatus?.running}
                className={`p-4 rounded-xl border-2 transition-colors text-left ${
                  serverMode === "http"
                    ? "border-accent bg-accent/10"
                    : "border-gray-700 hover:border-gray-600"
                } disabled:opacity-50`}
              >
                <p className="font-medium text-white">HTTP/SSE</p>
                <p className="text-sm text-gray-400 mt-1">
                  Network access with bearer auth
                </p>
              </button>
            </div>
          </div>

          {/* HTTP Port */}
          {serverMode === "http" && (
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                HTTP Port
              </label>
              <input
                type="number"
                value={httpPort}
                onChange={(e) => setHttpPort(parseInt(e.target.value) || 3000)}
                disabled={serverStatus?.running}
                className="w-32 bg-gray-800 border border-gray-700 rounded-xl px-4 py-2 text-white disabled:opacity-50"
              />
            </div>
          )}

          {/* Server Status & Control */}
          <div className="flex items-center justify-between pt-4 border-t border-gray-700">
            <div>
              <p className="text-sm text-gray-400">Server Status</p>
              <p className={`font-medium ${serverStatus?.running ? "text-green-400" : "text-gray-500"}`}>
                {serverStatus?.running ? `Running (${serverStatus.mode})` : "Stopped"}
              </p>
            </div>
            <button
              onClick={handleServerToggle}
              disabled={loading}
              className={`flex items-center gap-2 px-4 py-2 rounded-lg font-medium transition-colors ${
                serverStatus?.running
                  ? "bg-red-500/10 text-red-400 hover:bg-red-500/20"
                  : "bg-green-500/10 text-green-400 hover:bg-green-500/20"
              } disabled:opacity-50`}
            >
              {loading ? (
                <RefreshCw className="w-4 h-4 animate-spin" />
              ) : serverStatus?.running ? (
                "Stop Server"
              ) : (
                "Start Server"
              )}
            </button>
          </div>
        </div>
      </section>

      {/* MCP Client Configuration */}
      <section className="bg-surface-elevated rounded-xl border border-gray-800 overflow-hidden">
        <div className="px-6 py-4 border-b border-gray-800 bg-gray-800/30">
          <div className="flex items-center gap-3">
            <Terminal className="w-5 h-5 text-accent" />
            <h2 className="font-semibold text-white">MCP Client Configuration</h2>
          </div>
        </div>

        <div className="p-6 space-y-4">
          {/* Executable Path */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Server Executable Path
            </label>
            <div className="relative">
              <input
                type="text"
                value={executablePath}
                readOnly
                className="w-full bg-gray-900 border border-gray-700 rounded-lg px-4 py-3 pr-12 font-mono text-sm text-gray-300"
              />
              <button
                onClick={copyPath}
                className="absolute right-3 top-1/2 -translate-y-1/2 p-1.5 rounded bg-gray-800 text-gray-400 hover:text-white transition-colors"
              >
                {copiedPath ? <Check className="w-4 h-4 text-green-400" /> : <Copy className="w-4 h-4" />}
              </button>
            </div>
            <p className="text-xs text-gray-500 mt-1">
              Use this path when configuring MCP clients
            </p>
          </div>

          {/* Config Example */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">
              Example Configuration (JSON)
            </label>
            <div className="relative">
              <pre className="bg-gray-900 rounded-lg p-4 font-mono text-sm text-gray-300 overflow-x-auto">
{serverMode === "stdio"
  ? JSON.stringify({
      mcpServers: {
        "symbia-mcp-wallet": {
          command: executablePath,
          args: ["--stdio"]
        }
      }
    }, null, 2)
  : JSON.stringify({
      mcpServers: {
        "symbia-mcp-wallet": {
          url: `http://localhost:${httpPort}/sse`
        }
      }
    }, null, 2)
}
              </pre>
              <button
                onClick={copyConfig}
                className="absolute top-3 right-3 p-2 rounded-lg bg-gray-800 text-gray-400 hover:text-white transition-colors"
              >
                {copied ? <Check className="w-4 h-4 text-green-400" /> : <Copy className="w-4 h-4" />}
              </button>
            </div>
            {serverMode === "stdio" && (
              <div className="mt-3 p-3 bg-green-500/10 border border-green-500/30 rounded-lg">
                <p className="text-xs text-green-400">
                  <strong>No password needed!</strong> The CLI automatically uses a secure session token
                  created when you unlock the wallet in this app. Sessions last 24 hours.
                </p>
              </div>
            )}
            {serverMode === "http" && (
              <div className="mt-3 p-3 bg-blue-500/10 border border-blue-500/30 rounded-lg">
                <p className="text-xs text-blue-400">
                  <strong>Note:</strong> HTTP mode requires starting the server from this app first.
                  The wallet stays unlocked while the server is running.
                </p>
              </div>
            )}
            <p className="text-xs text-gray-500 mt-3">
              Copy this configuration to your MCP client's settings file
            </p>
          </div>
        </div>
      </section>

      {/* OpenTelemetry Settings */}
      <section className="bg-surface-elevated rounded-xl border border-gray-800 overflow-hidden">
        <div className="px-6 py-4 border-b border-gray-800 bg-gray-800/30">
          <div className="flex items-center gap-3">
            <Activity className="w-5 h-5 text-accent" />
            <h2 className="font-semibold text-white">Observability</h2>
          </div>
        </div>

        <div className="p-6 space-y-4">
          {/* Enable Toggle */}
          <div className="flex items-center justify-between py-3">
            <div>
              <p className="font-medium text-white">Enable OpenTelemetry</p>
              <p className="text-sm text-gray-400">Export traces and metrics to OTLP endpoint</p>
            </div>
            <button
              onClick={() => setOtelSettings({ ...otelSettings, enabled: !otelSettings.enabled })}
              className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                otelSettings.enabled ? "bg-accent" : "bg-gray-700"
              }`}
            >
              <span
                className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                  otelSettings.enabled ? "translate-x-6" : "translate-x-1"
                }`}
              />
            </button>
          </div>

          {otelSettings.enabled && (
            <>
              {/* OTLP Endpoint */}
              <div className="pt-3 border-t border-gray-700">
                <label className="block text-sm font-medium text-gray-300 mb-2">
                  OTLP Endpoint
                </label>
                <input
                  type="url"
                  value={otelSettings.endpoint || ""}
                  onChange={(e) => setOtelSettings({ ...otelSettings, endpoint: e.target.value || null })}
                  placeholder="http://localhost:4317 or https://otel.example.com:4317"
                  className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 text-white placeholder-gray-500 font-mono text-sm focus:border-accent focus:ring-1 focus:ring-accent"
                />
                <p className="text-xs text-gray-500 mt-1">
                  The gRPC or HTTP endpoint for your OpenTelemetry collector
                </p>
              </div>

              {/* Service Name */}
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-2">
                  Service Name
                </label>
                <input
                  type="text"
                  value={otelSettings.serviceName || ""}
                  onChange={(e) => setOtelSettings({ ...otelSettings, serviceName: e.target.value || null })}
                  placeholder="symbia-mcp-wallet"
                  className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 text-white placeholder-gray-500 font-mono text-sm focus:border-accent focus:ring-1 focus:ring-accent"
                />
                <p className="text-xs text-gray-500 mt-1">
                  Service name for traces and metrics (defaults to symbia-mcp-wallet)
                </p>
              </div>

              {/* Auth Header */}
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-2">
                  Authorization Header <span className="text-gray-500 font-normal">(optional)</span>
                </label>
                <div className="relative">
                  <input
                    type={showAuthHeader ? "text" : "password"}
                    value={otelSettings.authHeader || ""}
                    onChange={(e) => setOtelSettings({ ...otelSettings, authHeader: e.target.value || null })}
                    placeholder="Bearer your-token or Api-Key your-key"
                    className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 pr-12 text-white placeholder-gray-500 font-mono text-sm focus:border-accent focus:ring-1 focus:ring-accent"
                  />
                  <button
                    type="button"
                    onClick={() => setShowAuthHeader(!showAuthHeader)}
                    className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-300"
                  >
                    {showAuthHeader ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
                  </button>
                </div>
                <p className="text-xs text-gray-500 mt-1">
                  For cloud OTEL providers (Honeycomb, Grafana Cloud, etc.)
                </p>
              </div>

              {/* Export Options */}
              <div className="grid grid-cols-2 gap-4">
                <div className="flex items-center gap-3">
                  <input
                    type="checkbox"
                    id="export-traces"
                    checked={otelSettings.exportTraces}
                    onChange={(e) => setOtelSettings({ ...otelSettings, exportTraces: e.target.checked })}
                    className="w-4 h-4 rounded border-gray-600 bg-gray-800 text-accent focus:ring-accent"
                  />
                  <label htmlFor="export-traces" className="text-sm text-gray-300">
                    Export Traces
                  </label>
                </div>
                <div className="flex items-center gap-3">
                  <input
                    type="checkbox"
                    id="export-metrics"
                    checked={otelSettings.exportMetrics}
                    onChange={(e) => setOtelSettings({ ...otelSettings, exportMetrics: e.target.checked })}
                    className="w-4 h-4 rounded border-gray-600 bg-gray-800 text-accent focus:ring-accent"
                  />
                  <label htmlFor="export-metrics" className="text-sm text-gray-300">
                    Export Metrics
                  </label>
                </div>
              </div>

              {/* Save Button */}
              <div className="pt-3 border-t border-gray-700">
                <button
                  onClick={handleOtelSave}
                  disabled={otelSaving}
                  className="flex items-center gap-2 px-4 py-2 rounded-lg bg-accent hover:bg-accent-hover text-white transition-colors disabled:opacity-50"
                >
                  {otelSaving ? (
                    <Loader2 className="w-4 h-4 animate-spin" />
                  ) : otelSaved ? (
                    <>
                      <Check className="w-4 h-4" />
                      Saved
                    </>
                  ) : (
                    <>
                      <Save className="w-4 h-4" />
                      Save Settings
                    </>
                  )}
                </button>
              </div>
            </>
          )}

          {!otelSettings.enabled && (
            <button
              onClick={handleOtelSave}
              disabled={otelSaving}
              className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gray-800 hover:bg-gray-700 text-gray-300 transition-colors disabled:opacity-50"
            >
              {otelSaving ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : otelSaved ? (
                <>
                  <Check className="w-4 h-4 text-green-400" />
                  Saved
                </>
              ) : (
                <>
                  <Save className="w-4 h-4" />
                  Save Settings
                </>
              )}
            </button>
          )}
        </div>
      </section>

      {/* Security Settings */}
      <section className="bg-surface-elevated rounded-xl border border-gray-800 overflow-hidden">
        <div className="px-6 py-4 border-b border-gray-800 bg-gray-800/30">
          <div className="flex items-center gap-3">
            <Shield className="w-5 h-5 text-accent" />
            <h2 className="font-semibold text-white">Security</h2>
          </div>
        </div>

        <div className="p-6 space-y-4">
          <div className="flex items-center justify-between py-3">
            <div>
              <p className="font-medium text-white">Auto-lock Timeout</p>
              <p className="text-sm text-gray-400">Lock wallet after inactivity</p>
            </div>
            <select
              value={autoLockTimeout}
              onChange={(e) => handleAutoLockChange(parseInt(e.target.value))}
              className="bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-white"
            >
              <option value="5">5 minutes</option>
              <option value="15">15 minutes</option>
              <option value="30">30 minutes</option>
              <option value="60">1 hour</option>
              <option value="0">Never</option>
            </select>
          </div>

          <div className="flex items-center justify-between py-3 border-t border-gray-700">
            <div>
              <p className="font-medium text-white">Change Master Password</p>
              <p className="text-sm text-gray-400">Update your wallet encryption key</p>
            </div>
            <button className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gray-800 text-gray-300 hover:bg-gray-700 transition-colors">
              <Key className="w-4 h-4" />
              Change
            </button>
          </div>

          <div className="flex items-center justify-between py-3 border-t border-gray-700">
            <div>
              <p className="font-medium text-white">Export Backup</p>
              <p className="text-sm text-gray-400">Download encrypted wallet backup</p>
            </div>
            <button className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gray-800 text-gray-300 hover:bg-gray-700 transition-colors">
              <Download className="w-4 h-4" />
              Export
            </button>
          </div>

          <div className="flex items-center justify-between py-3 border-t border-gray-700">
            <div>
              <p className="font-medium text-white">Import Backup</p>
              <p className="text-sm text-gray-400">Restore from encrypted backup file</p>
            </div>
            <button className="flex items-center gap-2 px-4 py-2 rounded-lg bg-gray-800 text-gray-300 hover:bg-gray-700 transition-colors">
              <Upload className="w-4 h-4" />
              Import
            </button>
          </div>
        </div>
      </section>

      {/* Danger Zone */}
      <section className="bg-surface-elevated rounded-xl border border-red-500/30 overflow-hidden">
        <div className="px-6 py-4 border-b border-red-500/30 bg-red-500/5">
          <div className="flex items-center gap-3">
            <AlertTriangle className="w-5 h-5 text-red-400" />
            <h2 className="font-semibold text-red-400">Danger Zone</h2>
          </div>
        </div>

        <div className="p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="font-medium text-white">Reset Wallet</p>
              <p className="text-sm text-gray-400">
                Delete all integrations, credentials, and settings
              </p>
            </div>
            <button
              onClick={() => setShowResetConfirm(true)}
              className="flex items-center gap-2 px-4 py-2 rounded-lg bg-red-500/10 text-red-400 hover:bg-red-500/20 transition-colors"
            >
              <Trash2 className="w-4 h-4" />
              Reset
            </button>
          </div>
        </div>
      </section>

      {/* Version Info */}
      <div className="text-center text-sm text-gray-600">
        <p>Symbia Labs MCP Wallet v0.1.6-beta</p>
        <p className="mt-1 text-xs text-gray-700 font-mono break-all px-4">
          Server: {executablePath}
        </p>
        <p className="mt-2">
          Built with Tauri + React + Rust
        </p>
        <p className="mt-3 text-gray-700">
          Brought to you by the team at{" "}
          <a
            href="https://symbia.io"
            target="_blank"
            rel="noopener noreferrer"
            className="text-gray-500 hover:text-accent transition-colors"
          >
            Symbia Labs
          </a>
        </p>
      </div>

      {/* Reset Confirmation Modal */}
      {showResetConfirm && (
        <div className="fixed inset-0 bg-black/60 flex items-center justify-center p-4 z-50">
          <div className="bg-surface-elevated rounded-2xl border border-red-500/30 w-full max-w-md animate-slideIn">
            <div className="px-6 py-4 border-b border-red-500/30 bg-red-500/5">
              <div className="flex items-center gap-3">
                <AlertTriangle className="w-6 h-6 text-red-400" />
                <h2 className="text-lg font-semibold text-red-400">Confirm Reset</h2>
              </div>
            </div>

            <div className="p-6 space-y-4">
              <p className="text-gray-300">
                Are you sure you want to reset the wallet? This will permanently delete:
              </p>
              <ul className="list-disc list-inside text-sm text-gray-400 space-y-1">
                <li>All integrations and their configurations</li>
                <li>All stored API credentials</li>
                <li>All settings and preferences</li>
              </ul>
              <p className="text-red-400 font-medium">
                This action cannot be undone!
              </p>

              <div className="flex gap-3 pt-4">
                <button
                  onClick={() => setShowResetConfirm(false)}
                  disabled={resetting}
                  className="flex-1 px-4 py-3 rounded-xl border border-gray-700 text-gray-300 hover:bg-gray-800 transition-colors disabled:opacity-50"
                >
                  Cancel
                </button>
                <button
                  onClick={handleReset}
                  disabled={resetting}
                  className="flex-1 flex items-center justify-center gap-2 px-4 py-3 rounded-xl bg-red-500 hover:bg-red-600 text-white transition-colors disabled:opacity-50"
                >
                  {resetting ? (
                    <Loader2 className="w-4 h-4 animate-spin" />
                  ) : (
                    <>
                      <Trash2 className="w-4 h-4" />
                      Reset Wallet
                    </>
                  )}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
