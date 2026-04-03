#!/bin/bash
# Mutation Testing Script for pi-autoresearch
# This script runs mutation tests using cargo-darwin

set -e

echo "========================================="
echo "Running Mutation Tests with cargo-darwin"
echo "========================================="

# Check if cargo-darwin is installed
if ! command -v cargo-darwin &> /dev/null; then
    echo "cargo-darwin not found. Installing..."
    cargo install cargo-darwin
fi

echo ""
echo "Building project..."
cargo build --release

echo ""
echo "Running unit tests first..."
cargo test --lib

echo ""
echo "Running mutation tests..."
echo "This may take a while (typically 10-30 minutes)..."
echo ""

# Run mutation tests with cargo-darwin
cargo darwin --timeout 300 --jobs 4

echo ""
echo "========================================="
echo "Mutation Test Complete"
echo "========================================="
echo ""
echo "Reports generated in: target/darwin/"
echo ""
echo "Key Metrics:"
echo "- Total Mutations: See report for details"
echo "- Killed Mutations: Tests caught the mutation"
echo "- Surviving Mutations: Tests did not catch (weak tests)"
echo "- Timeout Mutations: Test took too long"
echo "- Equivalent Mutations: Mutation is equivalent to original code"
echo ""
echo "Kill Ratio = Killed / (Killed + Surviving)"
echo "Target Kill Ratio: 80%+"
echo ""

# Display summary if report exists
if [ -d "target/darwin" ]; then
    echo "Report location: target/darwin/"
    ls -la target/darwin/ 2>/dev/null || echo "No reports generated yet"
fi

# Try to find and display any JSON report
if [ -f "target/darwin/report.json" ]; then
    echo ""
    echo "Summary:"
    cat target/darwin/report.json | jq '.' 2>/dev/null || cat target/darwin/report.json
fi
