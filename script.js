const API_BASE_URL = "https://quantumcoin-ithu.onrender.com";

let currentUser = "";

function showRegister() {
  document.getElementById("login-section").classList.add("hidden");
  document.getElementById("register-section").classList.remove("hidden");
}

function showLogin() {
  document.getElementById("register-section").classList.add("hidden");
  document.getElementById("login-section").classList.remove("hidden");
}

function showDashboard() {
  document.getElementById("login-section").classList.add("hidden");
  document.getElementById("register-section").classList.add("hidden");
  document.getElementById("dashboard").classList.remove("hidden");
}

function login() {
  const username = document.getElementById("login-username").value;
  const password = document.getElementById("login-password").value;

  fetch(`${API_BASE_URL}/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ username, password }),
  })
    .then(res => res.json())
    .then(data => {
      if (data.success) {
        currentUser = username;
        document.getElementById("user-label").innerText = username;
        showDashboard();
        loadDashboard();
      } else {
        alert("Login failed: " + (data.message || "Invalid credentials"));
      }
    })
    .catch(err => alert("Error: " + err));
}

function register() {
  const username = document.getElementById("register-username").value;
  const password = document.getElementById("register-password").value;
  const agreed = document.getElementById("terms-checkbox").checked;

  if (!agreed) {
    alert("You must agree to the Terms & Privacy Policy.");
    return;
  }

  fetch(`${API_BASE_URL}/register`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ username, password }),
  })
    .then(res => res.json())
    .then(data => {
      if (data.success) {
        alert("Registration successful. Please log in.");
        showLogin();
      } else {
        alert("Registration failed: " + (data.message || "Try again."));
      }
    })
    .catch(err => alert("Error: " + err));
}

function loadDashboard() {
  getBalance();
  getRevStopStatus();
  getLastTransactions();
}

function getBalance() {
  fetch(`${API_BASE_URL}/balance/${currentUser}`)
    .then(res => res.json())
    .then(data => {
      document.getElementById("balance").innerText = data.balance || "0";
    })
    .catch(err => {
      console.error("Balance fetch error:", err);
      document.getElementById("balance").innerText = "Error";
    });
}

function sendCoins() {
  const to = document.getElementById("send-to").value;
  const amount = parseFloat(document.getElementById("send-amount").value);

  fetch(`${API_BASE_URL}/send`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ from: currentUser, to, amount }),
  })
    .then(res => res.json())
    .then(data => {
      if (data.success) {
        alert("Transaction sent!");
        getBalance();
        getLastTransactions();
      } else {
        alert("Failed to send: " + (data.message || "Unknown error"));
      }
    })
    .catch(err => alert("Error: " + err));
}

function mine() {
  fetch(`${API_BASE_URL}/mine/${currentUser}`)
    .then(res => res.json())
    .then(data => {
      if (data.success) {
        alert("Block mined!");
        getBalance();
        getLastTransactions();
      } else {
        alert("Mine failed: " + (data.message || "Try again later"));
      }
    })
    .catch(err => alert("Error: " + err));
}

function getLastTransactions() {
  fetch(`${API_BASE_URL}/transactions/${currentUser}`)
    .then(res => res.json())
    .then(data => {
      const list = document.getElementById("transactions-list");
      list.innerHTML = "";
      (data.transactions || []).slice(0, 5).forEach(tx => {
        const item = document.createElement("li");
        item.innerText = `To: ${tx.to}, Amount: ${tx.amount}`;
        list.appendChild(item);
      });
    })
    .catch(err => {
      console.error("Transaction fetch error:", err);
    });
}

function getRevStopStatus() {
  fetch(`${API_BASE_URL}/revstop/${currentUser}`)
    .then(res => res.json())
    .then(data => {
      document.getElementById("revstop-status").innerText = data.active ? "Active" : "Not Active";
    })
    .catch(err => {
      document.getElementById("revstop-status").innerText = "Error";
    });
}

function logout() {
  currentUser = "";
  location.reload();
}