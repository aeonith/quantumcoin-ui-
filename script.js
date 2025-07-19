// Placeholder wallet data
let wallet = {
    address: "TNZCY5NT+GORGIA+JCVIGAJUIBM...QNSATLVTHNBWXMZA783YP/ALNCM2GEAO1TZ==",
    balance: 1250000.00, // Genesis allocation
    price: 0.25 // USD per QTC
};

// Update wallet info display
function updateWalletDisplay() {
    document.getElementById("walletAddress").innerText = wallet.address;
    document.getElementById("walletBalance").innerText = `${wallet.balance.toLocaleString()} QTC`;
    document.getElementById("usdBalance").innerText = `$${(wallet.balance * wallet.price).toFixed(2)} USD`;
}

// Placeholder for buying coins
function buyCoins() {
    alert("Buy functionality is under development. Will be connected to Rust backend + payment gateway.");
}

// Placeholder for sending coins
function sendCoins() {
    alert("Send logic will hook into Rust transaction engine.");
}

// Placeholder for mining
function startMining() {
    alert("Mining is CPU-intensive and protected by RevStop. Coming soon.");
}

// Placeholder for RevStop status
function checkRevStop() {
    alert("RevStop security is ACTIVE. Only owner can disable via USB + Password.");
}

// Navigate to Terms and Privacy (when linked)
function openLegal(page) {
    alert(`Redirecting to ${page}.html — (Under Construction)`);
}

// KYC status check
function startKYC() {
    alert("KYC verification required for exchange integration. Feature under construction.");
}

// Load current price (ready for CoinGecko once listed)
function loadLivePrice() {
    const coingeckoAPI = `https://api.coingecko.com/api/v3/simple/price?ids=quantumcoin&vs_currencies=usd`;

    fetch(coingeckoAPI)
        .then(response => response.json())
        .then(data => {
            if (data.quantumcoin && data.quantumcoin.usd) {
                wallet.price = data.quantumcoin.usd;
                updateWalletDisplay();
            } else {
                console.warn("QuantumCoin not yet listed. Using placeholder price.");
            }
        })
        .catch(() => {
            console.warn("Could not fetch live price. CoinGecko may not be live yet.");
        });
}

// Event bindings
document.addEventListener("DOMContentLoaded", () => {
    updateWalletDisplay();
    loadLivePrice();

    document.getElementById("buyBtn").addEventListener("click", buyCoins);
    document.getElementById("sendBtn").addEventListener("click", sendCoins);
    document.getElementById("mineBtn").addEventListener("click", startMining);
    document.getElementById("revStopBtn").addEventListener("click", checkRevStop);
    document.getElementById("kycBtn").addEventListener("click", startKYC);
    document.getElementById("termsBtn").addEventListener("click", () => openLegal("terms"));
    document.getElementById("privacyBtn").addEventListener("click", () => openLegal("privacy"));
});