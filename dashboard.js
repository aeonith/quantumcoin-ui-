const api = 'https://quantumcoin-ui-1live.onrender.com'; // Your backend URL

window.onload = () => {
  fetchWalletInfo();
};

function fetchWalletInfo() {
  fetch(`${api}/wallet`)
    .then(res => res.json())
    .then(data => {
      document.getElementById('wallet-address').textContent = data.public_key;
      document.getElementById('wallet-balance').textContent = `${data.balance} QTC`;
    })
    .catch(err => updateStatus('Error fetching wallet: ' + err));
}

function refreshBalance() {
  fetchWalletInfo();
}

function mineCoins() {
  fetch(`${api}/mine`, { method: 'POST' })
    .then(res => res.json())
    .then(data => {
      updateStatus(`Mined block: ${data.block_hash}`);
      refreshBalance();
    })
    .catch(err => updateStatus('Mining error: ' + err));
}

function toggleRevStop() {
  fetch(`${api}/revstop/toggle`, { method: 'POST' })
    .then(res => res.json())
    .then(data => updateStatus(`RevStop status: ${data.status}`))
    .catch(err => updateStatus('RevStop error: ' + err));
}

function showSend() {
  const section = document.getElementById('send-section');
  section.style.display = section.style.display === 'none' ? 'block' : 'none';
}

function sendCoins() {
  const recipient = document.getElementById('send-to').value;
  const amount = parseFloat(document.getElementById('send-amount').value);

  fetch(`${api}/send`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ to: recipient, amount })
  })
    .then(res => res.json())
    .then(data => {
      updateStatus(`Sent ${amount} QTC to ${recipient}`);
      refreshBalance();
    })
    .catch(err => updateStatus('Send error: ' + err));
}

function exportWallet() {
  fetch(`${api}/wallet/export`)
    .then(res => res.blob())
    .then(blob => {
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = 'quantumcoin_wallet_backup.json';
      a.click();
    })
    .catch(err => updateStatus('Export failed: ' + err));
}

function setup2FA() {
  fetch(`${api}/2fa/setup`)
    .then(res => res.json())
    .then(data => {
      alert(`Your 2FA code: ${data.secret}\nScan it in Google Authenticator.`);
      updateStatus('2FA initialized.');
    })
    .catch(err => updateStatus('2FA setup failed: ' + err));
}

function lockWallet() {
  const confirm = prompt("Enter password to lock your wallet:");
  if (!confirm) return;

  fetch(`${api}/wallet/lock`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ password: confirm })
  })
    .then(res => res.json())
    .then(data => {
      updateStatus('Wallet locked.');
    })
    .catch(err => updateStatus('Lock failed: ' + err));
}

function logout() {
  window.location.href = 'login.html';
}

function updateStatus(msg) {
  document.getElementById('status').textContent = msg;
}