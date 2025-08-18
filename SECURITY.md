# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 2.0.x   | :white_check_mark: |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability in QuantumCoin, please report it responsibly.

### How to Report

**Email:** security@quantumcoincrypto.com

**PGP Key:**
```
-----BEGIN PGP PUBLIC KEY BLOCK-----

mQINBGWH2wsBEADO2Q+XYZ123...
[Full PGP key will be included in actual releases]
...
=ABCD
-----END PGP PUBLIC KEY BLOCK-----
```

**Key Fingerprint:** `1234 5678 9ABC DEF0 1234 5678 9ABC DEF0 1234 5678`

### What to Include

Please include as much detail as possible:

- **Vulnerability type** (e.g., consensus bug, cryptographic issue, network attack)
- **Affected components** (node, wallet, RPC, P2P protocol)
- **Steps to reproduce** with example code or commands
- **Potential impact** (fund theft, network disruption, privacy leak)
- **Suggested fix** if you have one
- **Your contact information** for follow-up questions

### Scope

#### In Scope
- **Consensus layer**: Block validation, transaction verification, fork choice
- **Cryptography**: Post-quantum signatures, key generation, address derivation  
- **P2P protocol**: Network messaging, peer discovery, sync mechanisms
- **RPC interface**: Authentication, input validation, data exposure
- **Wallet functionality**: Key storage, transaction creation, backup/restore
- **Economic attacks**: Fee manipulation, mining attacks, inflation bugs

#### Out of Scope
- **UI/UX issues** that don't affect security
- **Performance issues** without security implications
- **Third-party services** (explorers, exchanges, wallets not developed by us)
- **Social engineering** attacks against users
- **Physical attacks** on user devices
- **Theoretical attacks** without practical exploitation

### Response Process

1. **Acknowledgment** within 24 hours
2. **Initial assessment** within 72 hours  
3. **Regular updates** every 7 days during investigation
4. **Resolution timeline** provided after assessment
5. **Public disclosure** 90 days after fix deployment (or by mutual agreement)

### Severity Levels

#### Critical (CVSS 9.0-10.0)
- **Consensus failure** leading to chain split
- **Cryptographic break** compromising signatures
- **Remote code execution** on nodes
- **Private key exposure** in normal operation

**Response:** Immediate fix within 24-48 hours, emergency release

#### High (CVSS 7.0-8.9)  
- **Fund theft** through transaction manipulation
- **Network-wide DoS** attacks
- **Privacy breaches** exposing user data
- **Mining centralization** attacks

**Response:** Fix within 1-2 weeks, coordinated release

#### Medium (CVSS 4.0-6.9)
- **Local DoS** on individual nodes
- **Information disclosure** of non-sensitive data
- **Fee manipulation** attacks
- **Mempool spam** attacks

**Response:** Fix in next regular release (4-8 weeks)

#### Low (CVSS 0.1-3.9)
- **Minor information leaks**
- **Aesthetic security issues**
- **Rate limiting bypasses**

**Response:** Fix when convenient, document workarounds

## Responsible Disclosure Guidelines

### For Researchers

- **Give us time** to fix vulnerabilities before public disclosure
- **Don't exploit** vulnerabilities on mainnet or cause harm to users
- **Test on testnet** when possible for proof-of-concept
- **Coordinate disclosure** timing with our security team
- **Consider user safety** in your research and disclosure

### For the Community

- **Report** security issues through proper channels (not GitHub issues)
- **Don't share** exploit details publicly until fixes are deployed
- **Update** your nodes promptly when security releases are available
- **Follow** @QuantumCoinSec on Twitter for security announcements

## Bug Bounty Program

### Rewards

| Severity | Mainnet Reward | Testnet Reward |
|----------|----------------|----------------|
| Critical | $10,000-25,000 | $1,000-2,500   |
| High     | $5,000-10,000  | $500-1,000     |
| Medium   | $1,000-5,000   | $100-500       |
| Low      | $100-1,000     | $50-100        |

### Payment Methods
- **QuantumCoin (QTC)** - preferred
- **Bitcoin (BTC)** - accepted
- **USD equivalent** via bank transfer

### Eligibility
- **First reporter** of a vulnerability gets the reward
- **No previous public disclosure** of the issue
- **Follows responsible disclosure** guidelines
- **Provides sufficient detail** for reproduction

## Security Announcements

### Channels
- **Security mailing list**: security-announce@quantumcoincrypto.com
- **Twitter**: @QuantumCoinSec
- **GitHub**: Security advisories on main repository
- **Website**: https://quantumcoincrypto.com/security

### Release Security Notes
All releases include security impact assessment:
- **Security release**: Contains fixes for disclosed vulnerabilities
- **Regular release**: May contain proactive security improvements
- **Emergency release**: Critical security fixes requiring immediate upgrade

## Cryptographic Assumptions

### Post-Quantum Security
QuantumCoin uses **Dilithium2** from the NIST Post-Quantum Cryptography standardization:
- **Security level**: NIST Level 2 (equivalent to AES-128)
- **Quantum resistance**: Secure against known quantum algorithms
- **Classical security**: Based on lattice problems (LWE/SIS)

### Hash Functions
- **Block hashing**: BLAKE3 (cryptographically secure, fast)
- **Merkle trees**: BLAKE3 with domain separation
- **Address checksums**: BLAKE3 with version bytes

### Randomness Sources
- **Key generation**: OS cryptographic RNG (`/dev/urandom`, Windows CryptoAPI)
- **Nonce generation**: ChaCha20 DRBG with regular reseeding
- **Mining**: Hardware entropy where available

## Known Limitations

### Performance Trade-offs
- **Signature size**: Dilithium2 signatures are ~2.4KB (vs ~70 bytes for ECDSA)
- **Key size**: Public keys are ~1.3KB (vs ~33 bytes for ECDSA)  
- **Verification time**: ~10x slower than ECDSA (still <1ms per signature)

### Quantum Transition Risks
- **Algorithm updates**: May require hard fork if NIST parameters change
- **Backward compatibility**: Old signatures will remain valid
- **Migration path**: Planned for potential algorithm upgrades

## Audit History

### External Audits
- **Pending**: Comprehensive security audit scheduled for Q2 2025
- **Scope**: Consensus engine, cryptography, P2P protocol, wallet security
- **Auditor**: TBD (RFP process underway)

### Internal Reviews
- **Code review**: All commits require security-focused review
- **Fuzzing**: Continuous fuzzing of parsers and validators
- **Static analysis**: CodeQL, Clippy, and custom security lints

## Compliance

### Regulatory Considerations
- **KYC/AML**: Not implemented at protocol level (exchange responsibility)
- **Privacy**: Transaction privacy similar to Bitcoin (pseudonymous)
- **Auditability**: Full transaction history permanently recorded
- **Sanctions**: No protocol-level sanctions enforcement

### Standards Compliance
- **NIST**: Post-quantum cryptography follows NIST standards
- **RFC**: Network protocols follow relevant internet standards
- **ISO 27001**: Security practices align with international standards

---

**Last Updated:** January 15, 2025  
**Next Review:** April 15, 2025  
**Version:** 1.0
