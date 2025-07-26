const BACKEND_URL = "https://quantumcoin-ui-1rust1.onrender.com";

// Login
async function login() {
  const username = document.getElementById("login-username").value;
  const password = document.getElementById("login-password").value;

  if (!username || !password) {
    alert("Enter both username and password.");
    return;
  }

  try {
    const res = await fetch(`${BACKEND_URL}/login`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ username, password }),
    });

    const data = await res.json();
    if (res.ok) alert("✅ Login successful!");
    else alert(`Login failed: ${data.error || "Unknown error"}`);
  } catch (err) {
    console.error(err);
    alert("Server error during login.");
  }
}

// Register
async function register() {
  const username = document.getElementById("register-username").value;
  const password = document.getElementById("register-password").value;
  const agreed = document.getElementById("terms-checkbox").checked;

  if (!username || !password || !agreed) {
    alert("Please fill all fields and agree to the Terms.");
    return;
  }

  try {
    const res = await fetch(`${BACKEND_URL}/register`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ username, password }),
    });

    const data = await res.json();
    if (res.ok) alert("✅ Registration successful!");
    else alert(`Registration failed: ${data.error || "Unknown error"}`);
  } catch (err) {
    console.error(err);
    alert("Server error during registration.");
  }
}

// Terms modal
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