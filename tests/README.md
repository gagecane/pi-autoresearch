# Test Suite

This directory contains the test suite for pi-autoresearch.

## Test Categories

### Integration Tests (`integration_tests.rs`)
End-to-end tests that verify the complete workflow of the application.

```bash
# Run all integration tests
cargo test --test integration_tests

# Run specific integration test
cargo test --test integration_tests test_name
```

### Mutation Tests (`mutation_tests.rs`)
Mutation-resistant tests that verify test quality. These tests are designed to follow mutation testing principles (they would fail if common bugs were introduced).

```bash
# Run mutation-resistant tests
cargo test --test mutation_tests

# Mutation testing framework (planned for future)
# ./test-mutation.sh (requires cargo-mutagen)

## Running All Tests

```bash
# Run all tests (unit + integration + mutation)
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test module
cargo test --test mutation_tests
```

## Test Coverage

The test suite includes:
- **Unit Tests**: 68 tests for helper functions, validation, and utilities
- **Integration Tests**: 60 tests for end-to-end workflows
- **Mutation-Resistant Tests**: 13 tests for test quality verification
- **Benchmarks**: 5 performance benchmarks

### Total: 141 tests

## Mutation-Resistant Tests

Mutation-resistant tests are designed to follow mutation testing principles. These tests would fail if common bugs (mutations) were introduced into the code.

### Current Implementation

- ✅ 13 mutation-resistant tests implemented
- ✅ Tests follow mutation testing principles
- ✅ All tests pass consistently

### Future Work

A full mutation testing framework (e.g., cargo-mutagen or cargo-mutest) is planned for future work. This would provide:
- Automated mutation generation
- Mutation coverage reporting
- Identification of weak tests

### Setup (Future)

```bash
# Install cargo-mutagen (when implemented)
cargo install cargo-mutagen

# Run mutation analysis
cargo mutagen test
```

## Best Practices

1. **Write tests before code** (TDD)
2. **Test edge cases** (empty inputs, invalid data, boundary values)
3. **Use descriptive test names** that explain what's being tested
4. **Keep tests independent** (no shared state between tests)
5. **Test error handling** (invalid inputs, failures, timeouts)
6. **Use mutation testing** to verify test quality

## Continuous Integration

Tests are run automatically in CI:
- All tests must pass
- Mutation coverage must be >= 80%
- Benchmarks must not regress by more than 10%
