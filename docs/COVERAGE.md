# Code Coverage Guide

This document describes how to run code coverage analysis for pi-autoresearch.

## Requirements

- Rust toolchain with LLVM tools: `rustup component add llvm-tools-preview`
- cargo-llvm-cov: `cargo install cargo-llvm-cov`

## Quick Start

```bash
# Run coverage and generate all reports
./scripts/run-coverage.sh

# Run coverage with specific format
./scripts/run-coverage.sh lcov
./scripts/run-coverage.sh html
./scripts/run-coverage.sh cobertura
```

## Coverage Reports

### Text Report

View the summary:
```bash
cat coverage_report/text/index.txt
```

View detailed coverage for a file:
```bash
cat coverage_report/text/coverage/Users/sikashep/pi-autoresearch/src/main.rs.txt
```

### HTML Report

Generate and view:
```bash
./scripts/run-coverage.sh html
open coverage_report/html/index.html
```

### LCOV Report

For use with lcov tools:
```bash
./scripts/run-coverage.sh lcov
genhtml coverage_report/lcov.info -o coverage_report/lcov_html
```

### Cobertura Report

For CI/CD integration:
```bash
./scripts/run-coverage.sh cobertura
```

### Codecov Report

For uploading to Codecov:
```bash
./scripts/run-coverage.sh codecov
curl -s https://codecov.io/bash | bash -s -- -f coverage_report/codecov.json
```

## Current Coverage Status

As of 2026-04-01:

| Metric | Coverage |
|--------|----------|
| Regions | 78.37% |
| Functions | 83.51% |
| Lines | 80.20% |

## Coverage Goals

- **Minimum**: 75% line coverage
- **Target**: 85% line coverage
- **Ideal**: 90% line coverage

## Improving Coverage

### Uncovered Areas

To identify uncovered code:
```bash
# View uncovered lines in text format
cat coverage_report/text/coverage/Users/sikashep/pi-autoresearch/src/main.rs.txt | grep -E "^\s*0\s" | head -20
```

### Adding Tests

1. Identify uncovered functions/lines
2. Write unit tests for simple functions
3. Write integration tests for complex workflows
4. Run coverage to verify improvement

## CI/CD Integration

Add to your CI configuration:

```yaml
- name: Run coverage
  run: ./scripts/run-coverage.sh codecov

- name: Upload to Codecov
  uses: codecov/codecov-action@v3
  with:
    files: coverage_report/codecov.json
```

## Troubleshooting

### "llvm-tools-preview not found"

```bash
rustup component add llvm-tools-preview
```

### "LLVM_COV not found"

Set environment variables:
```bash
export LLVM_COV=$(rustc --print sysroot)/lib/rustlib/$RUSTUP_TOOLCHAIN/bin/llvm-cov
export LLVM_PROFDATA=$(rustc --print sysroot)/lib/rustlib/$RUSTUP_TOOLCHAIN/bin/llvm-profdata
```

### Coverage data not updating

Clean and rebuild:
```bash
cargo clean
./scripts/run-coverage.sh
```

## Related Documentation

- [Test Documentation](../tests/README.md)
- [Development Guide](../README.md#development)
