# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 2.0.x   | :white_check_mark: |
| < 2.0   | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability in QuantumCoin, please follow these steps:

1. **Do NOT** open a public GitHub issue for security vulnerabilities
2. Email security reports to: aeonith@quantumcoincrypto.com
3. Include detailed information about the vulnerability
4. Allow up to 48 hours for initial response
5. Coordinate responsible disclosure timeline

## Security Features

QuantumCoin implements several security measures:

- **Post-Quantum Cryptography**: Dilithium2 signatures for quantum resistance
- **DoS Protection**: Rate limiting and peer scoring
- **Input Validation**: Comprehensive transaction and block validation  
- **Secure Transport**: TLS/Noise protocol for P2P communication
- **Memory Safety**: Rust's memory safety guarantees
- **Fuzzing**: Continuous security testing of parsers and validators

## Security Audits

Security audits are planned before mainnet launch. Current security measures include:

- Static analysis with CodeQL
- Dependency vulnerability scanning with cargo-audit  
- Automated security testing in CI/CD pipeline
- Manual code review process

## Bug Bounty Program

A bug bounty program will be established before mainnet launch.

## Contact

For security-related questions: aeonith@quantumcoincrypto.com

PGP Key: [Coming Soon]
