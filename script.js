const API = 'https://quantumcoin-ithu.onrender.com';
let token = localStorage.getItem('qtc_token');

// helpers
function qs(sel){ return document.querySelector(sel) }
function show(el){ el.classList.remove('hidden') }
function hide(el){ el.classList.add('hidden') }
async function api(path, opts={}) {
  opts.headers = opts.headers||{};
  if(token) opts.headers['Authorization'] = `Bearer ${token}`;
  opts.headers['Content-Type'] = 'application/json';
  const res = await fetch(API+path, opts);
  if(res.status===401) { logout(); throw 'Unauthorized' }
  return res.json();
}

// UI elements
const loginForm    = qs('#login-form'),
      registerForm = qs('#register-form'),
      dash         = qs('#dashboard'),
      btnLogin     = qs('#btn-login'),
      btnRegister  = qs('#btn-register'),
      linkToReg    = qs('#link-to-register'),
      linkToLog    = qs('#link-to-login'),
      linkTerms    = qs('#link-terms'),
      linkTerms2   = qs('#link-terms-2'),
      linkPriv     = qs('#link-privacy'),
      linkPriv2    = qs('#link-privacy-2'),
      modalTerms   = qs('#modal-terms'),
      modalPriv    = qs('#modal-privacy'),
      closeTerms   = qs('#close-terms'),
      closePriv    = qs('#close-privacy');

// show/hide handlers
btnLogin.onclick    = ()=>{ hide(registerForm); show(loginForm) }
btnRegister.onclick = ()=>{ hide(loginForm); show(registerForm) }
linkToReg.onclick   = e=>{ e.preventDefault(); hide(loginForm); show(registerForm) }
linkToLog.onclick   = e=>{ e.preventDefault(); hide(registerForm); show(loginForm) }
linkTerms.onclick   = e=>{ e.preventDefault(); show(modalTerms) }
linkTerms2.onclick  = e=>{ e.preventDefault(); show(modalTerms) }
linkPriv.onclick    = e=>{ e.preventDefault(); show(modalPriv) }
linkPriv2.onclick   = e=>{ e.preventDefault(); show(modalPriv) }
closeTerms.onclick  = ()=>{ hide(modalTerms) }
closePriv.onclick   = ()=>{ hide(modalPriv) }

// auth
qs('#do-register').onclick = async ()=>{
  const u=qs('#reg-user').value, p=qs('#reg-pass').value, s=qs('#reg-2fa').value;
  const j= await api('/api/register',{method:'POST', body:JSON.stringify({username:u,password:p, two_fa_secret:s||null})});
  if(j.token){ token=j.token; localStorage.setItem('qtc_token',token); initDashboard() }
}
qs('#do-login').onclick = async ()=>{
  const u=qs('#login-user').value, p=qs('#login-pass').value;
  const j= await api('/api/login',{method:'POST', body:JSON.stringify({username:u,password:p})});
  if(j.token){ token=j.token; localStorage.setItem('qtc_token',token); initDashboard() }
}

// logout & reset
function logout(){
  token=null; localStorage.removeItem('qtc_token');
  show(loginForm); hide(dash); hide(registerForm);
}

// dashboard init
async function initDashboard(){
  hide(loginForm); hide(registerForm); show(dash);
  await refreshAll();
}

// refresh everything
async function refreshAll(){
  try{
    const user = await api('/api/me');
    qs('#user-name').textContent = user.username;
    const w = await api('/api/wallet');
    qs('#pubkey').textContent = w.public_key;
    qs('#balance-qtc').textContent = w.balance_qtc.toFixed(2);
    qs('#balance-usd').textContent = w.balance_usd.toFixed(2);
    qs('#btc-deposit').textContent = w.btc_deposit_address;
    const txs = await api('/api/transactions');
    const ul = qs('#tx-list'); ul.innerHTML = '';
    txs.slice(0,5).forEach(tx=>{
      const li = document.createElement('li');
      li.textContent = `${tx.type} ${tx.amount} QTC @ ${new Date(tx.timestamp).toLocaleString()}`;
      ul.appendChild(li);
    });
  }catch(e){
    console.error(e);
  }
}

qs('#btn-refresh').onclick = refreshAll;

// view private key
qs('#btn-view-priv').onclick = ()=>{
  show(qs('#twofa-entry'));
}
qs('#btn-confirm-2fa').onclick = async ()=>{
  const code = qs('#twofa-code').value;
  const j = await api('/api/wallet/private',{method:'POST',body:JSON.stringify({two_fa_code:code})});
  alert('Your private key:\n\n'+j.private_key);
  hide(qs('#twofa-entry'));
}

// mine pending
qs('#btn-mine').onclick = async ()=>{
  const r = await api('/api/mine',{method:'POST'});
  alert(r.message||'Mining started');
  refreshAll();
}

// revstop toggle
qs('#btn-revstop').onclick = async ()=>{
  const status = await api('/api/revstop');
  const action = status.locked ? 'unlock' : 'lock';
  await api(`/api/revstop/${action}`,{method:'POST'});
  alert(`RevStop is now ${action}ed`);
  refreshAll();
}

// on load: if already logged in, go straight to dashboard
window.onload = ()=>{
  if(token) initDashboard();
  else show(loginForm);
}