document.getElementById('register-form').addEventListener('submit', async (e) => {
  e.preventDefault();

  const username = document.getElementById('register-username').value;
  const password = document.getElementById('register-password').value;
  const agreed = document.getElementById('terms-checkbox').checked;

  if (!username || !password) {
    alert('Please fill out all fields.');
    return;
  }

  if (!agreed) {
    alert('You must agree to the Terms & Conditions.');
    return;
  }

  try {
    const response = await fetch('https://quantumcoin-ui-1rust1.onrender.com/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, password }),
    });

    const data = await response.json();

    if (response.ok) {
      alert('Registration successful!');
    } else {
      alert(`Registration failed: ${data.error || 'Unknown error'}`);
    }
  } catch (error) {
    console.error('Error during registration:', error);
    alert('Network error. Please try again later.');
  }
});