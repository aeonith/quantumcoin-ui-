document.addEventListener("DOMContentLoaded", () => {
  // Placeholder live price data — replace with CoinGecko later
  const priceEl = document.querySelector(".price");
  const changeEl = document.querySelector(".change");

  let fakePrice = 749.23;
  let fakeChange = 1.98;

  // Display fake data for now
  priceEl.textContent = `$${fakePrice.toFixed(2)}`;
  changeEl.innerHTML = `<span style="color:#00ff94;">▲ ${fakeChange.toFixed(2)}%</span>`;

  // Chart Setup using Chart.js
  const ctx = document.getElementById("priceChart").getContext("2d");
  new Chart(ctx, {
    type: "line",
    data: {
      labels: ["1H", "1D", "1W", "1Y"],
      datasets: [{
        label: "QTC Price",
        data: [710, 735, 749.23, 749.23],
        borderColor: "#00bfff",
        backgroundColor: "rgba(0, 191, 255, 0.1)",
        tension: 0.4
      }]
    },
    options: {
      scales: {
        x: { display: false },
        y: { display: false }
      },
      plugins: {
        legend: { display: false }
      }
    }
  });

  // Placeholder for CoinGecko Integration (once listed)
  /*
  async function fetchLivePrice() {
    try {
      const res = await fetch('https://api.coingecko.com/api/v3/simple/price?ids=quantumcoin&vs_currencies=usd');
      const data = await res.json();
      const livePrice = data.quantumcoin.usd;
      priceEl.textContent = `$${livePrice.toFixed(2)}`;
      // Add your price change logic here
    } catch (err) {
      console.error('Live price fetch failed:', err);
    }
  }

  setInterval(fetchLivePrice, 60000); // Poll every minute
  */

  // Buy button placeholder
  document.querySelector(".btn-primary").addEventListener("click", () => {
    alert("Buy QuantumCoin™ coming soon. Stay tuned!");
  });

  // Email Signup handler
  document.querySelector("form").addEventListener("submit", e => {
    e.preventDefault();
    const email = document.querySelector("input[type='email']").value;
    alert(`Thanks for signing up, ${email}!`);
    document.querySelector("input[type='email']").value = "";
  });
});