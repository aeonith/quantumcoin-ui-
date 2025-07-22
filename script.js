// Login/Signup Toggle
function toggleAuthMode() {
  const title = document.getElementById('authTitle');
  const btn = document.getElementById('authBtn');
  const confirm = document.getElementById('confirmPassword');
  const toggle = document.getElementById('toggleText');

  if (title.innerText === 'Sign Up') {
    title.innerText = 'Login';
    btn.innerText = 'Login';
    confirm.style.display = 'none';
    toggle.innerText = 'Sign Up';
  } else {
    title.innerText = 'Sign Up';
    btn.innerText = 'Create Account';
    confirm.style.display = 'block';
    toggle.innerText = 'Login';
  }
}
function switchMode() {
  toggleAuthMode();
}

// Reveal Animations on Scroll
const reveals = document.querySelectorAll('.section');
const observer = new IntersectionObserver(entries => {
  entries.forEach(entry => {
    if (entry.isIntersecting) {
      entry.target.classList.add('reveal');
    }
  });
}, { threshold: 0.2 });

reveals.forEach(section => {
  observer.observe(section);
});