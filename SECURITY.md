# Security Policy

## Supported Versions

We provide security updates for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 2.x.x   | :white_check_mark: |
| < 2.0   | :x:                |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report security vulnerabilities to:
- Email: security@quantumcoincrypto.com
- Use our GPG key for encryption: [link to public key]

Include the following information:
- Description of the vulnerability
- Steps to reproduce the issue
- Potential impact assessment  
- Any proof-of-concept or exploit code (if applicable)

## Response Timeline

- **Initial Response**: Within 24 hours of report
- **Assessment**: Within 72 hours 
- **Fix Development**: Based on severity (Critical: 7 days, High: 14 days, Medium: 30 days)
- **Public Disclosure**: After fix is deployed and users have time to upgrade

## Security Features

### Post-Quantum Cryptography
- Uses Dilithium2 signatures for quantum resistance
- All wallet operations use post-quantum secure primitives

### RevStop Protection  
- Per-wallet freeze capability for compromised accounts
- Cannot affect other users' funds
- Requires password authentication to disable

### Supply Chain Security
- SBOM (Software Bill of Materials) generated for all releases
- Container images signed with cosign
- Dependencies regularly audited with cargo-audit

### Network Security
- Rate limiting on all public endpoints
- CORS protection for web interfaces
- Structured logging for security monitoring

## Bug Bounty Program

We are considering a bug bounty program. Check this space for updates.

## Additional Resources

- [Threat Model](docs/threat-model.md)
- [Security Runbooks](docs/runbooks/)
