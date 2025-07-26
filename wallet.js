export async function getOrCreateWallet() {
  let wallet = JSON.parse(localStorage.getItem("wallet"));
  if (!wallet) {
    // 🧠 REAL backend call to Rust
    const res = await fetch('https://quantumcoin-ui-1live.onrender.com/api/create-wallet');
    if (!res.ok) {
      alert("❌ Failed to generate secure wallet.");
      throw new Error("Failed to create wallet");
    }
    wallet = await res.json();
    localStorage.setItem("wallet", JSON.stringify(wallet));
  }
  return wallet;
}
window.getOrCreateWallet = getOrCreateWallet;

export function showWalletInfo() {
  const wallet = JSON.parse(localStorage.getItem("wallet"));
  if (wallet) {
    alert("✅ Your QuantumCoin Wallet\n\n" +
          "Public Key:\n" + wallet.publicKey + "\n\n" +
          "Private Key:\n" + wallet.privateKey + "\n\n" +
          "(Keep your private key secret!)");
  } else {
    alert("⚠️ No wallet found.");
  }
}
window.showWalletInfo = showWalletInfo;