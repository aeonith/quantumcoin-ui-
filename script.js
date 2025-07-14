async function updateGenesisCountdown() {
  try {
    const res = await fetch("/status");
    const data = await res.json();

    const spent = data.genesis_spent || 0;
    const remaining = 1250000 - spent;

    document.getElementById("remainingQTC").innerText = remaining.toLocaleString();

    if (remaining <= 0) {
      document.getElementById("miningStatus").innerText = "✅ Mining is now UNLOCKED!";
    } else {
      document.getElementById("miningStatus").innerText = "⛏️ Mining unlocks when supply = 0 QTC";
    }

  } catch (err) {
    console.error("Error fetching status:", err);
    document.getElementById("remainingQTC").innerText = "Unavailable";
  }
}

setInterval(updateGenesisCountdown, 5000);
updateGenesisCountdown();
