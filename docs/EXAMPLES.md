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
