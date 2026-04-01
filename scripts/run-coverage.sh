#!/bin/bash
# Run coverage analysis for pi-autoresearch
# Usage: ./scripts/run-coverage.sh [format]
# Formats: lcov, cobertura, codecov, text, json (default: all)

set -e

# Set up LLVM tools
export LLVM_COV=/Users/sikashep/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin/llvm-cov
export LLVM_PROFDATA=/Users/sikashep/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin/llvm-profdata

# Create output directory
mkdir -p coverage_report

# Clean previous coverage data
cargo clean -p pi_autoresearch

# Run tests with coverage
echo "Running tests with coverage instrumentation..."

# Generate reports based on format argument
case "${1:-all}" in
    lcov)
        cargo llvm-cov --lcov --output-path coverage_report/lcov.info
        echo "LCOV report saved to coverage_report/lcov.info"
        ;;
    cobertura)
        cargo llvm-cov --cobertura --output-path coverage_report/cobertura.xml
        echo "Cobertura report saved to coverage_report/cobertura.xml"
        ;;
    codecov)
        cargo llvm-cov --codecov --output-path coverage_report/codecov.json
        echo "Codecov report saved to coverage_report/codecov.json"
        ;;
    text)
        cargo llvm-cov --text --output-dir coverage_report/text
        echo "Text report saved to coverage_report/text/"
        cat coverage_report/text/index.txt
        ;;
    json)
        cargo llvm-cov --json --output-path coverage_report/json.json
        echo "JSON report saved to coverage_report/json.json"
        ;;
    html)
        cargo llvm-cov --html --output-dir coverage_report/html
        echo "HTML report saved to coverage_report/html/"
        ;;
    all)
        # Generate all formats
        cargo llvm-cov --lcov --output-path coverage_report/lcov.info
        cargo llvm-cov --cobertura --output-path coverage_report/cobertura.xml
        cargo llvm-cov --codecov --output-path coverage_report/codecov.json
        cargo llvm-cov --text --output-dir coverage_report/text
        cargo llvm-cov --json --output-path coverage_report/json.json
        echo ""
        echo "Coverage reports generated:"
        echo "  - LCOV: coverage_report/lcov.info"
        echo "  - Cobertura: coverage_report/cobertura.xml"
        echo "  - Codecov: coverage_report/codecov.json"
        echo "  - Text: coverage_report/text/"
        echo "  - JSON: coverage_report/json.json"
        echo ""
        echo "Summary:"
        cat coverage_report/text/index.txt
        ;;
    *)
        echo "Unknown format: $1"
        echo "Usage: $0 [lcov|cobertura|codecov|text|json|html|all]"
        exit 1
        ;;
esac

echo ""
echo "Coverage complete!"
