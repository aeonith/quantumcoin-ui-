const api = "https://quantumcoin-ithu.onrender.com";

async function refreshWallet() {
  const res = await fetch(`${api}/wallet`);
  const data = await res.json();
  document.getElementById("walletAddress").textContent = data.address || "Unavailable";
  document.getElementById("walletBalance").textContent = data.balance || "0";
}

async function sendTransaction() {
  const recipient = document.getElementById("recipient").value;
  const amount = document.getElementById("amount").value;
  await fetch(`${api}/send`, {
    method: "POST",
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ recipient, amount })
  });
  alert("Transaction Sent!");
}

async function register() {
  const username = document.getElementById("username").value;
  const password = document.getElementById("password").value;
  const agreed = document.getElementById("agreeTerms").checked;

  if (!agreed) {
    alert("Please agree to the Terms & Conditions before registering.");
    return;
  }

  await fetch(`${api}/register`, {
    method: "POST",
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ username, password })
  });
  alert("Account created!");
}

function showTerms() {
  document.getElementById("termsModal").style.display = "block";
}

function closeTerms() {
  document.getElementById("termsModal").style.display = "none";
}

// Load wallet on first render
refreshWallet();