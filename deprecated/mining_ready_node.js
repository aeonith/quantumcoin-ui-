#!/usr/bin/env node
// Mining-Ready QuantumCoin Node - Bitcoin-like Implementation
// With proper /template and /submit endpoints for pool mining

const http = require('http');
const crypto = require('crypto');
const fs = require('fs');
const { v4: uuidv4 } = require('uuid');

const PORT = 8545;
const TARGET_BLOCK_TIME = 600; // 10 minutes (600 seconds) like Bitcoin
const DIFFICULTY_ADJUSTMENT_INTERVAL = 144; // Adjust every 144 blocks (~24 hours with 10min blocks)
const HALVING_INTERVAL = 105120; // Every 2 years as specified in the spec
const MAX_SUPPLY = 2200000000000000; // 22M QTC in satoshis

// Mining job storage
const miningJobs = new Map(); // job_id -> template

// Real blockchain state with proper Bitcoin-like structure
let blockchain = {
    chain: [],
    height: 0,
    difficulty: 0x1d00ffff, // Bitcoin-style difficulty (bits format)
    target: null,
    totalWork: BigInt(0),
    unspentOutputs: new Map(), // UTXO set
    mempool: new Map(), // Transaction mempool
    blockTimes: [], // Track block times for difficulty adjustment
    totalSupply: BigInt(0)
};

// Load saved state if it exists
const stateFile = 'quantumcoin_blockchain.json';
if (fs.existsSync(stateFile)) {
    try {
        const saved = JSON.parse(fs.readFileSync(stateFile));
        blockchain.chain = saved.chain || [];
        blockchain.height = saved.height || 0;
        blockchain.difficulty = saved.difficulty || 0x1d00ffff;
        blockchain.target = BigInt(saved.target || '0x00000000FFFF0000000000000000000000000000000000000000000000000000');
        blockchain.totalSupply = BigInt(saved.totalSupply || '0');
        blockchain.blockTimes = saved.blockTimes || [];
        console.log(`📂 Loaded blockchain state: ${blockchain.chain.length} blocks, height ${blockchain.height}`);
    } catch (err) {
        console.log('⚠️  Could not load saved state, starting fresh');
    }
}

// Convert difficulty bits to target (Bitcoin format)
function bitsToTarget(bits) {
    const exponent = bits >> 24;
    const mantissa = bits & 0xffffff;
    return BigInt(mantissa) * (BigInt(2) ** BigInt(8 * (exponent - 3)));
}

// Convert target back to bits
function targetToBits(target) {
    let targetHex = target.toString(16).padStart(64, '0');
    let bytes = Math.ceil(targetHex.length / 2);
    let mantissa = parseInt(targetHex.substring(0, 6), 16);
    if (mantissa > 0x7fffff) {
        mantissa >>= 8;
        bytes++;
    }
    return (bytes << 24) | mantissa;
}

// Calculate mining reward (with halving)
function getMiningReward(height) {
    const halvings = Math.floor(height / HALVING_INTERVAL);
    if (halvings >= 33) return 0; // All coins mined
    
    const baseReward = 5000000000; // 50 QTC in satoshis
    return Math.floor(baseReward / Math.pow(2, halvings));
}

// Real Bitcoin-like difficulty adjustment
function adjustDifficulty(blockchain) {
    if (blockchain.height % DIFFICULTY_ADJUSTMENT_INTERVAL !== 0) {
        return blockchain.difficulty; // No adjustment needed
    }
    
    if (blockchain.blockTimes.length < DIFFICULTY_ADJUSTMENT_INTERVAL) {
        return blockchain.difficulty; // Not enough data
    }
    
    // Get last 144 block times
    const recentTimes = blockchain.blockTimes.slice(-DIFFICULTY_ADJUSTMENT_INTERVAL);
    const timeSpan = recentTimes[recentTimes.length - 1] - recentTimes[0];
    const targetTimeSpan = TARGET_BLOCK_TIME * DIFFICULTY_ADJUSTMENT_INTERVAL;
    
    // Limit adjustment to 4x easier or 1/4 harder (Bitcoin rule)
    const maxAdjustment = 4;
    let adjustment = timeSpan / targetTimeSpan;
    
    if (adjustment > maxAdjustment) adjustment = maxAdjustment;
    if (adjustment < 1/maxAdjustment) adjustment = 1/maxAdjustment;
    
    // Calculate new target
    const currentTarget = bitsToTarget(blockchain.difficulty);
    const newTarget = currentTarget * BigInt(Math.floor(adjustment * 1000000)) / BigInt(1000000);
    
    // Convert back to bits format
    const newDifficulty = targetToBits(newTarget);
    
    console.log(`📊 Difficulty Adjustment at block ${blockchain.height}:`);
    console.log(`   Time span: ${Math.floor(timeSpan/60)} minutes (target: ${targetTimeSpan/60} minutes)`);
    console.log(`   Adjustment: ${(adjustment * 100).toFixed(2)}%`);
    console.log(`   Old difficulty: 0x${blockchain.difficulty.toString(16)}`);
    console.log(`   New difficulty: 0x${newDifficulty.toString(16)}`);
    
    return newDifficulty;
}

// Calculate merkle root (simplified)
function calculateMerkleRoot(transactions) {
    if (transactions.length === 0) return "0".repeat(64);
    
    let hashes = transactions.map(tx => 
        crypto.createHash('sha256').update(JSON.stringify(tx)).digest('hex')
    );
    
    while (hashes.length > 1) {
        const newHashes = [];
        for (let i = 0; i < hashes.length; i += 2) {
            const left = hashes[i];
            const right = hashes[i + 1] || hashes[i]; // Duplicate last if odd
            const combined = crypto.createHash('sha256')
                .update(crypto.createHash('sha256').update(left + right).digest())
                .digest('hex');
            newHashes.push(combined);
        }
        hashes = newHashes;
    }
    
    return hashes[0];
}

// Create genesis block
function createGenesisBlock() {
    const genesis = {
        version: 1,
        height: 0,
        timestamp: Math.floor(Date.now() / 1000),
        previousHash: "0".repeat(64),
        merkleRoot: "0".repeat(64),
        difficulty: 0x1d00ffff, // Initial difficulty
        nonce: 0,
        transactions: [{
            type: "coinbase",
            id: "genesis_coinbase",
            inputs: [],
            outputs: [],
            amount: 0,
            fee: 0,
            timestamp: Math.floor(Date.now() / 1000),
            message: "QuantumCoin Genesis - Fair Launch, 22M Cap, Zero Premine"
        }],
        hash: null
    };
    
    // Calculate genesis block hash
    const headerData = `${genesis.version}${genesis.previousHash}${genesis.merkleRoot}${genesis.timestamp}${genesis.difficulty}${genesis.nonce}`;
    genesis.hash = crypto.createHash('sha256')
        .update(crypto.createHash('sha256').update(headerData).digest())
        .digest('hex');
    
    return genesis;
}

// Initialize blockchain with genesis
if (blockchain.chain.length === 0) {
    const genesis = createGenesisBlock();
    blockchain.chain.push(genesis);
    blockchain.height = 0;
    blockchain.target = bitsToTarget(blockchain.difficulty);
    blockchain.blockTimes.push(genesis.timestamp);
    
    console.log('🎬 Genesis block created:');
    console.log(`   Hash: ${genesis.hash}`);
    console.log(`   Difficulty: 0x${genesis.difficulty.toString(16)}`);
    console.log(`   Target: 0x${blockchain.target.toString(16)}`);
}

// Validate proof of work using Bitcoin-style header hashing
function validateProofOfWork(version, prevHash, merkleRoot, timestamp, bits, nonce, target) {
    // Create Bitcoin-style header (80 bytes)
    const buffer = Buffer.alloc(80);
    let offset = 0;
    
    buffer.writeUInt32BE(version, offset); offset += 4;
    Buffer.from(prevHash, 'hex').reverse().copy(buffer, offset); offset += 32; // Little endian
    Buffer.from(merkleRoot, 'hex').reverse().copy(buffer, offset); offset += 32; // Little endian
    buffer.writeUInt32BE(timestamp, offset); offset += 4;
    buffer.writeUInt32BE(bits, offset); offset += 4;
    buffer.writeUInt32BE(nonce, offset); offset += 4;
    
    // Double SHA256
    const hash1 = crypto.createHash('sha256').update(buffer).digest();
    const hash2 = crypto.createHash('sha256').update(hash1).digest();
    
    // Convert to big endian hex
    const blockHash = hash2.reverse().toString('hex');
    const hashBigInt = BigInt('0x' + blockHash);
    
    return { valid: hashBigInt < target, hash: blockHash };
}

// Add new block to blockchain
function addBlock(newBlock) {
    // Validate the block
    const previousBlock = blockchain.chain[blockchain.chain.length - 1];
    if (newBlock.previousHash !== previousBlock.hash) {
        throw new Error('Invalid previous hash');
    }
    
    // Validate proof of work
    const pow = validateProofOfWork(
        newBlock.version,
        newBlock.previousHash,
        newBlock.merkleRoot,
        newBlock.timestamp,
        newBlock.difficulty,
        newBlock.nonce,
        blockchain.target
    );
    
    if (!pow.valid) {
        throw new Error('Invalid proof of work');
    }
    
    newBlock.hash = pow.hash;
    newBlock.height = blockchain.height + 1;
    
    // Add to blockchain
    blockchain.chain.push(newBlock);
    blockchain.height++;
    blockchain.blockTimes.push(newBlock.timestamp);
    
    // Update total supply
    const reward = getMiningReward(newBlock.height);
    blockchain.totalSupply += BigInt(reward);
    
    // Adjust difficulty if needed
    const newDifficulty = adjustDifficulty(blockchain);
    if (newDifficulty !== blockchain.difficulty) {
        blockchain.difficulty = newDifficulty;
        blockchain.target = bitsToTarget(newDifficulty);
    }
    
    console.log(`✅ Block ${newBlock.height} added to blockchain`);
    console.log(`   Hash: ${newBlock.hash}`);
    console.log(`   Reward: ${reward / 100000000} QTC`);
    console.log(`   Total Supply: ${Number(blockchain.totalSupply) / 100000000} QTC`);
    console.log(`   Difficulty: 0x${blockchain.difficulty.toString(16)}`);
    
    // Save state
    saveBlockchainState();
    
    return newBlock;
}

// Save blockchain state
function saveBlockchainState() {
    try {
        fs.writeFileSync(stateFile, JSON.stringify({
            chain: blockchain.chain,
            height: blockchain.height,
            difficulty: blockchain.difficulty,
            target: blockchain.target.toString(),
            totalSupply: blockchain.totalSupply.toString(),
            blockTimes: blockchain.blockTimes
        }, null, 2));
    } catch (err) {
        console.error('❌ Could not save blockchain state:', err.message);
    }
}

// Create mining template
function createMiningTemplate() {
    const previousBlock = blockchain.chain[blockchain.chain.length - 1];
    const timestamp = Math.floor(Date.now() / 1000);
    const height = blockchain.height + 1;
    const reward = getMiningReward(height);
    
    // Create coinbase transaction
    const coinbaseTx = {
        type: "coinbase",
        id: `coinbase_${height}_${timestamp}`,
        inputs: [{
            txid: "0".repeat(64),
            vout: 0xffffffff,
            scriptSig: `Block ${height} mined at ${timestamp}`
        }],
        outputs: [{
            value: reward,
            scriptPubKey: "coinbase_output"
        }],
        timestamp
    };
    
    const transactions = [coinbaseTx];
    const merkleRoot = calculateMerkleRoot(transactions);
    const job_id = uuidv4();
    
    const template = {
        job_id,
        version: 1,
        height,
        prev_hash: previousBlock.hash,
        merkle_root: merkleRoot,
        timestamp,
        bits: blockchain.difficulty,
        expires_in: 30, // seconds
        transactions
    };
    
    // Store job
    miningJobs.set(job_id, {
        template,
        created_at: Date.now(),
        expires_at: Date.now() + (30 * 1000)
    });
    
    // Clean old jobs
    for (const [id, job] of miningJobs) {
        if (Date.now() > job.expires_at) {
            miningJobs.delete(id);
        }
    }
    
    return template;
}

// Handle mining endpoints
function handleMiningEndpoint(req, res, path) {
    if (path === '/template') {
        // GET /template - return mining template
        const template = createMiningTemplate();
        
        res.writeHead(200, {
            'Content-Type': 'application/json',
            'Access-Control-Allow-Origin': '*'
        });
        res.end(JSON.stringify(template));
        
        console.log(`⛏️  Mining template created: job ${template.job_id}`);
        
    } else if (path === '/submit') {
        // POST /submit - submit mined block
        let body = '';
        
        req.on('data', chunk => {
            body += chunk.toString();
        });
        
        req.on('end', () => {
            try {
                const submission = JSON.parse(body);
                const { job_id, nonce, timestamp } = submission;
                
                const job = miningJobs.get(job_id);
                if (!job) {
                    res.writeHead(400, {'Content-Type': 'application/json'});
                    res.end(JSON.stringify({status: "rejected", reason: "unknown_job"}));
                    return;
                }
                
                if (Date.now() > job.expires_at) {
                    res.writeHead(400, {'Content-Type': 'application/json'});
                    res.end(JSON.stringify({status: "rejected", reason: "stale_job"}));
                    miningJobs.delete(job_id);
                    return;
                }
                
                const template = job.template;
                
                // Create block from template
                const newBlock = {
                    version: template.version,
                    height: template.height,
                    timestamp: timestamp || template.timestamp,
                    previousHash: template.prev_hash,
                    merkleRoot: template.merkle_root,
                    difficulty: template.bits,
                    nonce: nonce,
                    transactions: template.transactions
                };
                
                try {
                    const addedBlock = addBlock(newBlock);
                    
                    res.writeHead(200, {'Content-Type': 'application/json'});
                    res.end(JSON.stringify({
                        status: "accepted",
                        hash: addedBlock.hash,
                        height: addedBlock.height
                    }));
                    
                    console.log(`🎉 Block accepted! Hash: ${addedBlock.hash}`);
                    miningJobs.delete(job_id);
                    
                } catch (err) {
                    res.writeHead(400, {'Content-Type': 'application/json'});
                    res.end(JSON.stringify({
                        status: "rejected",
                        reason: err.message
                    }));
                    
                    console.log(`❌ Block rejected: ${err.message}`);
                }
                
            } catch (err) {
                res.writeHead(400, {'Content-Type': 'application/json'});
                res.end(JSON.stringify({
                    status: "rejected",
                    reason: "invalid_submission"
                }));
            }
        });
    } else {
        res.writeHead(404, {'Content-Type': 'application/json'});
        res.end(JSON.stringify({error: "endpoint_not_found"}));
    }
}

// Enhanced RPC handler (keeping existing RPC methods)
function handleRPC(req, res) {
    let body = '';
    
    req.on('data', chunk => {
        body += chunk.toString();
    });
    
    req.on('end', () => {
        try {
            const request = JSON.parse(body);
            console.log(`🔧 RPC: ${request.method}`);
            
            let response = {
                jsonrpc: "2.0",
                id: request.id,
                result: null,
                error: null
            };
            
            switch (request.method) {
                case 'getblockchaininfo':
                case 'getinfo':
                    response.result = {
                        version: "1.0.0",
                        network: "QuantumCoin Mainnet",
                        height: blockchain.height,
                        bestblockhash: blockchain.chain[blockchain.chain.length - 1].hash,
                        difficulty: blockchain.difficulty,
                        target: '0x' + blockchain.target.toString(16),
                        totalwork: blockchain.totalWork.toString(),
                        supply: {
                            max: MAX_SUPPLY,
                            current: Number(blockchain.totalSupply),
                            premine: 0
                        },
                        halvingInterval: HALVING_INTERVAL,
                        nextHalving: HALVING_INTERVAL - (blockchain.height % HALVING_INTERVAL),
                        targetBlockTime: TARGET_BLOCK_TIME,
                        difficultyAdjustmentInterval: DIFFICULTY_ADJUSTMENT_INTERVAL
                    };
                    break;
                    
                case 'getmininginfo':
                    const currentReward = getMiningReward(blockchain.height + 1);
                    const avgBlockTime = blockchain.blockTimes.length > 1 ? 
                        (blockchain.blockTimes[blockchain.blockTimes.length - 1] - blockchain.blockTimes[0]) / (blockchain.blockTimes.length - 1) : 0;
                    
                    response.result = {
                        height: blockchain.height,
                        difficulty: blockchain.difficulty,
                        target: '0x' + blockchain.target.toString(16),
                        reward: currentReward,
                        rewardQTC: currentReward / 100000000,
                        avgBlockTime: Math.floor(avgBlockTime),
                        targetBlockTime: TARGET_BLOCK_TIME,
                        blocksUntilAdjustment: DIFFICULTY_ADJUSTMENT_INTERVAL - (blockchain.height % DIFFICULTY_ADJUSTMENT_INTERVAL),
                        nextHalving: HALVING_INTERVAL - (blockchain.height % HALVING_INTERVAL)
                    };
                    break;
                    
                case 'getblock':
                    const blockHeight = request.params?.height || request.params?.[0] || blockchain.height;
                    const block = blockchain.chain[blockHeight];
                    
                    if (block) {
                        response.result = {
                            ...block,
                            confirmations: blockchain.height - block.height + 1,
                            size: JSON.stringify(block).length,
                            reward: getMiningReward(block.height)
                        };
                    } else {
                        response.error = "Block not found";
                    }
                    break;
                    
                default:
                    response.error = `Unknown method: ${request.method}`;
            }
            
            res.writeHead(200, {
                'Content-Type': 'application/json',
                'Access-Control-Allow-Origin': '*',
                'Access-Control-Allow-Methods': 'POST, GET, OPTIONS',
                'Access-Control-Allow-Headers': 'Content-Type'
            });
            
            res.end(JSON.stringify(response));
            
        } catch (err) {
            res.writeHead(400, {'Content-Type': 'application/json'});
            res.end(JSON.stringify({
                jsonrpc: "2.0",
                error: "Invalid JSON-RPC request",
                id: null
            }));
        }
    });
}

// Create server
const server = http.createServer((req, res) => {
    const url = new URL(req.url, `http://localhost:${PORT}`);
    const path = url.pathname;
    
    if (path === '/template' || path === '/submit') {
        handleMiningEndpoint(req, res, path);
    } else if (req.method === 'POST' && path === '/') {
        handleRPC(req, res);
    } else if (req.method === 'OPTIONS') {
        res.writeHead(200, {
            'Access-Control-Allow-Origin': '*',
            'Access-Control-Allow-Methods': 'POST, GET, OPTIONS',
            'Access-Control-Allow-Headers': 'Content-Type'
        });
        res.end();
    } else {
        res.writeHead(200, {'Content-Type': 'text/html'});
        res.end(`
<!DOCTYPE html>
<html>
<head><title>QuantumCoin Mining-Ready Node</title></head>
<body>
<h1>⚛️ QuantumCoin Mining-Ready Node</h1>
<p><strong>Network:</strong> QuantumCoin Mainnet (Bitcoin-like)</p>
<p><strong>Block Height:</strong> ${blockchain.height}</p>
<p><strong>Difficulty:</strong> 0x${blockchain.difficulty.toString(16)}</p>
<p><strong>Target:</strong> 0x${blockchain.target.toString(16)}</p>
<p><strong>Total Supply:</strong> ${Number(blockchain.totalSupply) / 100000000} QTC</p>
<p><strong>Current Reward:</strong> ${getMiningReward(blockchain.height + 1) / 100000000} QTC</p>
<p><strong>Target Block Time:</strong> ${TARGET_BLOCK_TIME} seconds (${TARGET_BLOCK_TIME/60} minutes)</p>
<hr>
<h3>⛏️ Mining Endpoints:</h3>
<ul>
<li><strong>GET /template</strong> - Get mining work template</li>
<li><strong>POST /submit</strong> - Submit mined block</li>
</ul>
<h3>🧪 RPC Methods:</h3>
<ul>
<li>getblockchaininfo - Full blockchain information</li>
<li>getmininginfo - Current mining parameters</li>
<li>getblock - Get specific block data</li>
</ul>
<hr>
<p><strong>Active Mining Jobs:</strong> ${miningJobs.size}</p>
</body>
</html>
        `);
    }
});

// Install uuid if not present
try {
    require('uuid');
} catch (err) {
    console.log('⚠️  Installing uuid dependency...');
    const { execSync } = require('child_process');
    try {
        execSync('npm install uuid', { stdio: 'inherit' });
        console.log('✅ uuid installed successfully');
    } catch (installErr) {
        console.log('❌ Could not install uuid. Please run: npm install uuid');
        process.exit(1);
    }
}

// Start server
server.listen(PORT, '0.0.0.0', () => {
    console.log('');
    console.log('⚛️ QuantumCoin Mining-Ready Node Started!');
    console.log('=========================================');
    console.log(`📡 RPC Server: http://localhost:${PORT}`);
    console.log(`⛏️  Mining API: http://localhost:${PORT}/template`);
    console.log(`📤 Submit API: http://localhost:${PORT}/submit`);
    console.log(`🌐 Network: QuantumCoin Mainnet`);
    console.log(`🔗 Mining Algorithm: Bitcoin-like SHA256d PoW`);
    console.log(`🎯 Target Block Time: ${TARGET_BLOCK_TIME} seconds (${TARGET_BLOCK_TIME/60} minutes)`);
    console.log(`📊 Difficulty Adjustment: Every ${DIFFICULTY_ADJUSTMENT_INTERVAL} blocks`);
    console.log(`💰 Current Mining Reward: ${getMiningReward(blockchain.height + 1) / 100000000} QTC`);
    console.log(`🔄 Halving Interval: ${HALVING_INTERVAL} blocks (~2 years)`);
    console.log(`💎 Max Supply: ${MAX_SUPPLY / 100000000} QTC`);
    console.log(`⚖️  Current Difficulty: 0x${blockchain.difficulty.toString(16)}`);
    console.log(`🎯 Current Target: 0x${blockchain.target.toString(16)}`);
    console.log(`📈 Current Height: ${blockchain.height}`);
    console.log(`💰 Total Supply: ${Number(blockchain.totalSupply) / 100000000} QTC`);
    console.log('');
    console.log('🔗 Ready for real Bitcoin-like pool mining!');
    console.log('');
});

// Graceful shutdown
process.on('SIGINT', () => {
    console.log('');
    console.log('🛑 Shutting down QuantumCoin mining-ready node...');
    
    saveBlockchainState();
    console.log(`💾 Blockchain state saved to ${stateFile}`);
    console.log('✅ Node stopped gracefully');
    
    server.close(() => {
        process.exit(0);
    });
});
