const API = 'https://quantumcoin-ithu.onrender.com';

async function loadWallet() {
  const res = await fetch(`${API}/wallet`);
  const data = await res.json();
  document.getElementById('walletAddress').textContent = data.address;
  document.getElementById('walletBalance').textContent = data.balance;
}

async function sendCoins() {
  const to = document.getElementById('sendTo').value;
  const amount = document.getElementById('sendAmount').value;
  const res = await fetch(`${API}/send`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ to, amount }),
  });
  const result = await res.json();
  alert(result.message);
  loadWallet();
}

function showTerms() {
  document.getElementById('termsPopup').style.display = 'flex';
}

function closeTerms() {
  document.getElementById('termsPopup').style.display = 'none';
}

async function register() {
  const user = document.getElementById('username').value;
  const pass = document.getElementById('password').value;
  const agree = document.getElementById('agree').checked;

  if (!agree) return alert("Please agree to the Terms & Conditions.");

  const res = await fetch(`${API}/register`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ user, pass }),
  });

  const result = await res.json();
  alert(result.message);
}