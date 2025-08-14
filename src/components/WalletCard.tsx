import { useEffect, useState } from "react";
import QRCode from "qrcode";
import { useRevStop } from "@/context/RevStopContext";
import { quantumAPI } from "@/lib/api";

export default function WalletCard() {
  const [address, setAddress] = useState<string>("");
  const [qrDataUrl, setQrDataUrl] = useState<string>("");
  const [balance, setBalance] = useState<number>(0);
  const [isGenerating, setIsGenerating] = useState(false);
  const [isLoadingBalance, setIsLoadingBalance] = useState(false);
  const [backendConnected, setBackendConnected] = useState(false);
  const { active: revStopActive } = useRevStop();

  // Check backend connectivity on mount
  useEffect(() => {
    quantumAPI.checkHealth().then(setBackendConnected);
  }, []);

  // Load saved address on mount
  useEffect(() => {
    try {
      const savedAddress = localStorage.getItem("qc_wallet_addr") || "";
      setAddress(savedAddress);
    } catch (error) {
      console.error("Error loading wallet data:", error);
    }
  }, []);

  // Update balance when address changes
  useEffect(() => {
    if (!address) return;
    
    const updateBalance = async () => {
      setIsLoadingBalance(true);
      try {
        const newBalance = await quantumAPI.wallet.getBalance(address);
        setBalance(newBalance);
      } catch (error) {
        console.error("Error updating balance:", error);
      } finally {
        setIsLoadingBalance(false);
      }
    };

    updateBalance();
    
    // Set up periodic balance updates
    const interval = setInterval(updateBalance, 30000); // Update every 30 seconds
    return () => clearInterval(interval);
  }, [address]);

  // Generate QR code when address changes
  useEffect(() => {
    if (!address) {
      setQrDataUrl("");
      return;
    }

    QRCode.toDataURL(address, {
      width: 256,
      margin: 2,
      color: { dark: '#000000', light: '#FFFFFF' }
    })
    .then(setQrDataUrl)
    .catch(error => {
      console.error("Error generating QR code:", error);
      setQrDataUrl("");
    });
  }, [address]);

  const generateWallet = async () => {
    setIsGenerating(true);
    try {
      // Use the API to generate address (connects to backend when available)
      const newAddress = await quantumAPI.wallet.generateAddress();
      
      // Save to localStorage
      localStorage.setItem("qc_wallet_addr", newAddress);
      localStorage.setItem("qc_wallet_balance", "0");
      
      setAddress(newAddress);
      setBalance(0);
      
      // Show backup warning
      alert("🔒 Wallet Generated!\n\n⚠️ CRITICAL: Back up your address immediately!\n\n• Copy and store offline (USB, paper)\n• Never share your private keys\n• Loss of backup = loss of funds");
    } catch (error) {
      console.error("Error generating wallet:", error);
      alert("Error generating wallet. Please try again.");
    } finally {
      setIsGenerating(false);
    }
  };

  const copyAddress = async () => {
    if (!address) return;
    
    try {
      await navigator.clipboard.writeText(address);
      // Visual feedback
      const button = document.getElementById('copy-btn');
      if (button) {
        const original = button.innerText;
        button.innerText = "Copied!";
        setTimeout(() => { button.innerText = original; }, 2000);
      }
    } catch (error) {
      console.error("Error copying address:", error);
      alert("Copy failed. Please select and copy manually.");
    }
  };

  const downloadBackup = () => {
    if (!address) return;
    
    const backupData = `QuantumCoin™ Wallet Backup
Generated: ${new Date().toISOString()}
Address: ${address}
RevStop Status: ${revStopActive ? 'ENABLED' : 'DISABLED'}
Backend Connected: ${backendConnected ? 'YES' : 'NO (Local Mode)'}

⚠️ SECURITY WARNING:
• Keep this file secure and offline
• Never share your private keys
• Make multiple backup copies
• Store in different locations

For support: https://github.com/aeonith/quantumcoin-ui-
`;

    const blob = new Blob([backupData], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `quantumcoin-wallet-backup-${Date.now()}.txt`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  return (
    <div className="quantum-card">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-xl font-semibold text-cyan-300">🔐 Quantum Wallet</h3>
        <div className="flex items-center gap-2">
          {address && (
            <div className={`px-3 py-1 rounded text-sm font-medium ${
              revStopActive 
                ? 'bg-green-700/30 text-green-300 border border-green-600/50' 
                : 'bg-red-700/30 text-red-300 border border-red-600/50'
            }`}>
              RevStop™ {revStopActive ? 'ACTIVE' : 'OFF'}
            </div>
          )}
          <div className={`px-2 py-1 rounded text-xs ${
            backendConnected 
              ? 'bg-green-700/30 text-green-300' 
              : 'bg-yellow-700/30 text-yellow-300'
          }`}>
            {backendConnected ? '🟢 Backend' : '🟡 Local'}
          </div>
        </div>
      </div>

      {!address ? (
        <div className="text-center">
          <div className="mb-4">
            <div className="text-lg mb-2">No Wallet Detected</div>
            <div className="text-sm opacity-75 mb-4">
              Generate a quantum-resistant wallet to start using QuantumCoin™
            </div>
          </div>
          
          <button 
            onClick={generateWallet}
            disabled={isGenerating}
            className={`quantum-button-primary ${isGenerating ? 'opacity-50 cursor-not-allowed' : ''}`}
          >
            {isGenerating ? "Generating..." : "🔑 Generate Wallet"}
          </button>
          
          <div className="mt-4 text-xs opacity-60">
            Uses post-quantum cryptography for maximum security
          </div>
        </div>
      ) : (
        <div>
          {/* Balance Display */}
          <div className="mb-4 p-4 bg-black/20 rounded-lg">
            <div className="text-sm opacity-75 mb-1">Balance</div>
            <div className="text-2xl font-bold text-cyan-300 flex items-center gap-2">
              {isLoadingBalance ? (
                <span className="animate-pulse">Loading...</span>
              ) : (
                `${balance.toLocaleString()} QTC`
              )}
              <button 
                onClick={() => quantumAPI.wallet.getBalance(address).then(setBalance)}
                className="text-sm px-2 py-1 rounded bg-cyan-700/30 hover:bg-cyan-700/50 transition-colors"
                title="Refresh balance"
              >
                🔄
              </button>
            </div>
          </div>

          {/* Address Display */}
          <div className="mb-4">
            <div className="text-sm opacity-75 mb-2">Your Address</div>
            <div className="p-3 bg-black/30 rounded font-mono text-sm break-all border border-cyan-700/30">
              {address}
            </div>
          </div>

          {/* Action Buttons */}
          <div className="flex flex-wrap gap-2 mb-4">
            <button 
              id="copy-btn"
              onClick={copyAddress} 
              className="quantum-button-primary"
            >
              📋 Copy Address
            </button>
            <button 
              onClick={downloadBackup} 
              className="quantum-button-secondary"
            >
              💾 Backup Wallet
            </button>
          </div>

          {/* QR Code */}
          {qrDataUrl && (
            <div className="text-center">
              <div className="text-sm opacity-75 mb-2">QR Code</div>
              <div className="inline-block p-4 bg-white rounded-lg">
                <img src={qrDataUrl} alt="Wallet QR Code" className="w-32 h-32" />
              </div>
              <div className="text-xs opacity-60 mt-2">
                Scan to receive QuantumCoin™
              </div>
            </div>
          )}

          {/* Security Notice */}
          <div className="mt-6 p-3 bg-amber-900/20 border border-amber-600/30 rounded-lg">
            <div className="text-amber-300 text-sm font-medium mb-1">🛡️ Security Notice</div>
            <div className="text-xs opacity-80">
              • Back up your wallet immediately after generation<br/>
              • Store backup offline (USB drive, paper)<br/>
              • Never share your private keys with anyone<br/>
              • RevStop™ provides additional transaction protection<br/>
              • {backendConnected ? 'Connected to secure Rust backend' : 'Running in local mode'}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
