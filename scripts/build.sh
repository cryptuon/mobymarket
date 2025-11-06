#!/bin/bash

# Moby Market - Build Script
# Builds all libraries and programs in the correct order

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."

    if ! command_exists cargo; then
        print_error "Cargo not found. Please install Rust."
        exit 1
    fi

    if ! command_exists solana; then
        print_warning "Solana CLI not found. Some features may not work."
    fi

    if ! command_exists anchor; then
        print_warning "Anchor CLI not found. Program builds will be skipped."
    fi

    print_status "Prerequisites check completed."
}

# Build foundation libraries in order
build_foundation() {
    print_status "Building foundation libraries..."

    local libs=("moby-math" "moby-types" "moby-oracle")

    for lib in "${libs[@]}"; do
        if [ -d "libs/$lib" ]; then
            print_status "Building $lib..."
            cargo build -p "$lib" --release
        else
            print_warning "Library $lib not found, skipping..."
        fi
    done

    print_status "Foundation libraries built successfully."
}

# Build business logic libraries
build_business_logic() {
    print_status "Building business logic libraries..."

    local libs=("moby-trading" "moby-privacy" "moby-governance" "moby-bridge")

    for lib in "${libs[@]}"; do
        if [ -d "libs/$lib" ]; then
            print_status "Building $lib..."
            cargo build -p "$lib" --release
        else
            print_warning "Library $lib not found, skipping..."
        fi
    done

    print_status "Business logic libraries built successfully."
}

# Build Anchor programs
build_programs() {
    if ! command_exists anchor; then
        print_warning "Anchor CLI not found. Skipping program builds."
        return 0
    fi

    if [ -f "Anchor.toml" ]; then
        print_status "Building Anchor programs..."
        anchor build
        print_status "Anchor programs built successfully."
    else
        print_warning "No Anchor.toml found. Skipping program builds."
    fi
}

# Run tests
run_tests() {
    if [ "${SKIP_TESTS:-false}" = "true" ]; then
        print_warning "Skipping tests (SKIP_TESTS=true)"
        return 0
    fi

    print_status "Running tests..."
    cargo test --workspace --release
    print_status "All tests passed."
}

# Run clippy
run_clippy() {
    if [ "${SKIP_CLIPPY:-false}" = "true" ]; then
        print_warning "Skipping clippy (SKIP_CLIPPY=true)"
        return 0
    fi

    print_status "Running clippy..."
    cargo clippy --workspace --all-targets -- -D warnings
    print_status "Clippy checks passed."
}

# Check formatting
check_formatting() {
    if [ "${SKIP_FORMAT:-false}" = "true" ]; then
        print_warning "Skipping format check (SKIP_FORMAT=true)"
        return 0
    fi

    print_status "Checking code formatting..."
    cargo fmt --all -- --check
    print_status "Code formatting is correct."
}

# Clean build artifacts
clean() {
    print_status "Cleaning build artifacts..."
    cargo clean

    if [ -d "target" ]; then
        rm -rf target
    fi

    if [ -d ".anchor" ]; then
        rm -rf .anchor
    fi

    print_status "Clean completed."
}

# Print usage information
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --clean          Clean build artifacts before building"
    echo "  --skip-tests     Skip running tests"
    echo "  --skip-clippy    Skip running clippy"
    echo "  --skip-format    Skip format checking"
    echo "  --foundation     Build only foundation libraries"
    echo "  --business       Build only business logic libraries"
    echo "  --programs       Build only Anchor programs"
    echo "  --help           Show this help message"
    echo ""
    echo "Environment variables:"
    echo "  SKIP_TESTS=true      Skip running tests"
    echo "  SKIP_CLIPPY=true     Skip running clippy"
    echo "  SKIP_FORMAT=true     Skip format checking"
}

# Main build function
main() {
    local clean_first=false
    local foundation_only=false
    local business_only=false
    local programs_only=false

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --clean)
                clean_first=true
                shift
                ;;
            --foundation)
                foundation_only=true
                shift
                ;;
            --business)
                business_only=true
                shift
                ;;
            --programs)
                programs_only=true
                shift
                ;;
            --skip-tests)
                export SKIP_TESTS=true
                shift
                ;;
            --skip-clippy)
                export SKIP_CLIPPY=true
                shift
                ;;
            --skip-format)
                export SKIP_FORMAT=true
                shift
                ;;
            --help)
                usage
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done

    print_status "Starting Moby Market build process..."

    # Clean if requested
    if [ "$clean_first" = true ]; then
        clean
    fi

    # Check prerequisites
    check_prerequisites

    # Build based on options
    if [ "$foundation_only" = true ]; then
        build_foundation
    elif [ "$business_only" = true ]; then
        build_business_logic
    elif [ "$programs_only" = true ]; then
        build_programs
    else
        # Full build
        check_formatting
        run_clippy
        build_foundation
        build_business_logic
        build_programs
        run_tests
    fi

    print_status "Build process completed successfully!"
}

# Trap errors and cleanup
trap 'print_error "Build failed on line $LINENO"' ERR

# Run main function with all arguments
main "$@"