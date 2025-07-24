const API_BASE = "https://quantumcoin-ithu.onrender.com"; // Backend URL

async function registerUser() {
  const username = document.getElementById("register-username")?.value.trim();
  const password = document.getElementById("register-password")?.value.trim();
  const terms = document.getElementById("terms").checked;

  if (!username || !password) {
    return alert("Please enter both username and password.");
  }

  if (!terms) {
    return alert("You must agree to the Terms & Conditions.");
  }

  try {
    const response = await fetch(`${API_BASE}/api/register`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ username, password })
    });

    const result = await response.json();
    if (response.ok) {
      alert("✅ Registration successful!");
      showDashboard(result);
    } else {
      alert("❌ " + result.message || "Registration failed");
    }
  } catch (error) {
    alert("❌ Register failed: " + error.message);
  }
}

async function loginUser() {
  const username = document.getElementById("login-username")?.value.trim();
  const password = document.getElementById("login-password")?.value.trim();

  if (!username || !password) {
    return alert("Please enter both username and password.");
  }

  try {
    const response = await fetch(`${API_BASE}/api/login`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ username, password })
    });

    const result = await response.json();
    if (response.ok) {
      alert("✅ Login successful!");
      showDashboard(result);
    } else {
      alert("❌ " + result.message || "Login failed");
    }
  } catch (error) {
    alert("❌ Login failed: " + error.message);
  }
}

function showDashboard(data) {
  const dashboard = document.getElementById("dashboard");
  dashboard.style.display = "block";
  dashboard.innerHTML = `
    <h3>Welcome, ${data.username || "user"}!</h3>
    <p>Your QuantumCoin Wallet: <strong>${data.wallet || "N/A"}</strong></p>
    <p>Balance: <strong>${data.balance || 0} QTC</strong></p>
  `;
}