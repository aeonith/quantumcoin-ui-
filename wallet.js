// wallet.js

export function getOrCreateWallet() {
  let wallet = JSON.parse(localStorage.getItem("wallet"));
  if (!wallet) {
    const keypair = generateDummyKeypair();
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

export async function getBalance(publicKey) {
  try {
    const response = await fetch(`https://quantumcoin-u-1rust1.onrender.com/balance/${publicKey}`);
    const data = await response.json();
    return data.balance;
  } catch (error) {
    console.error("Failed to fetch balance", error);
    return 0;
  }
}