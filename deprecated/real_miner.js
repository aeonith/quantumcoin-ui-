#!/usr/bin/env node
// Real QuantumCoin Miner - Bitcoin-like Implementation
// Proper proof-of-work mining with real computational difficulty

const http = require('http');
const crypto = require('crypto');
const { Worker, isMainThread, parentPort, workerData } = require('worker_threads');

const NODE_URL = 'http://localhost:8545';
const NUM_THREADS = require('os').cpus().length; // Use all CPU cores
const MINING_ADDRESS = "qtc1q" + crypto.randomBytes(20).toString('hex');

console.log('⚛️ QuantumCoin Real Miner Starting');
console.log('==================================');
console.log(`💰 Mining Address: ${MINING_ADDRESS}`);
console.log(`🖥️  CPU Threads: ${NUM_THREADS}`);
console.log(`🔥 Algorithm: Bitcoin-like SHA256d PoW`);
console.log('');

// RPC helper function
function rpcCall(method, params = {}) {
    return new Promise((resolve, reject) => {
        const data = JSON.stringify({
            jsonrpc: "2.0",
            method: method,
            params: params,
            id: Date.now()
        });

        const options = {
            hostname: 'localhost',
            port: 8545,
            path: '/',
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Content-Length': Buffer.byteLength(data)
            }
        };

        const req = http.request(options, (res) => {
            let body = '';
            res.on('data', (chunk) => body += chunk);
            res.on('end', () => {
                try {
                    const response = JSON.parse(body);
                    resolve(response);
                } catch (err) {
                    reject(err);
                }
            });
        });

        req.on('error', reject);
        req.write(data);
        req.end();
    });
}

// Bitcoin-like double SHA256 hashing
function doubleSha256(data) {
    const hash1 = crypto.createHash('sha256').update(data).digest();
    const hash2 = crypto.createHash('sha256').update(hash1).digest('hex');
    return hash2;
}

// Real mining worker (runs in separate thread)
const miningWorker = `
const crypto = require('crypto');
const { parentPort, workerData } = require('worker_threads');

function doubleSha256(data) {
    const hash1 = crypto.createHash('sha256').update(data).digest();
    const hash2 = crypto.createHash('sha256').update(hash1).digest('hex');
    return hash2;
}

function mine(blockTemplate, target, startNonce, nonceStep) {
    let nonce = startNonce;
    const targetBigInt = BigInt('0x' + target);
    let hashCount = 0;
    const startTime = Date.now();
    
    while (true) {
        // Create block data with current nonce
        const blockData = JSON.stringify({
            version: blockTemplate.version,
            height: blockTemplate.height,
            previousHash: blockTemplate.previousblockhash,
            timestamp: blockTemplate.timestamp,
            difficulty: blockTemplate.difficulty,
            nonce: nonce,
            transactions: blockTemplate.transactions || [],
            coinbase: {
                to: workerData.miningAddress,
                amount: blockTemplate.coinbasevalue
            }
        });
        
        // Double SHA256 (Bitcoin-like)
        const blockHash = doubleSha256(blockData);
        const hashBigInt = BigInt('0x' + blockHash);
        
        hashCount++;
        
        // Check if hash meets target
        if (hashBigInt < targetBigInt) {
            const endTime = Date.now();
            const timeElapsed = (endTime - startTime) / 1000;
            const hashRate = Math.floor(hashCount / timeElapsed);
            
            parentPort.postMessage({
                type: 'success',
                blockHash: blockHash,
                nonce: nonce,
                hashCount: hashCount,
                timeElapsed: timeElapsed,
                hashRate: hashRate,
                blockData: blockData
            });
            return;
        }
        
        nonce += nonceStep; // Each worker searches different nonce ranges
        
        // Report progress every 100,000 hashes
        if (hashCount % 100000 === 0) {
            const timeElapsed = (Date.now() - startTime) / 1000;
            const hashRate = Math.floor(hashCount / timeElapsed);
            
            parentPort.postMessage({
                type: 'progress',
                hashCount: hashCount,
                hashRate: hashRate,
                nonce: nonce
            });
        }
        
        // Check for stop signal every 1M hashes
        if (hashCount % 1000000 === 0) {
            // Allow other operations
            if (Date.now() - startTime > 60000) { // Stop after 1 minute to get new template
                parentPort.postMessage({
                    type: 'timeout'
                });
                return;
            }
        }
    }
}

// Start mining when worker receives data
mine(workerData.blockTemplate, workerData.target, workerData.startNonce, workerData.nonceStep);
`;

class RealMiner {
    constructor() {
        this.workers = [];
        this.mining = false;
        this.currentTemplate = null;
        this.totalHashCount = 0;
        this.startTime = Date.now();
        this.blocksFound = 0;
        this.totalEarnings = 0;
    }
    
    async getMiningTemplate() {
        try {
            const response = await rpcCall('getblocktemplate');
            if (response.result) {
                return response.result;
            }
        } catch (err) {
            console.error('❌ Failed to get block template:', err.message);
        }
        return null;
    }
    
    async getMiningInfo() {
        try {
            const response = await rpcCall('getmininginfo');
            if (response.result) {
                return response.result;
            }
        } catch (err) {
            console.error('❌ Failed to get mining info:', err.message);
        }
        return null;
    }
    
    async submitBlock(blockData) {
        try {
            const response = await rpcCall('submitblock', { blockdata: blockData });
            return response.result || response.error;
        } catch (err) {
            console.error('❌ Failed to submit block:', err.message);
            return false;
        }
    }
    
    startWorkers(blockTemplate, target) {
        this.stopWorkers();
        
        console.log(`🚀 Starting ${NUM_THREADS} mining workers...`);
        console.log(`🎯 Target: ${target}`);
        console.log(`💎 Reward: ${blockTemplate.coinbasevalue / 100000000} QTC`);
        console.log(`🏗️  Mining Block #${blockTemplate.height}`);
        
        for (let i = 0; i < NUM_THREADS; i++) {
            const worker = new Worker(miningWorker, {
                eval: true,
                workerData: {
                    blockTemplate: blockTemplate,
                    target: target,
                    startNonce: i * 1000000, // Each worker starts from different nonce
                    nonceStep: NUM_THREADS,   // Workers search different ranges
                    miningAddress: MINING_ADDRESS
                }
            });
            
            worker.on('message', async (message) => {
                if (message.type === 'success') {
                    console.log('\n🎉 BLOCK FOUND!');
                    console.log(`   Hash: ${message.blockHash}`);
                    console.log(`   Nonce: ${message.nonce}`);
                    console.log(`   Hashes: ${message.hashCount.toLocaleString()}`);
                    console.log(`   Time: ${message.timeElapsed.toFixed(2)}s`);
                    console.log(`   Hash Rate: ${message.hashRate.toLocaleString()} H/s`);
                    
                    // Submit the block
                    const result = await this.submitBlock(message.blockData);
                    if (result === "Block accepted") {
                        this.blocksFound++;
                        this.totalEarnings += blockTemplate.coinbasevalue;
                        
                        console.log('✅ Block accepted by network!');
                        console.log(`💰 Earned: ${blockTemplate.coinbasevalue / 100000000} QTC`);
                        console.log(`🏆 Total blocks found: ${this.blocksFound}`);
                        console.log(`💎 Total earnings: ${this.totalEarnings / 100000000} QTC`);
                    } else {
                        console.log('❌ Block rejected:', result);
                    }
                    
                    // Stop all workers and get new template
                    this.stopWorkers();
                    setTimeout(() => this.startMining(), 1000);
                    
                } else if (message.type === 'progress') {
                    // Update total hash count
                    this.totalHashCount += 100000;
                    
                    // Show mining progress every few seconds
                    const now = Date.now();
                    if (now - this.lastProgressReport > 5000) {
                        this.showProgress();
                        this.lastProgressReport = now;
                    }
                    
                } else if (message.type === 'timeout') {
                    // Worker timed out, restart with new template
                    console.log('⏰ Mining template expired, getting new one...');
                    this.stopWorkers();
                    setTimeout(() => this.startMining(), 500);
                }
            });
            
            worker.on('error', (err) => {
                console.error('💥 Worker error:', err);
            });
            
            this.workers.push(worker);
        }
        
        this.lastProgressReport = Date.now();
    }
    
    showProgress() {
        const timeElapsed = (Date.now() - this.startTime) / 1000;
        const totalHashRate = Math.floor(this.totalHashCount / timeElapsed);
        const difficulty = this.currentTemplate ? this.currentTemplate.difficulty : 0;
        
        process.stdout.write(`\\r⛏️  Mining... Hash Rate: ${totalHashRate.toLocaleString()} H/s | ` +
                            `Hashes: ${this.totalHashCount.toLocaleString()} | ` +
                            `Time: ${Math.floor(timeElapsed)}s | ` +
                            `Difficulty: 0x${difficulty.toString(16)}`);
    }
    
    stopWorkers() {
        this.workers.forEach(worker => {
            worker.terminate();
        });
        this.workers = [];
    }
    
    async startMining() {
        console.log('🔍 Getting mining template...');
        
        // Get mining info and template
        const [miningInfo, blockTemplate] = await Promise.all([
            this.getMiningInfo(),
            this.getMiningTemplate()
        ]);
        
        if (!blockTemplate || !miningInfo) {
            console.error('❌ Failed to get mining data, retrying in 10 seconds...');
            setTimeout(() => this.startMining(), 10000);
            return;
        }
        
        console.log('\n📊 Mining Information:');
        console.log(`   Current Height: ${miningInfo.height}`);
        console.log(`   Difficulty: 0x${miningInfo.difficulty.toString(16)}`);
        console.log(`   Target: ${miningInfo.target}`);
        console.log(`   Block Reward: ${miningInfo.rewardQTC} QTC`);
        console.log(`   Target Block Time: ${miningInfo.targetBlockTime}s (${miningInfo.targetBlockTime/60} minutes)`);
        console.log(`   Blocks Until Difficulty Adjustment: ${miningInfo.blocksUntilAdjustment}`);
        console.log(`   Blocks Until Halving: ${miningInfo.nextHalving}`);
        console.log('');
        
        this.currentTemplate = blockTemplate;
        this.mining = true;
        
        // Start mining workers
        this.startWorkers(blockTemplate, miningInfo.target.replace('0x', ''));
    }
    
    async stop() {
        console.log('\\n🛑 Stopping miner...');
        this.mining = false;
        this.stopWorkers();
        
        const timeElapsed = (Date.now() - this.startTime) / 1000;
        const avgHashRate = Math.floor(this.totalHashCount / timeElapsed);
        
        console.log('\\n📊 Final Mining Statistics:');
        console.log(`   Total Time: ${Math.floor(timeElapsed)} seconds (${Math.floor(timeElapsed/60)} minutes)`);
        console.log(`   Total Hashes: ${this.totalHashCount.toLocaleString()}`);
        console.log(`   Average Hash Rate: ${avgHashRate.toLocaleString()} H/s`);
        console.log(`   Blocks Found: ${this.blocksFound}`);
        console.log(`   Total Earnings: ${this.totalEarnings / 100000000} QTC`);
        console.log(`   Mining Address: ${MINING_ADDRESS}`);
        console.log('');
        process.exit(0);
    }
}

// Main execution
if (isMainThread) {
    const miner = new RealMiner();
    
    // Handle graceful shutdown
    process.on('SIGINT', () => miner.stop());
    process.on('SIGTERM', () => miner.stop());
    
    // Start mining after a brief delay
    console.log('🔥 Initializing real mining system...');
    console.log('⏱️  Starting in 3 seconds...');
    console.log('💡 Press Ctrl+C to stop mining');
    console.log('');
    
    setTimeout(() => {
        miner.startMining();
    }, 3000);
}
