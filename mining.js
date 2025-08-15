// REAL QUANTUMCOIN MINING SYSTEM - PRODUCTION GRADE
const API_BASE = 'http://localhost:8080';

// REAL mining state
let mining = false;
let hashRate = 0;
let totalMined = 0;
let blocksMined = 0;
let miningWorker = null;
let difficulty = 0;
let networkHashrate = 0;
let lastBlockReward = 0;
let estimatedEarnings = 0;

// REAL MINING FUNCTIONS - PRODUCTION CRYPTOCURRENCY MINING
async function startRealMining() {
    if (mining) return;
    
    const walletAddress = localStorage.getItem('walletAddress');
    if (!walletAddress) {
        alert('❌ Please generate a wallet first before mining');
        return;
    }

    try {
        // Start REAL mining via backend API
        const response = await fetch(`${API_BASE}/mining/start`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                minerAddress: walletAddress,
                threadCount: parseInt(document.getElementById('thread-count')?.value || '4'),
                difficulty: 'auto'
            })
        });

        if (!response.ok) {
            throw new Error(`Mining start failed: ${response.status}`);
        }

        const result = await response.json();
        
        if (result.success) {
            mining = true;
            updateMiningUI(true);
            showNotification('⛏️ Real mining started successfully!', 'success');
            
            // Start real-time mining monitoring
            startMiningMonitoring();
        } else {
            throw new Error(result.error || 'Mining failed to start');
        }
        
    } catch (error) {
        console.error('Real mining error:', error);
        showNotification(`❌ Mining failed: ${error.message}`, 'error');
    }
}

async function stopRealMining() {
    try {
        const response = await fetch(`${API_BASE}/mining/stop`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' }
        });

        const result = await response.json();
        
        mining = false;
        updateMiningUI(false);
        showNotification('⏹️ Mining stopped', 'info');
        
    } catch (error) {
        console.error('Stop mining error:', error);
        showNotification('❌ Failed to stop mining', 'error');
    }
}

function startMiningMonitoring() {
    // Monitor REAL mining status every 5 seconds
    const monitoringInterval = setInterval(async () => {
        if (!mining) {
            clearInterval(monitoringInterval);
            return;
        }
        
        try {
            const response = await fetch(`${API_BASE}/mining/status`);
            const status = await response.json();
            
            if (status.success) {
                updateMiningStats(status.mining);
                updateNetworkData(status.network);
                updateEarningsData(status.earnings);
            }
            
        } catch (error) {
            console.error('Mining monitoring error:', error);
        }
    }, 5000);
}

function updateMiningStats(miningData) {
    hashRate = miningData.hashrate || 0;
    blocksMined = miningData.blocksMined || 0;
    difficulty = miningData.difficulty || 0;
    
    document.getElementById('hashrate').textContent = `${hashRate.toLocaleString()} H/s`;
    document.getElementById('blocks-mined').textContent = blocksMined.toLocaleString();
    document.getElementById('difficulty').textContent = parseFloat(difficulty).toFixed(6);
    document.getElementById('mining-uptime').textContent = miningData.uptime || '0h 0m';
}

function updateNetworkData(networkData) {
    networkHashrate = networkData.networkHashrate || 0;
    blockchainHeight = networkData.height || 0;
    
    document.getElementById('network-height').textContent = blockchainHeight.toLocaleString();
    document.getElementById('network-hashrate').textContent = networkData.networkHashrate || '0 H/s';
    document.getElementById('total-supply').textContent = `${(networkData.totalSupply || 0).toLocaleString()} QTC`;
}

function updateEarningsData(earningsData) {
    estimatedEarnings = earningsData.todayQTC || 0;
    lastBlockReward = earningsData.lastReward || 0;
    
    document.getElementById('today-earnings').textContent = `${estimatedEarnings.toFixed(8)} QTC`;
    document.getElementById('total-earnings').textContent = `${(earningsData.totalQTC || 0).toFixed(8)} QTC`;
    document.getElementById('estimated-daily').textContent = `${(earningsData.estimatedDailyQTC || 0).toFixed(8)} QTC`;
}

function updateMiningUI(isMining) {
    const startBtn = document.getElementById('start-mining');
    const stopBtn = document.getElementById('stop-mining');
    const statusEl = document.getElementById('mining-status');
    
    if (startBtn) startBtn.disabled = isMining;
    if (stopBtn) stopBtn.disabled = !isMining;
    if (statusEl) statusEl.textContent = isMining ? 'Mining Active' : 'Mining Stopped';
}

async function startMining() {
    await startRealMining();
}

async function stopMining() {
    await stopRealMining();
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
