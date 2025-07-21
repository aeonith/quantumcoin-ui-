document.getElementById("kycForm").addEventListener("submit", function(e) {
  e.preventDefault();

  const email = document.getElementById("email").value;
  const inputCode = document.getElementById("codeInput").value;
  const actualCode = document.getElementById("codeDisplay").textContent.trim();
  const status = document.getElementById("statusMessage");

  if (inputCode === actualCode && email.includes("@")) {
    localStorage.setItem("kycVerified", "true");
    status.textContent = "✅ Verification successful!";
    status.style.color = "#00ff00";
  } else {
    status.textContent = "❌ Incorrect code or email.";
    status.style.color = "#ff5555";
  }
});