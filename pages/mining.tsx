import NavBar from "@/components/NavBar";
import { useEffect, useState } from "react";

interface MiningStats {
  mining: boolean;
  hashrate: number;
  blocksMined: number;
  totalEarnings: number;
  difficulty: number;
  networkHashrate: number;
  aiPerformanceBoost: number;
  aiSecurityLevel: number;
}

export default function Mining() {
  const [stats, setStats] = useState<MiningStats>({
    mining: false,
    hashrate: 0,
    blocksMined: 0,
    totalEarnings: 0,
    difficulty: 0,
    networkHashrate: 0,
    aiPerformanceBoost: 1.0,
    aiSecurityLevel: 1.0,
  });
  const [walletAddress, setWalletAddress] = useState<string>("");
  const [threadCount, setThreadCount] = useState<number>(4);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    // Load wallet address
    const addr = localStorage.getItem("qc_wallet_addr") || "";
    setWalletAddress(addr);

    // Start real-time monitoring
    const interval = setInterval(fetchMiningStats, 5000);
    fetchMiningStats();

    return () => clearInterval(interval);
  }, []);

  const fetchMiningStats = async () => {
    try {
      // Get AI-enhanced network metrics
      const response = await fetch("http://localhost:8080/network/metrics");
      const metrics = await response.json();

      // Get AI status
      const aiResponse = await fetch("http://localhost:8080/ai/status");
      const aiStatus = await aiResponse.json();

      setStats(prev => ({
        ...prev,
        difficulty: metrics.difficulty || 0,
        networkHashrate: metrics.hashrate || 0,
        aiPerformanceBoost: aiStatus.performance_boost || 1.0,
        aiSecurityLevel: aiStatus.security_level || 1.0,
      }));

    } catch (error) {
      console.error("Failed to fetch mining stats:", error);
    }
  };

  const startMining = async () => {
    if (!walletAddress) {
      alert("Please generate a wallet first");
      return;
    }

    setLoading(true);
    try {
      // Start real mining with AI optimization
      const response = await fetch("http://localhost:8080/mining/start", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          minerAddress: walletAddress,
          threadCount: threadCount,
          aiOptimized: true, // Enable AI optimizations
        }),
      });

      if (response.ok) {
        setStats(prev => ({ ...prev, mining: true }));
        // Start mining monitoring
        startMiningMonitoring();
      } else {
        throw new Error("Failed to start mining");
      }
    } catch (error) {
      alert(`Mining failed: ${error}`);
    }
    setLoading(false);
  };

  const stopMining = async () => {
    try {
      await fetch("http://localhost:8080/mining/stop", { method: "POST" });
      setStats(prev => ({ ...prev, mining: false, hashrate: 0 }));
    } catch (error) {
      console.error("Failed to stop mining:", error);
    }
  };

  const startMiningMonitoring = () => {
    const interval = setInterval(async () => {
      if (!stats.mining) {
        clearInterval(interval);
        return;
      }

      try {
        const response = await fetch("http://localhost:8080/mining/status");
        const status = await response.json();

        if (status.success) {
          setStats(prev => ({
            ...prev,
            hashrate: status.hashrate || 0,
            blocksMined: status.blocksMined || 0,
            totalEarnings: status.totalEarnings || 0,
          }));
        }
      } catch (error) {
        console.error("Mining monitoring error:", error);
      }
    }, 3000);
  };

  const mineBlock = async () => {
    if (!walletAddress) {
      alert("Please generate a wallet first");
      return;
    }

    setLoading(true);
    try {
      const response = await fetch(`http://localhost:8080/mine/${walletAddress}`, {
        method: "POST",
      });

      if (response.ok) {
        const block = await response.json();
        setStats(prev => ({
          ...prev,
          blocksMined: prev.blocksMined + 1,
          totalEarnings: prev.totalEarnings + 10, // Mining reward
        }));
        alert(`🎉 Block #${block.index} mined! Earned 10 QTC`);
        fetchMiningStats(); // Refresh stats
      } else {
        throw new Error("Mining failed");
      }
    } catch (error) {
      alert(`Mining failed: ${error}`);
    }
    setLoading(false);
  };

  return (
    <main className="min-h-screen bg-[#061018] text-cyan-100">
      <NavBar />
      <div className="mx-auto max-w-6xl px-4 py-8">
        <h2 className="text-2xl font-semibold text-cyan-300 mb-6">AI-Enhanced Mining</h2>

        {/* AI Status Panel */}
        <div className="rounded-xl p-5 bg-gradient-to-br from-[#0a1f2b] to-[#103042] border border-cyan-700/30 mb-6">
          <h3 className="text-lg font-semibold text-cyan-300 mb-3">🤖 AI Sentinel Status</h3>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <div className="text-sm opacity-80">Performance Boost</div>
              <div className="text-xl font-semibold text-green-400">
                {stats.aiPerformanceBoost.toFixed(2)}x
              </div>
            </div>
            <div>
              <div className="text-sm opacity-80">Security Level</div>
              <div className="text-xl font-semibold text-blue-400">
                {(stats.aiSecurityLevel * 100).toFixed(1)}%
              </div>
            </div>
            <div>
              <div className="text-sm opacity-80">AI Enhancement</div>
              <div className="text-xl font-semibold text-cyan-400">
                {stats.aiPerformanceBoost > 1.2 ? "ACTIVE" : "MONITORING"}
              </div>
            </div>
          </div>
        </div>

        {/* Mining Controls */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
          <div className="rounded-xl p-5 bg-[#0c2030] border border-cyan-700/30">
            <h3 className="text-lg font-semibold text-cyan-300 mb-4">Mining Control</h3>
            
            <div className="mb-4">
              <label className="block text-sm opacity-80 mb-2">Wallet Address</label>
              <input
                type="text"
                value={walletAddress}
                onChange={(e) => setWalletAddress(e.target.value)}
                className="w-full p-3 rounded bg-[#0b1b26] border border-cyan-700/30 font-mono text-sm"
                placeholder="Generate wallet first"
                readOnly
              />
            </div>

            <div className="mb-4">
              <label className="block text-sm opacity-80 mb-2">Thread Count</label>
              <input
                type="number"
                value={threadCount}
                onChange={(e) => setThreadCount(parseInt(e.target.value) || 4)}
                min="1"
                max="32"
                className="w-full p-3 rounded bg-[#0b1b26] border border-cyan-700/30"
              />
            </div>

            <div className="flex gap-3">
              {!stats.mining ? (
                <button
                  onClick={startMining}
                  disabled={loading || !walletAddress}
                  className="flex-1 py-3 rounded bg-green-600 text-white font-semibold disabled:opacity-50"
                >
                  {loading ? "Starting..." : "Start AI Mining"}
                </button>
              ) : (
                <button
                  onClick={stopMining}
                  className="flex-1 py-3 rounded bg-red-600 text-white font-semibold"
                >
                  Stop Mining
                </button>
              )}
              
              <button
                onClick={mineBlock}
                disabled={loading || !walletAddress}
                className="px-4 py-3 rounded bg-cyan-500 text-black font-semibold disabled:opacity-50"
              >
                Mine Block
              </button>
            </div>
          </div>

          {/* Mining Statistics */}
          <div className="rounded-xl p-5 bg-[#0c2030] border border-cyan-700/30">
            <h3 className="text-lg font-semibold text-cyan-300 mb-4">Statistics</h3>
            
            <div className="grid grid-cols-2 gap-4">
              <div>
                <div className="text-sm opacity-80">Status</div>
                <div className={`text-lg font-semibold ${stats.mining ? "text-green-400" : "text-red-400"}`}>
                  {stats.mining ? "MINING" : "STOPPED"}
                </div>
              </div>
              <div>
                <div className="text-sm opacity-80">Hashrate</div>
                <div className="text-lg font-semibold text-cyan-400">
                  {stats.hashrate.toLocaleString()} H/s
                </div>
              </div>
              <div>
                <div className="text-sm opacity-80">Blocks Mined</div>
                <div className="text-lg font-semibold text-yellow-400">
                  {stats.blocksMined}
                </div>
              </div>
              <div>
                <div className="text-sm opacity-80">Total Earned</div>
                <div className="text-lg font-semibold text-green-400">
                  {stats.totalEarnings.toFixed(8)} QTC
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Network Information */}
        <div className="rounded-xl p-5 bg-[#0c2030] border border-cyan-700/30">
          <h3 className="text-lg font-semibold text-cyan-300 mb-4">Network Status</h3>
          
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div>
              <div className="text-sm opacity-80">Network Hashrate</div>
              <div className="text-lg font-semibold text-cyan-400">
                {(stats.networkHashrate / 1000000).toFixed(2)} MH/s
              </div>
            </div>
            <div>
              <div className="text-sm opacity-80">Difficulty</div>
              <div className="text-lg font-semibold text-cyan-400">
                {stats.difficulty.toFixed(0)}
              </div>
            </div>
            <div>
              <div className="text-sm opacity-80">AI Performance</div>
              <div className="text-lg font-semibold text-purple-400">
                +{((stats.aiPerformanceBoost - 1) * 100).toFixed(1)}%
              </div>
            </div>
            <div>
              <div className="text-sm opacity-80">AI Security</div>
              <div className="text-lg font-semibold text-blue-400">
                {(stats.aiSecurityLevel * 100).toFixed(1)}%
              </div>
            </div>
          </div>

          <div className="mt-6 p-4 bg-[#0a1f2b] rounded border border-cyan-700/20">
            <div className="text-sm text-cyan-300 font-semibold mb-2">🧠 AI Enhancements Active:</div>
            <div className="text-sm opacity-80">
              • Real-time attack detection and prevention<br />
              • Dynamic difficulty and fee optimization<br />
              • Network performance auto-tuning<br />
              • Quantum-resistant security monitoring<br />
              • Mining efficiency maximization
            </div>
          </div>
        </div>
      </div>
    </main>
  );
}
