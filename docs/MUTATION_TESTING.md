# Mutation Testing Guide

This document describes the mutation testing setup for pi-autoresearch.

## What is Mutation Testing?

Mutation testing is a software testing method that evaluates the quality of test cases by systematically introducing small changes (mutations) to the source code and verifying that the test suite catches these changes.

### Key Concepts

- **Mutant**: A version of the code with a small intentional change
- **Killed Mutant**: A mutant that is detected by the test suite (test fails)
- **Surviving Mutant**: A mutant that is NOT detected (test passes - indicates weak test)
- **Kill Ratio**: Percentage of mutants killed by tests (target: 80%+)

### Types of Mutations

1. **Arithmetic Mutations**: Change `+` to `-`, `*` to `/`, etc.
2. **Boolean Mutations**: Change `&&` to `||`, `!` to identity, etc.
3. **Return Value Mutations**: Change return values
4. **Statement Removal**: Remove statements
5. **Operator Replacement**: Replace comparison operators

## Setup

### Prerequisites

```bash
# Install cargo-darwin
cargo install cargo-darwin

# Or use the provided script
./scripts/run-mutation-tests.sh
```

### Configuration

The mutation testing configuration is in `darwin.toml`:

```toml
[darwin]
jobs = 4                    # Parallel test processes
timeout = 300               # 5 minutes per mutation
coverage_threshold = 0.80   # Target 80% kill ratio

include = [
    "src/lib.rs",
    "src/cli.rs",
    "src/session.rs",
]

exclude = [
    "src/main.rs",
    "src/audit.rs",
]
```

## Running Mutation Tests

### Quick Run

```bash
# Run all mutation tests
./scripts/run-mutation-tests.sh
```

### Manual Run

```bash
# Build first
cargo build --release

# Run mutation tests
cargo darwin

# Run with specific options
cargo darwin --jobs 8 --timeout 600
```

### Run on Specific Module

```bash
# Test only cli module
cargo darwin --lib cli

# Test only session module
cargo darwin --lib session
```

## Interpreting Results

### Report Files

Reports are generated in `target/darwin/`:

- `target/darwin/report.json` - Machine-readable results
- `target/darwin/` - Contains individual mutant results

### Key Metrics

```json
{
  "total_mutations": 500,
  "killed": 420,
  "surviving": 50,
  "timeout": 20,
  "equivalent": 10,
  "kill_ratio": 0.89
}
```

**Kill Ratio Calculation:**
```
Kill Ratio = Killed / (Killed + Surviving)
           = 420 / (420 + 50)
           = 0.89 (89%)
```

### Result Categories

| Category | Meaning | Action |
|----------|---------|--------|
| **Killed** | Test caught the mutation | ✅ Good! |
| **Surviving** | Test did not catch | ⚠️ Improve test coverage |
| **Timeout** | Test took too long | ⏱️ Optimize test or increase timeout |
| **Equivalent** | Mutation is same as original | ℹ️ Ignore |

## Improving Test Quality

### If Kill Ratio is Low

1. **Identify surviving mutants**
   ```bash
   cargo darwin --show-surviving
   ```

2. **Review the mutation**
   - What change was made?
   - Why didn't the test catch it?

3. **Add/improve tests**
   - Add edge case tests
   - Add specific assertions
   - Test more input combinations

### Example: Improving a Test

**Before (weak test):**
```rust
#[test]
fn test_calculate_improvement() {
    let baseline = 100.0;
    let new_value = 80.0;
    let improvement = calculate_improvement(baseline, new_value);
    assert!(improvement > 0.0);  // Weak: only checks positive
}
```

**After (strong test):**
```rust
#[test]
fn test_calculate_improvement() {
    let baseline = 100.0;
    let new_value = 80.0;
    let improvement = calculate_improvement(baseline, new_value);
    assert!((improvement - 0.20).abs() < 0.001);  // Strong: exact value
}

#[test]
fn test_calculate_improvement_negative() {
    let baseline = 100.0;
    let new_value = 120.0;
    let improvement = calculate_improvement(baseline, new_value);
    assert!(improvement < 0.0);  // Edge case
}

#[test]
fn test_calculate_improvement_zero() {
    let baseline = 100.0;
    let new_value = 100.0;
    let improvement = calculate_improvement(baseline, new_value);
    assert_eq!(improvement, 0.0);  // Edge case
}
```

## Critical Functions

Some functions require higher mutation coverage:

| Function | Target Kill Ratio | Rationale |
|----------|------------------|-----------|
| `validate_config` | 95% | Configuration errors cause failures |
| `calculate_improvement` | 95% | Core metric calculation |
| `detect_stalled` | 90% | Experiment termination logic |
| `detect_converged` | 90% | Experiment termination logic |
| `run_experiment` | 85% | Main experiment flow |
| `measure_metric` | 90% | Measurement accuracy critical |

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Mutation Testing

on:
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 0 * * 0'  # Weekly on Sunday

jobs:
  mutation-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Install cargo-darwin
        run: cargo install cargo-darwin
      
      - name: Run mutation tests
        run: ./scripts/run-mutation-tests.sh
      
      - name: Upload report
        uses: actions/upload-artifact@v3
        with:
          name: mutation-report
          path: target/darwin/
```

## Best Practices

### Do ✅

- Run mutation tests regularly (weekly or on PR)
- Aim for 80%+ kill ratio
- Focus on critical functions first
- Use mutation testing to find weak tests
- Document surviving mutants and plan fixes

### Don't ❌

- Don't ignore surviving mutants
- Don't set unrealistic targets (95%+ is often impossible)
- Don't mutation test performance-critical code in CI (too slow)
- Don't use mutation testing instead of coverage testing (complementary)

## Troubleshooting

### Tests Take Too Long

```toml
# Reduce parallel jobs
[darwin]
jobs = 2

# Reduce timeout
timeout = 120  # 2 minutes
```

Or use command-line options:
```bash
cargo darwin --jobs 2 --timeout 120
```

### Too Many Surviving Mutants

1. Check if tests are too weak
2. Add more specific assertions
3. Test more edge cases
4. Consider if mutation is equivalent (false positive)

### cargo-darwin Not Found

```bash
# Install manually
cargo install cargo-darwin

# Check installation
cargo-darwin --version
```

## Resources

- [cargo-darwin Documentation](https://github.com/aschuner/cargo-darwin)
- [Mutation Testing Wikipedia](https://en.wikipedia.org/wiki/Mutation_testing)
- [Zoo Project - Mutation Testing Tools](https://zoo.cs.yale.edu/classes/cs201/materials/mutation/)

## Appendix: Mutation Types

### Arithmetic Mutations

| Original | Mutated |
|----------|---------|
| `a + b` | `a - b` |
| `a - b` | `a + b` |
| `a * b` | `a / b` |
| `a / b` | `a * b` |
| `a % b` | `a / b` |

### Boolean Mutations

| Original | Mutated |
|----------|---------|
| `a && b` | `a \\| b` |
| `a \\| b` | `a && b` |
| `!a` | `a` |
| `true` | `false` |
| `false` | `true` |

### Comparison Mutations

| Original | Mutated |
|----------|---------|
| `a < b` | `a <= b` |
| `a <= b` | `a < b` |
| `a > b` | `a >= b` |
| `a >= b` | `a > b` |
| `a == b` | `a != b` |
| `a != b` | `a == b` |

### Return Value Mutations

| Original | Mutated |
|----------|---------|
| `return x` | `return 0` |
| `return true` | `return false` |
| `return None` | `return Some(x)` |

### Statement Removal

| Original | Mutated |
|----------|---------|
| `x = f(); y = g();` | `y = g();` |
| `if (cond) { body }` | `if (false) { body }` |
