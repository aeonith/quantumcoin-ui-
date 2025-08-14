import NavBar from "@/components/NavBar";
import { useAuth } from "@/context/AuthContext";
import { useEffect, useState } from "react";
import { useRouter } from "next/router";

interface ExchangeStatus {
  float: number;
  enabled: boolean;
}

interface VerificationResult {
  ok: boolean;
  btc?: number;
  usd?: number;
  estimatedQtc?: number;
  error?: string;
}

const BTC_ADDRESS = process.env.NEXT_PUBLIC_BTC_ADDRESS || "bc1qv7tpdxqvgwutfrhf53nhwgp77j5lv7whnk433y";

export default function Exchange() {
  const { user, isLoading } = useAuth();
  const router = useRouter();
  const [status, setStatus] = useState<ExchangeStatus>({ float: 0, enabled: false });
  const [qtcAddress, setQtcAddress] = useState<string>("");
  const [btcTxid, setBtcTxid] = useState<string>("");
  const [message, setMessage] = useState<string>("");
  const [isVerifying, setIsVerifying] = useState(false);

  // Redirect to login if not authenticated
  useEffect(() => {
    if (!isLoading && !user) {
      router.push("/login");
    }
  }, [user, isLoading, router]);

  // Load exchange status
  useEffect(() => {
    fetch("/api/exchange-status")
      .then(r => r.json())
      .then(setStatus)
      .catch(() => setStatus({ float: 0, enabled: false }));
  }, []);

  // Load saved QTC address from wallet
  useEffect(() => {
    const savedAddress = localStorage.getItem("qc_wallet_addr") || "";
    setQtcAddress(savedAddress);
  }, []);

  const verifyAndCredit = async () => {
    if (!qtcAddress.trim()) {
      setMessage("❌ Please enter your QTC address");
      return;
    }

    if (!btcTxid.trim()) {
      setMessage("❌ Please enter your Bitcoin transaction ID");
      return;
    }

    if (!status.enabled || status.float <= 0) {
      setMessage("❌ No QTC available on exchange. Supply must be mined first.");
      return;
    }

    setIsVerifying(true);
    setMessage("🔍 Verifying Bitcoin transaction on-chain...");

    try {
      // Verify BTC transaction
      const verifyResponse = await fetch(`/api/verify-btc?txid=${encodeURIComponent(btcTxid)}`);
      const verification: VerificationResult = await verifyResponse.json();

      if (!verification.ok) {
        setMessage(`❌ ${verification.error || "Transaction verification failed"}`);
        return;
      }

      setMessage("💰 Bitcoin transaction verified! Crediting QTC...");

      // Credit QTC
      const creditResponse = await fetch("/api/credit-qtc", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          qtcAddr: qtcAddress,
          amountQtc: verification.estimatedQtc
        })
      });

      const creditResult = await creditResponse.json();

      if (!creditResult.ok) {
        setMessage(`❌ ${creditResult.error || "Failed to credit QTC"}`);
        return;
      }

      // Success!
      setMessage(`✅ Success! ${creditResult.credited} QTC credited to ${qtcAddress}${creditResult.simulated ? ' (SIMULATED)' : ''}`);
      
      // Update local balance if this is the user's wallet
      const userWallet = localStorage.getItem("qc_wallet_addr");
      if (userWallet === qtcAddress) {
        const currentBalance = parseFloat(localStorage.getItem("qc_wallet_balance") || "0");
        const newBalance = currentBalance + (creditResult.credited || 0);
        localStorage.setItem("qc_wallet_balance", newBalance.toString());
      }

      // Clear form
      setBtcTxid("");

    } catch (error) {
      console.error("Exchange error:", error);
      setMessage("❌ Network error. Please try again.");
    } finally {
      setIsVerifying(false);
    }
  };

  if (isLoading) {
    return (
      <div className="min-h-screen bg-quantum-dark flex items-center justify-center">
        <div className="text-cyan-300">Loading...</div>
      </div>
    );
  }

  if (!user) {
    return null; // Will redirect
  }

  return (
    <div className="min-h-screen bg-quantum-dark text-cyan-100">
      <NavBar />
      
      <main className="pt-20 pb-16">
        <div className="mx-auto max-w-4xl px-4 py-8">
          {/* Page Header */}
          <div className="mb-8">
            <h1 className="text-3xl font-bold text-cyan-300 mb-2">💱 BTC to QTC Exchange</h1>
            <p className="opacity-75">
              Buy QuantumCoin™ with Bitcoin using secure on-chain verification
            </p>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
            {/* Exchange Form */}
            <div className="space-y-6">
              {/* BTC Address */}
              <div className="quantum-card">
                <h3 className="text-lg font-semibold text-cyan-300 mb-3">
                  📤 Step 1: Send Bitcoin
                </h3>
                <div className="text-sm opacity-75 mb-2">Send BTC to this address:</div>
                <div className="p-3 bg-black/30 rounded font-mono text-sm break-all border border-cyan-700/30 mb-3">
                  {BTC_ADDRESS}
                </div>
                <button 
                  onClick={() => navigator.clipboard.writeText(BTC_ADDRESS)}
                  className="quantum-button-secondary text-sm"
                >
                  📋 Copy BTC Address
                </button>
              </div>

              {/* Verification Form */}
              <div className="quantum-card">
                <h3 className="text-lg font-semibold text-cyan-300 mb-3">
                  ✅ Step 2: Verify & Claim
                </h3>
                
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium text-cyan-300 mb-2">
                      Your QTC Address
                    </label>
                    <input
                      className="quantum-input"
                      placeholder="QTC address to receive coins"
                      value={qtcAddress}
                      onChange={(e) => setQtcAddress(e.target.value)}
                    />
                    {!qtcAddress && (
                      <div className="text-xs opacity-60 mt-1">
                        Generate a wallet first if you don't have an address
                      </div>
                    )}
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-cyan-300 mb-2">
                      Bitcoin Transaction ID
                    </label>
                    <input
                      className="quantum-input"
                      placeholder="Paste your BTC txid here"
                      value={btcTxid}
                      onChange={(e) => setBtcTxid(e.target.value)}
                    />
                    <div className="text-xs opacity-60 mt-1">
                      Found in your Bitcoin wallet after sending
                    </div>
                  </div>

                  <button
                    onClick={verifyAndCredit}
                    disabled={isVerifying || !qtcAddress || !btcTxid}
                    className={`w-full py-3 rounded font-semibold transition-colors ${
                      isVerifying || !qtcAddress || !btcTxid
                        ? 'bg-gray-600 text-gray-300 cursor-not-allowed'
                        : 'quantum-button-primary'
                    }`}
                  >
                    {isVerifying ? "🔍 Verifying..." : "🚀 Verify & Credit QTC"}
                  </button>
                </div>
              </div>
            </div>

            {/* Exchange Status & Info */}
            <div className="space-y-6">
              {/* Exchange Status */}
              <div className="quantum-card">
                <h3 className="text-lg font-semibold text-cyan-300 mb-3">📊 Exchange Status</h3>
                <div className="space-y-3">
                  <div className="flex justify-between">
                    <span>Status:</span>
                    <span className={status.enabled ? "text-green-300" : "text-red-300"}>
                      {status.enabled ? "🟢 ACTIVE" : "🔴 INACTIVE"}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span>Available QTC:</span>
                    <span className="font-mono">{status.float.toLocaleString()}</span>
                  </div>
                  <div className="text-xs opacity-60 mt-2">
                    {status.enabled 
                      ? "Exchange is operational with available supply"
                      : "Exchange disabled - QTC must be mined to increase supply"
                    }
                  </div>
                </div>
              </div>

              {/* How It Works */}
              <div className="quantum-card">
                <h3 className="text-lg font-semibold text-cyan-300 mb-3">❓ How It Works</h3>
                <div className="space-y-3 text-sm opacity-80">
                  <div className="flex gap-3">
                    <span className="text-cyan-300 font-bold">1.</span>
                    <span>Send Bitcoin to our verified address</span>
                  </div>
                  <div className="flex gap-3">
                    <span className="text-cyan-300 font-bold">2.</span>
                    <span>Copy the transaction ID from your Bitcoin wallet</span>
                  </div>
                  <div className="flex gap-3">
                    <span className="text-cyan-300 font-bold">3.</span>
                    <span>Paste the txid here for on-chain verification</span>
                  </div>
                  <div className="flex gap-3">
                    <span className="text-cyan-300 font-bold">4.</span>
                    <span>Receive QTC at current market rate</span>
                  </div>
                </div>
              </div>

              {/* Security Features */}
              <div className="quantum-card">
                <h3 className="text-lg font-semibold text-cyan-300 mb-3">🛡️ Security Features</h3>
                <div className="space-y-2 text-sm opacity-80">
                  <div className="flex items-center gap-2">
                    <span>✅</span>
                    <span>On-chain Bitcoin verification via mempool.space</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span>✅</span>
                    <span>Real-time BTC/USD price from CoinGecko</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span>✅</span>
                    <span>Supply-gated exchange (no infinite printing)</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span>✅</span>
                    <span>Post-quantum cryptographic security</span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Status Message */}
          {message && (
            <div className="mt-6 quantum-card">
              <div className="text-center">
                <div className="text-lg">{message}</div>
              </div>
            </div>
          )}
        </div>
      </main>
    </div>
  );
}
