# pi-autoresearch Examples

This document provides example use cases for pi-autoresearch.

## Example 1: Reduce Memory Usage

```bash
pi-autoresearch \
  --question "How can I reduce memory usage in my Rust application?" \
  --metric "peak_memory_mb" \
  --measure "cargo run --release -- --benchmark-memory" \
  --baseline 100.0 \
  --target-improvement 0.20 \
  --auto-approve
```

This will:
1. Measure the baseline memory usage (100 MB)
2. Run iterations to reduce memory usage by 20%
3. Save results to `autoresearch.jsonl`
4. Create a git branch with successful changes

## Example 2: Improve Performance

```bash
pi-autoresearch \
  --question "How can I improve the performance of my benchmark?" \
  --metric "execution_time_ms" \
  --measure "cargo bench --bench my_bench" \
  --baseline 500.0 \
  --target-improvement 0.30 \
  --max-iterations 25 \
  --auto-approve
```

This will:
1. Measure the baseline execution time (500 ms)
2. Run up to 25 iterations to reduce execution time by 30%
3. Track improvements and converge when threshold is reached

## Example 3: Use Config File

Create `~/.config/pi-autoresearch/config.json`:

```json
{
  "metric": "execution_time_ms",
  "measure": "cargo bench --bench performance",
  "baseline": 500.0,
  "target_improvement": 0.30,
  "max_iterations": 20
}
```

Run without specifying options:

```bash
pi-autoresearch --question "How can I improve performance?" --auto-approve
```

## Example 4: Verify Baseline

```bash
pi-autoresearch \
  --question "How can I reduce memory usage?" \
  --measure "cargo run --release -- --benchmark-memory" \
  --verify-baseline \
  --max-variance 0.02
```

This will:
1. Run the measurement command twice
2. Verify that the variance is within 2%
3. Record the baseline if verification passes

## Example 5: Resume Experiment

```bash
# View experiment history
pi-autoresearch --history

# Resume a specific experiment
pi-autoresearch --resume <SESSION_ID>
```

This will:
1. Load the previous experiment state
2. Continue from the best iteration
3. Use the best metric value as the new baseline

## Example 6: Compare Experiments

```bash
# Compare two experiments
pi-autoresearch \
  --compare-id1 <SESSION_ID1> \
  --compare-id2 <SESSION_ID2>
```

This will display:
- Side-by-side comparison of metrics
- Best improvement for each experiment
- Winner based on improvement percentage
- Detailed iteration breakdown

## Example 7: Dry Run

```bash
pi-autoresearch \
  --question "How can I reduce memory usage?" \
  --measure "cargo run --release -- --benchmark-memory" \
  --dry-run
```

This will:
1. Show what changes would be made
2. Skip actual code changes
3. Skip git branch creation
4. Skip session file writes

## Example 8: Clean Up Branches

```bash
# List all autoresearch branches
pi-autoresearch --list-branches

# Clean up branches older than 7 days
pi-autoresearch --cleanup-branches --cleanup-days 7

# Clean up branches older than 30 days
pi-autoresearch --cleanup-branches --cleanup-days 30
```

## Example 9: Skip Git Operations

```bash
pi-autoresearch \
  --question "How can I improve performance?" \
  --measure "cargo bench --bench my_bench" \
  --skip-git \
  --auto-approve
```

This will:
1. Run the experiment normally
2. Skip git branch creation and commits
3. Save results to session file only

## Example 10: Project-Specific Config

Create `./config.json` in your project directory:

```json
{
  "metric": "accuracy_percent",
  "measure": "cargo test --test accuracy_test",
  "target_improvement": 0.05,
  "max_iterations": 30
}
```

Run with project-specific config:

```bash
pi-autoresearch \
  --config ./config.json \
  --question "How can I improve test accuracy?" \
  --auto-approve
```

## Example 11: Beads Integration

```bash
pi-autoresearch \
  --question "How can I reduce memory usage?" \
  --beads-enabled \
  --auto-approve
```

This will:
1. Create a beads issue for the experiment
2. Update issue status during iterations
3. Close issue when experiment completes

## Example 12: Custom Session File

```bash
pi-autoresearch \
  --question "How can I improve performance?" \
  --session-file "experiments/performance.jsonl" \
  --auto-approve
```

This will:
1. Save results to `experiments/performance.jsonl`
2. Allow multiple experiment files for different projects

---

## Advanced Configuration Examples

### Example 13: Multi-Metric Optimization

Optimize multiple metrics sequentially:

```bash
# First, optimize for memory
pi-autoresearch \
  --question "How can I reduce memory usage?" \
  --metric "peak_memory_mb" \
  --measure "cargo run --release -- --benchmark-memory" \
  --baseline 100.0 \
  --target-improvement 0.20 \
  --session-file "experiments/memory.jsonl" \
  --auto-approve

# Then, optimize for performance without regressing memory
pi-autoresearch \
  --question "How can I improve performance without increasing memory?" \
  --metric "execution_time_ms" \
  --measure "cargo bench --bench my_bench" \
  --baseline 500.0 \
  --target-improvement 0.15 \
  --session-file "experiments/performance.jsonl" \
  --auto-approve
```

### Example 14: Aggressive Optimization with High Iteration Count

```bash
pi-autoresearch \
  --question "How can I maximize performance?" \
  --metric "execution_time_ms" \
  --measure "cargo bench --bench my_bench" \
  --baseline 500.0 \
  --target-improvement 0.50 \
  --max-iterations 50 \
  --iteration-timeout-minutes 30 \
  --total-timeout-minutes 480 \
  --stall-limit 5 \
  --convergence-threshold 0.005 \
  --convergence-window 10 \
  --auto-approve
```

This configuration:
- Runs up to 50 iterations
- Allows up to 30 minutes per iteration
- Total timeout of 8 hours
- Requires 10 consecutive iterations with <0.5% improvement to converge
- Allows 5 stalled iterations before backing off

### Example 15: Conservative Optimization with Strict Convergence

```bash
pi-autoresearch \
  --question "How can I safely improve performance?" \
  --metric "execution_time_ms" \
  --measure "cargo bench --bench my_bench" \
  --baseline 500.0 \
  --target-improvement 0.10 \
  --max-iterations 10 \
  --stall-limit 2 \
  --convergence-threshold 0.01 \
  --convergence-window 3 \
  --verify-baseline \
  --max-variance 0.01 \
  --auto-approve
```

This configuration:
- Limited to 10 iterations
- Strict baseline verification (1% variance)
- Converges quickly (3 iterations with <1% improvement)
- Stops after 2 stalled iterations

---

## CI/CD Integration Examples

### Example 16: GitHub Actions Integration

Create `.github/workflows/autoresearch.yml`:

```yaml
name: AutoResearch

on:
  schedule:
    - cron: '0 2 * * *'  # Run daily at 2 AM
  workflow_dispatch:  # Allow manual trigger

jobs:
  optimize:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      
      - name: Build pi-autoresearch
        run: |
          cargo build --release
      
      - name: Run optimization experiment
        run: |
          ./target/release/pi-autoresearch \
            --question "How can I improve CI build performance?" \
            --metric "build_time_seconds" \
            --measure "time cargo build --release" \
            --target-improvement 0.10 \
            --max-iterations 5 \
            --auto-approve \
            --skip-git
      
      - name: Upload session file
        uses: actions/upload-artifact@v3
        with:
          name: autoresearch-session
          path: autoresearch.jsonl
```

### Example 17: GitLab CI Integration

Create `.gitlab-ci.yml`:

```yaml
autoresearch:
  image: rust:latest
  script:
    - cargo build --release
    - ./target/release/pi-autoresearch \
        --question "How can I reduce test execution time?" \
        --metric "test_time_seconds" \
        --measure "time cargo test" \
        --target-improvement 0.15 \
        --max-iterations 10 \
        --auto-approve
  rules:
    - if: $CI_PIPELINE_SOURCE == "schedule"
  artifacts:
    paths:
      - autoresearch.jsonl
    expire_in: 30 days
```

### Example 18: Jenkins Pipeline

Create `Jenkinsfile`:

```groovy
pipeline {
    agent any
    
    stages {
        stage('AutoResearch') {
            steps {
                sh '''
                    cargo build --release
                    ./target/release/pi-autoresearch \
                        --question "How can I optimize build performance?" \
                        --metric "build_time_seconds" \
                        --measure "time cargo build" \
                        --target-improvement 0.10 \
                        --max-iterations 5 \
                        --auto-approve
                '''
            }
            post {
                always {
                    archiveArtifacts artifacts: 'autoresearch.jsonl'
                }
            }
        }
    }
}
```

---

## Beads Workflow Examples

### Example 19: Beads Integration with Auto-Approve

```bash
# Create and track experiment with beads
pi-autoresearch \
  --question "How can I reduce memory usage in the database module?" \
  --beads-enabled \
  --auto-approve \
  --measure "cargo run --release -- --benchmark-memory" \
  --target-improvement 0.20
```

This will:
1. Create a beads issue for the experiment
2. Update issue status during iterations
3. Link commits to the issue
4. Close issue when experiment completes

### Example 20: Beads with Manual Approval

```bash
# Create beads issue with manual approval for each iteration
pi-autoresearch \
  --question "How can I improve query performance?" \
  --beads-enabled \
  --measure "cargo bench --bench query_bench"
```

This will:
1. Create a beads issue
2. Prompt for approval after each iteration
3. Update issue with iteration results
4. Allow you to review changes before accepting

---

## Common Patterns and Best Practices

### Example 21: Nightly Optimization Job

Set up a cron job for nightly optimization:

```bash
# Add to crontab (crontab -e)
0 2 * * * cd /path/to/project && \
  pi-autoresearch \
    --question "How can I improve performance?" \
    --measure "cargo bench" \
    --target-improvement 0.05 \
    --max-iterations 10 \
    --auto-approve \
    --session-file "experiments/nightly-$(date +%Y%m%d).jsonl"
```

### Example 22: Pre-Commit Optimization

Create a git hook to run quick optimizations before commits:

```bash
# .git/hooks/pre-commit
#!/bin/bash

# Run quick performance check
pi-autoresearch \
  --question "Can we improve performance before this commit?" \
  --measure "cargo test --release" \
  --target-improvement 0.01 \
  --max-iterations 2 \
  --auto-approve \
  --skip-git \
  --session-file "/tmp/pre-commit-autoresearch.jsonl"
```

### Example 23: A/B Testing with Session Comparison

```bash
# Run two different optimization strategies
pi-autoresearch \
  --question "Strategy A: How can I optimize with caching?" \
  --measure "cargo bench" \
  --session-file "experiments/strategy-a.jsonl" \
  --auto-approve

pi-autoresearch \
  --question "Strategy B: How can I optimize with parallelization?" \
  --measure "cargo bench" \
  --session-file "experiments/strategy-b.jsonl" \
  --auto-approve

# Compare results
pi-autoresearch \
  --compare-id1 $(jq -r '.session_id' experiments/strategy-a.jsonl | head -1) \
  --compare-id2 $(jq -r '.session_id' experiments/strategy-b.jsonl | head -1)
```

### Example 24: Incremental Optimization

```bash
# First pass: Quick wins
pi-autoresearch \
  --question "What are the quick wins for performance?" \
  --measure "cargo bench" \
  --target-improvement 0.10 \
  --max-iterations 5 \
  --auto-approve

# Second pass: Deeper optimization
pi-autoresearch \
  --resume <SESSION_ID_FROM_FIRST_PASS> \
  --target-improvement 0.20 \
  --max-iterations 10 \
  --auto-approve
```

### Example 25: Team Collaboration Workflow

```bash
# Team member 1: Initial optimization
pi-autoresearch \
  --question "How can we improve the rendering pipeline?" \
  --measure "cargo bench --bench rendering" \
  --session-file "team/rendering-pipeline.jsonl" \
  --auto-approve

# Team member 2: Review and continue
git pull
pi-autoresearch \
  --resume <SESSION_ID> \
  --max-iterations 10 \
  --auto-approve

# Team lead: Compare all attempts
pi-autoresearch \
  --compare-id1 <SESSION_ID_1> \
  --compare-id2 <SESSION_ID_2>
```

---

## Troubleshooting Examples

### Example 26: Debug Mode

```bash
# Enable verbose logging for debugging
RUST_LOG=debug pi-autoresearch \
  --question "How can I improve performance?" \
  --measure "cargo bench" \
  --verbose
```

### Example 27: Test Measurement Command

```bash
# Test your measurement command before running experiment
$ cargo bench --bench my_bench
# Verify output contains a parseable number

# If using custom script
$ ./measure-performance.sh
# Should output: 123.45 (just the number)
```

### Example 28: Handle High Variance

```bash
# If measurements have high variance, run multiple times and average
#!/bin/bash
# measure-averaged.sh
for i in {1..5}; do
  cargo bench --bench my_bench 2>&1 | grep "time:" | awk '{print $2}'
done | awk '{sum+=$1} END {print sum/NR}'

# Use in pi-autoresearch
pi-autoresearch \
  --question "How can I improve performance?" \
  --measure "./measure-averaged.sh" \
  --verify-baseline \
  --max-variance 0.05
```

---

## Metric Detection Examples

### Example 29: Automatic Metric Detection

pi-autoresearch automatically detects the metric from your question:

```bash
# Memory-related questions
pi-autoresearch --question "How can I reduce memory usage?"
# Auto-detects: metric = "peak_memory_mb"

# Performance-related questions
pi-autoresearch --question "How can I speed up the build?"
# Auto-detects: metric = "execution_time_ms"

# Accuracy-related questions
pi-autoresearch --question "How can I improve test accuracy?"
# Auto-detects: metric = "accuracy_percent"
```

### Example 30: Custom Metric Names

```bash
# Use custom metric names for your specific use case
pi-autoresearch \
  --question "How can I improve user experience?" \
  --metric "user_satisfaction_score" \
  --measure "./run-user-survey.sh" \
  --baseline 3.5 \
  --target-improvement 0.10
```

---

## Configuration File Examples

### Example 31: Comprehensive Config File

Create `~/.config/pi-autoresearch/config.json`:

```json
{
  "metric": "execution_time_ms",
  "measure": "cargo bench --release",
  "baseline": 1000.0,
  "target_improvement": 0.20,
  "max_iterations": 20,
  "iteration_timeout_minutes": 30,
  "total_timeout_minutes": 480,
  "stall_limit": 3,
  "convergence_threshold": 0.01,
  "convergence_window": 5,
  "max_variance": 0.02,
  "session_file": "experiments/default.jsonl",
  "beads_enabled": false
}
```

### Example 32: Project-Specific Config

Create `./autoresearch-config.json` in your project:

```json
{
  "metric": "test_coverage_percent",
  "measure": "cargo tarpaulin --out Xml 2>/dev/null | grep -oP 'coverage=\K[0-9.]+'",
  "target_improvement": 0.05,
  "max_iterations": 15,
  "convergence_threshold": 0.005
}
```

Run with:
```bash
pi-autoresearch \
  --config ./autoresearch-config.json \
  --question "How can I improve test coverage?" \
  --auto-approve
```

### Example 33: Environment-Based Config Selection

```bash
# Development config
pi-autoresearch --config ./config-dev.json --question "..."

# Production config
pi-autoresearch --config ./config-prod.json --question "..."

# CI config
pi-autoresearch --config ./config-ci.json --question "..."
```

---

## Performance Tuning Examples

### Example 34: Fast Iterations for Quick Feedback

```bash
pi-autoresearch \
  --question "Quick performance check" \
  --measure "cargo test --release -- --test-threads=1" \
  --max-iterations 20 \
  --iteration-timeout-minutes 5 \
  --stall-limit 2 \
  --convergence-window 2 \
  --auto-approve
```

### Example 35: Thorough Optimization with Long Timeouts

```bash
pi-autoresearch \
  --question "Comprehensive performance optimization" \
  --measure "cargo bench --release -- --save-baseline" \
  --max-iterations 100 \
  --iteration-timeout-minutes 60 \
  --total-timeout-minutes 1440 \
  --stall-limit 10 \
  --convergence-threshold 0.001 \
  --convergence-window 20 \
  --auto-approve
```
