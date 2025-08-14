document.getElementById('register-form').addEventListener('submit', async (e) => {
  e.preventDefault();

  const email = document.getElementById('register-email').value;
  const password = document.getElementById('register-password').value;
  const confirm = document.getElementById('register-confirm').value;
  const agreed = document.getElementById('terms-checkbox').checked;

  if (!agreed) {
    return alert("You must agree to the Terms & Conditions.");
  }

  if (password !== confirm) {
    return alert("Passwords do not match.");
  }

  try {
    // For local development, simulate registration with localStorage
    const userData = {
      email: email,
      walletAddress: "QTC_" + Math.random().toString(36).substring(2, 15),
      balance: 100, // Welcome bonus
      registerTime: new Date().toISOString(),
      revStopEnabled: false
    };

    localStorage.setItem("qc_user", JSON.stringify(userData));
    localStorage.setItem("walletAddress", userData.walletAddress);
    localStorage.setItem("email", email);
    
    alert("✅ Registered successfully! Welcome bonus: 100 QTC");
    window.location.href = "wallet.html";
    
  } catch (err) {
    console.error(err);
    alert("Registration error. Try again later.");
  }
});