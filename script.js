const API = 'https://quantumcoin-ithu.onrender.com';

// 3s loop for the background video
const video = document.getElementById('bg-video');
setInterval(() => {
  video.currentTime = 0;
  video.play();
}, 3000);

// UI helpers
function showRegister(){
  document.getElementById('login-box').classList.add('hidden');
  document.getElementById('register-box').classList.remove('hidden');
}
function showLogin(){
  document.getElementById('register-box').classList.add('hidden');
  document.getElementById('login-box').classList.remove('hidden');
}
function showTerms(){
  document.getElementById('terms-modal').classList.remove('hidden');
}
function closeTerms(){
  document.getElementById('terms-modal').classList.add('hidden');
}
function showDashboard(){
  document.getElementById('login-box').classList.add('hidden');
  document.getElementById('register-box').classList.add('hidden');
  document.getElementById('dashboard').classList.remove('hidden');
}

// Authentication
async function login(){
  let u = document.getElementById('login-username').value,
      p = document.getElementById('login-password').value;
  let res = await fetch(`${API}/login`, {
    method:'POST', headers:{'Content-Type':'application/json'},
    body:JSON.stringify({username:u,password:p})
  });
  if(res.ok){
    window.token = (await res.json()).token;
    await loadWallet();
    showDashboard();
  } else {
    alert('Login failed');
  }
}

async function register(){
  if(!document.getElementById('terms-checkbox').checked){
    return alert('You must agree to the terms');
  }
  let u = document.getElementById('register-username').value,
      p = document.getElementById('register-password').value;
  let res = await fetch(`${API}/register`, {
    method:'POST', headers:{'Content-Type':'application/json'},
    body:JSON.stringify({username:u,password:p})
  });
  if(res.ok){
    alert('Registered! Please login.');
    showLogin();
  } else alert('Registration failed');
}

// Wallet & transactions
async function loadWallet(){
  let res = await fetch(`${API}/wallet`, {
    headers:{ 'Authorization':'Bearer '+window.token }
  });
  let data = await res.json();
  document.getElementById('wallet-address').innerText = data.address;
  document.getElementById('wallet-balance').innerText = data.balance;
}

async function refreshBalance(){
  await loadWallet();
}

async function sendQTC(){
  let to = document.getElementById('recipient').value,
      amt = parseFloat(document.getElementById('amount').value);
  let res = await fetch(`${API}/send`, {
    method:'POST',
    headers:{
      'Content-Type':'application/json',
      'Authorization':'Bearer '+window.token
    },
    body:JSON.stringify({to,amount:amt})
  });
  alert(res.ok ? 'Sent!' : 'Send failed');
  refreshBalance();
}