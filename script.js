const API_URL = "https://quantumcoin-ui-1rust1.onrender.com";

async function register() {
  const email = document.getElementById("register-email").value;
  const password = document.getElementById("register-password").value;
  const terms = document.getElementById("terms").checked;

  if (!email || !password || !terms) {
    alert("Please complete all fields and accept the terms.");
    return;
  }

  const res = await fetch(`${API_URL}/register`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email, password })
  });

  const data = await res.json();
  if (res.ok) {
    alert("Registered successfully!");
    localStorage.setItem("email", email);
    window.location.href = "dashboard.html";
  } else {
    alert(data.error || "Registration failed.");
  }
}

async function login() {
  const email = document.getElementById("login-email").value;
  const password = document.getElementById("login-password").value;

  if (!email || !password) {
    alert("Enter email and password.");
    return;
  }

  const res = await fetch(`${API_URL}/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ email, password })
  });

  const data = await res.json();
  if (res.ok) {
    alert("Login successful.");
    localStorage.setItem("email", email);
    localStorage.setItem("wallet", data.wallet_address);
    window.location.href = "dashboard.html";
  } else {
    alert(data.error || "Login failed.");
  }
}