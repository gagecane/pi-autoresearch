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

### Example 13: Complete Configuration with All Options

Create a comprehensive config file `~/.config/pi-autoresearch/config.json`:

```json
{
  "metric": "execution_time_ms",
  "measure": "cargo bench --bench my_bench -- --time 5s",
  "baseline": 500.0,
  "target_improvement": 0.25,
  "max_iterations": 30,
  "max_variance": 0.02,
  "iteration_timeout_minutes": 15,
  "total_timeout_minutes": 120,
  "stall_limit": 5,
  "convergence_threshold": 0.005,
  "convergence_window": 3,
  "session_file": "experiments/sessions.jsonl",
  "beads_enabled": false
}
```

This configuration:
- Sets a 25% improvement target
- Runs up to 30 iterations
- Times out individual iterations after 15 minutes
- Stops after 2 hours total
- Detects stalls after 5 consecutive non-improving iterations
- Converges when 3 consecutive iterations improve by less than 0.5%

### Example 14: Conservative Configuration for Critical Systems

```json
{
  "metric": "error_rate",
  "measure": "./run-load-test.sh --duration 60",
  "baseline": 0.01,
  "target_improvement": 0.10,
  "max_iterations": 10,
  "max_variance": 0.01,
  "iteration_timeout_minutes": 30,
  "total_timeout_minutes": 240,
  "stall_limit": 3,
  "convergence_threshold": 0.001,
  "convergence_window": 5
}
```

Conservative settings:
- Lower improvement target (10%)
- Fewer iterations (10)
- Stricter convergence (0.1% over 5 iterations)
- Longer timeouts for stability testing

### Example 15: Aggressive Configuration for Rapid Prototyping

```json
{
  "metric": "throughput_rps",
  "measure": "./benchmark-throughput.sh",
  "baseline": 1000.0,
  "target_improvement": 0.50,
  "max_iterations": 50,
  "max_variance": 0.05,
  "iteration_timeout_minutes": 5,
  "total_timeout_minutes": 60,
  "stall_limit": 10,
  "convergence_threshold": 0.01,
  "convergence_window": 2
}
```

Aggressive settings:
- High improvement target (50%)
- Many iterations (50)
- Short timeouts for rapid feedback
- Higher variance tolerance

---

## CI/CD Integration Examples

### Example 16: GitHub Actions Workflow

Create `.github/workflows/autoresearch.yml`:

```yaml
name: AutoResearch

on:
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 2 * * *'  # Run daily at 2 AM

jobs:
  autoresearch:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      
      - name: Build pi-autoresearch
        run: |\n          git clone https://github.com/your-org/pi-autoresearch
          cd pi-autoresearch
          cargo build --release
      
      - name: Run AutoResearch
        env:\n          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |\n          ./pi-autoresearch/target/release/pi-autoresearch \
            --question "How can I improve build performance?" \
            --metric "build_time_seconds" \
            --measure "time cargo build --release" \
            --baseline 120.0 \
            --target-improvement 0.20 \
            --max-iterations 10 \
            --auto-approve \
            --skip-git
      
      - name: Upload Session File
        uses: actions/upload-artifact@v4
        with:\n          name: autoresearch-session
          path: autoresearch.jsonl
```

### Example 17: GitLab CI Integration

Add to `.gitlab-ci.yml`:

```yaml
autoresearch:
  stage: test
  image: rust:latest
  script:
    - cargo build --release
    - ./target/release/pi-autoresearch \
        --question "How can I reduce test time?" \
        --metric "test_duration_seconds" \
        --measure "time cargo test" \
        --baseline 300.0 \
        --target-improvement 0.15 \
        --max-iterations 5 \
        --auto-approve \
        --skip-git
  artifacts:
    paths:
      - autoresearch.jsonl
    expire_in: 7 days
  rules:
    - if: $CI_PIPELINE_SOURCE == "schedule"
```

### Example 18: Jenkins Pipeline

Add to `Jenkinsfile`:

```groovy
pipeline {
    agent any
    
    stages {
        stage('AutoResearch') {
            steps {
                sh '''\n                    cargo build --release
                    ./target/release/pi-autoresearch \
                        --question "How can I improve deployment time?" \
                        --metric "deployment_time_seconds" \
                        --measure "./deploy.sh --dry-run --time" \
                        --baseline 60.0 \
                        --target-improvement 0.25 \
                        --max-iterations 8 \
                        --auto-approve \
                        --skip-git
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

### Example 19: Cron Job for Periodic Optimization

Add to crontab (`crontab -e`):

```bash
# Run AutoResearch daily at 3 AM
0 3 * * * cd /path/to/project && \
  /path/to/pi-autoresearch \
    --question "How can I reduce database query time?" \
    --metric "query_time_ms" \
    --measure "./run-db-benchmark.sh" \
    --baseline 100.0 \
    --target-improvement 0.10 \
    --max-iterations 5 \
    --auto-approve \
    --skip-git \
    >> /var/log/autoresearch.log 2>&1
```

---

## Beads Workflow Examples

### Example 20: Beads-Enabled Experiment with Auto-Approve

```bash
pi-autoresearch \
  --question "How can I reduce API latency?" \
  --metric "api_latency_ms" \
  --measure "./benchmark-api.sh" \
  --beads-enabled \
  --auto-approve
```

This will:
1. Create a beads issue: "Reduce API latency by X%"
2. Update issue with iteration progress
3. Close issue with results when complete
4. Link git branch to beads issue

### Example 21: Beads Workflow with Manual Approval

```bash
# Start experiment with beads tracking
pi-autoresearch \
  --question "How can I improve cache hit rate?" \
  --metric "cache_hit_rate" \
  --measure "./run-cache-benchmark.sh" \
  --beads-enabled
```

Workflow:
1. Experiment creates beads issue
2. Agent proposes changes
3. You manually review and approve each iteration
4. Beads issue updated with your decisions
5. Final results linked to issue

### Example 22: Beads Task Management

```bash
# List available beads tasks
bd ready

# View task details
bd show <TASK_ID>

# Start experiment linked to beads task
pi-autoresearch \
  --question "How can I reduce memory fragmentation?" \
  --beads-enabled \
  --auto-approve

# Update task status
bd update <TASK_ID> --claim

# Complete task when experiment finishes
bd close <TASK_ID>
```

---

## Common Patterns and Best Practices

### Example 23: A/B Testing Pattern

```bash
# Baseline measurement
pi-autoresearch \
  --question "What is the current performance baseline?" \
  --metric "request_latency_p99" \
  --measure "./measure-latency.sh --variant A" \
  --verify-baseline \
  --max-variance 0.01

# Test variant B
pi-autoresearch \
  --question "Does variant B improve latency?" \
  --metric "request_latency_p99" \
  --measure "./measure-latency.sh --variant B" \
  --baseline <BASELINE_FROM_A> \
  --target-improvement 0.05
```

### Example 24: Multi-Metric Optimization

```bash
# Optimize for speed first
pi-autoresearch \
  --question "How can I improve execution speed?" \
  --metric "execution_time_ms" \
  --measure "./benchmark-speed.sh" \
  --baseline 500.0 \
  --target-improvement 0.30 \
  --session-file "sessions/speed.jsonl" \
  --auto-approve

# Then optimize for memory (resuming from best speed iteration)
pi-autoresearch \
  --question "How can I reduce memory without losing speed gains?" \
  --metric "memory_usage_mb" \
  --measure "./benchmark-memory.sh" \
  --baseline 200.0 \
  --target-improvement 0.20 \
  --session-file "sessions/memory.jsonl" \
  --auto-approve

# Compare results
pi-autoresearch \
  --compare-id1 <SPEED_SESSION_ID> \
  --compare-id2 <MEMORY_SESSION_ID>
```

### Example 25: Incremental Improvement Pattern

```bash
# Phase 1: Quick wins (low target, few iterations)
pi-autoresearch \
  --question "What are the quick wins for performance?" \
  --metric "throughput_rps" \
  --measure "./benchmark.sh" \
  --baseline 1000.0 \
  --target-improvement 0.10 \
  --max-iterations 5 \
  --auto-approve

# Phase 2: Deeper optimization (higher target, more iterations)
pi-autoresearch \
  --question "What deeper optimizations are possible?" \
  --metric "throughput_rps" \
  --measure "./benchmark.sh" \
  --resume <PHASE1_SESSION_ID> \
  --target-improvement 0.25 \
  --max-iterations 15 \
  --auto-approve
```

### Example 26: Regression Prevention

```bash
# Before making changes, establish baseline
pi-autoresearch \
  --question "What is the current performance?" \
  --metric "test_pass_rate" \
  --measure "./run-tests.sh --report-rate" \
  --verify-baseline \
  --max-variance 0.001 \
  --session-file "baseline.jsonl"

# Make your changes

# After changes, verify no regression
pi-autoresearch \
  --question "Did changes cause regression?" \
  --metric "test_pass_rate" \
  --measure "./run-tests.sh --report-rate" \
  --baseline <BASELINE_VALUE> \
  --target-improvement 0.0 \
  --max-iterations 1
```

### Example 27: Team Collaboration Pattern

```bash
# Team member 1: Initial experiment
pi-autoresearch \
  --question "How can we improve query performance?" \
  --metric "query_time_ms" \
  --measure "./benchmark-queries.sh" \
  --baseline 200.0 \
  --target-improvement 0.40 \
  --session-file "team/queries.jsonl" \
  --auto-approve

# Team member 2: Review and resume
pi-autoresearch --history --session-file "team/queries.jsonl"
pi-autoresearch \
  --resume <SESSION_ID> \
  --session-file "team/queries.jsonl" \
  --auto-approve
```

### Example 28: Logging and Debugging

```bash
# Quiet mode (errors only)
pi-autoresearch \
  --question "How can I improve performance?" \
  --quiet \
  --auto-approve

# Verbose mode (debug output)
pi-autoresearch \
  --question "How can I improve performance?" \
  --verbose \
  --auto-approve

# Custom log level via environment
RUST_LOG=debug pi-autoresearch \
  --question "How can I improve performance?" \
  --auto-approve

# RUST_LOG with specific modules
RUST_LOG=pi_autoresearch=trace,info pi-autoresearch \
  --question "How can I improve performance?" \
  --auto-approve
```

### Example 29: Session File Management

```bash
# View all experiments
pi-autoresearch --history

# Compare multiple experiments
pi-autoresearch \
  --compare-id1 <SESSION_ID_1> \
  --compare-id2 <SESSION_ID_2>

# Resume best experiment
pi-autoresearch \
  --resume <BEST_SESSION_ID> \
  --target-improvement 0.10

# Archive old sessions
mv autoresearch.jsonl "archive/autoresearch-$(date +%Y%m%d).jsonl"
```

### Example 30: Git Branch Management

```bash
# List all autoresearch branches
pi-autoresearch --list-branches

# Clean up old branches (older than 7 days)
pi-autoresearch --cleanup-branches --cleanup-days 7

# Clean up very old branches (older than 30 days)
pi-autoresearch --cleanup-branches --cleanup-days 30

# Manual branch review before cleanup
git branch --list "autoresearch/*" --sort=-committerdate
```

---

## Troubleshooting Examples

### Example 31: High Variance in Measurements

```bash
# Problem: Measurements vary too much
# Solution: Increase verification runs and reduce max variance

pi-autoresearch \
  --question "How can I improve performance?" \
  --verify-baseline \
  --max-variance 0.005 \
  --auto-approve
```

### Example 32: Experiment Too Slow

```bash
# Problem: Each iteration takes too long
# Solution: Reduce timeout and use faster measurement

pi-autoresearch \
  --question "How can I improve performance?" \
  --measure "./fast-benchmark.sh" \
  --iteration-timeout 5 \
  --total-timeout 30 \
  --auto-approve
```

### Example 33: No Improvement Found

```bash
# Problem: Agent can't find improvements
# Solution: Relax target improvement and increase iterations

pi-autoresearch \
  --question "How can I improve performance?" \
  --target-improvement 0.05 \
  --max-iterations 30 \
  --stall-limit 10 \
  --auto-approve
```

---

## See Also

- [USAGE.md](./USAGE.md) - Complete usage guide
- [CONFIG.md](./CONFIG.md) - Configuration file reference
- [TROUBLESHOOTING.md](./TROUBLESHOOTING.md) - Common issues and solutions
- [API.md](./API.md) - Library API documentation
- [specs/](../specs/) - Technical specifications
- [README.md](../README.md) - Project overview
