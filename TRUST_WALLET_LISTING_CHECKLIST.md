# Trust Wallet Listing Checklist for QuantumCoin

## ✅ Basic Requirements Met

- [x] **Independent blockchain** (not a token)
- [x] **Open source** code on GitHub  
- [x] **Active development** with recent commits
- [x] **Block explorer** available
- [x] **Working testnet** (mainnet TBD)

## 📋 Chain Information

- **Name**: QuantumCoin
- **Symbol**: QTC  
- **Decimals**: 8
- **Chain ID**: `quantumcoin-mainnet-v2`
- **Total Supply**: 22,000,000 QTC
- **Website**: https://quantumcoin.network
- **Explorer**: https://explorer.quantumcoin.network

## 🎨 Logo Assets Ready

- [x] 256x256 PNG → `listing/logo-256.png`
- [x] 512x512 PNG → `listing/logo-512.png` 
- [x] 1024x1024 PNG → `listing/logo-1024.png`
- [x] SVG Vector → `listing/logo.svg`

## 🔧 Technical Integration

- [x] **REST API**: OpenAPI 3.0 specification
- [x] **Address Format**: Base64-encoded public keys
- [x] **Confirmations**: 6 blocks recommended
- [x] **Chain Metadata**: Complete `quantumcoin.chain.json`

## 🛡️ RevStop Disclosure

✅ **RevStop is per-wallet security feature that:**
- Cannot affect other users' funds
- Is OFF by default for exchanges  
- Requires password to enable/disable
- Similar to account recovery in banking

## 📋 Next Steps

1. Fork [Trust Wallet Assets](https://github.com/trustwallet/assets)
2. Create `blockchains/quantumcoin/` folder
3. Add logos and metadata
4. Submit PR when mainnet launches

See [quantumcoin.chain.json](listing/quantumcoin.chain.json) for complete metadata.
