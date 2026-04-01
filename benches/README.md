# Performance Benchmarks

This directory contains performance benchmarks for pi-autoresearch.

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench session_file_parsing

# Run with verbose output
cargo bench -- --verbose
```

## Benchmark Results

### Current Baseline (2026-04-01)

| Benchmark | Time | Iterations |
|-----------|------|------------|
| session_file_parsing | ~5.3µs | ~939k |
| config_file_loading | ~14.4µs | ~348k |
| metric_detection | ~560ns | ~9.0M |
| git_branch_name_generation | ~340ns | ~12M |
| iteration_record_creation | ~360ns | ~8.5M |

## Performance Regression Testing

### Setup Baseline

```bash
# Run benchmarks and save baseline
cargo bench -- --save-baseline
```

### Check for Regressions

```bash
# Compare current benchmarks against baseline
cargo bench -- --compare-baseline

# Fail if regression exceeds threshold (default: 10%)
cargo bench -- --check-regression

# Set custom threshold (20%)
cargo bench -- --check-regression --threshold 0.20
```

### CI/CD Integration

```yaml
- name: Run benchmarks
  run: cargo bench

- name: Check for regressions
  run: cargo bench -- --check-regression
```

## Benchmark Details

### session_file_parsing
Measures the time to parse a JSONL session file with baseline, iteration, and experiment records.

### config_file_loading
Measures the time to load and parse a configuration file.

### metric_detection
Measures the time to detect the appropriate metric from a research question.

### git_branch_name_generation
Measures the time to generate a unique git branch name.

### iteration_record_creation
Measures the time to create a JSON iteration record.

## Acceptable Degradation

- **Critical**: >20% degradation - Block merge
- **Warning**: 10-20% degradation - Review required
- **Acceptable**: <10% degradation - Normal variance

## Troubleshooting

### "Gnuplot not found"

```bash
# Install gnuplot for better plots
brew install gnuplot  # macOS
sudo apt-get install gnuplot  # Debian/Ubuntu
```

### Benchmarks not running

```bash
# Ensure release profile is configured
cargo build --release
```

### Flaky benchmarks

```bash
# Run with more samples
cargo bench -- --samples 200
```

## Related Documentation

- [Cargo Criterion Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Performance Testing Best Practices](https://www.kernel.org/doc/html/latest/dev-tools/criterion.html)

