import NavBar from "@/components/NavBar";
import WalletCard from "@/components/WalletCard";
import { useAuth } from "@/context/AuthContext";
import { useRevStop } from "@/context/RevStopContext";
import { useRouter } from "next/router";
import { useEffect } from "react";

export default function Wallet() {
  const { user, isLoading } = useAuth();
  const { active, enable, disable, isLoading: revStopLoading } = useRevStop();
  const router = useRouter();

  // Redirect to login if not authenticated
  useEffect(() => {
    if (!isLoading && !user) {
      router.push("/login");
    }
  }, [user, isLoading, router]);

  if (isLoading || !user) {
    return (
      <div className="min-h-screen bg-quantum-dark flex items-center justify-center">
        <div className="text-cyan-300">Loading...</div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-quantum-dark text-cyan-100">
      <NavBar />
      
      <main className="pt-20 pb-16">
        <div className="mx-auto max-w-4xl px-4 py-8">
          {/* Page Header */}
          <div className="mb-8">
            <h1 className="text-3xl font-bold text-cyan-300 mb-2">💰 Your Quantum Wallet</h1>
            <p className="opacity-75">
              Manage your quantum-resistant QuantumCoin™ address and enable RevStop™ protection
            </p>
          </div>

          {/* RevStop Status Bar */}
          <div className="quantum-card mb-6">
            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
              <div className="flex items-center gap-3">
                <span className="text-cyan-300 font-semibold">🛡️ RevStop™ Status:</span>
                <span className={`px-3 py-1 rounded font-medium ${
                  active 
                    ? "bg-green-700/50 text-green-200 border border-green-600/50" 
                    : "bg-red-800/50 text-red-200 border border-red-600/50"
                }`}>
                  {revStopLoading ? "Loading..." : (active ? "🟢 ACTIVE" : "🔴 DISABLED")}
                </span>
              </div>
              
              <div className="flex gap-2">
                {active ? (
                  <button 
                    onClick={disable}
                    className="px-4 py-2 rounded bg-red-600 text-white hover:bg-red-700 transition-colors"
                    disabled={revStopLoading}
                  >
                    Disable RevStop™
                  </button>
                ) : (
                  <button 
                    onClick={enable}
                    className="quantum-button-primary"
                    disabled={revStopLoading}
                  >
                    🔒 Enable RevStop™
                  </button>
                )}
              </div>
            </div>
            
            <div className="mt-3 text-sm opacity-75">
              {active 
                ? "🛡️ Maximum protection enabled. All transactions require additional verification."
                : "⚠️ Standard protection. Enable RevStop™ for enhanced transaction security."
              }
            </div>
          </div>

          {/* Wallet Card */}
          <WalletCard />

          {/* RevStop Information */}
          <div className="mt-8 quantum-card">
            <h3 className="text-lg font-semibold text-cyan-300 mb-3">🔒 About RevStop™</h3>
            <div className="space-y-3 text-sm opacity-80">
              <p>
                <strong className="text-cyan-300">RevStop™</strong> is QuantumCoin's revolutionary 
                transaction protection system. When enabled, it provides an additional layer of 
                security for your wallet.
              </p>
              <p>
                <strong>Features:</strong>
              </p>
              <ul className="list-disc list-inside pl-4 space-y-1">
                <li>Optional irreversible wallet lock capability</li>
                <li>Enhanced transaction verification</li>
                <li>Protection against unauthorized access</li>
                <li>Ideal for long-term holders and high-value wallets</li>
              </ul>
              <p className="text-amber-300">
                <strong>⚠️ Warning:</strong> Once RevStop™ is permanently activated on a wallet, 
                it cannot be reversed. Use the toggle above for testing - permanent activation 
                is a separate, irreversible process.
              </p>
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}
