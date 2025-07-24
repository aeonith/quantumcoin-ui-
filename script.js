const API_BASE = "https://quantumcoin-ithu.onrender.com";

// Utility to show/hide sections
function showSection(id) {
    document.querySelectorAll('.section').forEach(el => el.style.display = "none");
    document.getElementById(id).style.display = "block";
}

// Load wallet data
async function loadWallet() {
    const token = localStorage.getItem('token');
    if (!token) return;

    try {
        const res = await fetch(`${API_BASE}/wallet`, {
            headers: { Authorization: `Bearer ${token}` }
        });
        const data = await res.json();

        document.getElementById("wallet-address").textContent = data.address;
        document.getElementById("wallet-balance").textContent = `${data.balance} QTC`;
        localStorage.setItem('address', data.address);
    } catch (err) {
        console.error("Wallet load failed", err);
        document.getElementById("wallet-address").textContent = "Error";
        document.getElementById("wallet-balance").textContent = "Loading...";
    }
}

// Login
async function login() {
    const username = document.getElementById("login-username").value;
    const password = document.getElementById("login-password").value;

    const res = await fetch(`${API_BASE}/login`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username, password })
    });

    const result = await res.json();

    if (res.ok && result.token) {
        localStorage.setItem('token', result.token);
        showSection("wallet-ui");
        loadWallet();
    } else {
        alert(result.message || "Login failed");
    }
}

// Register
async function register() {
    const username = document.getElementById("register-username").value;
    const password = document.getElementById("register-password").value;

    const res = await fetch(`${API_BASE}/register`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username, password })
    });

    const result = await res.json();

    if (res.ok) {
        alert("Registered. Please log in.");
        showSection("auth");
    } else {
        alert(result.message || "Registration failed");
    }
}

// Send QTC
async function sendCoins() {
    const to = document.getElementById("send-to").value;
    const amount = document.getElementById("send-amount").value;

    const res = await fetch(`${API_BASE}/send`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${localStorage.getItem('token')}`
        },
        body: JSON.stringify({ to, amount })
    });

    const result = await res.json();
    alert(result.message || "Transaction submitted");
    loadWallet();
}

// Mine QTC
async function mine() {
    const res = await fetch(`${API_BASE}/mine`, {
        method: "POST",
        headers: {
            Authorization: `Bearer ${localStorage.getItem('token')}`
        }
    });

    const result = await res.json();
    alert(result.message || "Mining complete");
    loadWallet();
}

// Export Wallet with 2FA
async function exportWallet() {
    const password = prompt("Enter your password for 2FA export:");

    const res = await fetch(`${API_BASE}/export`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${localStorage.getItem('token')}`
        },
        body: JSON.stringify({ password })
    });

    const result = await res.json();
    if (res.ok) {
        alert("Exported:\n" + JSON.stringify(result));
    } else {
        alert(result.message || "Export failed");
    }
}

// Toggle RevStop
async function toggleRevStop(enable) {
    const endpoint = enable ? "/revstop/enable" : "/revstop/disable";
    const password = enable ? prompt("Enter password to enable RevStop:") : prompt("Password to disable RevStop:");

    const res = await fetch(`${API_BASE}${endpoint}`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${localStorage.getItem('token')}`
        },
        body: JSON.stringify({ password })
    });

    const result = await res.json();
    alert(result.message || "Updated");
}

// Show Last 5 Transactions
async function showLastTransactions() {
    const res = await fetch(`${API_BASE}/transactions`, {
        headers: { Authorization: `Bearer ${localStorage.getItem('token')}` }
    });

    const data = await res.json();
    const list = document.getElementById("tx-list");
    list.innerHTML = "";

    (data.transactions || []).slice(-5).forEach(tx => {
        const item = document.createElement("li");
        item.textContent = `${tx.amount} QTC to ${tx.to}`;
        list.appendChild(item);
    });
}

// Auto-login check on load
window.onload = function () {
    if (localStorage.getItem('token')) {
        showSection("wallet-ui");
        loadWallet();
    } else {
        showSection("auth");
    }
};