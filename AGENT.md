# QuantumCoin Development Guide

## Build & Run Commands
- **Rust Backend**: `cargo build` (compile), `cargo run` (start main backend server on port 8080)
- **Node.js API**: `npm start` (starts Express wrapper server)
- **Frontend**: Open `index.html` directly or use Vite dev server
- **Backend Service**: `cd backend && cargo run` (starts Rocket API server)
- **Tests**: No test framework configured yet

## Architecture
- **Hybrid codebase**: Rust blockchain core + Node.js API wrapper + vanilla JS frontend
- **Main modules**: blockchain.rs, wallet.rs, transaction.rs, revstop.rs (quantum-safe features)
- **Backend**: Separate Rocket-based API in `/backend` directory
- **Frontend**: Basic HTML/CSS/JS with React/Vite setup in progress
- **Database**: SQLite with sqlx (async SQL toolkit)
- **Cryptography**: Dilithium2 post-quantum signatures, Argon2 hashing

## Code Style & Conventions
- **Rust**: Standard rustfmt formatting, use `anyhow` for errors, async/await with tokio
- **JavaScript**: No strict linter configured, basic ES6+ conventions
- **Naming**: snake_case for Rust, camelCase for JS/TS
- **Imports**: Group external crates first, then local modules
- **Security**: Never log private keys, use secure RNG, validate all inputs
