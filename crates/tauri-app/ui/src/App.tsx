import { Routes, Route, Navigate } from "react-router-dom";
import { useState, useEffect } from "react";
import Layout from "./components/Layout";
import UnlockPage from "./pages/Unlock";
import DashboardPage from "./pages/Dashboard";
import IntegrationsPage from "./pages/Integrations";
import CredentialsPage from "./pages/Credentials";
import SettingsPage from "./pages/Settings";
import ServerLogsPage from "./pages/ServerLogs";
import { WalletState } from "./lib/types";
import { getWalletState } from "./lib/api";

function App() {
  const [walletState, setWalletState] = useState<WalletState>("loading");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    checkWalletState();
  }, []);

  const checkWalletState = async () => {
    try {
      const state = await getWalletState();
      setWalletState(state);
    } catch (error) {
      console.error("Failed to get wallet state:", error);
      setWalletState("not_initialized");
    } finally {
      setLoading(false);
    }
  };

  const handleUnlock = () => {
    setWalletState("unlocked");
  };

  const handleLock = () => {
    setWalletState("locked");
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-surface flex items-center justify-center">
        <div className="text-center">
          <div className="w-12 h-12 border-4 border-accent border-t-transparent rounded-full animate-spin mx-auto mb-4" />
          <p className="text-gray-400">Loading wallet...</p>
        </div>
      </div>
    );
  }

  // Show unlock screen if wallet is locked or not initialized
  if (walletState !== "unlocked") {
    return (
      <UnlockPage
        state={walletState}
        onUnlock={handleUnlock}
        onInitialize={() => setWalletState("locked")}
      />
    );
  }

  return (
    <Layout onLock={handleLock}>
      <Routes>
        <Route path="/" element={<DashboardPage />} />
        <Route path="/integrations" element={<IntegrationsPage />} />
        <Route path="/credentials" element={<CredentialsPage />} />
        <Route path="/settings" element={<SettingsPage />} />
        <Route path="/logs" element={<ServerLogsPage />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </Layout>
  );
}

export default App;
