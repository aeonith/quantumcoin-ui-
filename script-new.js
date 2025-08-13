// QuantumCoin Frontend - Connected to Ultra-Advanced Backend
const API_BASE = '/api/v1';

document.addEventListener('DOMContentLoaded', function() {
  const formButton = document.getElementById('form-button');
  const emailInput = document.getElementById('email');
  const passwordInput = document.getElementById('password');
  const confirmPasswordInput = document.getElementById('confirm-password');
  const toggleForm = document.getElementById('toggle-form');

  let isLoginForm = true;

  // Check authentication status
  checkAuthStatus();

  toggleForm.addEventListener('click', function(e) {
    e.preventDefault();

    if (isLoginForm) {
      confirmPasswordInput.style.display = 'block';
      formButton.textContent = '🔐 Create Quantum Wallet';
      toggleForm.textContent = 'Login';
      document.querySelector('.toggle').innerHTML = 'Already have an account? <a href="#" id="toggle-form">Login</a>';
      isLoginForm = false;
    } else {
      confirmPasswordInput.style.display = 'none';
      formButton.textContent = '🚀 Login';
      toggleForm.textContent = 'Register';
      document.querySelector('.toggle').innerHTML = 'New here? <a href="#" id="toggle-form">Register</a>';
      isLoginForm = true;
    }

    // Re-bind the event listener to the new element
    document.getElementById('toggle-form').addEventListener('click', arguments.callee);
  });

  formButton.addEventListener('click', async function() {
    const email = emailInput.value;
    const password = passwordInput.value;
    const confirmPassword = confirmPasswordInput.value;

    if (!email || !password) {
      showAlert('Please fill in all fields', 'error');
      return;
    }

    if (isLoginForm) {
      await login(email, password);
    } else {
      if (password !== confirmPassword) {
        showAlert('Passwords do not match', 'error');
        return;
      }
      await createQuantumWallet(password, confirmPassword);
    }
  });

  // API Functions
  async function login(username, password) {
    try {
      showLoading('Authenticating with quantum security...');
      
      const response = await fetch(`${API_BASE}/auth/login`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          username,
          password,
          two_factor_code: null
        })
      });

      const result = await response.json();
      hideLoading();

      if (result.success) {
        localStorage.setItem('quantum_token', result.data.access_token);
        localStorage.setItem('user_info', JSON.stringify(result.data.user_info));
        showAlert('🎉 Login successful! Welcome to the quantum era!', 'success');
        
        // Redirect to dashboard
        setTimeout(() => {
          window.location.href = 'dashboard.html';
        }, 1500);
      } else {
        showAlert(`Login failed: ${result.message}`, 'error');
      }
    } catch (error) {
      hideLoading();
      showAlert(`Network error: ${error.message}`, 'error');
    }
  }

  async function createQuantumWallet(password, confirmPassword) {
    try {
      showLoading('Creating quantum-resistant wallet...');
      
      const response = await fetch(`${API_BASE}/wallet/create`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          password,
          confirm_password: confirmPassword
        })
      });

      const result = await response.json();
      hideLoading();

      if (result.success) {
        showAlert('🚀 Quantum wallet created successfully!', 'success');
        
        // Display wallet info
        displayWalletInfo(result.data);
        
        // Switch to login mode
        setTimeout(() => {
          confirmPasswordInput.style.display = 'none';
          formButton.textContent = '🚀 Login';
          toggleForm.textContent = 'Register';
          document.querySelector('.toggle').innerHTML = 'New here? <a href="#" id="toggle-form">Register</a>';
          isLoginForm = true;
        }, 3000);
      } else {
        showAlert(`Wallet creation failed: ${result.message}`, 'error');
      }
    } catch (error) {
      hideLoading();
      showAlert(`Network error: ${error.message}`, 'error');
    }
  }

  function displayWalletInfo(walletData) {
    const modal = document.createElement('div');
    modal.className = 'quantum-modal';
    modal.innerHTML = `
      <div class="modal-content">
        <h3>🔐 Your Quantum Wallet</h3>
        <div class="wallet-info">
          <p><strong>Address:</strong> <code>${walletData.address.substring(0, 20)}...</code></p>
          <p><strong>Security Level:</strong> ⭐⭐⭐⭐⭐ (Quantum-Resistant)</p>
          <p><strong>Balance:</strong> ${walletData.balance} QTC</p>
        </div>
        <div class="security-notice">
          <p>🛡️ Your wallet is protected by:</p>
          <ul>
            <li>Dilithium2 post-quantum signatures</li>
            <li>AES-256-GCM encryption</li>
            <li>SHA-3 quantum-resistant hashing</li>
            <li>AI-powered fraud detection</li>
          </ul>
        </div>
        <button onclick="this.parentElement.parentElement.remove()">Continue</button>
      </div>
    `;
    document.body.appendChild(modal);
  }

  async function checkAuthStatus() {
    const token = localStorage.getItem('quantum_token');
    if (token) {
      try {
        const response = await fetch(`${API_BASE}/health`, {
          headers: {
            'Authorization': `Bearer ${token}`
          }
        });
        
        if (response.ok) {
          // User is authenticated, redirect to dashboard
          window.location.href = 'dashboard.html';
        }
      } catch (error) {
        // Token is invalid, continue with login page
        localStorage.removeItem('quantum_token');
        localStorage.removeItem('user_info');
      }
    }
  }

  function showLoading(message) {
    const loadingDiv = document.createElement('div');
    loadingDiv.id = 'loading';
    loadingDiv.innerHTML = `
      <div class="loading-content">
        <div class="quantum-spinner"></div>
        <p>${message}</p>
      </div>
    `;
    document.body.appendChild(loadingDiv);
  }

  function hideLoading() {
    const loading = document.getElementById('loading');
    if (loading) {
      loading.remove();
    }
  }

  function showAlert(message, type) {
    const alertDiv = document.createElement('div');
    alertDiv.className = `quantum-alert ${type}`;
    alertDiv.innerHTML = `
      <span>${message}</span>
      <button onclick="this.parentElement.remove()">×</button>
    `;
    document.body.appendChild(alertDiv);
    
    // Auto-hide success messages
    if (type === 'success') {
      setTimeout(() => {
        if (alertDiv.parentElement) {
          alertDiv.remove();
        }
      }, 5000);
    }
  }
});

// Real-time network stats display
async function loadNetworkStats() {
  try {
    const response = await fetch(`${API_BASE}/network/stats`);
    const result = await response.json();
    
    if (result.success) {
      const stats = result.data;
      
      // Update any network stats on the page
      const networkInfo = document.querySelector('.network-stats');
      if (networkInfo) {
        networkInfo.innerHTML = `
          <div class="stat">
            <span class="label">Total Supply:</span>
            <span class="value">${stats.total_supply.toLocaleString()} QTC</span>
          </div>
          <div class="stat">
            <span class="label">Transactions/sec:</span>
            <span class="value">${stats.transactions_per_second}</span>
          </div>
          <div class="stat">
            <span class="label">Environmental Score:</span>
            <span class="value">${stats.environmental_score.toFixed(1)}%</span>
          </div>
          <div class="stat">
            <span class="label">Quantum Security:</span>
            <span class="value">${stats.quantum_security_active ? '🛡️ ACTIVE' : '❌ INACTIVE'}</span>
          </div>
        `;
      }
    }
  } catch (error) {
    console.log('Network stats unavailable:', error);
  }
}

// Load network stats every 10 seconds
setInterval(loadNetworkStats, 10000);
loadNetworkStats();
