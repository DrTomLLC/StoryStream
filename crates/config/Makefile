# StoryStream Configuration System - Makefile
# Common development tasks

.PHONY: help build test lint doc bench examples clean check verify install

help: ## Show this help message
	@echo 'StoryStream Config System - Available targets:'
	@echo ''
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

build: ## Build the config system
	@echo "Building config system..."
	@cargo build --package storystream-config
	@echo "✓ Build complete"

build-release: ## Build with optimizations
	@echo "Building config system (release)..."
	@cargo build --package storystream-config --release
	@echo "✓ Release build complete"

test: ## Run all tests
	@echo "Running tests..."
	@cargo test --package storystream-config
	@echo "✓ All tests passed"

test-unit: ## Run unit tests only
	@echo "Running unit tests..."
	@cargo test --package storystream-config --lib
	@echo "✓ Unit tests passed"

test-integration: ## Run integration tests
	@echo "Running integration tests..."
	@cargo test --package storystream-config --test integration_tests
	@echo "✓ Integration tests passed"

test-edge: ## Run edge case tests
	@echo "Running edge case tests..."
	@cargo test --package storystream-config --test edge_case_tests
	@echo "✓ Edge case tests passed"

test-property: ## Run property-based tests
	@echo "Running property tests..."
	@cargo test --package storystream-config --test property_tests
	@echo "✓ Property tests passed"

test-smoke: ## Run smoke tests
	@echo "Running smoke tests..."
	@cargo test --package storystream-config --test smoke_test
	@echo "✓ Smoke tests passed"

test-all: test test-integration test-edge test-property test-smoke ## Run all test suites
	@echo "✓ All test suites passed"

lint: ## Run linting checks
	@echo "Running linting..."
	@cargo fmt --package storystream-config -- --check
	@cargo clippy --package storystream-config -- -D warnings
	@echo "✓ Linting passed"

fmt: ## Format code
	@echo "Formatting code..."
	@cargo fmt --package storystream-config
	@echo "✓ Code formatted"

doc: ## Build documentation
	@echo "Building documentation..."
	@cargo doc --package storystream-config --no-deps
	@echo "✓ Documentation built"

doc-open: ## Build and open documentation
	@echo "Building and opening documentation..."
	@cargo doc --package storystream-config --no-deps --open

bench: ## Run benchmarks
	@echo "Running benchmarks..."
	@cargo bench --package storystream-config
	@echo "✓ Benchmarks complete"

examples: ## Build all examples
	@echo "Building examples..."
	@cargo build --package storystream-config --examples
	@echo "✓ Examples built"

run-basic: ## Run basic usage example
	@cargo run --package storystream-config --example basic_usage

run-advanced: ## Run advanced usage example
	@cargo run --package storystream-config --example advanced_usage

run-tool: ## Run config tool
	@cargo run --package storystream-config --example config_tool -- show

clean: ## Clean build artifacts
	@echo "Cleaning..."
	@cargo clean
	@echo "✓ Clean complete"

check: lint test ## Run all checks (lint + test)
	@echo "✓ All checks passed"

verify: check test-all bench examples doc ## Full verification (all tests + benchmarks + examples + docs)
	@echo ""
	@echo "════════════════════════════════════════"
	@echo "✓ VERIFICATION COMPLETE"
	@echo "════════════════════════════════════════"
	@echo "  Tests:        PASSED"
	@echo "  Linting:      PASSED"
	@echo "  Benchmarks:   PASSED"
	@echo "  Examples:     PASSED"
	@echo "  Docs:         PASSED"
	@echo "════════════════════════════════════════"
	@echo "✓ Config system is production ready!"
	@echo ""

audit: ## Run security audit
	@echo "Running security audit..."
	@cargo audit
	@echo "✓ Security audit complete"

coverage: ## Generate code coverage report
	@echo "Generating coverage report..."
	@cargo tarpaulin --package storystream-config --out Html
	@echo "✓ Coverage report generated"

install-tools: ## Install development tools
	@echo "Installing development tools..."
	@cargo install cargo-audit
	@cargo install cargo-tarpaulin
	@cargo install cargo-watch
	@pip install pre-commit
	@echo "✓ Tools installed"

watch: ## Watch for changes and run tests
	@cargo watch -x 'test --package storystream-config'

watch-check: ## Watch for changes and run checks
	@cargo watch -x 'check --package storystream-config'

pre-commit: ## Run pre-commit hooks
	@pre-commit run --all-files

ci: check test-all audit ## Run CI checks locally
	@echo "✓ CI checks complete"

release-check: verify audit ## Pre-release verification
	@echo "✓ Ready for release"

stats: ## Show project statistics
	@echo "Project Statistics:"
	@echo ""
	@echo "Source files:"
	@find src -name '*.rs' | wc -l
	@echo ""
	@echo "Lines of code:"
	@find src -name '*.rs' -exec cat {} \; | wc -l
	@echo ""
	@echo "Test files:"
	@find tests -name '*.rs' | wc -l
	@echo ""
	@echo "Example files:"
	@find examples -name '*.rs' | wc -l

tree: ## Show project structure
	@tree -I 'target|.git' -L 3

.DEFAULT_GOAL := help