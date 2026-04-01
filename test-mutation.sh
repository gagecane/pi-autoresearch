#!/bin/bash
# Mutation Testing Script for pi-autoresearch
# This script runs mutation tests to verify test quality

set -e

echo "========================================"
echo "Running Mutation Tests"
echo "========================================"

# Check if cargo-mutagen is installed
if ! command -v cargo-mutagen &> /dev/null; then
    echo "Installing cargo-mutagen..."
    cargo install cargo-mutagen
fi

echo ""
echo "Running mutation tests with cargo-mutagen..."
echo ""

# Run mutation tests
cargo mutagen test --test mutation_tests -- --nocapture

# Generate report
echo ""
echo "Generating mutation testing report..."
cargo mutagen report --format html --output target/mutation-report

echo ""
echo "========================================"
echo "Mutation Testing Complete"
echo "========================================"
echo "Report available at: target/mutation-report/index.html"
echo ""

# Show summary
echo "Mutation Testing Summary:"
cargo mutagen stats
