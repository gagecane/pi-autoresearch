# Fuzz Testing for pi-autoresearch

This directory contains fuzz tests for the pi-autoresearch session file parsing functionality.

## Fuzz Targets

### 1. `fuzz_baseline_record`
Tests JSON parsing of `BaselineRecord` objects.

### 2. `fuzz_iteration_record`
Tests JSON parsing of `IterationRecord` objects.

### 3. `fuzz_experiment_session`
Tests JSON parsing of `ExperimentSession` objects.

### 4. `fuzz_session_file`
Tests full session file parsing (JSONL format with multi-line JSON support).

## Setup

### Install cargo-fuzz

```bash
rustup update nightly
cargo install cargo-fuzz
```

### Build Fuzz Targets

```bash
cd fuzz
cargo fuzz build
```

## Running Fuzz Tests

### Run a Specific Fuzz Target

```bash
cd fuzz
cargo fuzz run fuzz_baseline_record
cargo fuzz run fuzz_iteration_record
cargo fuzz run fuzz_experiment_session
cargo fuzz run fuzz_session_file
```

### Run All Fuzz Targets

```bash
cd fuzz
for target in fuzz_baseline_record fuzz_iteration_record fuzz_experiment_session fuzz_session_file; do
    cargo fuzz run $target &
done
wait
```

### Run with Custom Options

```bash
# Run for a specific duration
cargo fuzz run fuzz_session_file -max_time=300

# Run with specific corpus
cargo fuzz run fuzz_session_file -artifact_prefix=corpus/

# Run in single mode (no forking)
cargo fuzz run fuzz_session_file -jobs=1
```

## Corpus Management

### Initial Corpus

The fuzzer will automatically create a corpus directory at `fuzz/corpus/<target_name>/`.

You can seed the corpus with known valid inputs:

```bash
# Add valid BaselineRecord JSON
echo '{"timestamp":"2024-01-01T00:00:00Z","git_commit":"abc123","metric":"latency","measurement_command":"echo 100","value":100.0,"verification_runs":[100.0,101.0],"variance":0.01,"within_threshold":true}' > fuzz/corpus/fuzz_baseline_record/valid_baseline

# Add valid IterationRecord JSON
echo '{"iteration":1,"timestamp":"2024-01-01T00:00:00Z","agent_action":"Optimized query","metric_value":95.0,"improvement":0.05,"kept":true}' > fuzz/corpus/fuzz_iteration_record/valid_iteration

# Add valid ExperimentSession JSON (multi-line)
cat > fuzz/corpus/fuzz_experiment_session/valid_session << 'EOF'
{
  "session_id": "test-123",
  "question": "How to optimize?",
  "design": {
    "question": "How to optimize?",
    "metric": "latency",
    "measurement_command": "echo 100",
    "baseline": 100.0,
    "target_improvement": 0.20,
    "max_iterations": 10
  },
  "baseline_record": {
    "timestamp": "2024-01-01T00:00:00Z",
    "git_commit": "abc123",
    "metric": "latency",
    "measurement_command": "echo 100",
    "value": 100.0,
    "verification_runs": [100.0, 101.0],
    "variance": 0.01,
    "within_threshold": true
  },
  "iterations": [],
  "best_iteration": null,
  "start_time": "2024-01-01T00:00:00Z",
  "end_time": null,
  "status": "running"
}
EOF
```

### Corpus Generation

The fuzzer will automatically generate and save interesting inputs to the corpus directory. These can be used to:
- Seed future fuzzing runs
- Create regression tests
- Document edge cases

## Crash Analysis

When the fuzzer finds a crash, it will save the input to `fuzz/artifacts/<target_name>/crash-*`.

### Analyzing a Crash

```bash
# Run with the crashing input
cargo fuzz run fuzz_session_file artifacts/fuzz_session_file/crash-*

# View the crashing input
cat artifacts/fuzz_session_file/crash-* | xxd
```

### Common Crash Types

1. **JSON Parse Errors** - Malformed JSON that causes panic
2. **Integer Overflow** - Large numbers causing overflow
3. **Stack Overflow** - Deeply nested structures
4. **Memory Exhaustion** - Very large inputs

## CI/CD Integration

Add fuzz testing to your CI pipeline:

```yaml
# .github/workflows/fuzz.yml
name: Fuzz Testing

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 0 * * *'  # Daily at midnight

jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: nightly
          override: true
      - run: cargo install cargo-fuzz
      - run: cd fuzz && cargo fuzz build
      - run: cd fuzz && cargo fuzz run fuzz_session_file -max_time=60
```

## Best Practices

1. **Run fuzzers regularly** - Schedule daily fuzzing runs
2. **Keep corpus up to date** - Add new valid inputs as you discover them
3. **Analyze crashes quickly** - Fix bugs found by fuzzing promptly
4. **Add regression tests** - Convert crashing inputs to unit tests
5. **Monitor coverage** - Track code coverage improvements over time

## Troubleshooting

### Fuzzer not finding bugs

- Increase fuzzing time
- Add more diverse corpus inputs
- Check if the fuzzer is actually exercising the code

### Fuzzer too slow

- Reduce input size limits
- Run multiple fuzzers in parallel
- Use a faster machine

### Out of memory errors

- Reduce the maximum input size
- Check for memory leaks in the target code
- Increase system memory limits

## Resources

- [cargo-fuzz Documentation](https://github.com/rust-fuzz/cargo-fuzz)
- [libfuzzer-sys Documentation](https://docs.rs/libfuzzer-sys/)
- [Rust Fuzzing Book](https://rust-fuzz.github.io/book/)
