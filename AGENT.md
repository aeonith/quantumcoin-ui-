# QuantumCoin Development Guide

## Build & Run Commands
- **Modern UI**: `npm run dev` (Next.js development server on port 3000)
- **Production Build**: `npm run build && npm start` (production Next.js)
- **Legacy UI**: `START_POWERSHELL_SERVER.bat` (static files on port 8000)
- **Development Setup**: `START_DEV_SERVER.bat` (automated setup with fallbacks)
- **Rust Backend**: `cargo build` (compile), `cargo run` (start main backend server on port 8080)
- **Backend Service**: `cd backend && cargo run` (starts Rocket API server)
- **Legacy Node API**: `npm run legacy-server` (Express wrapper server)
- **Tests**: No test framework configured yet

## Architecture
- **Hybrid codebase**: Rust blockchain core + Node.js API wrapper + Next.js React frontend
- **Main modules**: blockchain.rs, wallet.rs, transaction.rs, revstop.rs (quantum-safe features)
- **Backend**: Separate Rocket-based API in `/backend` directory
- **Frontend**: Modern Next.js + React with TypeScript, plus legacy HTML fallbacks
- **Database**: SQLite with sqlx (async SQL toolkit)
- **Cryptography**: Dilithium2 post-quantum signatures, Argon2 hashing
- **Exchange**: BTC-to-QTC with on-chain verification via mempool.space

## Code Style & Conventions
- **Rust**: Standard rustfmt formatting, use `anyhow` for errors, async/await with tokio
- **JavaScript**: No strict linter configured, basic ES6+ conventions
- **Naming**: snake_case for Rust, camelCase for JS/TS
- **Imports**: Group external crates first, then local modules
- **Security**: Never log private keys, use secure RNG, validate all inputs
