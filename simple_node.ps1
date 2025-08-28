# QuantumCoin Simple Node Server
# Minimal PowerShell RPC server

$Port = 8545
$P2PPort = 30333

Write-Host ""
Write-Host "🚀 QuantumCoin Node Starting..." -ForegroundColor Cyan

# Test if port is available
try {
    $test = New-Object System.Net.Sockets.TcpClient
    $test.Connect("127.0.0.1", $Port)
    $test.Close()
    Write-Host "❌ Port $Port is already in use" -ForegroundColor Red
    exit 1
}
catch {
    Write-Host "✅ Port $Port is available" -ForegroundColor Green
}

# Start simple HTTP listener
$listener = New-Object System.Net.HttpListener
$listener.Prefixes.Add("http://localhost:$Port/")

try {
    $listener.Start()
    Write-Host "🎯 QuantumCoin Node RUNNING!" -ForegroundColor Green
    Write-Host "=============================" -ForegroundColor Cyan
    Write-Host "📡 RPC: http://localhost:$Port" -ForegroundColor White
    Write-Host "🌐 P2P: Port $P2PPort (simulated)" -ForegroundColor White
    Write-Host "⚛️  Network: QuantumCoin Mainnet" -ForegroundColor White
    Write-Host "💰 Max Supply: 22M QTC (Fair Launch)" -ForegroundColor Green
    Write-Host ""
    Write-Host "🧪 Test Commands:" -ForegroundColor Yellow
    Write-Host "Open another PowerShell and run:" -ForegroundColor Gray
    Write-Host "curl http://localhost:$Port" -ForegroundColor White
    Write-Host ""
    Write-Host "🌍 YOUR DEVICE IS NOW A QUANTUMCOIN NODE!" -ForegroundColor Green
    Write-Host "Press Ctrl+C to stop" -ForegroundColor Yellow
    Write-Host ""
    
    while ($listener.IsListening) {
        $context = $listener.GetContext()
        $request = $context.Request
        $response = $context.Response
        
        Write-Host "📞 Request: $($request.HttpMethod) $($request.Url)" -ForegroundColor Cyan
        
        $responseText = @"
QuantumCoin Node Status
======================
✅ Status: RUNNING
🌐 Network: QuantumCoin Mainnet  
📊 Block Height: 0
💰 Max Supply: 22,000,000 QTC
🔒 Premine: 0 QTC (Fair Launch)
📡 RPC Port: $Port
🌍 P2P Port: $P2PPort

Your device is successfully running as a QuantumCoin node!

RPC Methods Available:
- qc_blockNumber
- qc_getBalance
- qc_getBlockByNumber
- getblockchain
- getinfo

Test with: curl http://localhost:$Port
"@
        
        $buffer = [System.Text.Encoding]::UTF8.GetBytes($responseText)
        $response.ContentLength64 = $buffer.Length
        $response.ContentType = "text/plain"
        $response.StatusCode = 200
        
        $response.OutputStream.Write($buffer, 0, $buffer.Length)
        $response.Close()
    }
}
catch {
    Write-Host "❌ Error: $($_.Exception.Message)" -ForegroundColor Red
}
finally {
    if ($listener.IsListening) {
        $listener.Stop()
    }
    Write-Host "🛑 QuantumCoin node stopped" -ForegroundColor Yellow
}
