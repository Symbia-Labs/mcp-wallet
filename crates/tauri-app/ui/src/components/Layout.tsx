import { NavLink } from "react-router-dom";
import {
  LayoutDashboard,
  Puzzle,
  Key,
  Settings,
  Lock,
  Wallet,
  Server,
} from "lucide-react";
import { useState, useEffect } from "react";
import { getServerStatus } from "../lib/api";
import { ServerStatus } from "../lib/types";

interface LayoutProps {
  children: React.ReactNode;
  onLock: () => void;
}

const navItems = [
  { to: "/", icon: LayoutDashboard, label: "Dashboard" },
  { to: "/integrations", icon: Puzzle, label: "Integrations" },
  { to: "/credentials", icon: Key, label: "Credentials" },
  { to: "/settings", icon: Settings, label: "Settings" },
];

export default function Layout({ children, onLock }: LayoutProps) {
  const [serverStatus, setServerStatus] = useState<ServerStatus | null>(null);

  useEffect(() => {
    const checkStatus = async () => {
      try {
        const status = await getServerStatus();
        setServerStatus(status);
      } catch (error) {
        console.error("Failed to get server status:", error);
      }
    };
    checkStatus();
    const interval = setInterval(checkStatus, 5000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="min-h-screen bg-surface flex">
      {/* Sidebar */}
      <aside className="w-64 bg-surface-elevated border-r border-gray-800 flex flex-col">
        {/* Logo */}
        <div className="p-6 border-b border-gray-800">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-accent to-purple-600 flex items-center justify-center">
              <Wallet className="w-5 h-5 text-white" />
            </div>
            <div>
              <h1 className="font-bold text-lg text-white">MCP Wallet</h1>
              <p className="text-xs text-gray-500">Secure API Gateway</p>
            </div>
          </div>
        </div>

        {/* Navigation */}
        <nav className="flex-1 p-4">
          <ul className="space-y-1">
            {navItems.map((item) => (
              <li key={item.to}>
                <NavLink
                  to={item.to}
                  className={({ isActive }) =>
                    `flex items-center gap-3 px-4 py-3 rounded-lg transition-colors ${
                      isActive
                        ? "bg-accent/20 text-accent"
                        : "text-gray-400 hover:bg-gray-800 hover:text-white"
                    }`
                  }
                >
                  <item.icon className="w-5 h-5" />
                  <span>{item.label}</span>
                </NavLink>
              </li>
            ))}
          </ul>
        </nav>

        {/* Server Status */}
        <div className="p-4 border-t border-gray-800">
          <div className="flex items-center gap-3 px-4 py-3 rounded-lg bg-gray-800/50">
            <Server className="w-5 h-5 text-gray-400" />
            <div className="flex-1">
              <p className="text-sm text-gray-300">MCP Server</p>
              <p className="text-xs text-gray-500">
                {serverStatus?.running ? (
                  <span className="text-green-400">Running ({serverStatus.mode})</span>
                ) : (
                  <span className="text-gray-500">Stopped</span>
                )}
              </p>
            </div>
            <div
              className={`w-2 h-2 rounded-full ${
                serverStatus?.running ? "bg-green-400" : "bg-gray-600"
              }`}
            />
          </div>
        </div>

        {/* Lock Button */}
        <div className="p-4 border-t border-gray-800">
          <button
            onClick={onLock}
            className="flex items-center gap-3 w-full px-4 py-3 rounded-lg text-gray-400 hover:bg-gray-800 hover:text-white transition-colors"
          >
            <Lock className="w-5 h-5" />
            <span>Lock Wallet</span>
          </button>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 overflow-auto">
        <div className="p-8">{children}</div>
      </main>
    </div>
  );
}
