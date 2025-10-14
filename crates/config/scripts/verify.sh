#!/bin/bash
# StoryStream Configuration System - Verification Script
# This script verifies the entire config system is working correctly

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0

# Function to print colored output
print_status() {
    local status=$1
    local message=$2

    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

    if [ "$status" == "PASS" ]; then
        echo -e "${GREEN}✓${NC} $message"
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
    elif [ "$status" == "FAIL" ]; then
        echo -e "${RED}✗${NC} $message"
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
    elif [ "$status" == "WARN" ]; then
        echo -e "${YELLOW}⚠${NC} $message"
    else
        echo -e "${BLUE}ℹ${NC} $message"
    fi
}

print_header() {
    echo ""
    echo -e "${BLUE}════════════════════════════════════════${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}════════════════════════════════════════${NC}"
    echo ""
}

print_section() {
    echo ""
    echo -e "${YELLOW}▶ $1${NC}"
    echo ""
}

# Start verification
clear
print_header "StoryStream Config System Verification"

# 1. Environment Check
print_section "1. Environment Check"

if command -v cargo &> /dev/null; then
    RUST_VERSION=$(cargo --version)
    print_status "PASS" "Cargo found: $RUST_VERSION"
else
    print_status "FAIL" "Cargo not found"
    exit 1
fi

if command -v rustc &> /dev/null; then
    RUSTC_VERSION=$(rustc --version)
    print_status "PASS" "Rustc found: $RUSTC_VERSION"
else
    print_status "FAIL" "Rustc not found"
    exit 1
fi

# 2. Build Check
print_section "2. Build Check"

if cargo build --package storystream-config 2>&1 | grep -q "Finished"; then
    print_status "PASS" "Debug build successful"
else
    print_status "FAIL" "Debug build failed"
fi

if cargo build --package storystream-config --release 2>&1 | grep -q "Finished"; then
    print_status "PASS" "Release build successful"
else
    print_status "FAIL" "Release build failed"
fi

# 3. Test Suite
print_section "3. Test Suite"

if cargo test --package storystream-config --quiet 2>&1 | grep -q "test result: ok"; then
    print_status "PASS" "Unit tests passed"
else
    print_status "FAIL" "Unit tests failed"
fi

if cargo test --package storystream-config --test integration_tests --quiet 2>&1 | grep -q "test result: ok"; then
    print_status "PASS" "Integration tests passed"
else
    print_status "FAIL" "Integration tests failed"
fi

if cargo test --package storystream-config --test edge_case_tests --quiet 2>&1 | grep -q "test result: ok"; then
    print_status "PASS" "Edge case tests passed"
else
    print_status "FAIL" "Edge case tests failed"
fi

if cargo test --package storystream-config --test property_tests --quiet 2>&1 | grep -q "test result: ok"; then
    print_status "PASS" "Property tests passed"
else
    print_status "FAIL" "Property tests failed"
fi

if cargo test --package storystream-config --test smoke_test --quiet 2>&1 | grep -q "test result: ok"; then
    print_status "PASS" "Smoke tests passed"
else
    print_status "FAIL" "Smoke tests failed"
fi

# 4. Code Quality
print_section "4. Code Quality"

if cargo fmt --package storystream-config -- --check &> /dev/null; then
    print_status "PASS" "Code formatting correct"
else
    print_status "FAIL" "Code formatting issues found"
fi

if cargo clippy --package storystream-config -- -D warnings &> /dev/null; then
    print_status "PASS" "Clippy checks passed"
else
    print_status "FAIL" "Clippy warnings found"
fi

# 5. Documentation
print_section "5. Documentation"

if cargo doc --package storystream-config --no-deps &> /dev/null; then
    print_status "PASS" "Documentation builds successfully"
else
    print_status "FAIL" "Documentation build failed"
fi

# Check for required documentation files
REQUIRED_DOCS=(
    "README.md"
    "INTEGRATION_GUIDE.md"
    "TROUBLESHOOTING.md"
    "MIGRATION_GUIDE.md"
    "COMPLETION_SUMMARY.md"
    "QA_CHECKLIST.md"
    "CHANGELOG.md"
)

for doc in "${REQUIRED_DOCS[@]}"; do
    if [ -f "crates/config/$doc" ]; then
        print_status "PASS" "Documentation exists: $doc"
    else
        print_status "FAIL" "Missing documentation: $doc"
    fi
done

# 6. Examples
print_section "6. Examples"

if cargo build --package storystream-config --example basic_usage &> /dev/null; then
    print_status "PASS" "basic_usage example builds"
else
    print_status "FAIL" "basic_usage example build failed"
fi

if cargo build --package storystream-config --example advanced_usage &> /dev/null; then
    print_status "PASS" "advanced_usage example builds"
else
    print_status "FAIL" "advanced_usage example build failed"
fi

if cargo build --package storystream-config --example config_tool &> /dev/null; then
    print_status "PASS" "config_tool example builds"
else
    print_status "FAIL" "config_tool example build failed"
fi

# 7. Security
print_section "7. Security"

# Check for dangerous patterns in source code
if ! grep -r "\.unwrap()" crates/config/src --exclude-dir=tests &> /dev/null; then
    print_status "PASS" "No unwrap() in production code"
else
    print_status "FAIL" "Found unwrap() in production code"
fi

if ! grep -r "\.expect(" crates/config/src --exclude-dir=tests &> /dev/null; then
    print_status "PASS" "No expect() in production code"
else
    print_status "FAIL" "Found expect() in production code"
fi

if ! grep -r "panic!(" crates/config/src &> /dev/null; then
    print_status "PASS" "No panic!() in production code"
else
    print_status "FAIL" "Found panic!() in production code"
fi

if ! grep -r "todo!()" crates/config/src &> /dev/null; then
    print_status "PASS" "No todo!() in production code"
else
    print_status "FAIL" "Found todo!() in production code"
fi

if ! grep -r "unimplemented!()" crates/config/src &> /dev/null; then
    print_status "PASS" "No unimplemented!() in production code"
else
    print_status "FAIL" "Found unimplemented!() in production code"
fi

# 8. Benchmarks
print_section "8. Benchmarks"

if cargo bench --package storystream-config --no-run &> /dev/null; then
    print_status "PASS" "Benchmarks compile"
else
    print_status "FAIL" "Benchmark compilation failed"
fi

# 9. File Structure
print_section "9. File Structure"

REQUIRED_FILES=(
    "crates/config/Cargo.toml"
    "crates/config/src/lib.rs"
    "crates/config/src/error.rs"
    "crates/config/src/validation.rs"
    "crates/config/src/manager.rs"
    "crates/config/src/persistence.rs"
    "crates/config/src/migration.rs"
    "crates/config/src/player_config.rs"
    "crates/config/src/library_config.rs"
    "crates/config/src/app_config.rs"
    "crates/config/src/watcher.rs"
    "crates/config/src/schema.rs"
    "crates/config/src/backup.rs"
)

for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        print_status "PASS" "Required file exists: $file"
    else
        print_status "FAIL" "Missing required file: $file"
    fi
done

# 10. Functional Test
print_section "10. Functional Test"

# Create temporary test directory
TEST_DIR=$(mktemp -d)
export CONFIG_TEST_DIR="$TEST_DIR"

# Run config tool commands
if cargo run --package storystream-config --example config_tool -- --help &> /dev/null; then
    print_status "PASS" "Config tool runs"
else
    print_status "FAIL" "Config tool failed to run"
fi

# Cleanup
rm -rf "$TEST_DIR"

# Final Summary
print_header "Verification Summary"

echo -e "Total Checks:  ${BLUE}$TOTAL_CHECKS${NC}"
echo -e "Passed:        ${GREEN}$PASSED_CHECKS${NC}"
echo -e "Failed:        ${RED}$FAILED_CHECKS${NC}"
echo ""

if [ $FAILED_CHECKS -eq 0 ]; then
    echo -e "${GREEN}✓ ALL CHECKS PASSED${NC}"
    echo -e "${GREEN}✓ Config system is production ready!${NC}"
    echo ""
    exit 0
else
    echo -e "${RED}✗ SOME CHECKS FAILED${NC}"
    echo -e "${RED}✗ Please review failures above${NC}"
    echo ""
    exit 1
fi