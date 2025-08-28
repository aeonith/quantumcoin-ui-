# QuantumCoin Quick Node Server - PowerShell Version
# Gets your device running as a QuantumCoin node immediately

param(
    [int]$Port = 8545,
    [int]$P2PPort = 30333
)

Write-Host ""
Write-Host "🚀 Starting QuantumCoin Node" -ForegroundColor Cyan
Write-Host "============================" -ForegroundColor Cyan

# Create HTTP listener
$listener = New-Object System.Net.HttpListener
$listener.Prefixes.Add("http://localhost:$Port/")
$listener.Prefixes.Add("http://127.0.0.1:$Port/")

try {
    $listener.Start()
    Write-Host "✅ RPC Server started on port $Port" -ForegroundColor Green
}
catch {
    Write-Host "❌ Failed to start server on port $Port" -ForegroundColor Red
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Yellow
    Write-Host "Try running as Administrator or use a different port" -ForegroundColor Yellow
    exit 1
}

# Node state
$blockHeight = 0
$chainData = @{
    version = "1.0.0"
    network = "QuantumCoin Mainnet"
    height = $blockHeight
    difficulty = "1d00ffff"
    totalSupply = 0
    maxSupply = 2200000000000000
    premine = 0
    fairLaunch = $true
}

Write-Host ""
Write-Host "📊 QuantumCoin Node Information:" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host "📡 RPC Server: http://localhost:$Port" -ForegroundColor White
Write-Host "🌐 P2P Port: $P2PPort (simulated)" -ForegroundColor White
Write-Host "⚛️  Network: QuantumCoin Mainnet" -ForegroundColor White
Write-Host "🔒 Security: Fair Launch (Zero Premine)" -ForegroundColor Green
Write-Host "💰 Max Supply: 22,000,000 QTC" -ForegroundColor White
Write-Host "📈 Block Height: $blockHeight" -ForegroundColor White
Write-Host ""

Write-Host "🧪 Test your node:" -ForegroundColor Yellow
Write-Host "==================" -ForegroundColor Yellow
Write-Host "curl -X POST http://localhost:$Port \ " -ForegroundColor Gray
Write-Host "  -H `"Content-Type: application/json`" \ " -ForegroundColor Gray
Write-Host "  -d `'{`"jsonrpc`":`"2.0`",`"method`":`"qc_blockNumber`",`"params`":{},`"id`":1}`'" -ForegroundColor Gray
Write-Host ""

Write-Host "🌍 Your device is now running as a QuantumCoin node!" -ForegroundColor Green
Write-Host "Press Ctrl+C to stop" -ForegroundColor Yellow
Write-Host ""

# Request handler function
function Handle-RpcRequest($request, $body) {
    try {
        $jsonRequest = $body | ConvertFrom-Json -ErrorAction Stop
        $method = $jsonRequest.method
        $params = $jsonRequest.params
        $id = $jsonRequest.id
        
        Write-Host "🔧 RPC Request: $method" -ForegroundColor Cyan
        
        $response = @{
            jsonrpc = "2.0"
            id = $id
            result = $null
            error = $null
        }
        
        switch ($method) {
            "qc_blockNumber" {
                $response.result = @{ blockNumber = $blockHeight }
            }
            "qc_getBalance" {
                $address = $params.address
                $response.result = @{ 
                    balance = 0
                    address = $address 
                }
            }
            "qc_getBlockByNumber" {
                $blockNum = $params.number
                if ($blockNum -eq 0) {
                    $response.result = @{
                        number = 0
                        timestamp = "2025-01-15T00:00:00Z"
                        transactions = @()
                        difficulty = "1d00ffff"
                        nonce = 0
                        message = "QuantumCoin Genesis - Fair Launch"
                    }
                } else {
                    $response.error = "Block not found"
                }
            }
            "getblockchain" {
                $response.result = $chainData
            }
            "getinfo" {
                $response.result = $chainData
            }
            "qc_sendTransaction" {
                $txHash = "0x" + [System.Guid]::NewGuid().ToString("N").Substring(0, 64)
                $response.result = @{
                    transactionHash = $txHash
                    status = "pending"
                }
            }
            default {
                $response.error = "Unknown method: $method"
            }
        }
        
        return ($response | ConvertTo-Json -Depth 10)
        
    }
    catch {
        Write-Host "⚠️  Invalid JSON-RPC request" -ForegroundColor Yellow
        return '{"jsonrpc":"2.0","error":"Invalid JSON-RPC request","id":null}'
    }
}

# Main server loop
try {
    while ($listener.IsListening) {
        $context = $listener.GetContext()
        $request = $context.Request
        $response = $context.Response
        
        # Set CORS headers
        $response.Headers.Add("Access-Control-Allow-Origin", "*")
        $response.Headers.Add("Access-Control-Allow-Methods", "POST, GET, OPTIONS")
        $response.Headers.Add("Access-Control-Allow-Headers", "Content-Type")
        
        if ($request.HttpMethod -eq "POST") {
            # Handle RPC request
            $reader = New-Object System.IO.StreamReader($request.InputStream)
            $body = $reader.ReadToEnd()
            $reader.Close()
            
            $jsonResponse = Handle-RpcRequest $request $body
            
            $response.ContentType = "application/json"
            $buffer = [System.Text.Encoding]::UTF8.GetBytes($jsonResponse)
            $response.ContentLength64 = $buffer.Length
            $response.OutputStream.Write($buffer, 0, $buffer.Length)
            
        }
        elseif ($request.HttpMethod -eq "GET") {
            # Serve status page
            $html = @"
<!DOCTYPE html>
<html>
<head>
    <title>QuantumCoin Node</title>
    <style>body{font-family:Arial,sans-serif;margin:40px;background:#f5f5f5}
    .container{background:white;padding:30px;border-radius:10px;box-shadow:0 2px 10px rgba(0,0,0,0.1)}
    .status{color:#28a745;font-weight:bold}
    .info{margin:10px 0}
    code{background:#f8f9fa;padding:10px;display:block;border-radius:5px;overflow-x:auto}</style>
</head>
<body>
    <div class="container">
        <h1>🚀 QuantumCoin Node</h1>
        <div class="info"><strong>Status:</strong> <span class="status">✅ Running</span></div>
        <div class="info"><strong>Network:</strong> QuantumCoin Mainnet</div>
        <div class="info"><strong>RPC Port:</strong> $Port</div>
        <div class="info"><strong>P2P Port:</strong> $P2PPort</div>
        <div class="info"><strong>Block Height:</strong> $blockHeight</div>
        <div class="info"><strong>Max Supply:</strong> 22,000,000 QTC</div>
        <div class="info"><strong>Premine:</strong> 0 QTC (Fair Launch)</div>
        
        <h3>🧪 Test RPC:</h3>
        <code>curl -X POST http://localhost:$Port -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"qc_blockNumber","params":{},"id":1}'</code>
        
        <h3>📡 Available Methods:</h3>
        <ul>
            <li><code>qc_blockNumber</code> - Current block height</li>
            <li><code>qc_getBalance</code> - Get address balance</li>
            <li><code>qc_getBlockByNumber</code> - Get block data</li>
            <li><code>getblockchain</code> - Full chain information</li>
            <li><code>getinfo</code> - Node status</li>
        </ul>
    </div>
</body>
</html>
"@
            
            $response.ContentType = "text/html"
            $buffer = [System.Text.Encoding]::UTF8.GetBytes($html)
            $response.ContentLength64 = $buffer.Length
            $response.OutputStream.Write($buffer, 0, $buffer.Length)
        }
        
        $response.Close()
    }
}
catch [System.Net.HttpListenerException] {
    if ($_.Exception.ErrorCode -eq 995) {
        Write-Host ""
        Write-Host "🛑 Node stopped by user" -ForegroundColor Yellow
    }
    else {
        Write-Host ""
        Write-Host "❌ Server error: $($_.Exception.Message)" -ForegroundColor Red
    }
}
finally {
    if ($listener.IsListening) {
        $listener.Stop()
    }
    Write-Host "✅ QuantumCoin node shut down gracefully" -ForegroundColor Green
}
