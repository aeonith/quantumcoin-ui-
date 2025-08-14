// Backend API endpoint
const API_BASE = 'http://localhost:8080';

// Mining state
let mining = false;
let hashRate = 0;
let totalMined = 0;
let miningWorker;

async function startMining() {
    if (mining) return;
    
    mining = true;
    document.getElementById('mining-status').textContent = 'Mining...';
    document.getElementById('start-mining').disabled = true;
    document.getElementById('stop-mining').disabled = false;
    
    // Get wallet address for mining rewards
    const walletAddress = localStorage.getItem('walletAddress') || 'qtc_default_miner_address';
    
    const miningInterval = setInterval(async () => {
        if (!mining) {
            clearInterval(miningInterval);
            return;
        }
        
        // Simulate hash rate
        hashRate = Math.floor(Math.random() * 1000) + 500;
        document.getElementById('hashrate').textContent = hashRate + ' H/s';
        
        // Attempt to mine a block (random chance)
        if (Math.random() < 0.1) {
            try {
                const response = await fetch(`${API_BASE}/mine/${walletAddress}`, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json'
                    }
                });
                
                if (response.ok) {
                    const block = await response.json();
                    totalMined += 10; // Mining reward
                    document.getElementById('total-mined').textContent = totalMined + ' QTC';
                    showNotification(`Block #${block.index} mined! Reward: 10 QTC`);
                    
                    // Update balance display if available
                    updateBalance();
                }
            } catch (error) {
                console.error('Mining error:', error);
                showNotification('Mining error - using simulation mode');
                // Fallback to simulation
                totalMined += 10;
                document.getElementById('total-mined').textContent = totalMined + ' QTC';
                showNotification('Block mined! Reward: 10 QTC (simulated)');
            }
        }
    }, 3000); // Mine every 3 seconds for demo
}

function stopMining() {
    mining = false;
    document.getElementById('mining-status').textContent = 'Stopped';
    document.getElementById('start-mining').disabled = false;
    document.getElementById('stop-mining').disabled = true;
    document.getElementById('hashrate').textContent = '0 H/s';
}

async function updateBalance() {
    const walletAddress = localStorage.getItem('walletAddress');
    if (!walletAddress) return;
    
    try {
        const response = await fetch(`${API_BASE}/balance/${walletAddress}`);
        if (response.ok) {
            const data = await response.json();
            const balanceElement = document.getElementById('current-balance');
            if (balanceElement) {
                balanceElement.textContent = data.balance + ' QTC';
            }
        }
    } catch (error) {
        console.error('Balance update error:', error);
    }
}

function showNotification(message) {
    const notification = document.createElement('div');
    notification.className = 'notification';
    notification.textContent = message;
    notification.style.cssText = `
        position: fixed;
        top: 20px;
        right: 20px;
        background: #4CAF50;
        color: white;
        padding: 15px;
        border-radius: 5px;
        z-index: 1000;
        animation: slideIn 0.5s ease-in-out;
        box-shadow: 0 4px 8px rgba(0,0,0,0.2);
    `;
    
    document.body.appendChild(notification);
    
    setTimeout(() => {
        notification.remove();
    }, 4000);
}

// Initialize mining page
document.addEventListener('DOMContentLoaded', function() {
    // Generate wallet address if not exists
    if (!localStorage.getItem('walletAddress')) {
        const randomSuffix = Math.random().toString(36).substr(2, 20);
        localStorage.setItem('walletAddress', `qtc_${randomSuffix}`);
    }
    
    // Display wallet address
    const walletElement = document.getElementById('wallet-address');
    if (walletElement) {
        walletElement.textContent = localStorage.getItem('walletAddress');
    }
    
    updateBalance();
});
