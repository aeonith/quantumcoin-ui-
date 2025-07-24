const API_BASE = "https://quantumcoin-ithu.onrender.com";

// Handle login
async function loginUser() {
  const username = document.getElementById("login-username").value;
  const password = document.getElementById("login-password").value;

  try {
    const res = await fetch(`${API_BASE}/api/login`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ username, password })
    });

    if (!res.ok) throw new Error("Login failed.");
    const data = await res.json();
    alert(`✅ Login successful! Welcome ${data.username || username}`);
  } catch (err) {
    alert("❌ Login error: " + err.message);
    console.error(err);
  }
}

// Handle registration
async function registerUser() {
  const username = document.getElementById("register-username").value;
  const password = document.getElementById("register-password").value;
  const agreed = document.getElementById("terms-checkbox").checked;

  if (!agreed) {
    alert("⚠️ You must agree to the Terms & Conditions.");
    return;
  }

  try {
    const res = await fetch(`${API_BASE}/api/register`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ username, password })
    });

    if (!res.ok) throw new Error("Registration failed.");
    const data = await res.json();
    alert(`✅ Registered as ${data.username || username}`);
  } catch (err) {
    alert("❌ Registration error: " + err.message);
    console.error(err);
  }
}

// Toggle between login and register view
function toggleAuthView() {
  const loginForm = document.getElementById("login-form");
  const registerForm = document.getElementById("register-form");
  loginForm.style.display = loginForm.style.display === "none" ? "block" : "none";
  registerForm.style.display = registerForm.style.display === "none" ? "block" : "none";
}

// Background video autoplay fix
document.addEventListener("DOMContentLoaded", function () {
  const video = document.getElementById("background-video");
  if (video) {
    video.play().catch(err => {
      console.warn("Background video autoplay failed:", err);
    });
  }
});