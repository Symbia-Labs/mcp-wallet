import { useEffect, useState, useRef } from "react";
import { Link } from "react-router-dom";
import {
  ArrowLeft,
  Play,
  Pause,
  Trash2,
  Download,
  Circle,
  ArrowUp,
  ArrowDown,
  Server,
  Activity,
} from "lucide-react";
import { getServerStatus } from "../lib/api";

interface LogEntry {
  id: string;
  timestamp: Date;
  direction: "in" | "out";
  type: "request" | "response" | "notification";
  method?: string;
  data: object;
}

// Mock MCP log generator for demo
function generateMockLogEntry(): LogEntry {
  const methods = [
    "initialize",
    "tools/list",
    "tools/call",
    "notifications/initialized",
  ];
  const direction = Math.random() > 0.5 ? "in" : "out";
  const method = methods[Math.floor(Math.random() * methods.length)];
  const isNotification = method.startsWith("notifications/");

  const mockData: Record<string, () => object> = {
    initialize: () =>
      direction === "in"
        ? {
            jsonrpc: "2.0",
            id: Math.floor(Math.random() * 1000),
            method: "initialize",
            params: {
              protocolVersion: "2024-11-05",
              capabilities: { tools: {} },
              clientInfo: { name: "MCP Client", version: "1.0.0" },
            },
          }
        : {
            jsonrpc: "2.0",
            id: Math.floor(Math.random() * 1000),
            result: {
              protocolVersion: "2024-11-05",
              capabilities: { tools: { listChanged: true } },
              serverInfo: { name: "MCP Wallet", version: "0.1.0" },
            },
          },
    "tools/list": () =>
      direction === "in"
        ? {
            jsonrpc: "2.0",
            id: Math.floor(Math.random() * 1000),
            method: "tools/list",
          }
        : {
            jsonrpc: "2.0",
            id: Math.floor(Math.random() * 1000),
            result: {
              tools: [
                { name: "openai_chat_completions_create", description: "..." },
                { name: "stripe_customers_list", description: "..." },
              ],
            },
          },
    "tools/call": () => {
      const tools = [
        "openai_chat_completions_create",
        "stripe_customers_list",
        "github_repos_list",
        "anthropic_messages_create",
      ];
      const tool = tools[Math.floor(Math.random() * tools.length)];
      return direction === "in"
        ? {
            jsonrpc: "2.0",
            id: Math.floor(Math.random() * 1000),
            method: "tools/call",
            params: {
              name: tool,
              arguments: { model: "gpt-4", messages: [] },
            },
          }
        : {
            jsonrpc: "2.0",
            id: Math.floor(Math.random() * 1000),
            result: {
              content: [{ type: "text", text: "Response from " + tool }],
            },
          };
    },
    "notifications/initialized": () => ({
      jsonrpc: "2.0",
      method: "notifications/initialized",
    }),
  };

  return {
    id: crypto.randomUUID(),
    timestamp: new Date(),
    direction,
    type: isNotification ? "notification" : direction === "in" ? "request" : "response",
    method,
    data: mockData[method](),
  };
}

export default function ServerLogsPage() {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [isLive, setIsLive] = useState(true);
  const [serverRunning, setServerRunning] = useState(false);
  const [filter, setFilter] = useState<"all" | "in" | "out">("all");
  const logsEndRef = useRef<HTMLDivElement>(null);
  const intervalRef = useRef<ReturnType<typeof setInterval>>();

  useEffect(() => {
    checkServerStatus();
  }, []);

  useEffect(() => {
    if (isLive && serverRunning) {
      // Simulate incoming MCP messages
      intervalRef.current = setInterval(() => {
        if (Math.random() > 0.3) {
          setLogs((prev) => [...prev.slice(-500), generateMockLogEntry()]);
        }
      }, 800 + Math.random() * 1200);
    }

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, [isLive, serverRunning]);

  useEffect(() => {
    if (isLive) {
      logsEndRef.current?.scrollIntoView({ behavior: "smooth" });
    }
  }, [logs, isLive]);

  const checkServerStatus = async () => {
    const status = await getServerStatus();
    setServerRunning(status.running);
  };

  const filteredLogs =
    filter === "all" ? logs : logs.filter((l) => l.direction === filter);

  const clearLogs = () => setLogs([]);

  const downloadLogs = () => {
    const content = logs
      .map(
        (l) =>
          `${l.timestamp.toISOString()} [${l.direction.toUpperCase()}] ${JSON.stringify(l.data)}`
      )
      .join("\n");
    const blob = new Blob([content], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `mcp-logs-${new Date().toISOString().split("T")[0]}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="h-full flex flex-col animate-fadeIn">
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-4">
          <Link
            to="/"
            className="p-2 rounded-lg text-gray-400 hover:text-white hover:bg-gray-800 transition-colors"
          >
            <ArrowLeft className="w-5 h-5" />
          </Link>
          <div>
            <h1 className="text-2xl font-bold text-white">Server Logs</h1>
            <p className="text-gray-400 text-sm">Live MCP protocol traffic</p>
          </div>
        </div>

        <div className="flex items-center gap-2">
          {/* Server Status */}
          <div
            className={`flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm ${
              serverRunning
                ? "bg-green-500/10 text-green-400"
                : "bg-gray-500/10 text-gray-400"
            }`}
          >
            <Circle
              className={`w-2 h-2 ${serverRunning ? "fill-green-400" : "fill-gray-400"}`}
            />
            {serverRunning ? "Server Running" : "Server Stopped"}
          </div>
        </div>
      </div>

      {/* Controls */}
      <div className="flex items-center justify-between mb-4 bg-surface-elevated rounded-lg border border-gray-800 p-3">
        <div className="flex items-center gap-2">
          {/* Live Toggle */}
          <button
            onClick={() => setIsLive(!isLive)}
            className={`flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm font-medium transition-colors ${
              isLive
                ? "bg-green-500/10 text-green-400"
                : "bg-gray-700 text-gray-300"
            }`}
          >
            {isLive ? (
              <>
                <Pause className="w-4 h-4" />
                Pause
              </>
            ) : (
              <>
                <Play className="w-4 h-4" />
                Resume
              </>
            )}
          </button>

          {/* Filter */}
          <div className="flex items-center bg-gray-800 rounded-lg p-1">
            <button
              onClick={() => setFilter("all")}
              className={`px-3 py-1 rounded text-sm transition-colors ${
                filter === "all"
                  ? "bg-gray-700 text-white"
                  : "text-gray-400 hover:text-white"
              }`}
            >
              All
            </button>
            <button
              onClick={() => setFilter("in")}
              className={`flex items-center gap-1 px-3 py-1 rounded text-sm transition-colors ${
                filter === "in"
                  ? "bg-gray-700 text-blue-400"
                  : "text-gray-400 hover:text-white"
              }`}
            >
              <ArrowDown className="w-3 h-3" />
              In
            </button>
            <button
              onClick={() => setFilter("out")}
              className={`flex items-center gap-1 px-3 py-1 rounded text-sm transition-colors ${
                filter === "out"
                  ? "bg-gray-700 text-green-400"
                  : "text-gray-400 hover:text-white"
              }`}
            >
              <ArrowUp className="w-3 h-3" />
              Out
            </button>
          </div>

          <span className="text-xs text-gray-500 ml-2">
            {filteredLogs.length} messages
          </span>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={clearLogs}
            className="flex items-center gap-1 px-3 py-1.5 rounded-lg text-sm text-gray-400 hover:text-white hover:bg-gray-700 transition-colors"
          >
            <Trash2 className="w-4 h-4" />
            Clear
          </button>
          <button
            onClick={downloadLogs}
            className="flex items-center gap-1 px-3 py-1.5 rounded-lg text-sm text-gray-400 hover:text-white hover:bg-gray-700 transition-colors"
          >
            <Download className="w-4 h-4" />
            Export
          </button>
        </div>
      </div>

      {/* Log View */}
      <div className="flex-1 bg-gray-900 rounded-lg border border-gray-800 overflow-hidden">
        {!serverRunning ? (
          <div className="flex flex-col items-center justify-center h-full text-gray-400">
            <Server className="w-12 h-12 mb-4 text-gray-600" />
            <p className="text-lg font-medium text-gray-300">Server not running</p>
            <p className="text-sm mt-1">Start the MCP server to see live traffic</p>
            <Link
              to="/"
              className="mt-4 px-4 py-2 rounded-lg bg-accent/10 text-accent hover:bg-accent/20 transition-colors"
            >
              Go to Dashboard
            </Link>
          </div>
        ) : filteredLogs.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-gray-400">
            <Activity className="w-12 h-12 mb-4 text-gray-600" />
            <p className="text-lg font-medium text-gray-300">Waiting for traffic...</p>
            <p className="text-sm mt-1">MCP messages will appear here</p>
          </div>
        ) : (
          <div className="h-full overflow-y-auto font-mono text-sm">
            {filteredLogs.map((log) => (
              <LogEntryRow key={log.id} entry={log} />
            ))}
            <div ref={logsEndRef} />
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="mt-4 flex items-center justify-between text-xs text-gray-500">
        <div>
          Protocol: MCP 2024-11-05 • Transport: stdio
        </div>
        <div className="flex items-center gap-2">
          <span className="flex items-center gap-1">
            <ArrowDown className="w-3 h-3 text-blue-400" />
            Incoming
          </span>
          <span className="flex items-center gap-1">
            <ArrowUp className="w-3 h-3 text-green-400" />
            Outgoing
          </span>
        </div>
      </div>
    </div>
  );
}

function LogEntryRow({ entry }: { entry: LogEntry }) {
  const [expanded, setExpanded] = useState(false);

  const getMethodLabel = (method?: string): string => {
    if (!method) return "unknown";
    const parts = method.split("/");
    return parts[parts.length - 1];
  };

  const typeColors: Record<string, string> = {
    request: "text-blue-400",
    response: "text-green-400",
    notification: "text-yellow-400",
  };

  return (
    <div
      className={`border-b border-gray-800 hover:bg-gray-800/50 transition-colors ${
        expanded ? "bg-gray-800/30" : ""
      }`}
    >
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full px-4 py-2 flex items-center gap-3 text-left"
      >
        {/* Direction indicator */}
        <span className={entry.direction === "in" ? "text-blue-400" : "text-green-400"}>
          {entry.direction === "in" ? (
            <ArrowDown className="w-4 h-4" />
          ) : (
            <ArrowUp className="w-4 h-4" />
          )}
        </span>

        {/* Timestamp */}
        <span className="text-gray-500 w-28 flex-shrink-0">
          {entry.timestamp.toLocaleTimeString("en-US", {
            hour12: false,
            hour: "2-digit",
            minute: "2-digit",
            second: "2-digit",
          })}.{String(entry.timestamp.getMilliseconds()).padStart(3, "0")}
        </span>

        {/* Type badge */}
        <span
          className={`px-2 py-0.5 rounded text-xs font-semibold ${typeColors[entry.type]} bg-gray-800`}
        >
          {entry.type}
        </span>

        {/* Method */}
        <span className="text-white font-medium">{getMethodLabel(entry.method)}</span>

        {/* Preview */}
        <span className="text-gray-500 truncate flex-1">
          {JSON.stringify(entry.data).slice(0, 80)}
          {JSON.stringify(entry.data).length > 80 ? "..." : ""}
        </span>
      </button>

      {/* Expanded view */}
      {expanded && (
        <div className="px-4 pb-3 pl-16">
          <pre className="text-xs text-gray-300 bg-gray-900 rounded p-3 overflow-x-auto">
            {JSON.stringify(entry.data, null, 2)}
          </pre>
        </div>
      )}
    </div>
  );
}
