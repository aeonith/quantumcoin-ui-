// wallet.js

export function getOrCreateWallet() {
  let wallet = JSON.parse(localStorage.getItem("wallet"));
  if (!wallet) {
    const keypair = generateDummyKeypair(); // Replace this with real post-quantum wallet logic
    wallet = {
      publicKey: keypair.publicKey,
      privateKey: keypair.privateKey,
    };
    localStorage.setItem("wallet", JSON.stringify(wallet));
  }
  return wallet;
}

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

function generateDummyKeypair() {
  const random = Math.random().toString(36).substring(2);
  return {
    publicKey: btoa("QTC_" + random),
    privateKey: btoa("PRIV_" + random),
  };
}