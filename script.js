const ctx = document.getElementById('priceChart').getContext('2d');
const chart = new Chart(ctx, {
  type: 'line',
  data: {
    labels: ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'],
    datasets: [{
      label: 'Price',
      data: [700, 705, 710, 715, 730, 740, 749],
      borderColor: '#57a8ff',
      fill: false,
      tension: 0.4,
    }]
  },
  options: {
    responsive: true,
    plugins: {
      legend: { display: false }
    },
    scales: {
      x: {
        display: false
      },
      y: {
        display: false
      }
    }
  }
});
