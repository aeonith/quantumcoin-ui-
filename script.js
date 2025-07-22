async function fetchWallet() {
  const res = await fetch('/wallet');
  const wallet = await res.json();
  document.getElementById('address').innerText = wallet.address;
}

async function fetchBalance() {
  const res = await fetch('/balance');
  const data = await res.json();
  document.getElementById('balance').innerText = data.balance;
}

async function fetchPrice() {
  const res = await fetch('/price');
  const data = await res.json();
  document.getElementById('price').innerText = `$${parseFloat(data.price).toFixed(4)}`;
}

async function send() {
  const recipient = document.getElementById('recipient').value;
  const amount = parseInt(document.getElementById('amount').value);
  const password = document.getElementById('password').value;

  const res = await fetch('/send', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ recipient, amount, password })
  });

  const msg = await res.text();
  alert(msg);
  fetchBalance();
}

async function mine() {
  const res = await fetch('/mine', { method: 'POST' });
  const msg = await res.text();
  alert(msg);
  fetchBalance();
}

fetchWallet();
fetchBalance();
fetchPrice();