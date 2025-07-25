const API = "https://quantumcoin-ithu.onrender.com";

async function login() {
  const u = document.getElementById("login-username").value;
  const p = document.getElementById("login-password").value;
  const res = await fetch(`${API}/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ username: u, password: p })
  });
  const { success, token } = await res.json();
  if (!success) return alert("Login failed");
  localStorage.setItem("token", token);
  alert("Logged in!");
  loadWallet();
}

async function register() {
  if (!document.getElementById("terms-checkbox").checked)
    return alert("You must agree to terms");
  const u = document.getElementById("register-username").value;
  const p = document.getElementById("register-password").value;
  const res = await fetch(`${API}/register`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ username: u, password: p })
  });
  const { success } = await res.json();
  if (!success) return alert("Registration failed");
  alert("Account created! Please login.");
}

// After login, grab wallet & balance:
async function loadWallet() {
  const token = localStorage.getItem("token");
  const hdr = { "Authorization": `Bearer ${token}` };

  let res = await fetch(`${API}/wallet/address`, { headers: hdr });
  document.getElementById("wallet-address").innerText = await res.text();

  res = await fetch(`${API}/wallet/balance`, { headers: hdr });
  document.getElementById("wallet-balance").innerText = await res.text();
}

async function refreshBalance() {
  await loadWallet();
}

async function send() {
  const token = localStorage.getItem("token");
  const hdr = {
    "Authorization": `Bearer ${token}`,
    "Content-Type": "application/json"
  };
  const recipient = document.getElementById("recipient").value;
  const amount = parseFloat(document.getElementById("amount").value);
  const res = await fetch(`${API}/send`, {
    method: "POST",
    headers: hdr,
    body: JSON.stringify({ recipient, amount })
  });
  if (res.ok) {
    alert("Transaction sent!");
    refreshBalance();
  } else {
    alert("Send failed");
  }
}

// On page load, if we have a token, auto-load wallet:
window.addEventListener("DOMContentLoaded", () => {
  if (localStorage.getItem("token")) loadWallet();
});