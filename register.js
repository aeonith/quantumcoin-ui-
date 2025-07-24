document.getElementById("register-form").addEventListener("submit", async (e) => {
  e.preventDefault();

  const username = document.getElementById("username").value.trim();
  const email = document.getElementById("email").value.trim();
  const password = document.getElementById("password").value.trim();
  const storeKeys = document.getElementById("storeKeysOption").checked;

  const response = await fetch("https://quantumcoin-ui-1live.onrender.com/api/register", {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      username,
      email,
      password,
      storeKeys
    })
  });

  const result = await response.json();

  if (response.ok) {
    alert("Registration successful! Now login.");
    window.location.href = "login.html";
  } else {
    alert(`Error: ${result.message}`);
  }
});