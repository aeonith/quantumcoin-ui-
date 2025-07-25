// ⏳ Background Video - Set 3 second loop manually
const bgVideo = document.getElementById("bg-video");
bgVideo.addEventListener("loadedmetadata", () => {
  bgVideo.currentTime = 0;
  setInterval(() => {
    bgVideo.currentTime = 0;
    bgVideo.play();
  }, 3000);
});

// 🔐 Login Function
async function login() {
  const username = document.getElementById("login-username").value.trim();
  const password = document.getElementById("login-password").value.trim();

  if (!username || !password) {
    alert("Please enter both username and password.");
    return;
  }

  const res = await fetch("/api/login", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ username, password })
  });

  const data = await res.json();
  if (data.success) {
    alert("Login successful!");
    localStorage.setItem("wallet", data.wallet);
    loadWallet(data.wallet);
  } else {
    alert("Login failed: " + data.message);
  }
}

// 🧾 Register Function (Shows Modal)
function register() {
  const modal = document.getElementById("terms-modal");
  modal.style.display = "block";
}

// 🧾 Accept Terms and Proceed with Registration
async function acceptTermsAndRegister() {
  const checkbox = document.getElementById("terms-checkbox-modal");
  if (!checkbox.checked) {
    alert("You must agree to the Terms & Conditions.");
    return;
  }

  const username = document.getElementById("register-username").value.trim();
  const password = document.getElementById("register-password").value.trim();

  if (!username || !password) {
    alert("Enter a username and password.");
    return;
  }

  const res = await fetch("/api/register", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ username, password })
  });

  const data = await res.json();
  if (data.success) {
    alert("Registration successful!");
    localStorage.setItem("wallet", data.wallet);
    loadWallet(data.wallet);
    document.getElementById("terms-modal").style.display = "none";
  } else {
    alert("Registration failed: " + data.message);
  }
}

// 👛 Load Wallet Info
async function loadWallet(address) {
  document.getElementById("wallet-address").textContent = address;

  const res = await fetch(`/api/balance/${address}`);
  const data = await res.json();
  document.getElementById("wallet-balance").textContent = data.balance || 0;
}

// 🔁 Refresh Wallet
function refreshBalance() {
  const wallet = localStorage.getItem("wallet");
  if (wallet) loadWallet(wallet);
  else alert("Login required to load balance.");
}

// 🚀 Send Coins
async function send() {
  const from = localStorage.getItem("wallet");
  const to = document.getElementById("recipient").value.trim();
  const amount = document.getElementById("amount").value.trim();

  if (!from || !to || !amount) {
    alert("All fields are required.");
    return;
  }

  const res = await fetch("/api/send", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ from, to, amount })
  });

  const data = await res.json();
  if (data.success) {
    alert("Transaction sent!");
    refreshBalance();
  } else {
    alert("Failed to send: " + data.message);
  }
}

// ❌ Close Modal
window.onclick = function (event) {
  const modal = document.getElementById("terms-modal");
  if (event.target == modal) modal.style.display = "none";
};