// src/wallet.js
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

function generateDummyKeypair() {
  const random = Math.random().toString(36).substring(2);
  return {
    publicKey: btoa("QTC_" + random),
    privateKey: btoa("PRIV_" + random),
  };
}