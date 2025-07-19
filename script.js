document.addEventListener('DOMContentLoaded', () => {
  const priceEl = document.getElementById('price');
  const changeEl = document.getElementById('change');
  const ctx = document.getElementById('price-chart').getContext('2d');
  let chart;

  function fetchData(days) {
    // 1) Current price & 24h change
    fetch('https://api.coingecko.com/api/v3/simple/price?ids=quantumcoin&vs_currencies=usd&include_24hr_change=true')
      .then(r => r.json())
      .then(data => {
        const p = data.quantumcoin.usd;
        const c = data.quantumcoin.usd_24h_change;
        priceEl.textContent = `$${p.toFixed(2)}`;
        const sign = c >= 0 ? '▲' : '▼';
        changeEl.textContent = `${sign}${Math.abs(c).toFixed(2)} (${c.toFixed(2)}%)`;
      })
      .catch(console.error);

    // 2) Historical for chart
    fetch(`https://api.coingecko.com/api/v3/coins/quantumcoin/market_chart?vs_currency=usd&days=${days}`)
      .then(r => r.json())
      .then(data => {
        const labels = data.prices.map(p => new Date(p[0]).toLocaleTimeString());
        const values = data.prices.map(p => p[1]);
        if (chart) chart.destroy();
        chart = new Chart(ctx, {
          type: 'line',
          data: {
            labels,
            datasets: [{
              data: values,
              borderColor: '#00aaff',
              fill: false,
              pointRadius: 0
            }]
          },
          options: {
            scales: { x: { display: false }, y: { ticks: { color: 'white' } } },
            plugins: { legend: { display: false } }
          }
        });
      })
      .catch(console.error);
  }

  // timeframe buttons
  document.querySelectorAll('.timeframe-buttons button').forEach(btn => {
    btn.addEventListener('click', () => {
      const days = btn.dataset.days;
      fetchData(days);
    });
  });

  // initial load (1D)
  fetchData(1);

  // signup form
  document.getElementById('signup-form').addEventListener('submit', e => {
    e.preventDefault();
    const email = document.getElementById('email').value;
    // → hook this to your backend/email-capture logic
    alert(`Thanks for signing up, ${email}!`);
  });

  // buy button
  document.getElementById('buy-btn').addEventListener('click', () => {
    // → hook this to your on-chain purchase flow
    alert('Redirecting to your QuantumCoin™ purchase flow…');
  });
});