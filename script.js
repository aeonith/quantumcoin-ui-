function toggleAuthMode() {
  const title = document.getElementById('authTitle');
  const btn = document.getElementById('authBtn');
  const confirmPass = document.getElementById('confirmPassword');
  const toggle = document.getElementById('toggleText');

  if (title.innerText === 'Sign Up') {
    title.innerText = 'Login';
    btn.innerText = 'Login';
    confirmPass.style.display = 'none';
    toggle.innerText = 'Sign Up';
  } else {
    title.innerText = 'Sign Up';
    btn.innerText = 'Create Account';
    confirmPass.style.display = 'block';
    toggle.innerText = 'Login';
  }
}
function switchMode() {
  toggleAuthMode();
}