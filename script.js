function login() {
  const username = document.getElementById("login-username").value;
  const password = document.getElementById("login-password").value;

  fetch("https://quantumcoin-ithu.onrender.com/api/login", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ username, password }),
  })
    .then(res => res.json())
    .then(data => alert(data.message || "Logged in"))
    .catch(err => alert("Login error"));
}

function register() {
  const username = document.getElementById("register-username").value;
  const password = document.getElementById("register-password").value;
  const agreed = document.getElementById("terms-agree-box").checked;

  if (!agreed) {
    alert("You must agree to the terms and conditions to register.");
    return;
  }

  fetch("https://quantumcoin-ithu.onrender.com/api/register", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ username, password }),
  })
    .then(res => res.json())
    .then(data => alert(data.message || "Account created"))
    .catch(err => alert("Registration error"));
}

function refreshBalance() {
  fetch("https://quantumcoin-ithu.onrender.com/api/balance")
    .then(res => res.json())
    .then(data => {
      document.getElementById("wallet-balance").textContent = data.balance;
      document.getElementById("wallet-address").textContent = data.address;
    });
}

function send() {
  const recipient = document.getElementById("recipient").value;
  const amount = document.getElementById("amount").value;

  fetch("https://quantumcoin-ithu.onrender.com/api/send", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ recipient, amount }),
  })
    .then(res => res.json())
    .then(data => alert(data.message || "Sent"))
    .catch(err => alert("Send failed"));
}

function showTerms() {
  document.getElementById("terms-modal").classList.remove("hidden");
}

function closeTerms() {
  document.getElementById("terms-modal").classList.add("hidden");
}