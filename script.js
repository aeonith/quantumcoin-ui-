const API = 'https://quantumcoin-ithu.onrender.com'; // your Rust backend

async function login() {
  const user = document.getElementById('login-username').value;
  const pass = document.getElementById('login-password').value;
  // call your /login endpoint (implement on backend)
  try {
    let res = await fetch(`${API}/login`, {
      method:'POST',
      headers:{'Content-Type':'application/json'},
      body: JSON.stringify({user,pass})
    });
    if (!res.ok) throw new Error(await res.text());
    await loadWallet();
    alert('Logged in!');
  } catch(e) {
    alert('Login failed: '+e.message);
  }
}

async function register() {
  const user = document.getElementById('register-username').value;
  const pass = document.getElementById('register-password').value;
  const agreed = document.getElementById('terms-checkbox').checked;
  if (!agreed) return alert('You must agree to Terms & Conditions.');
  try {
    let res = await fetch(`${API}/register`, {
      method:'POST',
      headers:{'Content-Type':'application/json'},
      body: JSON.stringify({user,pass})
    });
    if (!res.ok) throw new Error(await res.text());
    await loadWallet();
    alert('Registered & logged in!');
  } catch(e) {
    alert('Register failed: '+e.message);
  }
}

function toggleForms(){
  document.getElementById('login-box').classList.toggle('hidden');
  document.getElementById('register-box').classList.toggle('hidden');
}

// fetch wallet address & balance
async function loadWallet(){
  document.getElementById('wallet-address').innerText = '…';
  document.getElementById('wallet-balance').innerText = '…';
  try {
    let addr = await fetch(`${API}/address`).then(r=>r.text());
    document.getElementById('wallet-address').innerText = addr;
    let bal  = await fetch(`${API}/balance`).then(r=>r.text());
    document.getElementById('wallet-balance').innerText = bal;
  } catch(e){
    document.getElementById('wallet-address').innerText = 'Error';
    document.getElementById('wallet-balance').innerText = 'Error';
  }
}

// manual refresh
function refreshBalance(){ loadWallet(); }

// send QTC
async function send(){
  const to = document.getElementById('recipient').value;
  const amt= document.getElementById('amount').value;
  try {
    let res = await fetch(`${API}/send`, {
      method:'POST',
      headers:{'Content-Type':'application/json'},
      body: JSON.stringify({ recipient:to, amount: Number(amt) })
    });
    if (!res.ok) throw new Error(await res.text());
    alert('Transaction sent!');
    loadWallet();
  } catch(e){
    alert('Send failed: '+e.message);
  }
}

// on load
window.addEventListener('load', ()=>{
  loadWallet();
});