#!/bin/bash

# Moby Market - Test Script
# Comprehensive testing script for all libraries and programs

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TEST_TIMEOUT=${TEST_TIMEOUT:-300}
COVERAGE_THRESHOLD=${COVERAGE_THRESHOLD:-80}

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

print_debug() {
    echo -e "${BLUE}[DEBUG]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
check_prerequisites() {
    print_status "Checking test prerequisites..."

    if ! command_exists cargo; then
        print_error "Cargo not found. Please install Rust."
        exit 1
    fi

    # Check for optional tools
    if command_exists cargo-nextest; then
        export HAS_NEXTEST=true
        print_debug "cargo-nextest found - will use for parallel testing"
    else
        export HAS_NEXTEST=false
        print_debug "cargo-nextest not found - using standard cargo test"
    fi

    if command_exists cargo-llvm-cov; then
        export HAS_COVERAGE=true
        print_debug "cargo-llvm-cov found - coverage reporting enabled"
    else
        export HAS_COVERAGE=false
        print_debug "cargo-llvm-cov not found - coverage reporting disabled"
    fi

    print_status "Prerequisites check completed."
}

# Run unit tests for a specific library
test_library() {
    local lib_name="$1"
    local lib_path="libs/$lib_name"

    if [ ! -d "$lib_path" ]; then
        print_warning "Library $lib_name not found, skipping..."
        return 0
    fi

    print_status "Testing library: $lib_name"

    if [ "$HAS_NEXTEST" = true ]; then
        cargo nextest run -p "$lib_name" --no-fail-fast
    else
        cargo test -p "$lib_name" --lib --bins
    fi

    print_status "Library $lib_name tests completed."
}

# Run integration tests
run_integration_tests() {
    print_status "Running integration tests..."

    local test_dirs=(
        "tests/integration"
        "tests/e2e"
    )

    for test_dir in "${test_dirs[@]}"; do
        if [ -d "$test_dir" ]; then
            print_status "Running tests in $test_dir..."
            if [ "$HAS_NEXTEST" = true ]; then
                cargo nextest run --test-threads 1 --manifest-path "$test_dir/../Cargo.toml"
            else
                cargo test --manifest-path "$test_dir/../Cargo.toml"
            fi
        else
            print_debug "Test directory $test_dir not found, skipping..."
        fi
    done

    print_status "Integration tests completed."
}

# Run performance benchmarks
run_benchmarks() {
    if [ "${SKIP_BENCHMARKS:-false}" = "true" ]; then
        print_warning "Skipping benchmarks (SKIP_BENCHMARKS=true)"
        return 0
    fi

    print_status "Running performance benchmarks..."

    # Check if any benchmarks exist
    if find . -name "*.rs" -path "*/benches/*" | grep -q .; then
        cargo bench --workspace
        print_status "Benchmarks completed."
    else
        print_debug "No benchmarks found, skipping..."
    fi
}

# Generate code coverage report
generate_coverage() {
    if [ "$HAS_COVERAGE" = false ]; then
        print_warning "Coverage tool not available, skipping coverage report..."
        return 0
    fi

    if [ "${SKIP_COVERAGE:-false}" = "true" ]; then
        print_warning "Skipping coverage (SKIP_COVERAGE=true)"
        return 0
    fi

    print_status "Generating code coverage report..."

    # Generate coverage data
    cargo llvm-cov clean --workspace
    cargo llvm-cov --workspace --lcov --output-path coverage.lcov

    # Generate HTML report
    if command_exists genhtml; then
        genhtml coverage.lcov --output-directory coverage_html
        print_status "HTML coverage report generated in coverage_html/"
    fi

    # Check coverage threshold
    local coverage_percent
    coverage_percent=$(cargo llvm-cov --workspace | grep -oP 'TOTAL.*\K\d+(?=\.\d+%)' || echo "0")

    if [ "$coverage_percent" -lt "$COVERAGE_THRESHOLD" ]; then
        print_error "Coverage $coverage_percent% is below threshold $COVERAGE_THRESHOLD%"
        return 1
    else
        print_status "Coverage $coverage_percent% meets threshold $COVERAGE_THRESHOLD%"
    fi
}

# Run property-based tests
run_property_tests() {
    print_status "Running property-based tests..."

    # Look for proptest files
    if find . -name "*.rs" -exec grep -l "proptest" {} \; | grep -q .; then
        cargo test --workspace --release proptest
        print_status "Property tests completed."
    else
        print_debug "No property tests found, skipping..."
    fi
}

# Run Solana program tests
test_programs() {
    if ! command_exists anchor; then
        print_warning "Anchor CLI not found. Skipping program tests."
        return 0
    fi

    if [ -f "Anchor.toml" ]; then
        print_status "Running Anchor program tests..."
        anchor test
        print_status "Anchor program tests completed."
    else
        print_debug "No Anchor.toml found. Skipping program tests."
    fi
}

# Run security tests
run_security_tests() {
    print_status "Running security tests..."

    # Check for unsafe code
    print_status "Checking for unsafe code..."
    if find . -name "*.rs" -path "./libs/*" -exec grep -l "unsafe" {} \; | head -5; then
        print_warning "Unsafe code found - manual review required"
    else
        print_status "No unsafe code found in libraries"
    fi

    # Run cargo audit if available
    if command_exists cargo-audit; then
        print_status "Running security audit..."
        cargo audit
    else
        print_debug "cargo-audit not found, skipping security audit"
    fi

    # Run cargo deny if available
    if command_exists cargo-deny; then
        print_status "Running cargo deny checks..."
        cargo deny check
    else
        print_debug "cargo-deny not found, skipping deny checks"
    fi
}

# Test specific components
test_foundation() {
    print_status "Testing foundation libraries..."
    local libs=("moby-math" "moby-types" "moby-oracle")

    for lib in "${libs[@]}"; do
        test_library "$lib"
    done
}

test_business_logic() {
    print_status "Testing business logic libraries..."
    local libs=("moby-trading" "moby-privacy" "moby-governance" "moby-bridge")

    for lib in "${libs[@]}"; do
        test_library "$lib"
    done
}

# Clean test artifacts
clean_test_artifacts() {
    print_status "Cleaning test artifacts..."

    # Remove coverage files
    find . -name "*.lcov" -delete
    find . -name "*.profraw" -delete
    rm -rf coverage_html/

    # Remove test databases
    find . -name "*.db" -path "*/target/*" -delete

    # Remove temporary test files
    find . -name "test_*.tmp" -delete

    print_status "Test artifacts cleaned."
}

# Print usage information
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --foundation     Test only foundation libraries"
    echo "  --business       Test only business logic libraries"
    echo "  --integration    Run only integration tests"
    echo "  --programs       Test only Anchor programs"
    echo "  --benchmarks     Run only benchmarks"
    echo "  --coverage       Generate only coverage report"
    echo "  --security       Run only security tests"
    echo "  --clean          Clean test artifacts"
    echo "  --help           Show this help message"
    echo ""
    echo "Environment variables:"
    echo "  SKIP_BENCHMARKS=true    Skip benchmark tests"
    echo "  SKIP_COVERAGE=true      Skip coverage generation"
    echo "  TEST_TIMEOUT=300        Test timeout in seconds"
    echo "  COVERAGE_THRESHOLD=80   Minimum coverage percentage"
}

# Main test function
main() {
    local foundation_only=false
    local business_only=false
    local integration_only=false
    local programs_only=false
    local benchmarks_only=false
    local coverage_only=false
    local security_only=false
    local clean_only=false

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --foundation)
                foundation_only=true
                shift
                ;;
            --business)
                business_only=true
                shift
                ;;
            --integration)
                integration_only=true
                shift
                ;;
            --programs)
                programs_only=true
                shift
                ;;
            --benchmarks)
                benchmarks_only=true
                shift
                ;;
            --coverage)
                coverage_only=true
                shift
                ;;
            --security)
                security_only=true
                shift
                ;;
            --clean)
                clean_only=true
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

    print_status "Starting Moby Market test suite..."

    # Clean if requested
    if [ "$clean_only" = true ]; then
        clean_test_artifacts
        exit 0
    fi

    # Check prerequisites
    check_prerequisites

    # Run specific test suites based on options
    if [ "$foundation_only" = true ]; then
        test_foundation
    elif [ "$business_only" = true ]; then
        test_business_logic
    elif [ "$integration_only" = true ]; then
        run_integration_tests
    elif [ "$programs_only" = true ]; then
        test_programs
    elif [ "$benchmarks_only" = true ]; then
        run_benchmarks
    elif [ "$coverage_only" = true ]; then
        generate_coverage
    elif [ "$security_only" = true ]; then
        run_security_tests
    else
        # Full test suite
        test_foundation
        test_business_logic
        run_integration_tests
        test_programs
        run_property_tests
        run_security_tests
        run_benchmarks
        generate_coverage
    fi

    print_status "Test suite completed successfully!"
}

# Trap errors and cleanup
trap 'print_error "Test failed on line $LINENO"; clean_test_artifacts' ERR

# Run main function with all arguments
main "$@"