import { useEffect, useState, useMemo } from "react";
import { Link } from "react-router-dom";
import {
  Puzzle,
  Key,
  Server,
  Play,
  Square,
  RefreshCw,
  ExternalLink,
  Activity,
  Shield,
  Zap,
  Search,
  ChevronRight,
  Loader2,
  Wrench,
} from "lucide-react";
import { Integration, Credential, ServerStatus, Operation } from "../lib/types";
import {
  listIntegrations,
  listCredentials,
  getServerStatus,
  startServer,
  stopServer,
  getOperations,
} from "../lib/api";

export default function DashboardPage() {
  const [integrations, setIntegrations] = useState<Integration[]>([]);
  const [credentials, setCredentials] = useState<Credential[]>([]);
  const [serverStatus, setServerStatus] = useState<ServerStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [serverLoading, setServerLoading] = useState(false);
  const [showOpsModal, setShowOpsModal] = useState(false);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      const [ints, creds, status] = await Promise.all([
        listIntegrations(),
        listCredentials(),
        getServerStatus(),
      ]);
      setIntegrations(ints);
      setCredentials(creds);
      setServerStatus(status);
    } catch (error) {
      console.error("Failed to load dashboard data:", error);
    } finally {
      setLoading(false);
    }
  };

  const handleServerToggle = async () => {
    if (!serverStatus) return;
    setServerLoading(true);
    try {
      if (serverStatus.running) {
        await stopServer();
      } else {
        await startServer("stdio");
      }
      const status = await getServerStatus();
      setServerStatus(status);
    } catch (error) {
      console.error("Failed to toggle server:", error);
    } finally {
      setServerLoading(false);
    }
  };

  const activeIntegrations = integrations.filter((i) => i.status === "active");
  const totalOperations = integrations.reduce((sum, i) => sum + i.operationCount, 0);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="w-8 h-8 border-4 border-accent border-t-transparent rounded-full animate-spin" />
      </div>
    );
  }

  return (
    <div className="space-y-8 animate-fadeIn">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Dashboard</h1>
        <p className="text-gray-400 mt-1">
          Monitor your MCP wallet and server status
        </p>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          icon={Puzzle}
          label="Integrations"
          value={integrations.length}
          subtext={`${activeIntegrations.length} active`}
          color="text-purple-400"
          bgColor="bg-purple-400/10"
          to="/integrations"
        />
        <StatCard
          icon={Key}
          label="Credentials"
          value={credentials.length}
          subtext="API keys stored"
          color="text-blue-400"
          bgColor="bg-blue-400/10"
          to="/credentials"
        />
        <StatCard
          icon={Zap}
          label="MCP Tools"
          value={totalOperations}
          subtext="Available operations"
          color="text-yellow-400"
          bgColor="bg-yellow-400/10"
          onClick={() => setShowOpsModal(true)}
        />
        <StatCard
          icon={Activity}
          label="Server Status"
          value={serverStatus?.running ? "Running" : "Stopped"}
          subtext={serverStatus?.running ? serverStatus.mode : "Not started"}
          color={serverStatus?.running ? "text-green-400" : "text-gray-400"}
          bgColor={serverStatus?.running ? "bg-green-400/10" : "bg-gray-400/10"}
          to="/logs"
        />
      </div>

      {/* Server Control */}
      <div className="bg-surface-elevated rounded-xl border border-gray-800 p-6">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <div className="w-12 h-12 rounded-xl bg-accent/10 flex items-center justify-center">
              <Server className="w-6 h-6 text-accent" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-white">MCP Server</h2>
              <p className="text-sm text-gray-400">
                {serverStatus?.running
                  ? `Running in ${serverStatus.mode} mode`
                  : "Start the server to expose integrations via MCP"}
              </p>
            </div>
          </div>
          <div className="flex items-center gap-3">
            <button
              onClick={loadData}
              className="p-2 rounded-lg text-gray-400 hover:bg-gray-800 hover:text-white transition-colors"
              title="Refresh"
            >
              <RefreshCw className="w-5 h-5" />
            </button>
            <button
              onClick={handleServerToggle}
              disabled={serverLoading}
              className={`flex items-center gap-2 px-4 py-2 rounded-lg font-medium transition-colors ${
                serverStatus?.running
                  ? "bg-red-500/10 text-red-400 hover:bg-red-500/20"
                  : "bg-green-500/10 text-green-400 hover:bg-green-500/20"
              } disabled:opacity-50`}
            >
              {serverLoading ? (
                <div className="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin" />
              ) : serverStatus?.running ? (
                <>
                  <Square className="w-4 h-4" />
                  Stop
                </>
              ) : (
                <>
                  <Play className="w-4 h-4" />
                  Start
                </>
              )}
            </button>
          </div>
        </div>

        {/* MCP Client Config Link */}
        {serverStatus?.running && (
          <div className="mt-6 pt-6 border-t border-gray-700">
            <Link
              to="/settings"
              className="flex items-center gap-2 text-sm text-accent hover:text-accent-hover transition-colors"
            >
              <ExternalLink className="w-4 h-4" />
              View MCP client configuration
            </Link>
          </div>
        )}
      </div>

      {/* Quick Actions */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Link
          to="/integrations"
          className="bg-surface-elevated rounded-xl border border-gray-800 p-6 hover:border-accent/50 transition-colors group"
        >
          <div className="flex items-center gap-4">
            <div className="w-12 h-12 rounded-xl bg-purple-400/10 flex items-center justify-center">
              <Puzzle className="w-6 h-6 text-purple-400" />
            </div>
            <div className="flex-1">
              <h3 className="font-semibold text-white group-hover:text-accent transition-colors">
                Add Integration
              </h3>
              <p className="text-sm text-gray-400">
                Browse and add OpenAPI integrations
              </p>
            </div>
            <ExternalLink className="w-5 h-5 text-gray-600 group-hover:text-accent transition-colors" />
          </div>
        </Link>

        <Link
          to="/credentials"
          className="bg-surface-elevated rounded-xl border border-gray-800 p-6 hover:border-accent/50 transition-colors group"
        >
          <div className="flex items-center gap-4">
            <div className="w-12 h-12 rounded-xl bg-blue-400/10 flex items-center justify-center">
              <Key className="w-6 h-6 text-blue-400" />
            </div>
            <div className="flex-1">
              <h3 className="font-semibold text-white group-hover:text-accent transition-colors">
                Manage Credentials
              </h3>
              <p className="text-sm text-gray-400">
                Add and manage API keys securely
              </p>
            </div>
            <ExternalLink className="w-5 h-5 text-gray-600 group-hover:text-accent transition-colors" />
          </div>
        </Link>
      </div>

      {/* Security Info */}
      <div className="bg-accent/5 rounded-xl border border-accent/20 p-6">
        <div className="flex gap-4">
          <Shield className="w-6 h-6 text-accent flex-shrink-0" />
          <div>
            <h3 className="font-semibold text-white mb-2">Security Features</h3>
            <ul className="text-sm text-gray-400 space-y-1">
              <li>AES-256-GCM encryption for all stored credentials</li>
              <li>Argon2id key derivation from your master password</li>
              <li>OS Keychain integration when available (macOS, Windows, Linux)</li>
              <li>Credentials never leave your device unencrypted</li>
            </ul>
          </div>
        </div>
      </div>

      {/* All Operations Modal */}
      {showOpsModal && (
        <AllOperationsModal
          integrations={integrations}
          onClose={() => setShowOpsModal(false)}
        />
      )}
    </div>
  );
}

interface StatCardProps {
  icon: React.ComponentType<{ className?: string }>;
  label: string;
  value: string | number;
  subtext: string;
  color: string;
  bgColor: string;
  to?: string;
  onClick?: () => void;
}

function StatCard({ icon: Icon, label, value, subtext, color, bgColor, to, onClick }: StatCardProps) {
  const content = (
    <div className="bg-surface-elevated rounded-xl border border-gray-800 p-5 hover:border-gray-700 transition-colors cursor-pointer">
      <div className="flex items-start justify-between">
        <div className={`w-10 h-10 rounded-lg ${bgColor} flex items-center justify-center`}>
          <Icon className={`w-5 h-5 ${color}`} />
        </div>
      </div>
      <div className="mt-4">
        <p className="text-2xl font-bold text-white">{value}</p>
        <p className="text-sm text-gray-500">{label}</p>
        <p className={`text-xs mt-1 ${color}`}>{subtext}</p>
      </div>
    </div>
  );

  if (to) {
    return <Link to={to}>{content}</Link>;
  }
  if (onClick) {
    return <button onClick={onClick} className="text-left w-full">{content}</button>;
  }
  return content;
}

// Modal showing all operations across all integrations
interface AllOperationsModalProps {
  integrations: Integration[];
  onClose: () => void;
}

interface OperationWithIntegration extends Operation {
  integrationKey: string;
  integrationName: string;
}

function AllOperationsModal({ integrations, onClose }: AllOperationsModalProps) {
  const [operations, setOperations] = useState<OperationWithIntegration[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState("");
  const [expandedOp, setExpandedOp] = useState<string | null>(null);

  useEffect(() => {
    loadAllOperations();
  }, [integrations]);

  const loadAllOperations = async () => {
    try {
      const allOps: OperationWithIntegration[] = [];
      for (const integration of integrations.filter(i => i.status === "active")) {
        const ops = await getOperations(integration.key);
        ops.forEach(op => {
          allOps.push({
            ...op,
            integrationKey: integration.key,
            integrationName: integration.name,
          });
        });
      }
      setOperations(allOps);
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
        op.path.toLowerCase().includes(query) ||
        op.integrationName.toLowerCase().includes(query)
    );
  }, [operations, searchQuery]);

  const groupedOps = useMemo(() => {
    const groups: Record<string, OperationWithIntegration[]> = {};
    filteredOps.forEach(op => {
      if (!groups[op.integrationKey]) {
        groups[op.integrationKey] = [];
      }
      groups[op.integrationKey].push(op);
    });
    return groups;
  }, [filteredOps]);

  const methodColors: Record<string, string> = {
    GET: "bg-green-500/10 text-green-400 border-green-500/30",
    POST: "bg-blue-500/10 text-blue-400 border-blue-500/30",
    PUT: "bg-yellow-500/10 text-yellow-400 border-yellow-500/30",
    DELETE: "bg-red-500/10 text-red-400 border-red-500/30",
    PATCH: "bg-purple-500/10 text-purple-400 border-purple-500/30",
  };

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center p-4 z-50">
      <div className="bg-surface-elevated rounded-2xl border border-gray-800 w-full max-w-3xl max-h-[85vh] flex flex-col animate-slideIn">
        {/* Header */}
        <div className="px-6 py-4 border-b border-gray-800 flex-shrink-0">
          <div className="flex items-center justify-between mb-3">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-lg bg-yellow-500/10 flex items-center justify-center">
                <Zap className="w-5 h-5 text-yellow-400" />
              </div>
              <div>
                <h2 className="text-lg font-semibold text-white">All MCP Tools</h2>
                <p className="text-sm text-gray-400">
                  {operations.length} tools across {Object.keys(groupedOps).length} integrations
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
              placeholder="Search all operations..."
              className="w-full bg-gray-800 border border-gray-700 rounded-lg pl-10 pr-4 py-2 text-sm text-white placeholder-gray-500 focus:border-accent focus:ring-1 focus:ring-accent transition-colors"
            />
          </div>
        </div>

        {/* Operations List */}
        <div className="flex-1 overflow-y-auto p-4 space-y-4">
          {loading ? (
            <div className="flex items-center justify-center h-32">
              <Loader2 className="w-6 h-6 text-accent animate-spin" />
            </div>
          ) : Object.keys(groupedOps).length === 0 ? (
            <div className="text-center py-8 text-gray-400">
              {searchQuery ? "No operations match your search" : "No active integrations with operations"}
            </div>
          ) : (
            Object.entries(groupedOps).map(([key, ops]) => (
              <div key={key} className="space-y-2">
                <h3 className="text-sm font-semibold text-gray-300 flex items-center gap-2">
                  <Wrench className="w-4 h-4 text-gray-500" />
                  {ops[0].integrationName}
                  <span className="text-xs text-gray-500 font-normal">({ops.length} tools)</span>
                </h3>
                {ops.map((op) => {
                  const isExpanded = expandedOp === `${key}-${op.id}`;
                  return (
                    <div
                      key={`${key}-${op.id}`}
                      className="bg-gray-800/50 rounded-lg border border-gray-700 overflow-hidden"
                    >
                      <button
                        onClick={() => setExpandedOp(isExpanded ? null : `${key}-${op.id}`)}
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
                          <div className="font-medium text-white truncate">{op.name}</div>
                          <div className="text-xs text-gray-500 font-mono truncate">{op.path}</div>
                        </div>
                        <ChevronRight
                          className={`w-4 h-4 text-gray-500 transition-transform ${
                            isExpanded ? "rotate-90" : ""
                          }`}
                        />
                      </button>

                      {isExpanded && (
                        <div className="px-4 pb-4 border-t border-gray-700 pt-3 space-y-3">
                          <p className="text-sm text-gray-300">{op.description}</p>
                          {op.parameters.length > 0 && (
                            <div>
                              <h4 className="text-xs font-semibold text-gray-400 uppercase tracking-wide mb-2">
                                Parameters
                              </h4>
                              <div className="space-y-2">
                                {op.parameters.map((param) => (
                                  <div key={param.name} className="bg-gray-900/50 rounded px-3 py-2">
                                    <div className="flex items-center gap-2 mb-1">
                                      <code className="text-sm text-accent font-mono">{param.name}</code>
                                      <span className="text-xs text-gray-500">{param.type}</span>
                                      <span
                                        className={`text-xs px-1.5 py-0.5 rounded ${
                                          param.required
                                            ? "bg-red-500/10 text-red-400"
                                            : "bg-gray-700 text-gray-400"
                                        }`}
                                      >
                                        {param.required ? "required" : "optional"}
                                      </span>
                                      <span className="text-xs text-gray-600">({param.location})</span>
                                    </div>
                                    {param.description && (
                                      <p className="text-xs text-gray-400">{param.description}</p>
                                    )}
                                  </div>
                                ))}
                              </div>
                            </div>
                          )}
                          {op.parameters.length === 0 && (
                            <p className="text-xs text-gray-500 italic">No parameters required</p>
                          )}
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            ))
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
