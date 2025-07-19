// ——— fetch live data ———
const priceEl  = document.getElementById('price');
const changeEl = document.getElementById('change');
const ctx       = document.getElementById('priceChart').getContext('2d');
let chart;

// CoinGecko IDs (swap in your token ID once listed)
const COIN_ID = 'quantumcoin'; // placeholder

async function fetchData(days = 1) {
  const now = Math.floor(Date.now() / 1000);
  const from = now - days*24*60*60;
  const url = `https://api.coingecko.com/api/v3/coins/${COIN_ID}/market_chart/range?vs_currency=usd&from=${from}&to=${now}`;
  const res = await fetch(url);
  return res.json();
}

async function updateChart(days = 1) {
  const data = await fetchData(days);
  const prices = data.prices.map(p => ({ x: new Date(p[0]), y: p[1] }));
  const latest = prices[prices.length-1].y;
  const previous = prices[0].y;
  const diff = latest - previous;
  const pct  = (diff / previous) * 100;

  priceEl.textContent = `$${latest.toFixed(2)}`;
  changeEl.textContent = `${diff>=0?'▲':'▼'}${Math.abs(diff).toFixed(2)} (${pct.toFixed(2)}%)`;
  changeEl.style.color = diff >= 0 ? '#33cc33' : '#ff3333';

  if (chart) {
    chart.data.datasets[0].data = prices;
    chart.update();
  } else {
    chart = new Chart(ctx, {
      type: 'line',
      data: { datasets: [{
        data: prices,
        borderColor: '#3399ff',
        borderWidth: 2,
        pointRadius: 0,
        fill: false,
      }]},
      options: {
        scales: {
          x: { type: 'time', time: { unit: 'hour', tooltipFormat: 'MMM d, h:mm a' } },
          y: { grid: { color: 'rgba(255,255,255,.1)' } }
        },
        plugins: { legend: false, tooltip: { backgroundColor: '#222' } },
        maintainAspectRatio: false
      }
    });
  }
}

// interval buttons
document.querySelectorAll('.intervals button').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelector('.intervals button.active').classList.remove('active');
    btn.classList.add('active');
    const daysMap = { '1h': 1/24, '1d': 1, '1w': 7, '1y': 365 };
    updateChart(daysMap[btn.dataset.interval]);
  });
});

// initial load
updateChart(1);

// ——— signup form handler ———
document.getElementById('signupForm').addEventListener('submit', e => {
  e.preventDefault();
  const email = document.getElementById('email').value;
  alert(`Thanks! We’ll let you know when QuantumCoin™ is live: ${email}`);
  // TODO → hook into your mailing-list backend
});