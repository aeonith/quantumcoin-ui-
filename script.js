document.getElementById('signup-form').addEventListener('submit', function (e) {
  e.preventDefault();
  alert('Thanks for signing up! You’ll be notified when QuantumCoin goes live.');
});

document.getElementById('buy-btn').addEventListener('click', function () {
  alert('QuantumCoin is not yet listed. Please check back soon!');
});

// OPTIONAL: Future CoinGecko connection
async function fetchQuantumPrice() {
  try {
    const res = await fetch('https://api.coingecko.com/api/v3/simple/price?ids=quantumcoin&vs_currencies=usd');
    const data = await res.json();
    if (data.quantumcoin && data.quantumcoin.usd) {
      document.getElementById('price').innerText = `$${data.quantumcoin.usd.toFixed(2)}`;
    }
  } catch (e) {
    console.log('Waiting for CoinGecko listing...');
  }
}

// Call this function once listed on CoinGecko
// fetchQuantumPrice();