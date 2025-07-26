// wallet.js
export function getOrCreateWallet() {
  let wallet = JSON.parse(localStorage.getItem("wallet"));
  if (!wallet) {
    const keypair = generateKeypair(); // Real keypair generation goes here eventually
    wallet = {
      publicKey: keypair.publicKey,
      privateKey: keypair.privateKey,
    };
    localStorage.setItem("wallet", JSON.stringify(wallet));
  }
  return wallet;
}

function generateKeypair() {
  const randomId = Math.random().toString(36).substring(2) + Date.now();
  return {
    publicKey: btoa("QTC_" + randomId),
    privateKey: btoa("PRIV_" + randomId),
  };
}

export function showWalletInfo() {
  const wallet = getOrCreateWallet();
  const infoBox = document.createElement("div");
  infoBox.className = "wallet-info";
  infoBox.innerHTML = `
    <h3>🎉 Wallet Created</h3>
    <p><strong>Public Address:</strong> ${wallet.publicKey}</p>
    <p class="warning">💾 This wallet is stored locally. Please back it up if needed.</p>
  `;
  document.querySelector(".box").appendChild(infoBox);
}