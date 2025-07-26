const BACKEND = "https://quantumcoin-ui-1rust1.onrender.com";

// Auth
async function login(){
  const u = document.getElementById("login-username").value;
  const p = document.getElementById("login-password").value;
  if (!u||!p) return alert("Enter both username/password.");
  try {
    const r = await fetch(`${BACKEND}/login`, {
      method:"POST", headers:{"Content-Type":"application/json"},
      body: JSON.stringify({username:u,password:p})
    });
    const d=await r.json();
    if(r.ok){
      alert("Logged in!");
      // store token & refresh wallet
      localStorage.token = d.token;
      loadWallet();
    } else alert("Login fail: "+(d.error||"unknown"));
  } catch(e){
    alert("Server error during login.");
    console.error(e);
  }
}

// Registration
async function register(){
  const u=document.getElementById("register-username").value;
  const p=document.getElementById("register-password").value;
  const chk = document.getElementById("terms-checkbox").checked;
  if (!u||!p) return alert("Fill username/password.");
  if (!chk) return alert("You must agree to Terms.");
  try {
    const r=await fetch(`${BACKEND}/register`, {
      method:"POST", headers:{"Content-Type":"application/json"},
      body: JSON.stringify({username:u,password:p})
    });
    const d=await r.json();
    if(r.ok){
      alert("Registered successfully!");
      closeTerms();
    } else alert("Register failed: "+(d.error||"unknown"));
  } catch(e){
    alert("Server error during registration.");
    console.error(e);
  }
}

function showTerms(){
  document.getElementById("terms-modal").style.display="flex";
}

function closeTerms(){
  document.getElementById("terms-modal").style.display="none";
}

// Wallet & balance
async function loadWallet(){
  try {
    const r=await fetch(`${BACKEND}/wallet`, {
      headers:{"Authorization":"Bearer "+localStorage.token}
    });
    const d=await r.json();
    if(r.ok){
      document.getElementById("wallet-address").innerText = d.address;
      document.getElementById("wallet-balance").innerText = d.balance;
    } else throw d;
  } catch(e){
    alert("Error loading wallet");
    console.error(e);
  }
}

async function refreshBalance(){
  await loadWallet();
}

// Send QTC
async function sendQTC(){
  const to = document.getElementById("send-to").value;
  const amt = document.getElementById("send-amount").value;
  if(!to||!amt) return alert("Provide recipient & amount.");
  try {
    const r=await fetch(`${BACKEND}/send`, {
      method:"POST", headers:{
        "Content-Type":"application/json",
        "Authorization":"Bearer "+localStorage.token
      },
      body: JSON.stringify({to,amount:amt})
    });
    const d=await r.json();
    if(r.ok){
      alert("Sent successfully!");
      loadWallet();
    } else alert("Send failed: "+(d.error||"unknown"));
  } catch(e){
    alert("Server error sending QTC.");
    console.error(e);
  }
}

// BTC monitoring (if backend handles transfers automatically)
// Poll every 10s to check for incoming BTC confirmations and mint QTC
setInterval(async ()=>{
  try {
    await fetch(`${BACKEND}/check-btc`, {
      headers:{"Authorization":"Bearer "+localStorage.token}
    });
  } catch(e){
    console.warn("BTC monitor failed", e);
  }
}, 10000);

// Modal outside click
window.onclick = e => {
  if(e.target.id==="terms-modal") closeTerms();
};

// Load wallet on page load if already logged in
window.addEventListener("load", () => {
  if(localStorage.token) loadWallet();
});