// Standalone QuantumCoin Server - No Dependencies Required
const http = require('http');
const fs = require('fs');
const path = require('path');
const url = require('url');

// Blockchain state
let blockchain = {
    chain: [
        {
            index: 0,
            timestamp: new Date().toISOString(),
            transactions: [],
            previousHash: "0",
            hash: "genesis",
            nonce: 0,
            merkleRoot: "0"
        }
    ],
    pendingTransactions: [],
    balances: {}
};

// MIME types for static files
const mimeTypes = {
    '.html': 'text/html',
    '.js': 'text/javascript',
    '.css': 'text/css',
    '.json': 'application/json',
    '.png': 'image/png',
    '.jpg': 'image/jpeg',
    '.gif': 'image/gif',
    '.ico': 'image/x-icon'
};

// Simple hash function
function simpleHash(data) {
    let hash = 0;
    for (let i = 0; i < data.length; i++) {
        const char = data.charCodeAt(i);
        hash = ((hash << 5) - hash) + char;
        hash = hash & hash; // Convert to 32-bit integer
    }
    return Math.abs(hash).toString(16);
}

// Serve static files
function serveStaticFile(req, res, filePath) {
    fs.readFile(filePath, (err, data) => {
        if (err) {
            res.writeHead(404);
            res.end('File not found');
            return;
        }
        
        const ext = path.extname(filePath);
        const contentType = mimeTypes[ext] || 'text/plain';
        
        res.writeHead(200, { 'Content-Type': contentType });
        res.end(data);
    });
}

// API handlers
const apiHandlers = {
    '/api/blockchain': (req, res) => {
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify(blockchain.chain));
    },
    
    '/api/balance': (req, res) => {
        const urlParts = url.parse(req.url, true);
        const address = urlParts.pathname.split('/')[3];
        const balance = blockchain.balances[address] || 0;
        
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ address, balance }));
    },
    
    '/api/transaction': (req, res) => {
        if (req.method === 'POST') {
            let body = '';
            req.on('data', chunk => body += chunk);
            req.on('end', () => {
                try {
                    const transaction = JSON.parse(body);
                    blockchain.pendingTransactions.push(transaction);
                    
                    res.writeHead(200, { 'Content-Type': 'application/json' });
                    res.end(JSON.stringify({ status: 'Transaction added to pending pool' }));
                } catch (error) {
                    res.writeHead(400, { 'Content-Type': 'application/json' });
                    res.end(JSON.stringify({ error: 'Invalid transaction data' }));
                }
            });
        }
    },
    
    '/api/mine': (req, res) => {
        if (req.method === 'POST') {
            const urlParts = url.parse(req.url, true);
            const rewardAddress = urlParts.pathname.split('/')[3];
            
            // Create mining reward transaction
            const rewardTx = {
                id: `reward_${Date.now()}`,
                from: "",
                to: rewardAddress,
                amount: 10,
                timestamp: new Date().toISOString(),
                signature: "mining_reward"
            };
            
            blockchain.pendingTransactions.push(rewardTx);
            
            // Create new block
            const newBlock = {
                index: blockchain.chain.length,
                timestamp: new Date().toISOString(),
                transactions: [...blockchain.pendingTransactions],
                previousHash: blockchain.chain[blockchain.chain.length - 1].hash,
                hash: simpleHash(JSON.stringify(blockchain.pendingTransactions) + Date.now()),
                nonce: Math.floor(Math.random() * 1000000),
                merkleRoot: simpleHash(JSON.stringify(blockchain.pendingTransactions))
            };
            
            // Update balances
            blockchain.pendingTransactions.forEach(tx => {
                if (tx.from) {
                    blockchain.balances[tx.from] = (blockchain.balances[tx.from] || 0) - tx.amount;
                }
                blockchain.balances[tx.to] = (blockchain.balances[tx.to] || 0) + tx.amount;
            });
            
            blockchain.chain.push(newBlock);
            blockchain.pendingTransactions = [];
            
            res.writeHead(200, { 'Content-Type': 'application/json' });
            res.end(JSON.stringify(newBlock));
        }
    }
};

// Create server
const server = http.createServer((req, res) => {
    const parsedUrl = url.parse(req.url, true);
    const pathname = parsedUrl.pathname;
    
    // Enable CORS
    res.setHeader('Access-Control-Allow-Origin', '*');
    res.setHeader('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE');
    res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
    
    if (req.method === 'OPTIONS') {
        res.writeHead(200);
        res.end();
        return;
    }
    
    // Handle API routes
    for (const route in apiHandlers) {
        if (pathname.startsWith(route)) {
            apiHandlers[route](req, res);
            return;
        }
    }
    
    // Serve static files
    let filePath = pathname === '/' ? './index.html' : `.${pathname}`;
    
    // Security check - prevent directory traversal
    if (filePath.includes('..')) {
        res.writeHead(403);
        res.end('Forbidden');
        return;
    }
    
    // Check if file exists
    fs.access(filePath, fs.constants.F_OK, (err) => {
        if (err) {
            res.writeHead(404);
            res.end('File not found');
        } else {
            serveStaticFile(req, res, filePath);
        }
    });
});

const PORT = 8080;
server.listen(PORT, () => {
    console.log(`🚀 QuantumCoin Server running on http://localhost:${PORT}`);
    console.log(`📝 API Endpoints:`);
    console.log(`   GET  /api/blockchain - Get full blockchain`);
    console.log(`   GET  /api/balance/<address> - Get wallet balance`);
    console.log(`   POST /api/transaction - Create new transaction`);
    console.log(`   POST /api/mine/<address> - Mine new block`);
    console.log(`🌐 Frontend: Open http://localhost:${PORT} in your browser`);
});
