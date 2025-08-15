import NavBar from "@/components/NavBar";
import { useEffect, useState } from "react";

const BTC_ADDRESS = process.env.NEXT_PUBLIC_BTC_ADDRESS || "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh";

export default function Exchange(){
  const [status, setStatus] = useState<{float:number; enabled:boolean}>({float:0, enabled:false});
  const [qtcAddr, setQtcAddr] = useState<string>("");
  const [txid, setTxid] = useState<string>("");
  const [msg, setMsg] = useState<string>("");
  const [loading, setLoading] = useState(false);

  useEffect(()=>{
    fetch("/api/exchange-status").then(r=>r.json()).then(setStatus).catch(()=>setStatus({float:0,enabled:false}));
    // Pre-fill QTC address if user has wallet
    const addr = localStorage.getItem("qc_wallet_addr");
    if (addr) setQtcAddr(addr);
  },[]);

  const verifyAndCredit = async()=>{
    if (!txid) {
      setMsg("Please enter BTC transaction ID");
      return;
    }
    if (!qtcAddr) {
      setMsg("Please enter your QTC address");
      return;
    }

    setLoading(true);
    setMsg("Verifying BTC transaction on-chain via mempool.space…");
    
    try {
      const v = await fetch(`/api/verify-btc?txid=${encodeURIComponent(txid)}`).then(r=>r.json());
      if(!v.ok){ 
        setMsg(v.error || "Invalid transaction"); 
        setLoading(false);
        return; 
      }
      
      if(!status.enabled || status.float<=0){ 
        setMsg("❌ No QTC available on exchange. Miners must mine more supply first."); 
        setLoading(false);
        return; 
      }

      setMsg("✅ BTC verified! Crediting QTC to your wallet…");
      const c = await fetch(`/api/credit-qtc`, {
        method:"POST", 
        headers:{"Content-Type":"application/json"}, 
        body:JSON.stringify({ qtcAddr, amountQtc: v.estimatedQtc })
      }).then(r=>r.json());

      if(!c.ok){ 
        setMsg(c.error || "Credit failed"); 
        setLoading(false);
        return; 
      }

      setMsg(`🎉 SUCCESS: ${c.credited} QTC credited to ${qtcAddr}. ${c.simulated ? "(Demo mode - will be real when backend is live)" : ""}`);
      
      // Refresh exchange status
      fetch("/api/exchange-status").then(r=>r.json()).then(setStatus).catch(()=>{});
      
    } catch (error) {
      setMsg("❌ Verification failed. Please try again.");
    }
    setLoading(false);
  };

  return (
    <main className="min-h-screen bg-[#061018] text-cyan-100">
      <NavBar/>
      <div className="mx-auto max-w-2xl px-4 py-8">
        <h2 className="text-2xl font-semibold text-cyan-300 mb-2">Buy QTC with BTC</h2>
        <p className="opacity-80 mb-6">Send BTC to our address, then paste your BTC transaction ID to claim QTC at current rate. Supply-limited by mining.</p>

        <div className="rounded-xl p-5 bg-gradient-to-br from-[#0a1f2b] to-[#103042] border border-cyan-700/30 mb-6">
          <div className="text-sm opacity-80 mb-2">💰 Send BTC to this address:</div>
          <div className="font-mono break-all text-amber-300 bg-black/30 p-3 rounded">{BTC_ADDRESS}</div>
          <div className="text-xs opacity-60 mt-2">⚠️ Only send from your own wallet. Minimum: 0.001 BTC</div>
        </div>

        <div className="rounded-xl p-5 bg-[#0c2030] border border-cyan-700/30 mb-6">
          <div className="mb-3 text-sm font-semibold">Step 1: Your QTC receiving address</div>
          <input 
            className="w-full p-3 rounded bg-[#0b1b26] border border-cyan-700/30 mb-4" 
            placeholder="QTC address (from your wallet)" 
            value={qtcAddr} 
            onChange={e=>setQtcAddr(e.target.value)}
          />
          
          <div className="mb-3 text-sm font-semibold">Step 2: Your BTC transaction ID</div>
          <input 
            className="w-full p-3 rounded bg-[#0b1b26] border border-cyan-700/30 mb-4" 
            placeholder="Paste BTC txid here" 
            value={txid} 
            onChange={e=>setTxid(e.target.value)}
          />
          
          <button 
            onClick={verifyAndCredit} 
            disabled={loading}
            className="w-full py-3 rounded bg-cyan-500 text-black font-semibold disabled:opacity-50"
          >
            {loading ? "Verifying..." : "Verify & Credit QTC"}
          </button>
        </div>

        <div className="rounded-xl p-4 bg-[#09202a] border border-cyan-700/30 mb-4">
          <div className="text-sm font-semibold mb-2">📊 Exchange Status:</div>
          {status.enabled ? (
            <div className="text-green-300">✅ Available: {status.float.toLocaleString()} QTC</div>
          ) : (
            <div className="text-red-300">❌ Exchange disabled (supply: {status.float} QTC)</div>
          )}
        </div>

        {msg && (
          <div className="rounded-xl p-4 bg-[#132b3a] border border-cyan-700/30">
            <div className="text-sm">{msg}</div>
          </div>
        )}
      </div>
    </main>
  );
}
