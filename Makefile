# QuantumCoin Development Makefile

.PHONY: help dev test build lint typecheck fmt clean up down logs smoke install audit

# Default target
help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-15s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# Development
dev: ## Start development servers
	@echo "🚀 Starting development environment..."
	@if [ -d "ui" ]; then \
		echo "Starting UI dev server..."; \
		cd ui && pnpm dev & \
	fi
	@echo "Starting Rust components..."
	cargo run --bin quantumcoin &
	@echo "Development servers started. UI: http://localhost:3000, API: http://localhost:8080"

install: ## Install all dependencies
	@echo "📦 Installing dependencies..."
	@if [ -f "ui/package.json" ]; then \
		echo "Installing UI dependencies..."; \
		cd ui && pnpm install; \
	fi
	@echo "Installing Rust dependencies..."
	cargo fetch

# Testing
test: ## Run all tests
	@echo "🧪 Running tests..."
	cargo test --workspace --all-features
	@if [ -d "ui" ]; then \
		echo "Running UI tests..."; \
		cd ui && pnpm test; \
	fi

test-rust: ## Run only Rust tests
	cargo test --workspace --all-features

test-ui: ## Run only UI tests  
	@if [ -d "ui" ]; then \
		cd ui && pnpm test; \
	else \
		echo "UI directory not found"; \
	fi

test-e2e: ## Run end-to-end tests
	@if [ -d "ui" ]; then \
		cd ui && pnpm test:e2e; \
	else \
		echo "UI directory not found"; \
	fi

# Building
build: ## Build all components
	@echo "🔨 Building all components..."
	cargo build --release --workspace
	@if [ -d "ui" ]; then \
		echo "Building UI..."; \
		cd ui && pnpm build; \
	fi

build-rust: ## Build only Rust components
	cargo build --release --workspace

build-ui: ## Build only UI
	@if [ -d "ui" ]; then \
		cd ui && pnpm build; \
	else \
		echo "UI directory not found"; \
	fi

# Code quality
lint: ## Lint all code
	@echo "🔍 Linting code..."
	cargo clippy --workspace --all-targets --all-features -- -D warnings
	@if [ -d "ui" ]; then \
		echo "Linting UI..."; \
		cd ui && pnpm lint; \
	fi

fmt: ## Format all code
	@echo "✨ Formatting code..."
	cargo fmt --all
	@if [ -d "ui" ]; then \
		echo "Formatting UI..."; \
		cd ui && pnpm format; \
	fi

typecheck: ## Type check TypeScript
	@if [ -d "ui" ]; then \
		cd ui && pnpm typecheck; \
	else \
		echo "UI directory not found"; \
	fi

audit: ## Security audit
	@echo "🔒 Running security audit..."
	cargo audit
	@if [ -d "ui" ]; then \
		echo "Auditing UI dependencies..."; \
		cd ui && pnpm audit; \
	fi

# Docker operations
up: ## Start services with docker-compose
	docker-compose up -d

down: ## Stop docker-compose services
	docker-compose down

logs: ## Show docker-compose logs
	docker-compose logs -f

# Testing and validation
smoke: ## Run smoke tests
	@echo "🧨 Running smoke tests..."
	@chmod +x scripts/smoke.sh
	@./scripts/smoke.sh

lighthouse: ## Run Lighthouse performance tests
	@if [ -d "ui" ]; then \
		cd ui && pnpm lighthouse; \
	else \
		echo "UI directory not found"; \
	fi

# Database operations (if applicable)
db-migrate: ## Run database migrations
	@if command -v sqlx >/dev/null 2>&1; then \
		sqlx migrate run; \
	else \
		echo "sqlx CLI not found. Install with: cargo install sqlx-cli"; \
	fi

db-reset: ## Reset database
	@if command -v sqlx >/dev/null 2>&1; then \
		sqlx database drop -y && sqlx database create && sqlx migrate run; \
	else \
		echo "sqlx CLI not found. Install with: cargo install sqlx-cli"; \
	fi

# Cleanup
clean: ## Clean build artifacts
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	@if [ -d "ui" ]; then \
		cd ui && rm -rf .next dist build; \
	fi
	@if [ -d "node_modules" ]; then \
		rm -rf node_modules; \
	fi

# SBOM and security
sbom: ## Generate Software Bill of Materials
	@if command -v syft >/dev/null 2>&1; then \
		syft packages . --output spdx-json > quantumcoin.sbom.json; \
		echo "SBOM generated: quantumcoin.sbom.json"; \
	else \
		echo "syft not found. Install from: https://github.com/anchore/syft"; \
	fi

scan: ## Scan for vulnerabilities  
	@if command -v grype >/dev/null 2>&1; then \
		grype . --output table; \
	else \
		echo "grype not found. Install from: https://github.com/anchore/grype"; \
	fi

# Release preparation
pre-release: lint test build audit sbom scan ## Run all pre-release checks
	@echo "✅ Pre-release checks completed"

# Development utilities
watch: ## Watch for changes and rebuild
	cargo watch -x check -x test -x run

watch-ui: ## Watch UI for changes
	@if [ -d "ui" ]; then \
		cd ui && pnpm dev; \
	else \
		echo "UI directory not found"; \
	fi

# Documentation
docs: ## Generate documentation
	cargo doc --workspace --all-features --open
	@if [ -d "ui" ]; then \
		echo "UI docs available at ui/docs/"; \
	fi

# Check all quality gates (used by CI)
ci: lint typecheck test build ## Run all CI checks locally
	@echo "✅ All CI checks passed"
