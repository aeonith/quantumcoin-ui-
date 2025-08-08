// QuantumCoin Mining Interface JavaScript

const API_BASE = '/rpc';
let miningActive = false;
let miningWorker = null;
let miningStartTime = null;
let currentNonce = 0;
let blocksMined = 0;
let hashRate = 0;
let statsInterval = null;

// Initialize mining interface
document.addEventListener('DOMContentLoaded', function() {
    loadConfig();
    updateNetworkStats();
    
    // Update stats every 5 seconds
    statsInterval = setInterval(updateStats, 5000);
    
    // Handle mining mode change
    document.getElementById('miningMode').addEventListener('change', function() {
        const poolGroup = document.getElementById('poolAddressGroup');
        poolGroup.style.display = this.value === 'pool' ? 'block' : 'none';
    });
    
    // Auto-generate address if empty
    const addressInput = document.getElementById('miningAddress');
    if (!addressInput.value) {
        generateAddress();
    }
});

// RPC call helper
async function rpcCall(method, params = []) {
    try {
        const response = await fetch(API_BASE, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                jsonrpc: '2.0',
                method: method,
                params: params,
                id: Date.now()
            })
        });

        const data = await response.json();
        
        if (data.error) {
            throw new Error(data.error.message);
        }
        
        return data.result;
    } catch (error) {
        console.error('RPC Error:', error);
        addLogEntry('error', `RPC Error: ${error.message}`);
        throw error;
    }
}

// Start mining
async function startMining() {
    const address = document.getElementById('miningAddress').value.trim();
    const threadCount = parseInt(document.getElementById('threadCount').value);
    const mode = document.getElementById('miningMode').value;
    
    if (!address) {
        alert('Please enter a mining address');
        return;
    }
    
    if (miningActive) {
        addLogEntry('warning', 'Mining is already active');
        return;
    }
    
    try {
        addLogEntry('info', `Starting ${mode} mining with ${threadCount} threads`);
        addLogEntry('info', `Mining address: ${address}`);
        
        miningActive = true;
        miningStartTime = Date.now();
        currentNonce = 0;
        
        // Update UI
        updateMiningStatus('mining', 'Mining Active');
        document.getElementById('startMiningBtn').disabled = true;
        document.getElementById('stopMiningBtn').disabled = false;
        
        if (mode === 'solo') {
            await startSoloMining(address, threadCount);
        } else {
            await startPoolMining(address, threadCount);
        }
        
    } catch (error) {
        addLogEntry('error', `Failed to start mining: ${error.message}`);
        stopMining();
    }
}

// Solo mining implementation
async function startSoloMining(address, threadCount) {
    addLogEntry('info', 'Initializing solo mining...');
    
    // Start mining loop
    for (let i = 0; i < threadCount; i++) {
        setTimeout(() => mineBlock(address, i), i * 100);
    }
    
    addLogEntry('success', `Solo mining started with ${threadCount} threads`);
}

// Pool mining implementation
async function startPoolMining(address, threadCount) {
    const poolAddress = document.getElementById('poolAddress').value.trim();
    
    if (!poolAddress) {
        throw new Error('Pool address required for pool mining');
    }
    
    addLogEntry('info', `Connecting to mining pool: ${poolAddress}`);
    
    // TODO: Implement pool mining protocol
    addLogEntry('warning', 'Pool mining not yet fully implemented');
    
    // For now, fall back to solo mining
    await startSoloMining(address, threadCount);
}

// Main mining loop
async function mineBlock(address, threadId) {
    while (miningActive) {
        try {
            // Get block template
            const template = await rpcCall('getblocktemplate');
            
            addLogEntry('info', `Thread ${threadId}: Mining block template received`);
            
            // Mine the block
            const result = await mineBlockTemplate(template, address, threadId);
            
            if (result.success) {
                // Submit the block
                await rpcCall('submitblock', [JSON.stringify(result.block)]);
                
                blocksMined++;
                addLogEntry('success', `🎉 Block mined! Hash: ${result.block.hash}`);
                addLogEntry('success', `💰 Reward: ${template.coinbasevalue / 100000000} QTC`);
                
                // Update stats
                document.getElementById('blocksMined').textContent = blocksMined;
                
                // Brief pause before next block
                await sleep(1000);
            }
            
        } catch (error) {
            addLogEntry('error', `Thread ${threadId} error: ${error.message}`);
            await sleep(5000); // Wait before retrying
        }
    }
}

// Mining algorithm implementation
async function mineBlockTemplate(template, address, threadId) {
    const target = template.target;
    const maxNonce = Math.pow(2, 32);
    const startTime = Date.now();
    let attempts = 0;
    
    // Create block structure
    const block = {
        index: template.height || 0,
        timestamp: Date.now(),
        transactions: template.transactions || [],
        previous_hash: template.previousblockhash || "0",
        hash: "",
        nonce: Math.floor(Math.random() * maxNonce) // Start with random nonce
    };
    
    addLogEntry('info', `Thread ${threadId}: Starting proof of work, target: ${target}`);
    
    for (let nonce = block.nonce; nonce < maxNonce && miningActive; nonce++) {
        block.nonce = nonce;
        currentNonce = nonce;
        attempts++;
        
        // Calculate hash
        block.hash = await calculateBlockHash(block);
        
        // Update progress every 10000 attempts
        if (attempts % 10000 === 0) {
            const progress = (nonce / maxNonce) * 100;
            updateProgress(progress, attempts, Date.now() - startTime);
            
            // Update hash rate
            hashRate = attempts / ((Date.now() - startTime) / 1000);
            document.getElementById('hashRate').textContent = Math.round(hashRate);
            
            // Allow UI to update
            await sleep(1);
        }
        
        // Check if hash meets target
        if (block.hash.startsWith(target)) {
            const timeElapsed = (Date.now() - startTime) / 1000;
            addLogEntry('success', `Thread ${threadId}: Found valid hash in ${timeElapsed.toFixed(2)}s`);
            addLogEntry('success', `Hash: ${block.hash}`);
            addLogEntry('success', `Nonce: ${nonce}, Attempts: ${attempts}`);
            
            return { success: true, block: block };
        }
    }
    
    return { success: false };
}

// Calculate block hash (simplified)
async function calculateBlockHash(block) {
    const data = `${block.index}${block.timestamp}${block.previous_hash}${block.nonce}${JSON.stringify(block.transactions)}`;
    
    // Use Web Crypto API for hashing
    const encoder = new TextEncoder();
    const data_encoded = encoder.encode(data);
    const hashBuffer = await crypto.subtle.digest('SHA-256', data_encoded);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    const hashHex = hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
    
    return hashHex;
}

// Stop mining
function stopMining() {
    if (!miningActive) {
        addLogEntry('warning', 'Mining is not active');
        return;
    }
    
    miningActive = false;
    miningStartTime = null;
    
    // Update UI
    updateMiningStatus('stopped', 'Mining Stopped');
    document.getElementById('startMiningBtn').disabled = false;
    document.getElementById('stopMiningBtn').disabled = true;
    
    // Reset progress
    updateProgress(0, 0, 0);
    
    addLogEntry('info', 'Mining stopped');
}

// Update mining status
function updateMiningStatus(status, text) {
    const indicator = document.getElementById('miningStatus');
    const statusText = document.getElementById('miningStatusText');
    
    indicator.className = `status-indicator status-${status}`;
    statusText.textContent = text;
}

// Update progress display
function updateProgress(percent, attempts, timeMs) {
    document.getElementById('blockProgress').style.width = `${Math.min(percent, 100)}%`;
    document.getElementById('progressPercent').textContent = `${percent.toFixed(2)}%`;
    document.getElementById('currentNonce').textContent = currentNonce.toLocaleString();
    document.getElementById('attempts').textContent = attempts.toLocaleString();
    document.getElementById('blockTime').textContent = formatTime(timeMs / 1000);
}

// Update network statistics
async function updateNetworkStats() {
    try {
        const [miningInfo, blockchainInfo] = await Promise.all([
            rpcCall('getmininginfo'),
            rpcCall('getblockchaininfo')
        ]);
        
        document.getElementById('difficulty').textContent = miningInfo.difficulty;
        document.getElementById('networkHashRate').textContent = formatHashRate(miningInfo.networkhashps);
        document.getElementById('estimatedReward').textContent = (blockchainInfo.current_reward / 100000000).toFixed(8);
        
    } catch (error) {
        addLogEntry('warning', 'Failed to update network stats');
    }
}

// Update mining statistics
function updateStats() {
    if (miningActive && miningStartTime) {
        const elapsed = (Date.now() - miningStartTime) / 1000;
        document.getElementById('miningTime').textContent = formatTime(elapsed);
    }
    
    updateNetworkStats();
}

// Generate new mining address
async function generateAddress() {
    try {
        const address = await rpcCall('getnewaddress');
        document.getElementById('miningAddress').value = address;
        addLogEntry('info', `Generated new address: ${address}`);
    } catch (error) {
        // Fallback to client-side generation
        const address = generateClientAddress();
        document.getElementById('miningAddress').value = address;
        addLogEntry('info', `Generated client address: ${address}`);
    }
}

// Client-side address generation (simplified)
function generateClientAddress() {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    let address = 'QTC';
    for (let i = 0; i < 32; i++) {
        address += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return address;
}

// Save configuration
function saveConfig() {
    const config = {
        address: document.getElementById('miningAddress').value,
        threads: document.getElementById('threadCount').value,
        mode: document.getElementById('miningMode').value,
        poolAddress: document.getElementById('poolAddress').value
    };
    
    localStorage.setItem('quantumcoin_mining_config', JSON.stringify(config));
    addLogEntry('info', 'Configuration saved');
}

// Load configuration
function loadConfig() {
    const saved = localStorage.getItem('quantumcoin_mining_config');
    if (saved) {
        try {
            const config = JSON.parse(saved);
            document.getElementById('miningAddress').value = config.address || '';
            document.getElementById('threadCount').value = config.threads || '4';
            document.getElementById('miningMode').value = config.mode || 'solo';
            document.getElementById('poolAddress').value = config.poolAddress || '';
            
            // Update pool address visibility
            const poolGroup = document.getElementById('poolAddressGroup');
            poolGroup.style.display = config.mode === 'pool' ? 'block' : 'none';
            
            addLogEntry('info', 'Configuration loaded');
        } catch (error) {
            addLogEntry('warning', 'Failed to load saved configuration');
        }
    }
}

// Add log entry
function addLogEntry(type, message) {
    const log = document.getElementById('miningLog');
    const entry = document.createElement('div');
    entry.className = 'log-entry';
    
    const timestamp = new Date().toLocaleTimeString();
    entry.innerHTML = `
        <span class="log-timestamp">[${timestamp}]</span>
        <span class="log-${type}">${message}</span>
    `;
    
    log.appendChild(entry);
    log.scrollTop = log.scrollHeight;
    
    // Keep only last 100 entries
    while (log.children.length > 100) {
        log.removeChild(log.firstChild);
    }
}

// Clear log
function clearLog() {
    document.getElementById('miningLog').innerHTML = '';
    addLogEntry('info', 'Mining log cleared');
}

// Utility functions
function formatTime(seconds) {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = Math.floor(seconds % 60);
    return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
}

function formatHashRate(hashRate) {
    if (hashRate === 0) return '0 H/s';
    const units = ['H/s', 'KH/s', 'MH/s', 'GH/s', 'TH/s'];
    const i = Math.floor(Math.log(hashRate) / Math.log(1000));
    return `${(hashRate / Math.pow(1000, i)).toFixed(2)} ${units[i]}`;
}

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

// Cleanup on page unload
window.addEventListener('beforeunload', function() {
    if (miningActive) {
        stopMining();
    }
    if (statsInterval) {
        clearInterval(statsInterval);
    }
});
