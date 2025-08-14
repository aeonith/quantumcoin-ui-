@echo off
echo.
echo =====================================
echo    QuantumCoin Development Server
echo =====================================
echo.

REM Check if Node.js is installed
node --version >nul 2>&1
if errorlevel 1 (
    echo ERROR: Node.js is not installed!
    echo Please install Node.js from https://nodejs.org/
    pause
    exit /b 1
)

echo Node.js detected. Installing dependencies...
echo.

REM Install dependencies
call npm install

if errorlevel 1 (
    echo.
    echo ERROR: Failed to install dependencies!
    echo Falling back to legacy PowerShell server...
    echo.
    call START_POWERSHELL_SERVER.bat
    exit /b 1
)

echo.
echo Dependencies installed successfully!
echo.
echo Starting QuantumCoin Next.js development server...
echo.
echo The following will be available:
echo - Modern UI: http://localhost:3000
echo - Legacy UI: http://localhost:8000 (if PowerShell server is running)
echo.

REM Start Next.js development server
call npm run dev

echo.
echo Development server stopped.
pause
