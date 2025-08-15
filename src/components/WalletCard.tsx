import { useEffect, useState } from "react";

const genMockAddress = () => "QTC" + btoa(String(Math.random())).slice(0,42);

export default function WalletCard(){
  const [addr, setAddr] = useState<string>("");
  const [balance, setBalance] = useState<number>(0);

  useEffect(()=>{ 
    const a=localStorage.getItem("qc_wallet_addr")||""; 
    setAddr(a);
    // Fetch balance from backend
    if (a) {
      fetch(`http://localhost:8080/balance/${a}`)
        .then(r => r.json())
        .then(data => setBalance(data.balance || 0))
        .catch(() => setBalance(0));
    }
  },[]);

  const generate = async () => { 
    try {
      // Try to generate from backend first
      const response = await fetch('http://localhost:8080/wallet/generate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' }
      });
      
      if (response.ok) {
        const data = await response.json();
        const a = data.address;
        localStorage.setItem("qc_wallet_addr", a);
        setAddr(a);
        alert("Real quantum-resistant wallet generated! BACK IT UP.");
      } else {
        throw new Error("Backend not available");
      }
    } catch (error) {
      // Fallback to mock if backend not available
      const a = genMockAddress(); 
      localStorage.setItem("qc_wallet_addr", a); 
      setAddr(a); 
      alert("Wallet generated (demo mode). BACK IT UP."); 
    }
  };

  const copy = async () => { if(addr) await navigator.clipboard.writeText(addr); };

  return (
    <div className="rounded-xl p-5 bg-gradient-to-br from-[#0c2030] to-[#0e2c3f] border border-cyan-700/30">
      <h3 className="text-xl font-semibold text-cyan-300 mb-2">Quantum Wallet</h3>
      {!addr ? (
        <>
          <p className="opacity-80 mb-3">No wallet detected.</p>
          <button onClick={generate} className="px-4 py-2 rounded bg-cyan-500 text-black">Generate Wallet</button>
        </>
      ) : (
        <>
          <div className="text-sm opacity-80 mb-2">Address</div>
          <div className="break-all font-mono bg-black/30 p-2 rounded mb-2">{addr}</div>
          <div className="text-sm opacity-80 mb-2">Balance: <span className="text-cyan-300">{balance.toFixed(8)} QTC</span></div>
          <div className="mt-3 flex gap-2">
            <button onClick={copy} className="px-3 py-2 rounded bg-cyan-500 text-black">Copy</button>
            <a download="wallet.txt" href={`data:text/plain,${encodeURIComponent(addr)}`} className="px-3 py-2 rounded bg-cyan-700/30 border border-cyan-700/50">Download .txt</a>
          </div>
          <p className="mt-4 text-amber-300 text-sm">⚠️ Backup tip: save your address + keys offline (USB). Never share your private key.</p>
        </>
      )}
    </div>
  );
}
