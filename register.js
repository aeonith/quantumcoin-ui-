document.getElementById('register-form').addEventListener('submit', async (e) => {
  e.preventDefault();

  const email = document.getElementById('register-email').value;
  const password = document.getElementById('register-password').value;
  const confirm = document.getElementById('register-confirm').value;
  const agreed = document.getElementById('terms-checkbox').checked;

  if (!agreed) {
    return alert("You must agree to the Terms & Conditions.");
  }

  if (password !== confirm) {
    return alert("Passwords do not match.");
  }

  try {
    const response = await fetch('https://quantumcoin-ui-1rust1.onrender.com/register', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, password })
    });

    const data = await response.json();

    if (response.ok) {
      alert("✅ Registered successfully!");
      localStorage.setItem("walletAddress", data.walletAddress);
      window.location.href = "wallet.html";
    } else {
      alert(`❌ Registration failed: ${data.error || "Unknown error"}`);
    }
  } catch (err) {
    console.error(err);
    alert("🌐 Network error. Try again later.");
  }
});