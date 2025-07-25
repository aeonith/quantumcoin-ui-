const API = 'https://quantumcoin-ithu.onrender.com';

// helpers
function $(id){ return document.getElementById(id); }
function show(id){ $(id).style.display = 'block'; }
function hide(id){ $(id).style.display = 'none'; }

/* LOGIN / REGISTER */
async function login(){
  const user = $('login-username').value;
  const pass = $('login-password').value;
  try {
    let res = await fetch(`${API}/login`, {
      method:'POST',
      headers:{'Content-Type':'application/json'},
      body: JSON.stringify({ username:user, password:pass }),
    });
    if(!res.ok) throw new Error(await res.text());
    // on success
    hide('login-card');
    hide('register-card');
    show('dashboard');
    loadWallet();
  } catch(e){
    alert('Login error: '+e.message);
  }
}

function showRegister(){
  hide('login-card');
  show('register-card');
}
function showLogin(){
  hide('register-card');
  show('login-card');
}

/* TERMS MODAL */
function openTerms(){ show('terms-modal'); }
function closeTerms(){ hide('terms-modal'); }

/* REGISTER */
async function register(){
  if(!$('terms-checkbox').checked){
    return alert('You must agree to the Terms & Conditions');
  }
  const user = $('register-username').value;
  const pass = $('register-password').value;
  try {
    let res = await fetch(`${API}/register`, {
      method:'POST',
      headers:{'Content-Type':'application/json'},
      body: JSON.stringify({ username:user, password:pass }),
    });
    if(!res.ok) throw new Error(await res.text());
    alert('Account created! Please log in.');
    showLogin();
  } catch(e){
    alert('Register error: '+e.message);
  }
}

/* WALLET */
async function loadWallet(){
  try {
    let addr = await fetch(`${API}/address`).then(r=>r.text());
    $('wallet-address').textContent = addr;
    refreshBalance();
  } catch {
    $('wallet-address').textContent = 'Error';
  }
}
async function refreshBalance(){
  try {
    let bal = await fetch(`${API}/balance`).then(r=>r.text());
    $('wallet-balance').textContent = bal;
  } catch {
    $('wallet-balance').textContent = 'Error';
  }
}

/* SEND QTC */
async function sendQTC(){
  const to = $('recipient').value;
  const amt = parseFloat($('amount').value);
  if(!to||!amt) return alert('Address & amount required');
  try {
    let res = await fetch(`${API}/send`, {
      method:'POST',
      headers:{'Content-Type':'application/json'},
      body: JSON.stringify({ to, amount:amt }),
    });
    alert(await res.text());
    refreshBalance();
  } catch(e){
    alert('Send error: '+e.message);
  }
}