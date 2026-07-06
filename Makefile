.PHONY: help build test deploy clean install

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-15s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

install: ## Install all dependencies
	@echo "Installing Rust dependencies..."
	cd contract && cargo fetch
	@echo "Installing Node.js dependencies..."
	cd frontend && npm install
	@echo "✓ Dependencies installed"

build: ## Build contract and frontend
	@echo "Building contract..."
	cd contract && cargo build --target wasm32-unknown-unknown --release
	@echo "Building frontend..."
	cd frontend && npm run build
	@echo "✓ Build complete"

test: ## Run all tests
	@echo "Running contract tests..."
	cd contract && cargo test
	@echo "Running frontend type check..."
	cd frontend && npm run type-check
	@echo "✓ All tests passed"

lint: ## Run linters
	@echo "Linting contract..."
	cd contract && cargo fmt --check && cargo clippy -- -D warnings
	@echo "Linting frontend..."
	cd frontend && npm run lint
	@echo "✓ Linting complete"

fmt: ## Format code
	@echo "Formatting contract..."
	cd contract && cargo fmt
	@echo "Formatting frontend..."
	cd frontend && npm run lint -- --fix
	@echo "✓ Formatting complete"

deploy-testnet: ## Deploy to Stellar testnet
	@echo "Deploying to testnet..."
	cd scripts && ./deploy.sh
	@echo "✓ Deployed to testnet"

deploy-mainnet: ## Deploy to Stellar mainnet (requires confirmation)
	@echo "⚠️  WARNING: Deploying to MAINNET"
	@read -p "Are you sure? [y/N] " -n 1 -r; \
	echo; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		cd scripts && NETWORK=mainnet ./deploy.sh; \
		echo "✓ Deployed to mainnet"; \
	else \
		echo "Deployment cancelled"; \
	fi

integration-test: ## Run integration tests on testnet
	@echo "Running integration tests..."
	cd scripts && ./test-integration.sh
	@echo "✓ Integration tests complete"

clean: ## Clean build artifacts
	@echo "Cleaning contract artifacts..."
	cd contract && cargo clean
	@echo "Cleaning frontend artifacts..."
	cd frontend && rm -rf .next out node_modules/.cache
	@echo "✓ Clean complete"

dev: ## Start development environment
	@echo "Starting frontend dev server..."
	cd frontend && npm run dev

optimize: ## Optimize WASM binary
	@echo "Optimizing WASM..."
	@if command -v wasm-opt >/dev/null 2>&1; then \
		wasm-opt -Oz \
			contract/target/wasm32-unknown-unknown/release/lumiswap_launch.wasm \
			-o contract/target/wasm32-unknown-unknown/release/lumiswap_launch_optimized.wasm; \
		ls -lh contract/target/wasm32-unknown-unknown/release/lumiswap_launch*.wasm; \
		echo "✓ Optimization complete"; \
	else \
		echo "⚠️  wasm-opt not found. Install binaryen: https://github.com/WebAssembly/binaryen"; \
	fi

check: ## Check contract without building
	cd contract && cargo check

watch: ## Watch for changes and rebuild
	cd contract && cargo watch -x 'test'

size: ## Show WASM binary size
	@ls -lh contract/target/wasm32-unknown-unknown/release/*.wasm 2>/dev/null || echo "No WASM files found. Run 'make build' first."

docs: ## Generate documentation
	@echo "Generating contract docs..."
	cd contract && cargo doc --no-deps --open
	@echo "✓ Documentation generated"

setup: ## Initial project setup
	@echo "Setting up project..."
	rustup target add wasm32-unknown-unknown
	cargo install stellar-cli --locked
	cd frontend && npm install
	cp frontend/.env.example frontend/.env.local
	@echo "✓ Setup complete"
	@echo ""
	@echo "Next steps:"
	@echo "  1. Edit frontend/.env.local with your configuration"
	@echo "  2. Run 'make build' to build the project"
	@echo "  3. Run 'make test' to run tests"
	@echo "  4. Run 'make deploy-testnet' to deploy to testnet"
