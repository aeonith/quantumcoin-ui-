#!/usr/bin/env node
// QuantumCoin Simple Miner
// Mines blocks and interacts with the QuantumCoin node

const http = require('http');
const crypto = require('crypto');

const NODE_URL = 'http://localhost:8545';
let MINING_REWARD = 5000000000; // 50 QTC in satoshis (8 decimal places)
let TARGET_DIFFICULTY = "1d00ffff";

// Mining address (generate a simple address)
const MINING_ADDRESS = "qtc1q" + crypto.randomBytes(20).toString('hex');

console.log('⛏️  QuantumCoin Simple Miner Starting');
console.log('====================================');
console.log(`💰 Mining Address: ${MINING_ADDRESS}`);
console.log(`🎯 Target Difficulty: ${TARGET_DIFFICULTY}`);
console.log(`💎 Block Reward: ${MINING_REWARD / 100000000} QTC`);
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

// Simple proof-of-work mining function
function mine(blockData) {
    let nonce = 0;
    const target = parseInt(TARGET_DIFFICULTY, 16);
    
    console.log(`⚡ Mining block with data: ${JSON.stringify(blockData).substring(0, 100)}...`);
    
    const startTime = Date.now();
    
    while (true) {
        const blockString = JSON.stringify({
            ...blockData,
            nonce: nonce,
            timestamp: Date.now(),
            miner: MINING_ADDRESS
        });
        
        const hash = crypto.createHash('sha256').update(blockString).digest('hex');
        const hashNum = parseInt(hash.substring(0, 8), 16);
        
        if (hashNum < target) {
            const endTime = Date.now();
            const timeElapsed = (endTime - startTime) / 1000;
            const hashRate = Math.round(nonce / timeElapsed);
            
            console.log(`🎉 Block mined!`);
            console.log(`   Hash: ${hash}`);
            console.log(`   Nonce: ${nonce}`);
            console.log(`   Time: ${timeElapsed}s`);
            console.log(`   Hash Rate: ${hashRate} H/s`);
            
            return {
                hash: hash,
                nonce: nonce,
                timestamp: Date.now(),
                miner: MINING_ADDRESS,
                data: blockData
            };
        }
        
        nonce++;
        
        // Show progress every 10000 attempts
        if (nonce % 10000 === 0) {
            const hashRate = Math.round(nonce / ((Date.now() - startTime) / 1000));
            process.stdout.write(`\r🔍 Mining... Nonce: ${nonce}, Hash Rate: ${hashRate} H/s`);
        }
    }
}

// Main mining loop
async function startMining() {
    let blockHeight = 0;
    let mineCount = 0;
    
    console.log('🚀 Starting mining operations...');
    console.log('');
    
    while (true) {
        try {
            // Get current blockchain status
            const nodeInfo = await rpcCall('getinfo');
            if (nodeInfo.result) {
                console.log(`📊 Node Status: Height ${nodeInfo.result.height}, Peers ${nodeInfo.result.peers}`);
            }
            
            // Create a new block to mine
            const blockData = {
                height: blockHeight + 1,
                previousHash: blockHeight === 0 ? "0".repeat(64) : crypto.randomBytes(32).toString('hex'),
                transactions: [
                    {
                        type: "coinbase",
                        to: MINING_ADDRESS,
                        amount: MINING_REWARD,
                        fee: 0,
                        timestamp: Date.now(),
                        hash: crypto.randomBytes(32).toString('hex')
                    }
                ],
                difficulty: TARGET_DIFFICULTY
            };
            
            // Mine the block
            console.log(`\n⛏️  Mining Block #${blockData.height}...`);
            const minedBlock = mine(blockData);
            
            mineCount++;
            blockHeight++;
            
            console.log(`\n✅ Successfully mined block #${blockHeight}!`);
            console.log(`💰 Earned ${MINING_REWARD / 100000000} QTC`);
            console.log(`📈 Total blocks mined: ${mineCount}`);
            console.log(`💎 Total earnings: ${(mineCount * MINING_REWARD) / 100000000} QTC`);
            
            // Try to submit block to node (this is just a simulation since our node is simple)
            try {
                const submitResult = await rpcCall('qc_sendTransaction', {
                    from: "coinbase",
                    to: MINING_ADDRESS,
                    amount: MINING_REWARD,
                    blockHash: minedBlock.hash
                });
                
                if (submitResult.result) {
                    console.log(`🎯 Block submitted: ${submitResult.result.transactionHash}`);
                }
            } catch (err) {
                console.log(`⚠️  Block submission failed (node simulation)`);
            }
            
            console.log('\n' + '='.repeat(50));
            
            // Wait a bit before mining next block
            await new Promise(resolve => setTimeout(resolve, 2000));
            
        } catch (err) {
            console.error('❌ Mining error:', err.message);
            console.log('⏳ Retrying in 5 seconds...');
            await new Promise(resolve => setTimeout(resolve, 5000));
        }
    }
}

// Handle graceful shutdown
process.on('SIGINT', () => {
    console.log('\n\n🛑 Stopping miner...');
    console.log('💰 Mining session completed');
    console.log(`🏆 Final mining address: ${MINING_ADDRESS}`);
    process.exit(0);
});

// Start mining
console.log('🔥 Initializing miner...');
setTimeout(() => {
    startMining().catch(err => {
        console.error('💥 Fatal mining error:', err);
        process.exit(1);
    });
}, 2000);

console.log('⏱️  Miner will start in 2 seconds...');
console.log('💡 Press Ctrl+C to stop mining');
