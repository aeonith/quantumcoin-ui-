const API_URL = "https://quantumcoin-ui-1rust1.onrender.com";

async function register() {
  const email = document.getElementById("register-email").value;
  const password = document.getElementById("register-password").value;
  const agreed = document.getElementById("terms").checked;

  if (!email || !password || !agreed) {
    alert("Please fill in all fields and agree to the Terms.");
    return;
  }

  const res = await fetch(`${API_URL}/register`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email, password })
  });

  const data = await res.json();
  if (res.ok) {
    alert("Account created. Now login.");
  } else {
    alert(data.error || "Registration failed.");
  }
}

async function login() {
  const email = document.getElementById("login-email").value;
  const password = document.getElementById("login-password").value;

  if (!email || !password) {
    alert("Please enter email and password.");
    return;
  }

  const res = await fetch(`${API_URL}/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email, password })
  });

  const data = await res.json();

  if (res.ok && data.wallet_address) {
    document.getElementById("btc-address").textContent = data.wallet_address;
    document.getElementById("btc-info").style.display = "block";
    alert("Login successful.");
  } else {
    alert(data.error || "Login failed.");
  }
}