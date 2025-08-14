document.getElementById("login-form").addEventListener("submit", async function (e) {
  e.preventDefault();

  const email = document.getElementById("email").value.trim();
  const password = document.getElementById("password").value.trim();

  if (!email || !password) {
    alert("Please enter both email and password.");
    return;
  }

  try {
    // For local development, simulate login with localStorage
    const userData = {
      email: email,
      walletAddress: "QTC_" + Math.random().toString(36).substring(2, 15),
      balance: Math.floor(Math.random() * 1000),
      loginTime: new Date().toISOString(),
      revStopEnabled: false
    };

    localStorage.setItem("qc_user", JSON.stringify(userData));
    localStorage.setItem("walletAddress", userData.walletAddress);
    localStorage.setItem("email", email);
    
    // Redirect to wallet
    window.location.href = "wallet.html";
    
  } catch (error) {
    console.error("Login error:", error);
    alert("An error occurred. Please try again later.");
  }
});