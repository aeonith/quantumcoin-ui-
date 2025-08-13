# Contributing to QuantumCoin

Thank you for your interest in contributing to QuantumCoin! This document provides guidelines for contributing to the project.

## Development Environment Setup

### Prerequisites
- **Node.js**: >= 18.18.0
- **Rust**: Latest stable (install via [rustup](https://rustup.rs/))
- **pnpm**: Latest (preferred over npm)
- **Docker**: For local testing
- **Git**: For version control

### Quick Start

```bash
# Clone the repository
git clone https://github.com/aeonith/quantumcoin-ui-.git
cd quantumcoin-ui-

# Install dependencies
pnpm install

# Build Rust components
cargo build

# Run tests
make test

# Start development environment
make dev
```

### Using DevContainer (Recommended)

We provide a DevContainer for consistent development environments:

```bash
# Open in VS Code with DevContainer extension
code .
# VS Code will prompt to reopen in container
```

## Project Structure

```
quantumcoin-ui-/
├── crates/                 # Rust workspace crates
│   ├── node/               # Blockchain node implementation
│   ├── wallet/             # Wallet cryptography and management
│   └── cli/                # Command-line interface
├── services/               # Service implementations
│   ├── explorer/           # Rust-based block explorer API
│   └── explorer-proxy/     # Node.js fallback proxy
├── ui/                     # Next.js frontend application
├── infra/                  # Infrastructure and deployment
├── docs/                   # Documentation
├── config/                 # Canonical configuration
└── openapi/                # API specifications
```

## Economics Constants

All economic parameters are defined in `/config/chain.toml`. This is the **SINGLE SOURCE OF TRUTH**:

- **Total Supply**: 22,000,000 QTC
- **Halving Period**: 2 years  
- **Halving Duration**: 66 years total
- **Block Time**: 600 seconds (10 minutes)

Never hardcode these values - always import from the canonical source.

## Development Workflow

### Making Changes

1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/your-feature-name`
3. **Make** your changes following our coding standards
4. **Test** your changes: `make test`
5. **Lint** your code: `make lint`
6. **Commit** using conventional commits
7. **Push** and create a pull request

### Commit Message Format

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): description

[optional body]

[optional footer(s)]
```

Types:
- `feat`: New feature
- `fix`: Bug fix  
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Adding tests
- `chore`: Maintenance tasks

Examples:
```
feat(wallet): add RevStop protection mechanism
fix(explorer): resolve transaction pagination bug
docs(readme): update installation instructions
```

### Code Quality Standards

#### Rust
- Run `cargo fmt` for formatting
- Run `cargo clippy -- -D warnings` for linting
- Maintain test coverage above 80%
- Use `#[deny(missing_docs)]` for public APIs
- Never use `.unwrap()` in production code - prefer proper error handling

#### TypeScript/JavaScript
- Use strict TypeScript configuration
- Follow ESLint rules
- Prefer functional programming patterns
- Use proper error boundaries in React
- Never commit `console.log` statements

#### General
- Write self-documenting code
- Add comments for complex business logic
- Update documentation for API changes
- Include tests for new functionality

## Security Guidelines

### Cryptography
- Never implement custom cryptographic primitives
- Use only approved libraries: `pqcrypto-dilithium` <= 0.3
- Validate all inputs at API boundaries
- Use secure random number generation

### RevStop Feature
- RevStop only affects the local wallet, not network consensus
- Must be OFF by default for exchange integrations
- Requires strong password authentication

### Sensitive Data
- Never commit private keys or secrets
- Use environment variables for configuration
- Sanitize logs to prevent information disclosure
- Follow principle of least privilege

## Testing

### Rust Testing
```bash
# Unit tests
cargo test

# Integration tests  
cargo test --test integration_tests

# Property-based tests with proptest
cargo test --features proptest

# Fuzzing (optional)
cargo fuzz run tx_parser
```

### UI Testing
```bash
# Unit tests
pnpm test

# Component tests
pnpm test:components

# E2E tests
pnpm test:e2e

# Lighthouse performance tests
pnpm test:lighthouse
```

## Documentation

- Update relevant documentation for code changes
- Use JSDoc for TypeScript functions
- Use rustdoc for Rust public APIs
- Include examples in documentation
- Update README.md for user-facing changes

## Release Process

1. **Version Bump**: Update version in relevant files
2. **Changelog**: Add entry to CHANGELOG.md
3. **Tag**: Create git tag with `v` prefix (e.g., `v2.1.0`)
4. **CI/CD**: Automated build and deployment via GitHub Actions
5. **SBOM**: Software Bill of Materials automatically generated
6. **Signing**: Container images signed with cosign

## Getting Help

- **Discord**: [Join our community](https://discord.gg/quantumcoin)
- **GitHub Issues**: For bug reports and feature requests
- **GitHub Discussions**: For questions and general discussion
- **Email**: development@quantumcoincrypto.com

## Code of Conduct

This project adheres to a code of conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to conduct@quantumcoincrypto.com.

## License

By contributing to QuantumCoin, you agree that your contributions will be licensed under the MIT License.
