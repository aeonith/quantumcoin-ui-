const BACKEND_URL = "https://quantumcoin-ui-1rust1.onrender.com";

// Login Handler
async function login() {
  const email = document.getElementById("login-username").value;
  const password = document.getElementById("login-password").value;

  if (!email || !password) {
    alert("Please enter both email and password.");
    return;
  }

  try {
    const res = await fetch(`${BACKEND_URL}/login`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email, password }),
    });

    const data = await res.json();

    if (res.ok) {
      alert("Login successful!");
    } else {
      alert(`Login failed: ${data.error || "Unknown error"}`);
    }
  } catch (err) {
    alert("Server error during login.");
    console.error(err);
  }
}

// Terms modal logic
function openTerms() {
  document.getElementById("terms-modal").style.display = "block";
}

function closeTerms() {
  document.getElementById("terms-modal").style.display = "none";
}

window.onclick = function (event) {
  const modal = document.getElementById("terms-modal");
  if (event.target == modal) {
    modal.style.display = "none";
  }
};