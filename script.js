// Animate wallet balance from 0 → actual
function refreshBalance() {
  const el = document.getElementById('walletBalance');
  const target = 1250;  // replace with real API data
  let i = 0;
  el.textContent = '0';
  const step = Math.ceil(target / 30);
  const ticker = setInterval(() => {
    i += step;
    if (i >= target) {
      i = target;
      clearInterval(ticker);
    }
    el.textContent = i;
  }, 30);
}

// Simple scroll-in observer
const observer = new IntersectionObserver(entries => {
  entries.forEach(e => {
    if (e.isIntersecting) {
      e.target.classList.add('visible');
      observer.unobserve(e.target);
    }
  });
}, { threshold: 0.2 });

document.querySelectorAll('header, .panel, footer')
  .forEach(el => observer.observe(el));