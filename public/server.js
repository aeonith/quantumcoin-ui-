const express = require('express');
const cors = require('cors');
const bodyParser = require('body-parser');
const fs = require('fs');
const path = require('path');

const app = express();
const PORT = process.env.PORT || 8080;

// Middleware
app.use(cors());
app.use(bodyParser.json());
app.use(bodyParser.urlencoded({ extended: true }));

// Serve static files
app.use(express.static('.'));

// Mock blockchain state
let blockchainState = {
    height: 142857,
    difficulty: 4,
    hashRate: 125.67,
    networkHashRate: 1256700,
    totalSupply: 8532149,
    circulatingSupply: 8532149
};

let miningStats = {
    isRunning: false,
    hashRate: 0,
    blocksMined: 0,
    currentDifficulty: 4,
    estimatedReward: 25.5,
    miningTime: '00:00:00'
};

// API Routes
app.get('/api/status', (req, res) => {
    res.json({
        status: 'ok',
        blockchain: blockchainState,
        mining: miningStats
    });
});

app.get('/api/blockchain/info', (req, res) => {
    res.json(blockchainState);
});

app.get('/api/mining/status', (req, res) => {
    res.json(miningStats);
});

app.post('/api/mining/start', (req, res) => {
    miningStats.isRunning = true;
    miningStats.hashRate = Math.random() * 100 + 50;
    res.json({ status: 'Mining started', mining: miningStats });
});

app.post('/api/mining/stop', (req, res) => {
    miningStats.isRunning = false;
    miningStats.hashRate = 0;
    res.json({ status: 'Mining stopped', mining: miningStats });
});

app.get('/api/wallet/balance/:address', (req, res) => {
    const address = req.params.address;
    const mockBalance = Math.random() * 1000;
    res.json({
        address,
        balance: mockBalance.toFixed(8),
        transactions: []
    });
});

app.post('/api/wallet/generate', (req, res) => {
    const newAddress = 'qtc_' + Math.random().toString(36).substr(2, 32);
    res.json({
        address: newAddress,
        publicKey: Math.random().toString(36).substr(2, 64),
        message: 'New quantum-safe wallet generated'
    });
});

// RPC endpoint
app.post('/rpc', (req, res) => {
    const { method, params = [] } = req.body;
    
    switch (method) {
        case 'getinfo':
            res.json({
                result: blockchainState,
                error: null,
                id: req.body.id || 1
            });
            break;
        case 'getmininginfo':
            res.json({
                result: miningStats,
                error: null,
                id: req.body.id || 1
            });
            break;
        case 'mine_block':
            if (miningStats.isRunning) {
                miningStats.blocksMined++;
                blockchainState.height++;
                res.json({
                    result: {
                        blockNumber: blockchainState.height,
                        reward: 25.5,
                        time: new Date().toISOString()
                    },
                    error: null,
                    id: req.body.id || 1
                });
            } else {
                res.json({
                    result: null,
                    error: 'Mining not started',
                    id: req.body.id || 1
                });
            }
            break;
        default:
            res.json({
                result: null,
                error: 'Method not found',
                id: req.body.id || 1
            });
    }
});

app.listen(PORT, () => {
    console.log(`QuantumCoin Node.js Backend running on port ${PORT}`);
    console.log(`RPC endpoint: http://localhost:${PORT}/rpc`);
    console.log(`API endpoint: http://localhost:${PORT}/api`);
});
