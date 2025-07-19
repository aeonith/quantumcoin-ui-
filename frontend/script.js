const API = "http://localhost:8080";

async function updateAddress() {
  document.getElementById("address").textContent =
    "Address: " + await fetch(`${API}/address`).then(r => r.text());
}

async function updateBalance() {
  document.getElementById("balance").textContent =
    "Balance: " + await fetch(`${API}/balance`).then(r => r.text()) + " QTC";
}

document.getElementById("mine-btn").onclick = async () => {
  document.getElementById("status").textContent = await fetch(`${API}/mine`, { method: "POST" }).then(r => r.text());
  await updateBalance();
};

document.getElementById("send-btn").onclick = async () => {
  const tx = { from: "", to: "", amount: 1.0 };
  document.getElementById("status").textContent = await fetch(`${API}/send`, {
    method: "POST",
    headers: { "Content-Type":"application/json" },
    body: JSON.stringify(tx)
  }).then(r => r.text());
  await updateBalance();
};

document.getElementById("lock-btn").onclick = async () => {
  document.getElementById("status").textContent = await fetch(`${API}/revstop/lock`, { method: "POST" }).then(r => r.text());
};

document.getElementById("unlock-btn").onclick = async () => {
  document.getElementById("status").textContent = await fetch(`${API}/revstop/unlock`, { method: "POST" }).then(r => r.text());
};

window.onload = async () => {
  await updateAddress();
  await updateBalance();
};