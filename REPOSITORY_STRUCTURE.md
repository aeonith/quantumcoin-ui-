# QuantumCoin Blockchain Repository

This repository contains the core QuantumCoin blockchain implementation. **All website/frontend logic has been removed** and moved to a separate repository.

## What's Included

### Core Blockchain
- `src/` - Rust blockchain implementation
- `crates/` - Modular blockchain components
- `backend/` - API server for RPC interface
- `Cargo.toml` - Rust project configuration

### Configuration
- `chain_spec.toml` - Blockchain parameters
- `config/` - Network configuration
- `.env.example` - Environment template

### Scripts & Tools
- `scripts/` - Genesis generation and utilities
- Build scripts (`build.sh`, `cargo build`)
- Testing scripts (`test_health.sh`)

### Documentation
- `README.md` - Project documentation
- `SECURITY.md` - Security guidelines
- `docs/` - Technical documentation
- `exchange-pack/` - Exchange integration docs

### CI/CD & Deployment
- `.github/` - GitHub Actions workflows
- `Dockerfile` - Container configuration
- Release artifacts and deployment scripts

## What Was Removed

- ❌ `app/` - Next.js frontend pages
- ❌ `src/components/` - React components  
- ❌ `src/context/` - React contexts
- ❌ `src/lib/` - Frontend utilities
- ❌ `src/styles/` - CSS styling
- ❌ `services/` - Web API services
- ❌ Frontend configuration files
- ❌ Node.js dependencies

## Repository Focus

This repository now exclusively focuses on:

✅ **Blockchain Core** - Consensus, validation, mining  
✅ **Post-Quantum Security** - Dilithium signatures, RevStop  
✅ **RPC Interface** - Exchange integration endpoints  
✅ **Network Protocol** - P2P communication  
✅ **Storage Layer** - UTXO management, block storage  

For the website/frontend, use the separate web repository.
