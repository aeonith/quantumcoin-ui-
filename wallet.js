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

// Export functions for global use
window.sendTransaction = sendTransaction;
window.copyWalletAddress = copyWalletAddress;
