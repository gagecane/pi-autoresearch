# pi-autoresearch Usage Guide

This guide provides detailed instructions on how to use pi-autoresearch.

## Command-Line Options

### Required Options

- `--question <QUESTION>`: Research question to explore

### Metric Options

- `--metric <METRIC>`: Metric name for measurement (auto-detected if not specified)
- `--measure <COMMAND>`: Measurement command to run
- `--baseline <VALUE>`: Baseline value (auto-measured if not specified)
- `--target-improvement <RATIO>`: Target improvement ratio (e.g., 0.30 for 30%)

### Control Options

- `--max-iterations <N>`: Maximum iterations (default: 20)
- `--iteration-timeout-minutes <N>`: Timeout per iteration in minutes
- `--total-timeout-minutes <N>`: Total timeout in minutes
- `--stall-limit <N>`: Stall limit before backing off
- `--convergence-threshold <VALUE>`: Convergence threshold
- `--convergence-window <N>`: Convergence window size

### Verification Options

- `--verify-baseline`: Verify baseline measurement
- `--max-variance <VALUE>`: Maximum variance between baseline measurements (default: 0.05)

### Session Management

- `--session-file <PATH>`: Session file path (default: autoresearch.jsonl)
- `--resume <SESSION_ID>`: Resume a previous experiment
- `--history`: List prior experiments from session file
- `--compare-id1 <ID1>`: First session ID for comparison
- `--compare-id2 <ID2>`: Second session ID for comparison

### Output Options

- `--verbose`: Verbose output with per-iteration details
- `--quiet`: Quiet mode - only show final result
- `--auto-approve`: Auto-approve design without confirmation

### Git Options

- `--skip-git`: Skip git operations (branch creation, commits, pushes)
- `--list-branches`: List all autoresearch branches
- `--cleanup-branches`: Clean up old autoresearch branches
- `--cleanup-days <N>`: Only remove branches older than N days (default: 7)

### Other Options

- `--dry-run`: Dry run - show what would be done without making changes
- `--config <PATH>`: Path to config file (defaults to ~/.config/pi-autoresearch/config.json)
- `--beads-enabled`: Enable beads (bd) integration for issue tracking

## Metric Detection

pi-autoresearch auto-detects metrics based on question keywords:

- "memory" → `peak_memory_mb`
- "speed"/"performance" → `execution_time_ms`
- "accuracy" → `accuracy_percent`
- Default: `metric_value`

## Workflow

### 1. Define Research Question

```bash
pi-autoresearch --question "How can I reduce memory usage in my application?"
```

### 2. Verify Baseline (Optional)

```bash
pi-autoresearch --question "..." --verify-baseline
```

### 3. Run Experiment

```bash
pi-autoresearch --question "..." --auto-approve
```

### 4. View Results

```bash
# View experiment history
pi-autoresearch --history

# Compare two experiments
pi-autoresearch --compare-id1 <SESSION_ID1> --compare-id2 <SESSION_ID2>
```

### 5. Resume Experiment (if needed)

```bash
pi-autoresearch --resume <SESSION_ID>
```

### 6. Clean Up

```bash
# List autoresearch branches
pi-autoresearch --list-branches

# Clean up old branches
pi-autoresearch --cleanup-branches --cleanup-days 7
```

## Session File Format

Experiments are saved in JSONL format (one JSON object per line):

```json
{"session_id": "uuid-1", "question": "How to reduce memory?", "status": "completed", "baseline": 100.0, "best_improvement": 0.25}
{"session_id": "uuid-2", "question": "How to improve speed?", "status": "in_progress", "baseline": 500.0, "iterations": 3}
```

Each line is a complete JSON object containing:
- `session_id`: Unique identifier for the experiment
- `question`: Research question being explored
- `status`: `in_progress`, `completed`, or `failed`
- `baseline`: Baseline measurement value
- `best_improvement`: Best improvement achieved (if completed)
- `iterations`: Array of iteration results
- `start_time`, `end_time`: Timestamps for the experiment

## Error Handling

- Invalid config values are caught early with clear error messages
- Missing session files are handled gracefully
- Git operations can be skipped with `--skip-git`
- Dry-run mode allows previewing changes without side effects
