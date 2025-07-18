document.addEventListener('DOMContentLoaded', () => {
  const priceEl = document.getElementById('qtc-price');
  const changeEl = document.getElementById('qtc-change');

  // Placeholder for future API hook
  async function fetchLiveQTC() {
    try {
      const res = await fetch('https://api.quantumcoin.io/price'); // Replace with real endpoint
      const data = await res.json();

      priceEl.textContent = `$${data.price.toFixed(2)}`;
      changeEl.textContent = `${data.change > 0 ? '▲' : '▼'} ${data.change.toFixed(2)} (${data.percent.toFixed(2)}%)`;
      changeEl.style.color = data.change > 0 ? '#4fd47e' : '#f55252';
    } catch (err) {
      console.error("Live price fetch failed:", err);
    }
  }

  // Simulate now, hook real later
  fetchLiveQTC();
});