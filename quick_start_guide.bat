@echo off
echo ========================================
echo QuantumCoin Blockchain Quick Start Guide
echo ========================================
echo.
echo Step 1: Restart VS Code completely
echo   - Close VS Code entirely
echo   - Reopen VS Code and this project
echo.
echo Step 2: Test your setup by running:
echo   test_blockchain.bat
echo.
echo Step 3: Start your blockchain network:
echo   cargo run --release --bin quantumcoin-integrated node --genesis
echo.
echo Step 4: Open in browser:
echo   - Main UI: index.html
echo   - Block Explorer: explorer.html  
echo   - Mining Interface: mining.html
echo.
echo Step 5: Start mining (in new terminal):
echo   cargo run --release --bin quantumcoin-integrated mine --threads 4 --address YOUR_ADDRESS
echo.
echo ========================================
echo Your QuantumCoin blockchain is ready!
echo ========================================
pause
