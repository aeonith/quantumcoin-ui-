import { useEffect, useState } from "react";
import QRCode from "qrcode";

const genMockAddress = () => "QTC" + btoa(crypto.getRandomValues(new Uint8Array(16)).toString()).slice(0,42);

export default function WalletCard(){
  const [addr, setAddr] = useState<string>("");
  const [qr, setQr] = useState<string>("");

  useEffect(()=>{ const a=localStorage.getItem("qc_wallet_addr")||""; setAddr(a); },[]);
  useEffect(()=>{ if(!addr) return; QRCode.toDataURL(addr).then(setQr).catch(()=>setQr("")); },[addr]);

  const generate = () => { const a=genMockAddress(); localStorage.setItem("qc_wallet_addr",a); setAddr(a); alert("Wallet generated. BACK IT UP. Store offline."); };
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
          <div className="break-all font-mono bg-black/30 p-2 rounded">{addr}</div>
          <div className="mt-3 flex gap-2">
            <button onClick={copy} className="px-3 py-2 rounded bg-cyan-500 text-black">Copy</button>
            <a download="wallet.txt" href={`data:text/plain,${encodeURIComponent(addr)}`} className="px-3 py-2 rounded bg-cyan-700/30 border border-cyan-700/50">Download .txt</a>
          </div>
          {qr && <img src={qr} alt="qr" className="mt-4 w-32 h-32" />}
          <p className="mt-4 text-amber-300 text-sm">Backup tip: save your address + keys offline (USB). Never share your private key.</p>
        </>
      )}
    </div>
  );
}
