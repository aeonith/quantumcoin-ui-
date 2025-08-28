# QuantumCoin Node - Windows Service Deployment Script
# Requires NSSM (Non-Sucking Service Manager)

param(
    [string]$InstallPath = "C:\qtc",
    [string]$DataPath = "$env:APPDATA\.qtc",
    [switch]$Install,
    [switch]$Uninstall,
    [switch]$Help
)

if ($Help) {
    Write-Host "QuantumCoin Windows Service Deployment" -ForegroundColor Cyan
    Write-Host "======================================"
    Write-Host ""
    Write-Host "Usage:"
    Write-Host "  .\deploy_windows_service.ps1 -Install    # Install service"
    Write-Host "  .\deploy_windows_service.ps1 -Uninstall  # Remove service"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -InstallPath  Installation path (default: C:\qtc)"
    Write-Host "  -DataPath     Data directory (default: %APPDATA%\.qtc)"
    exit 0
}

$ServiceName = "QuantumCoinNode"
$DisplayName = "QuantumCoin Node"
$Description = "QuantumCoin Post-Quantum Cryptocurrency Node"

function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Install-NSSM {
    if (!(Get-Command nssm -ErrorAction SilentlyContinue)) {
        Write-Host "⬇️  Downloading NSSM (Non-Sucking Service Manager)..." -ForegroundColor Yellow
        
        $nssmUrl = "https://nssm.cc/release/nssm-2.24.zip"
        $tempPath = "$env:TEMP\nssm.zip"
        $extractPath = "$env:TEMP\nssm"
        
        try {
            Invoke-WebRequest -Uri $nssmUrl -OutFile $tempPath
            Expand-Archive -Path $tempPath -DestinationPath $extractPath -Force
            
            $nssmExe = Get-ChildItem -Path $extractPath -Name "nssm.exe" -Recurse | Select-Object -First 1
            $nssmPath = Join-Path $extractPath $nssmExe.DirectoryName
            
            Copy-Item "$nssmPath\nssm.exe" "$env:WINDIR\System32\" -Force
            Write-Host "✅ NSSM installed successfully" -ForegroundColor Green
        }
        catch {
            Write-Host "❌ Failed to install NSSM: $_" -ForegroundColor Red
            Write-Host "Please download NSSM manually from https://nssm.cc/" -ForegroundColor Yellow
            exit 1
        }
    } else {
        Write-Host "✅ NSSM is already installed" -ForegroundColor Green
    }
}

if (-not (Test-Administrator)) {
    Write-Host "❌ This script must be run as Administrator" -ForegroundColor Red
    Write-Host "Right-click PowerShell and 'Run as Administrator'" -ForegroundColor Yellow
    exit 1
}

Write-Host "🚀 QuantumCoin Node - Windows Service Deployment" -ForegroundColor Cyan
Write-Host "==============================================="

if ($Uninstall) {
    Write-Host "🗑️  Removing QuantumCoin service..." -ForegroundColor Yellow
    
    try {
        nssm stop $ServiceName 2>$null
        nssm remove $ServiceName confirm 2>$null
        Write-Host "✅ Service removed successfully" -ForegroundColor Green
    }
    catch {
        Write-Host "⚠️  Service may not exist or already removed" -ForegroundColor Yellow
    }
    
    exit 0
}

if ($Install) {
    Install-NSSM
    
    # Check if binary exists
    $nodeBinary = ".\target\release\qc-node.exe"
    if (!(Test-Path $nodeBinary)) {
        Write-Host "❌ Binary not found: $nodeBinary" -ForegroundColor Red
        Write-Host "Run: cargo build --workspace --release" -ForegroundColor Yellow
        exit 1
    }
    
    Write-Host "📁 Creating directories..." -ForegroundColor Cyan
    New-Item -Path $InstallPath -ItemType Directory -Force | Out-Null
    New-Item -Path "$InstallPath\config" -ItemType Directory -Force | Out-Null
    New-Item -Path "$InstallPath\logs" -ItemType Directory -Force | Out-Null
    New-Item -Path $DataPath -ItemType Directory -Force | Out-Null
    
    Write-Host "📦 Copying files..." -ForegroundColor Cyan
    Copy-Item $nodeBinary "$InstallPath\" -Force
    Copy-Item "chain_spec.toml" "$InstallPath\" -Force
    Copy-Item "config\node.toml" "$InstallPath\config\" -Force
    
    if (Test-Path "genesis.json") {
        Copy-Item "genesis.json" "$InstallPath\" -Force
    } else {
        Write-Host "⚠️  genesis.json not found - will be generated on first run" -ForegroundColor Yellow
    }
    
    # Update config paths for Windows
    $configContent = Get-Content "$InstallPath\config\node.toml"
    $configContent = $configContent -replace 'path = "~/.qtc"', "path = `"$DataPath`""
    $configContent = $configContent -replace 'spec = "./chain_spec.toml"', "spec = `"$InstallPath\chain_spec.toml`""
    $configContent = $configContent -replace 'genesis = "./genesis.json"', "genesis = `"$InstallPath\genesis.json`""
    $configContent | Set-Content "$InstallPath\config\node.toml"
    
    Write-Host "🔧 Installing Windows service..." -ForegroundColor Cyan
    
    # Remove existing service if it exists
    nssm stop $ServiceName 2>$null
    nssm remove $ServiceName confirm 2>$null
    
    # Install new service
    nssm install $ServiceName "$InstallPath\qc-node.exe"
    nssm set $ServiceName Parameters "--config `"$InstallPath\config\node.toml`""
    nssm set $ServiceName DisplayName $DisplayName
    nssm set $ServiceName Description $Description
    nssm set $ServiceName Start SERVICE_AUTO_START
    nssm set $ServiceName AppDirectory $InstallPath
    
    # Set up logging
    nssm set $ServiceName AppStdout "$InstallPath\logs\stdout.log"
    nssm set $ServiceName AppStderr "$InstallPath\logs\stderr.log"
    nssm set $ServiceName AppRotateFiles 1
    nssm set $ServiceName AppRotateSeconds 86400
    nssm set $ServiceName AppRotateBytes 10485760
    
    # Start the service
    Write-Host "🎯 Starting service..." -ForegroundColor Cyan
    nssm start $ServiceName
    
    Start-Sleep -Seconds 3
    $status = nssm status $ServiceName
    
    if ($status -eq "SERVICE_RUNNING") {
        Write-Host "✅ QuantumCoin Node service installed and started!" -ForegroundColor Green
    } else {
        Write-Host "⚠️  Service installed but may not be running. Status: $status" -ForegroundColor Yellow
    }
    
    Write-Host ""
    Write-Host "📊 Service Management:" -ForegroundColor Cyan
    Write-Host "  nssm start $ServiceName"
    Write-Host "  nssm stop $ServiceName"
    Write-Host "  nssm restart $ServiceName"
    Write-Host "  nssm status $ServiceName"
    Write-Host ""
    Write-Host "📋 Service Manager: services.msc" -ForegroundColor Cyan
    Write-Host "📂 Install Path: $InstallPath" -ForegroundColor Cyan
    Write-Host "📁 Data Path: $DataPath" -ForegroundColor Cyan
    Write-Host "📄 Logs: $InstallPath\logs\" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "🧪 Test RPC:" -ForegroundColor Cyan
    Write-Host "  curl -X POST http://127.0.0.1:8545 \\"
    Write-Host "    -H 'Content-Type: application/json' \\"
    Write-Host "    -d '{\"jsonrpc\":\"2.0\",\"method\":\"qc_blockNumber\",\"params\":{},\"id\":1}'"
    
} else {
    Write-Host "Use -Install to install the service or -Help for usage information" -ForegroundColor Yellow
}
