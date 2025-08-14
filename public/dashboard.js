let walletAddress = null;

window.onload = async () => {
  try {
    const res = await fetch("/api/wallet");
    const data = await res.json();

    if (data && data.address) {
      walletAddress = data.address;
      document.getElementById("wallet-address").innerHTML = `Address: <span>${walletAddress}</span>`;
      await refreshBalance(); // Load balance after address
    } else {
      document.getElementById("wallet-address").innerHTML = `Address: <span style="color:red;">Unavailable</span>`;
    }
  } catch (err) {
    console.error(err);
    document.getElementById("wallet-address").innerHTML = `Address: <span style="color:red;">Error</span>`;
  }
};

async function refreshBalance() {
  if (!walletAddress) return;

  try {
    const res = await fetch(`/api/balance/${walletAddress}`);
    const data = await res.json();
    document.getElementById("wallet-balance").innerHTML = `Balance: <span>${data.balance} QTC</span>`;
  } catch (err) {
    console.error("Balance load failed", err);
    document.getElementById("wallet-balance").innerHTML = `Balance: <span style="color:red;">Error</span>`;
  }
}

async function sendQTC() {
  const recipient = document.getElementById("recipient").value;
  const amount = parseFloat(document.getElementById("amount").value);

  if (!recipient || !amount || isNaN(amount)) {
    alert("Enter valid address and amount.");
    return;
  }

  try {
    const res = await fetch("/api/send", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ from: walletAddress, to: recipient, amount }),
    });

    const result = await res.json();
    if (result.success) {
      alert("Transaction sent!");
      await refreshBalance();
    } else {
      alert("Transaction failed: " + result.error);
    }
  } catch (err) {
    console.error(err);
    alert("Error sending transaction.");
  }
}