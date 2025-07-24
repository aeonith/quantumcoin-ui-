document.getElementById('register-form').addEventListener('submit', async (e) => {
  e.preventDefault();

  const email = document.getElementById('reg-email').value;
  const password = document.getElementById('reg-password').value;
  const confirm = document.getElementById('reg-confirm').value;

  if (password !== confirm) {
    alert('Passwords do not match.');
    return;
  }

  try {
    const response = await fetch('https://quantumcoin-ui-1live.onrender.com/api/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, password }),
    });

    const data = await response.json();

    if (response.ok) {
      alert('Registration successful! Redirecting to login...');
      window.location.href = 'login.html';
    } else {
      alert(`Registration failed: ${data.message || 'Unknown error'}`);
    }
  } catch (error) {
    console.error('Error during registration:', error);
    alert('Network error. Please try again later.');
  }
});