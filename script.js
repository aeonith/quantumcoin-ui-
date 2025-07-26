const backendURL = "https://quantumcoin-ui-1rust1.onrender.com";

async function register() {
  const email = document.getElementById("registerEmail").value;
  const password = document.getElementById("registerPassword").value;

  try {
    const res = await fetch(`${backendURL}/register`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email, password }),
    });

    const data = await res.json();
    if (res.ok) {
      alert("🎉 Registration successful!");
    } else {
      alert(`❌ Failed: ${data.message || "Unknown error"}`);
    }
  } catch (err) {
    console.error("Error:", err);
    alert("❌ Registration failed. Backend not reachable.");
  }
}

async function login() {
  const email = document.getElementById("loginEmail").value;
  const password = document.getElementById("loginPassword").value;

  try {
    const res = await fetch(`${backendURL}/login`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email, password }),
    });

    const data = await res.json();
    if (res.ok && data.wallet_address) {
      alert(`✅ Login successful! Your Wallet: ${data.wallet_address}`);
      // You could store to localStorage or redirect here
    } else {
      alert("❌ Login failed. Check credentials.");
    }
  } catch (err) {
    console.error("Login Error:", err);
    alert("❌ Login failed. Backend not reachable.");
  }
}