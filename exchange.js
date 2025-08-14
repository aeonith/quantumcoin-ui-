// QuantumCoin BTC Exchange - Legacy HTML Implementation
const BTC_ADDRESS = 'bc1qv7tpdxqvgwutfrhf53nhwgp77j5lv7whnk433y';

document.addEventListener('DOMContentLoaded', function() {
    initializeExchange();
    loadExchangeStatus();
});

function initializeExchange() {
    // Display BTC address
    const btcAddressElement = document.getElementById('btc-address');
    if (btcAddressElement) {
        btcAddressElement.textContent = BTC_ADDRESS;
    }

    // Load saved QTC address if available
    const qtcAddressInput = document.getElementById('qtc-address');
    const savedQtcAddress = localStorage.getItem('qc_wallet_addr');
    if (savedQtcAddress && qtcAddressInput) {
        qtcAddressInput.value = savedQtcAddress;
    }
}

async function loadExchangeStatus() {
    const statusElement = document.getElementById('exchange-status');
    
    try {
        // Check if we're running on Vercel or locally
        const isVercel = window.location.hostname.includes('vercel.app');
        const baseUrl = isVercel ? window.location.origin : '';
        
        const response = await fetch(`${baseUrl}/api/exchange-status`);
        const status = await response.json();
        
        statusElement.innerHTML = `
            <div style="display: flex; justify-content: space-between; margin: 10px 0;">
                <span>Status:</span>
                <span style="color: ${status.enabled ? '#00ff88' : '#ff6666'};">
                    ${status.enabled ? '🟢 ACTIVE' : '🔴 INACTIVE'}
                </span>
            </div>
            <div style="display: flex; justify-content: space-between; margin: 10px 0;">
                <span>Available QTC:</span>
                <span>${status.float.toLocaleString()}</span>
            </div>
            <div style="font-size: 0.9rem; opacity: 0.8; margin-top: 10px;">
                ${status.enabled 
                    ? 'Exchange is operational with available supply' 
                    : 'Exchange disabled - QTC must be mined to increase supply'}
            </div>
        `;
    } catch (error) {
        console.error('Error loading exchange status:', error);
        statusElement.innerHTML = `
            <div style="color: #fbbf24;">⚠️ Unable to load exchange status</div>
            <div style="font-size: 0.9rem; opacity: 0.8; margin-top: 10px;">
                Running in offline mode
            </div>
        `;
    }
}

function copyBtcAddress() {
    navigator.clipboard.writeText(BTC_ADDRESS).then(() => {
        showStatus('✅ Bitcoin address copied to clipboard!', 'success');
    }).catch(err => {
        console.error('Error copying address:', err);
        showStatus('❌ Copy failed. Please select and copy manually.', 'error');
    });
}

async function verifyAndCredit(event) {
    event.preventDefault();
    
    const qtcAddress = document.getElementById('qtc-address').value.trim();
    const btcTxid = document.getElementById('btc-txid').value.trim();
    const verifyBtn = document.getElementById('verify-btn');
    
    if (!qtcAddress) {
        showStatus('❌ Please enter your QTC address', 'error');
        return;
    }
    
    if (!btcTxid) {
        showStatus('❌ Please enter your Bitcoin transaction ID', 'error');
        return;
    }
    
    // Disable button and show loading
    verifyBtn.disabled = true;
    verifyBtn.textContent = '🔍 Verifying...';
    showStatus('🔍 Verifying Bitcoin transaction on blockchain...', 'info');
    
    try {
        // Check if we're running on Vercel or locally
        const isVercel = window.location.hostname.includes('vercel.app');
        const baseUrl = isVercel ? window.location.origin : '';
        
        // Verify BTC transaction
        const verifyResponse = await fetch(`${baseUrl}/api/verify-btc?txid=${encodeURIComponent(btcTxid)}`);
        const verification = await verifyResponse.json();
        
        if (!verification.ok) {
            showStatus(`❌ ${verification.error || 'Transaction verification failed'}`, 'error');
            return;
        }
        
        showStatus('💰 Bitcoin transaction verified! Crediting QTC...', 'info');
        
        // Credit QTC
        const creditResponse = await fetch(`${baseUrl}/api/credit-qtc`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                qtcAddr: qtcAddress,
                amountQtc: verification.estimatedQtc
            })
        });
        
        const creditResult = await creditResponse.json();
        
        if (!creditResult.ok) {
            showStatus(`❌ ${creditResult.error || 'Failed to credit QTC'}`, 'error');
            return;
        }
        
        // Success!
        showStatus(
            `✅ Success! ${creditResult.credited} QTC credited to ${qtcAddress}${creditResult.simulated ? ' (SIMULATED)' : ''}`, 
            'success'
        );
        
        // Update local balance if this is the user's wallet
        const userWallet = localStorage.getItem('qc_wallet_addr');
        if (userWallet === qtcAddress) {
            const currentBalance = parseFloat(localStorage.getItem('qc_wallet_balance') || '0');
            const newBalance = currentBalance + (creditResult.credited || 0);
            localStorage.setItem('qc_wallet_balance', newBalance.toString());
        }
        
        // Clear transaction ID
        document.getElementById('btc-txid').value = '';
        
    } catch (error) {
        console.error('Exchange error:', error);
        showStatus('❌ Network error. Please try again.', 'error');
    } finally {
        // Re-enable button
        verifyBtn.disabled = false;
        verifyBtn.textContent = '🚀 Verify & Credit QTC';
    }
}

function showStatus(message, type) {
    const statusElement = document.getElementById('status-message');
    statusElement.style.display = 'block';
    statusElement.textContent = message;
    statusElement.className = `status-message status-${type}`;
    
    // Auto-hide success messages after 5 seconds
    if (type === 'success') {
        setTimeout(() => {
            statusElement.style.display = 'none';
        }, 5000);
    }
}
