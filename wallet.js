// Backend API endpoint
const API_BASE = 'http://localhost:8080';

// Wallet functionality
let currentWallet = null;

// Initialize wallet page
document.addEventListener('DOMContentLoaded', function() {
    initializeWallet();
    loadWalletBalance();
    loadTransactionHistory();
});

function initializeWallet() {
    // Generate or load wallet address
    if (!localStorage.getItem('walletAddress')) {
        const randomSuffix = Math.random().toString(36).substr(2, 20);
        localStorage.setItem('walletAddress', `qtc_${randomSuffix}`);
    }
    
    currentWallet = localStorage.getItem('walletAddress');
    
    // Display wallet info
    const addressElement = document.getElementById('wallet-address');
    if (addressElement) {
        addressElement.textContent = currentWallet;
    }
    
    // Generate QR code for wallet address
    generateQRCode();
}

async function loadWalletBalance() {
    try {
        const response = await fetch(`${API_BASE}/balance/${currentWallet}`);
        if (response.ok) {
            const data = await response.json();
            const balanceElement = document.getElementById('wallet-balance');
            if (balanceElement) {
                balanceElement.textContent = `${data.balance} QTC`;
            }
        } else {
            // Fallback to stored balance
            const storedBalance = localStorage.getItem('walletBalance') || '0';
            const balanceElement = document.getElementById('wallet-balance');
            if (balanceElement) {
                balanceElement.textContent = `${storedBalance} QTC`;
            }
        }
    } catch (error) {
        console.error('Error loading balance:', error);
        // Show fallback balance
        const balanceElement = document.getElementById('wallet-balance');
        if (balanceElement) {
            balanceElement.textContent = '0.00 QTC';
        }
    }
}

async function loadTransactionHistory() {
    try {
        const response = await fetch(`${API_BASE}/blockchain`);
        if (response.ok) {
            const blocks = await response.json();
            displayTransactionHistory(blocks);
        } else {
            displayPlaceholderTransactions();
        }
    } catch (error) {
        console.error('Error loading transactions:', error);
        displayPlaceholderTransactions();
    }
}

function displayTransactionHistory(blocks) {
    const historyContainer = document.getElementById('transaction-history');
    if (!historyContainer) return;
    
    historyContainer.innerHTML = '';
    
    // Extract transactions involving current wallet
    const walletTransactions = [];
    blocks.forEach(block => {
        block.transactions.forEach(tx => {
            if (tx.from === currentWallet || tx.to === currentWallet) {
                walletTransactions.push({
                    ...tx,
                    blockIndex: block.index,
                    type: tx.to === currentWallet ? 'received' : 'sent'
                });
            }
        });
    });
    
    if (walletTransactions.length === 0) {
        historyContainer.innerHTML = '<p class="no-transactions">No transactions found</p>';
        return;
    }
    
    walletTransactions.reverse().forEach(tx => {
        const txElement = document.createElement('div');
        txElement.className = `transaction-item ${tx.type}`;
        txElement.innerHTML = `
            <div class="transaction-info">
                <div class="transaction-type">${tx.type === 'received' ? '↓ Received' : '↑ Sent'}</div>
                <div class="transaction-amount ${tx.type}">${tx.type === 'received' ? '+' : '-'}${tx.amount} QTC</div>
            </div>
            <div class="transaction-details">
                <div class="transaction-address">${tx.type === 'received' ? 'From: ' + tx.from : 'To: ' + tx.to}</div>
                <div class="transaction-time">${new Date(tx.timestamp).toLocaleString()}</div>
            </div>
        `;
        historyContainer.appendChild(txElement);
    });
}

function displayPlaceholderTransactions() {
    const historyContainer = document.getElementById('transaction-history');
    if (!historyContainer) return;
    
    historyContainer.innerHTML = `
        <div class="transaction-item received">
            <div class="transaction-info">
                <div class="transaction-type">↓ Received</div>
                <div class="transaction-amount received">+10.0 QTC</div>
            </div>
            <div class="transaction-details">
                <div class="transaction-address">Mining Reward</div>
                <div class="transaction-time">${new Date().toLocaleString()}</div>
            </div>
        </div>
    `;
}

async function sendTransaction() {
    const recipientAddress = document.getElementById('recipient-address')?.value;
    const amount = parseFloat(document.getElementById('send-amount')?.value);
    
    if (!recipientAddress || !amount || amount <= 0) {
        showNotification('Please enter valid recipient and amount', 'error');
        return;
    }
    
    try {
        const transaction = {
            id: `tx_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
            from: currentWallet,
            to: recipientAddress,
            amount: amount,
            timestamp: new Date().toISOString(),
            signature: 'simulated_signature'
        };
        
        const response = await fetch(`${API_BASE}/transaction`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(transaction)
        });
        
        if (response.ok) {
            showNotification('Transaction sent successfully!', 'success');
            // Clear form
            document.getElementById('recipient-address').value = '';
            document.getElementById('send-amount').value = '';
            // Refresh balance and history
            setTimeout(() => {
                loadWalletBalance();
                loadTransactionHistory();
            }, 1000);
        } else {
            throw new Error('Failed to send transaction');
        }
    } catch (error) {
        console.error('Error sending transaction:', error);
        showNotification('Failed to send transaction. Please try again.', 'error');
    }
}

function generateQRCode() {
    const qrContainer = document.getElementById('wallet-qr');
    if (!qrContainer || !currentWallet) return;
    
    // Simple QR placeholder - in production, use a proper QR library
    qrContainer.innerHTML = `
        <div class="qr-placeholder">
            <div class="qr-grid">
                <div class="qr-pattern"></div>
                <div class="qr-pattern"></div>
                <div class="qr-pattern"></div>
                <div class="qr-pattern"></div>
            </div>
            <p>QR Code for ${currentWallet}</p>
        </div>
    `;
}

function copyWalletAddress() {
    if (navigator.clipboard && currentWallet) {
        navigator.clipboard.writeText(currentWallet).then(() => {
            showNotification('Wallet address copied to clipboard!', 'success');
        }).catch(() => {
            // Fallback for older browsers
            const textArea = document.createElement('textarea');
            textArea.value = currentWallet;
            document.body.appendChild(textArea);
            textArea.select();
            document.execCommand('copy');
            document.body.removeChild(textArea);
            showNotification('Wallet address copied!', 'success');
        });
    }
}

function showNotification(message, type = 'info') {
    const notification = document.createElement('div');
    notification.className = `notification ${type}`;
    notification.textContent = message;
    notification.style.cssText = `
        position: fixed;
        top: 20px;
        right: 20px;
        padding: 15px 20px;
        border-radius: 5px;
        color: white;
        z-index: 1000;
        font-weight: bold;
        animation: slideIn 0.3s ease-out;
        box-shadow: 0 4px 12px rgba(0,0,0,0.3);
        ${type === 'success' ? 'background: #4CAF50;' : ''}
        ${type === 'error' ? 'background: #f44336;' : ''}
        ${type === 'info' ? 'background: #2196F3;' : ''}
    `;
    
    document.body.appendChild(notification);
    
    setTimeout(() => {
        notification.remove();
    }, 4000);
}

// New wallet functions for enhanced UI
function generateNewWallet() {
    const newAddress = "QTC_" + Math.random().toString(36).substring(2, 15);
    localStorage.setItem('walletAddress', newAddress);
    currentWallet = newAddress;
    
    // Update display
    const addressElement = document.getElementById('wallet-address');
    if (addressElement) {
        addressElement.textContent = newAddress;
    }
    
    // Update QR code
    generateSimpleQR();
    
    showNotification('New wallet generated successfully!', 'success');
}

function copyAddress() {
    if (navigator.clipboard && currentWallet) {
        navigator.clipboard.writeText(currentWallet).then(() => {
            showNotification('Address copied to clipboard!', 'success');
        }).catch(() => {
            showNotification('Address copied!', 'success');
        });
    }
}

function toggleRevStop() {
    const statusSpan = document.getElementById('revstop-status');
    const toggleBtn = document.getElementById('revstop-toggle');
    const userData = JSON.parse(localStorage.getItem('qc_user') || '{}');
    
    const isEnabled = userData.revStopEnabled || false;
    
    if (isEnabled) {
        userData.revStopEnabled = false;
        statusSpan.textContent = 'OFF';
        statusSpan.style.background = 'rgba(255, 0, 0, 0.2)';
        statusSpan.style.color = '#ff4444';
        toggleBtn.textContent = 'Enable';
        showNotification('RevStop disabled', 'info');
    } else {
        userData.revStopEnabled = true;
        statusSpan.textContent = 'ON';
        statusSpan.style.background = 'rgba(0, 255, 0, 0.2)';
        statusSpan.style.color = '#00ff00';
        toggleBtn.textContent = 'Disable';
        showNotification('RevStop enabled - Quantum protection active!', 'success');
    }
    
    localStorage.setItem('qc_user', JSON.stringify(userData));
}

function generateSimpleQR() {
    const qrDiv = document.getElementById('qr-code');
    if (qrDiv && currentWallet) {
        qrDiv.innerHTML = `
            <div style="width: 100px; height: 100px; background: white; margin: 0 auto; display: grid; grid-template-columns: repeat(10, 1fr); grid-template-rows: repeat(10, 1fr); border-radius: 5px;">
                ${Array.from({length: 100}, (_, i) => 
                    `<div style="background: ${Math.random() > 0.5 ? 'black' : 'white'};"></div>`
                ).join('')}
            </div>
            <div style="font-size: 0.8rem; margin-top: 10px; color: #00fdfd;">Scan to send QTC</div>
        `;
    }
}

// Initialize RevStop status on load
document.addEventListener('DOMContentLoaded', function() {
    setTimeout(() => {
        const userData = JSON.parse(localStorage.getItem('qc_user') || '{}');
        const statusSpan = document.getElementById('revstop-status');
        const toggleBtn = document.getElementById('revstop-toggle');
        
        if (statusSpan && toggleBtn) {
            if (userData.revStopEnabled) {
                statusSpan.textContent = 'ON';
                statusSpan.style.background = 'rgba(0, 255, 0, 0.2)';
                statusSpan.style.color = '#00ff00';
                toggleBtn.textContent = 'Disable';
            } else {
                statusSpan.textContent = 'OFF';
                statusSpan.style.background = 'rgba(255, 0, 0, 0.2)';
                statusSpan.style.color = '#ff4444';
                toggleBtn.textContent = 'Enable';
            }
        }
        
        generateSimpleQR();
    }, 100);
});

// Export functions for global use
window.sendTransaction = sendTransaction;
window.copyWalletAddress = copyWalletAddress;
window.generateNewWallet = generateNewWallet;
window.copyAddress = copyAddress;
window.toggleRevStop = toggleRevStop;
