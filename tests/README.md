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
Mutation tests that verify test quality by introducing deliberate bugs and checking if existing tests catch them.

```bash
# Run mutation tests
cargo test --test mutation_tests

# Run mutation analysis (requires cargo-mutagen)
./test-mutation.sh
```

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
- **Integration Tests**: 37 tests for end-to-end workflows
- **Mutation Tests**: 14 tests for test quality verification
- **Benchmarks**: 5 performance benchmarks

### Total: 119 tests

## Mutation Testing

Mutation testing helps identify weak tests by introducing deliberate bugs (mutants) into the code and checking if existing tests catch them.

### Setup

```bash
# Install cargo-mutagen
cargo install cargo-mutagen

# Or use the provided script
./test-mutation.sh
```

### Running Mutation Tests

```bash
# Run mutation analysis
cargo mutagen test

# Generate HTML report
cargo mutagen report --format html

# View mutation coverage stats
cargo mutagen stats
```

### Mutation Coverage Goals

- Target: 80% mutation coverage
- Current: See `target/mutation-report/index.html`

### Mutators Applied

- **Arithmetic**: `+`, `-`, `*`, `/`, `%` operators
- **Logical**: `&&`, `||`, `!` operators
- **Conditional**: `if`, `match`, comparison operators
- **Return Value**: Changed return values
- **Remove Statement**: Removed statements

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
