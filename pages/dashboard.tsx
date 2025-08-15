import NavBar from "@/components/NavBar";
import { useEffect, useState } from "react";
import { useRevStop } from "@/context/RevStopContext";

interface DashboardData {
  balance: number;
  networkStatus: {
    height: number;
    peers: number;
    mempool: number;
    hashrate: number;
    difficulty: number;
  };
  aiStatus: {
    active: boolean;
    performanceBoost: number;
    securityLevel: number;
    riskLevel: number;
    optimizationsActive: number;
  };
  recentBlocks: Array<{
    height: number;
    hash: string;
    timestamp: string;
    txCount: number;
  }>;
}

export default function Dashboard() {
  const { active: revStopActive } = useRevStop();
  const [data, setData] = useState<DashboardData>({
    balance: 0,
    networkStatus: { height: 0, peers: 0, mempool: 0, hashrate: 0, difficulty: 0 },
    aiStatus: { active: false, performanceBoost: 1.0, securityLevel: 1.0, riskLevel: 0.0, optimizationsActive: 0 },
    recentBlocks: [],
  });
  const [walletAddress, setWalletAddress] = useState<string>("");

  useEffect(() => {
    const addr = localStorage.getItem("qc_wallet_addr") || "";
    setWalletAddress(addr);

    fetchDashboardData();
    const interval = setInterval(fetchDashboardData, 10000); // Update every 10 seconds

    return () => clearInterval(interval);
  }, []);

  const fetchDashboardData = async () => {
    try {
      // Fetch wallet balance
      let balance = 0;
      if (walletAddress) {
        const balanceResponse = await fetch(`http://localhost:8080/balance/${walletAddress}`);
        const balanceData = await balanceResponse.json();
        balance = balanceData.balance || 0;
      }

      // Fetch network metrics with AI enhancements
      const networkResponse = await fetch("http://localhost:8080/network/metrics");
      const networkData = await networkResponse.json();

      // Fetch AI status
      const aiResponse = await fetch("http://localhost:8080/ai/status");
      const aiData = await aiResponse.json();

      // Fetch recent blocks
      const blocksResponse = await fetch("http://localhost:8080/blockchain");
      const blocksData = await blocksResponse.json();
      const recentBlocks = blocksData.slice(-5).reverse().map((block: any) => ({
        height: block.index,
        hash: block.hash.substring(0, 12) + "...",
        timestamp: new Date(block.timestamp).toLocaleTimeString(),
        txCount: block.transactions.length,
      }));

      setData({
        balance,
        networkStatus: {
          height: networkData.height || 0,
          peers: networkData.peer_count || 0,
          mempool: networkData.mempool_size || 0,
          hashrate: networkData.hashrate || 0,
          difficulty: networkData.difficulty || 0,
        },
        aiStatus: {
          active: aiData.ai_active || false,
          performanceBoost: aiData.performance_boost || 1.0,
          securityLevel: aiData.security_level || 1.0,
          riskLevel: aiData.risk_level || 0.0,
          optimizationsActive: aiData.optimizations_active || 0,
        },
        recentBlocks,
      });

    } catch (error) {
      console.error("Failed to fetch dashboard data:", error);
    }
  };

  const getRiskLevelColor = (risk: number) => {
    if (risk < 0.3) return "text-green-400";
    if (risk < 0.7) return "text-yellow-400";
    return "text-red-400";
  };

  const getRiskLevelText = (risk: number) => {
    if (risk < 0.3) return "LOW";
    if (risk < 0.7) return "MEDIUM";
    return "HIGH";
  };

  return (
    <main className="min-h-screen bg-[#061018] text-cyan-100">
      <NavBar />
      <div className="mx-auto max-w-6xl px-4 py-8">
        <h2 className="text-2xl font-semibold text-cyan-300 mb-6">AI-Enhanced Dashboard</h2>

        {/* Wallet Summary */}
        <div className="rounded-xl p-5 bg-gradient-to-br from-[#0a1f2b] to-[#103042] border border-cyan-700/30 mb-6">
          <h3 className="text-lg font-semibold text-cyan-300 mb-3">Wallet Summary</h3>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <div className="text-sm opacity-80">Balance</div>
              <div className="text-2xl font-bold text-green-400">{data.balance.toFixed(8)} QTC</div>
            </div>
            <div>
              <div className="text-sm opacity-80">Address</div>
              <div className="text-sm font-mono text-cyan-400 break-all">
                {walletAddress || "No wallet generated"}
              </div>
            </div>
            <div>
              <div className="text-sm opacity-80">RevStop™ Protection</div>
              <div className={`text-lg font-semibold ${revStopActive ? "text-green-400" : "text-red-400"}`}>
                {revStopActive ? "ACTIVE" : "DISABLED"}
              </div>
            </div>
          </div>
        </div>

        {/* AI Status Panel */}
        <div className="rounded-xl p-5 bg-gradient-to-br from-[#1a0a2b] to-[#2a1042] border border-purple-700/30 mb-6">
          <h3 className="text-lg font-semibold text-purple-300 mb-3">🤖 AI Sentinel Intelligence</h3>
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div>
              <div className="text-sm opacity-80">AI Status</div>
              <div className={`text-lg font-semibold ${data.aiStatus.active ? "text-green-400" : "text-yellow-400"}`}>
                {data.aiStatus.active ? "ENHANCING" : "LEARNING"}
              </div>
            </div>
            <div>
              <div className="text-sm opacity-80">Performance Boost</div>
              <div className="text-lg font-semibold text-purple-400">
                +{((data.aiStatus.performanceBoost - 1) * 100).toFixed(1)}%
              </div>
            </div>
            <div>
              <div className="text-sm opacity-80">Security Level</div>
              <div className="text-lg font-semibold text-blue-400">
                {(data.aiStatus.securityLevel * 100).toFixed(1)}%
              </div>
            </div>
            <div>
              <div className="text-sm opacity-80">Risk Assessment</div>
              <div className={`text-lg font-semibold ${getRiskLevelColor(data.aiStatus.riskLevel)}`}>
                {getRiskLevelText(data.aiStatus.riskLevel)}
              </div>
            </div>
          </div>
          
          <div className="mt-4 p-3 bg-black/20 rounded text-sm">
            🧠 <strong>Active AI Systems:</strong> {data.aiStatus.optimizationsActive} optimizations running
            • Attack Detection • Network Optimization • Performance Tuning • Security Enhancement
          </div>
        </div>

        {/* Network Status */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
          <div className="rounded-xl p-5 bg-[#0c2030] border border-cyan-700/30">
            <h3 className="text-lg font-semibold text-cyan-300 mb-4">Network Status</h3>
            <div className="space-y-3">
              <div className="flex justify-between">
                <span className="opacity-80">Block Height</span>
                <span className="font-semibold">{data.networkStatus.height.toLocaleString()}</span>
              </div>
              <div className="flex justify-between">
                <span className="opacity-80">Connected Peers</span>
                <span className="font-semibold">{data.networkStatus.peers}</span>
              </div>
              <div className="flex justify-between">
                <span className="opacity-80">Mempool Size</span>
                <span className="font-semibold">{data.networkStatus.mempool} txs</span>
              </div>
              <div className="flex justify-between">
                <span className="opacity-80">Network Hashrate</span>
                <span className="font-semibold">{(data.networkStatus.hashrate / 1000000).toFixed(2)} MH/s</span>
              </div>
              <div className="flex justify-between">
                <span className="opacity-80">Difficulty</span>
                <span className="font-semibold">{data.networkStatus.difficulty.toLocaleString()}</span>
              </div>
            </div>
          </div>

          {/* Recent Blocks */}
          <div className="rounded-xl p-5 bg-[#0c2030] border border-cyan-700/30">
            <h3 className="text-lg font-semibold text-cyan-300 mb-4">Recent Blocks</h3>
            <div className="space-y-2">
              {data.recentBlocks.map((block, index) => (
                <div key={index} className="flex justify-between items-center p-2 bg-black/20 rounded">
                  <div>
                    <div className="font-semibold">#{block.height}</div>
                    <div className="text-xs opacity-60 font-mono">{block.hash}</div>
                  </div>
                  <div className="text-right">
                    <div className="text-sm">{block.txCount} txs</div>
                    <div className="text-xs opacity-60">{block.timestamp}</div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* AI Learning Progress */}
        <div className="rounded-xl p-5 bg-gradient-to-br from-[#0a2b1f] to-[#104230] border border-green-700/30">
          <h3 className="text-lg font-semibold text-green-300 mb-3">🧠 AI Learning System</h3>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="text-center">
              <div className="text-2xl font-bold text-green-400">
                {data.aiStatus.optimizationsActive}
              </div>
              <div className="text-sm opacity-80">Active Optimizations</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-cyan-400">
                {((1 - data.aiStatus.riskLevel) * 100).toFixed(0)}%
              </div>
              <div className="text-sm opacity-80">Network Safety</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-purple-400">
                {(data.aiStatus.performanceBoost * data.aiStatus.securityLevel).toFixed(2)}
              </div>
              <div className="text-sm opacity-80">AI Efficiency Score</div>
            </div>
          </div>
          
          <div className="mt-4 text-sm opacity-80">
            The AI Sentinel continuously learns from blockchain data to enhance security, optimize performance, 
            and prevent attacks. It grows smarter with every block, making QuantumCoin more advanced than Bitcoin.
          </div>
        </div>
      </div>
    </main>
  );
}
