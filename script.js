const API_URL = "https://quantumcoin-ithu.onrender.com";

// REGISTER USER
async function registerUser() {
  const username = document.getElementById("register-username").value.trim();
  const password = document.getElementById("register-password").value.trim();
  const agreed = document.getElementById("terms").checked;

  if (!username || !password) {
    alert("Please enter both a username and password.");
    return;
  }

  if (!agreed) {
    alert("You must agree to the Terms & Conditions.");
    return;
  }

  try {
    const res = await fetch(`${API_URL}/api/register`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify({ username, password })
    });

    const data = await res.json();
    if (!res.ok) throw new Error(data.message || "Register failed.");

    alert("✅ Registered! Now login.");
  } catch (err) {
    alert("❌ " + err.message);
  }
}

// LOGIN USER
async function loginUser() {
  const username = document.getElementById("login-username").value.trim();
  const password = document.getElementById("login-password").value.trim();

  if (!username || !password) {
    alert("Please enter both username and password.");
    return;
  }

  try {
    const res = await fetch(`${API_URL}/api/login`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json"
      },
      body: JSON.stringify({ username, password })
    });

    const data = await res.json();
    if (!res.ok) throw new Error(data.message || "Login failed.");

    alert(`✅ Welcome ${data.username || username}`);
  } catch (err) {
    alert("❌ " + err.message);
  }
}