#!/bin/bash

echo "🚀 Starting QuantumCoin Mining Dashboard..."

# Check if we have a web server available
if command -v npx >/dev/null 2>&1; then
    echo "Using npx serve..."
    npx serve . -p 3000
elif command -v python3 >/dev/null 2>&1; then
    echo "Using Python HTTP server..."
    python3 -m http.server 3000
elif command -v python >/dev/null 2>&1; then
    echo "Using Python 2 HTTP server..."
    python -m SimpleHTTPServer 3000
else
    echo "❌ No web server available. Please install Node.js or Python."
    echo "You can also open the file directly, but some features may not work."
    echo "File location: file://$(pwd)/mining_dashboard.html"
    exit 1
fi
