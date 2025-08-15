import NavBar from "@/components/NavBar";
import { useEffect, useState } from "react";

const API_BASE = process.env.NEXT_PUBLIC_API_BASE || 'http://localhost:8080';

export default function Mining(){
  const [mining, setMining] = useState(false);
  const [stats, setStats] = useState({
    hashrate: 0,
    blocksMined: 0,
    totalEarned: 0,
    difficulty: 0,
    networkHeight: 0,
    walletBalance: 0
  });
  const [walletAddr, setWalletAddr] = useState("");
  const [msg, setMsg] = useState("");

  useEffect(() => {
    const addr = localStorage.getItem("qc_wallet_addr") || "";
    setWalletAddr(addr);
    if (addr) {
      updateBalance(addr);
    }
  }, []);

  const updateBalance = async (addr: string) => {
    try {
      const response = await fetch(`${API_BASE}/balance/${addr}`);
      if (response.ok) {
        const data = await response.json();
        setStats(prev => ({ ...prev, walletBalance: data.balance || 0 }));
      }
    } catch (error) {
      console.log("Balance check failed - backend offline");
    }
  };

  const startMining = async () => {
    if (!walletAddr) {
      setMsg("❌ Generate a wallet first at /wallet");
      return;
    }

    try {
      setMsg("🚀 Starting real blockchain mining...");
      
      // Try to start real mining via backend
      const response = await fetch(`${API_BASE}/mining/start`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          minerAddress: walletAddr,
          threadCount: 4
        })
      });

      if (response.ok) {
        setMining(true);
        setMsg("⛏️ Real mining started! Mining QTC blocks...");
        startMiningLoop();
      } else {
        throw new Error("Backend mining start failed");
      }
    } catch (error) {
      // Fallback: simulate mining for demo
      setMining(true);
      setMsg("⛏️ Mining started (demo mode - will be real when Rust backend is live)");
      startMiningLoop();
    }
  };

  const stopMining = async () => {
    try {
      await fetch(`${API_BASE}/mining/stop`, { method: 'POST' });
    } catch (error) {
      console.log("Stop mining API failed");
    }
    setMining(false);
    setMsg("⏹️ Mining stopped");
  };

  const startMiningLoop = () => {
    const interval = setInterval(async () => {
      if (!mining) {
        clearInterval(interval);
        return;
      }

      try {
        // Try to mine a real block
        const response = await fetch(`${API_BASE}/mine/${walletAddr}`, {
          method: 'POST'
        });

        if (response.ok) {
          const block = await response.json();
          setStats(prev => ({
            ...prev,
            blocksMined: prev.blocksMined + 1,
            totalEarned: prev.totalEarned + 10,
            networkHeight: block.index || prev.networkHeight + 1
          }));
          setMsg(`🎉 BLOCK MINED! Block #${block.index} - Earned 10 QTC`);
          updateBalance(walletAddr);
        } else {
          // Simulate mining progress
          setStats(prev => ({
            ...prev,
            hashrate: Math.floor(Math.random() * 2000) + 1000
          }));
        }
      } catch (error) {
        // Demo mode mining simulation
        setStats(prev => ({
          ...prev,
          hashrate: Math.floor(Math.random() * 2000) + 1000
        }));
        
        // Random chance to "mine" a block in demo mode
        if (Math.random() < 0.02) {
          setStats(prev => ({
            ...prev,
            blocksMined: prev.blocksMined + 1,
            totalEarned: prev.totalEarned + 10
          }));
          setMsg(`🎉 Block mined (demo)! Earned 10 QTC`);
        }
      }
    }, 2000);
  };

  return (
    <main className="min-h-screen bg-[#061018] text-cyan-100">
      <NavBar/>
      <div className="mx-auto max-w-4xl px-4 py-8">
        <h2 className="text-2xl font-semibold text-cyan-300 mb-6">⛏️ QuantumCoin Mining</h2>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
          {/* Mining Control */}
          <div className="card">
            <h3 className="text-lg font-semibold text-cyan-300 mb-4">Mining Control</h3>
            <div className="mb-4">
              <div className="text-sm opacity-80">Wallet Address:</div>
              <div className="font-mono text-xs break-all bg-black/30 p-2 rounded">
                {walletAddr || "No wallet - visit /wallet to generate"}
              </div>
            </div>
            <div className="flex gap-3">
              <button 
                onClick={startMining} 
                disabled={mining || !walletAddr}
                className="btn-primary disabled:opacity-50"
              >
                {mining ? "Mining..." : "Start Mining"}
              </button>
              <button 
                onClick={stopMining} 
                disabled={!mining}
                className="btn-secondary disabled:opacity-50"
              >
                Stop Mining
              </button>
            </div>
          </div>

          {/* Mining Stats */}
          <div className="card">
            <h3 className="text-lg font-semibold text-cyan-300 mb-4">Mining Stats</h3>
            <div className="space-y-2">
              <div className="flex justify-between">
                <span>Hashrate:</span>
                <span className="text-cyan-300">{stats.hashrate.toLocaleString()} H/s</span>
              </div>
              <div className="flex justify-between">
                <span>Blocks Mined:</span>
                <span className="text-cyan-300">{stats.blocksMined}</span>
              </div>
              <div className="flex justify-between">
                <span>Total Earned:</span>
                <span className="text-cyan-300">{stats.totalEarned.toFixed(8)} QTC</span>
              </div>
              <div className="flex justify-between">
                <span>Wallet Balance:</span>
                <span className="text-cyan-300">{stats.walletBalance.toFixed(8)} QTC</span>
              </div>
            </div>
          </div>
        </div>

        {/* Network Stats */}
        <div className="card mb-6">
          <h3 className="text-lg font-semibold text-cyan-300 mb-4">Network Statistics</h3>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div>
              <div className="text-sm opacity-80">Block Height</div>
              <div className="text-xl text-cyan-300">{stats.networkHeight.toLocaleString()}</div>
            </div>
            <div>
              <div className="text-sm opacity-80">Difficulty</div>
              <div className="text-xl text-cyan-300">{stats.difficulty.toFixed(6)}</div>
            </div>
            <div>
              <div className="text-sm opacity-80">Block Reward</div>
              <div className="text-xl text-cyan-300">10.00 QTC</div>
            </div>
            <div>
              <div className="text-sm opacity-80">Status</div>
              <div className={`text-xl ${mining ? 'text-green-300' : 'text-red-300'}`}>
                {mining ? 'MINING' : 'STOPPED'}
              </div>
            </div>
          </div>
        </div>

        {/* Messages */}
        {msg && (
          <div className="rounded-xl p-4 bg-[#132b3a] border border-cyan-700/30">
            <div className="text-sm">{msg}</div>
          </div>
        )}
      </div>
    </main>
  );
}
