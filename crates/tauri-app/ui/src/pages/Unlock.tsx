import { useState } from "react";
import { Wallet, Eye, EyeOff, Shield, Lock } from "lucide-react";
import { WalletState } from "../lib/types";
import { initializeWallet, unlockWallet } from "../lib/api";

interface UnlockPageProps {
  state: WalletState;
  onUnlock: () => void;
  onInitialize: () => void;
}

export default function UnlockPage({ state, onUnlock, onInitialize }: UnlockPageProps) {
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  const isInitializing = state === "not_initialized";

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setLoading(true);

    try {
      if (isInitializing) {
        if (password !== confirmPassword) {
          setError("Passwords do not match");
          setLoading(false);
          return;
        }
        if (password.length < 8) {
          setError("Password must be at least 8 characters");
          setLoading(false);
          return;
        }
        await initializeWallet(password);
        onInitialize();
      }

      await unlockWallet(password);
      onUnlock();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to unlock wallet");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-surface flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        {/* Logo */}
        <div className="text-center mb-8">
          <div className="w-20 h-20 rounded-2xl bg-gradient-to-br from-accent to-purple-600 flex items-center justify-center mx-auto mb-6 shadow-lg shadow-accent/20">
            <Wallet className="w-10 h-10 text-white" />
          </div>
          <h1 className="text-3xl font-bold text-white mb-2">MCP Wallet</h1>
          <p className="text-gray-400">
            {isInitializing
              ? "Create a secure wallet to manage your API integrations"
              : "Enter your password to unlock"}
          </p>
        </div>

        {/* Form Card */}
        <div className="bg-surface-elevated rounded-2xl border border-gray-800 p-8">
          <form onSubmit={handleSubmit} className="space-y-6">
            {/* Password Field */}
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-2">
                {isInitializing ? "Create Password" : "Password"}
              </label>
              <div className="relative">
                <Lock className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-500" />
                <input
                  type={showPassword ? "text" : "password"}
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="Enter your password"
                  className="w-full bg-gray-800 border border-gray-700 rounded-xl pl-12 pr-12 py-3 text-white placeholder-gray-500 focus:border-accent focus:ring-1 focus:ring-accent transition-colors"
                  required
                  autoFocus
                />
                <button
                  type="button"
                  onClick={() => setShowPassword(!showPassword)}
                  className="absolute right-4 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-300"
                >
                  {showPassword ? (
                    <EyeOff className="w-5 h-5" />
                  ) : (
                    <Eye className="w-5 h-5" />
                  )}
                </button>
              </div>
            </div>

            {/* Confirm Password (only when initializing) */}
            {isInitializing && (
              <div>
                <label className="block text-sm font-medium text-gray-300 mb-2">
                  Confirm Password
                </label>
                <div className="relative">
                  <Lock className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-500" />
                  <input
                    type={showPassword ? "text" : "password"}
                    value={confirmPassword}
                    onChange={(e) => setConfirmPassword(e.target.value)}
                    placeholder="Confirm your password"
                    className="w-full bg-gray-800 border border-gray-700 rounded-xl pl-12 pr-12 py-3 text-white placeholder-gray-500 focus:border-accent focus:ring-1 focus:ring-accent transition-colors"
                    required
                  />
                </div>
              </div>
            )}

            {/* Error Message */}
            {error && (
              <div className="bg-red-500/10 border border-red-500/20 rounded-lg px-4 py-3 text-red-400 text-sm">
                {error}
              </div>
            )}

            {/* Submit Button */}
            <button
              type="submit"
              disabled={loading}
              className="w-full bg-accent hover:bg-accent-hover text-white font-medium py-3 rounded-xl transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
            >
              {loading ? (
                <div className="w-5 h-5 border-2 border-white/20 border-t-white rounded-full animate-spin" />
              ) : (
                <>
                  <Shield className="w-5 h-5" />
                  {isInitializing ? "Create Wallet" : "Unlock Wallet"}
                </>
              )}
            </button>
          </form>

          {/* Security Note */}
          {isInitializing && (
            <div className="mt-6 pt-6 border-t border-gray-800">
              <div className="flex gap-3 text-sm text-gray-400">
                <Shield className="w-5 h-5 text-accent flex-shrink-0" />
                <p>
                  Your password is used to derive an encryption key using Argon2id.
                  API keys are encrypted with AES-256-GCM and stored securely.
                </p>
              </div>
            </div>
          )}
        </div>

        {/* Branding */}
        <p className="text-center text-xs text-gray-700 mt-6">
          Brought to you by the team at{" "}
          <a
            href="https://symbia.io"
            target="_blank"
            rel="noopener noreferrer"
            className="text-gray-600 hover:text-accent transition-colors"
          >
            Symbia Labs
          </a>
        </p>
      </div>
    </div>
  );
}
