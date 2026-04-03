# CLI Interface Specification

## Overview

The `pi-autoresearch` tool provides a command-line interface for autonomous research experiments. This specification documents all CLI arguments, their behavior, and interactions.

## Basic Usage

```bash
pi-autoresearch [OPTIONS]
```

## Required Arguments

No arguments are strictly required. The tool will prompt for a research question if `--question` is not provided.

## Optional Arguments

### Experiment Parameters

| Argument | Short | Type | Default | Description |
|----------|-------|------|---------|-------------|
| `--question` | - | String | (prompt) | Research question to explore |
| `--auto-approve` | - | Bool | false | Auto-approve design without confirmation |
| `--metric` | - | String | (auto) | Metric name for measurement |
| `--measure` | - | String | (auto) | Measurement command |
| `--baseline` | - | Float | (auto) | Baseline value |
| `--target-improvement` | - | Float | 0.30 | Target improvement ratio (e.g., 0.30 for 30%) |
| `--max-iterations` | - | Integer | - | Maximum iterations (must be set to run iterations) |

### Timeout and Convergence Parameters

| Argument | Short | Type | Default | Description |
|----------|-------|------|---------|-------------|
| `--iteration-timeout-minutes` | - | Integer | 10 | Iteration timeout in minutes |
| `--total-timeout-minutes` | - | Integer | 120 | Total timeout in minutes |
| `--stall-limit` | - | Integer | 5 | Stall limit before backing off |
| `--convergence-threshold` | - | Float | 0.01 | Convergence threshold |
| `--convergence-window` | - | Integer | 3 | Convergence window size |

### Verification Parameters

| Argument | Short | Type | Default | Description |
|----------|-------|------|---------|-------------|
| `--verify-baseline` | - | Bool | false | Verify baseline measurement |
| `--max-variance` | - | Float | 0.05 | Maximum variance between baseline measurements |

### Session Management

| Argument | Short | Type | Default | Description |
|----------|-------|------|---------|-------------|
| `--session-file` | - | String | autoresearch.jsonl | Session file path |
| `--resume` | - | String | - | Resume a previous experiment by session ID |
| `--history` | - | Bool | false | List prior experiments from session file |

### Git Integration

| Argument | Short | Type | Default | Description |
|----------|-------|------|---------|-------------|
| `--skip-git` | - | Bool | false | Skip git operations (branch creation, commits, pushes) |
| `--list-branches` | - | Bool | false | List all autoresearch branches |
| `--cleanup-branches` | - | Bool | false | Clean up old autoresearch branches |
| `--cleanup-days` | - | Integer | 7 | Only remove branches older than N days |

### Integration and Configuration

| Argument | Short | Type | Default | Description |
|----------|-------|------|---------|-------------|
| `--beads-enabled` | - | Bool | false | Enable beads (bd) integration for issue tracking |
| `--config` | `-c` | String | ~/.config/pi-autoresearch/config.json | Path to config file |

### Export and Reporting

| Argument | Short | Type | Default | Description |
|----------|-------|------|---------|-------------|
| `--export` | - | Enum | - | Export format (csv, json, pdf, markdown) |
| `--export-path` | - | String | export_{session_id}_{format}.{ext} | Output path for exported results |
| `--visualize` | - | Enum | - | Visualization format (html, png, both) |
| `--visualize-path` | - | String | visualize_{session_id}_{format}.{ext} | Output path for visualization |
| `--visualize-open` | - | Bool | false | Automatically open visualization in browser |

### Notifications

| Argument | Short | Type | Default | Description |
|----------|-------|------|---------|-------------|
| `--notify-provider` | - | Enum | - | Notification provider (webhook, slack, email) |
| `--notify-url` | - | String | - | Webhook/Slack URL for notifications |
| `--notify-email` | - | String | - | Email recipient for notifications |
| `--notify-milestone` | - | Integer | - | Send notification every N iterations |

### Audit Logging

| Argument | Short | Type | Default | Description |
|----------|-------|------|---------|-------------|
| `--audit-log-path` | - | String | - | Path to audit log file |
| `--audit-log-format` | - | Enum | json | Audit log format (json, csv, text) |

### Metrics and Monitoring

| Argument | Short | Type | Default | Description |
|----------|-------|------|---------|-------------|
| `--metrics-enabled` | - | Bool | false | Enable metrics server |
| `--metrics-port` | - | Integer | 9090 | Port for metrics server |

### Output Control

| Argument | Short | Type | Default | Description |
|----------|-------|------|---------|-------------|
| `--verbose` | - | Bool | false | Verbose output with per-iteration details |
| `--quiet` | - | Bool | false | Quiet mode - only show final result |
| `--dry-run` | - | Bool | false | Show what would be done without making changes |

### Experiment Comparison

| Argument | Short | Type | Default | Description |
|----------|-------|------|---------|-------------|
| `--compare-id1` | - | String | - | First session ID for comparison |
| `--compare-id2` | - | String | - | Second session ID for comparison |

## Argument Precedence

When a value can be specified in multiple places, the following precedence applies:

1. **CLI arguments** (highest priority)
2. **Config file** values
3. **Default values** (lowest priority)

**Exception**: `--max-variance` and `--session-file` always use CLI values only (config file values are ignored for these arguments).

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success - target improvement achieved or operation completed |
| 1 | Failure - target improvement not met or error occurred |

## Special Modes

### Dry Run Mode

When `--dry-run` is specified:
- No code changes are applied
- No git branches are created
- No session files are written
- All operations are simulated and displayed

### Resume Mode

When `--resume <SESSION_ID>` is specified:
- Loads the experiment from the session file
- Continues from the best iteration's metric value
- Merges new iterations with existing session data

### Comparison Mode

When `--compare-id1` and `--compare-id2` are specified:
- Loads both experiments from the session file
- Displays side-by-side comparison
- Shows iteration details for both experiments

### History Mode

When `--history` is specified:
- Lists all experiments from the session file
- Shows summary for each experiment
- Does not run any iterations

### Branch Management Modes

- `--list-branches`: Lists all autoresearch branches with ages
- `--cleanup-branches`: Removes old autoresearch branches
- `--cleanup-days N`: Sets age threshold for cleanup (default: 7 days)

## Examples

### Basic Experiment

```bash
pi-autoresearch --question "How can we reduce memory usage?" --max-iterations 10
```

### With Custom Metric

```bash
pi-autoresearch --question "Optimize performance" --metric "execution_time_ms" --measure "hyperfine ./benchmark" --baseline 1000 --max-iterations 20
```

### Resume Previous Experiment

```bash
pi-autoresearch --resume abc123def456 --max-iterations 10
```

### Compare Two Experiments

```bash
pi-autoresearch --compare-id1 exp1 --compare-id2 exp2
```

### Dry Run

```bash
pi-autoresearch --question "Test optimization" --max-iterations 5 --dry-run
```

### With Config File

```bash
pi-autoresearch --config /path/to/config.json --question "Optimize" --max-iterations 10
```

### Clean Up Old Branches

```bash
pi-autoresearch --cleanup-branches --cleanup-days 14
```

### Export Results

```bash
pi-autoresearch --question "Optimize" --max-iterations 10 --export json --export-path results.json
```

### With Notifications

```bash
pi-autoresearch --question "Optimize" --max-iterations 10 --notify-provider webhook --notify-url "https://hooks.example.com/xxx" --notify-milestone 5
```

### With Audit Logging

```bash
pi-autoresearch --question "Optimize" --max-iterations 10 --audit-log-path audit.log --audit-log-format json
```

### With Visualization

```bash
pi-autoresearch --question "Optimize" --max-iterations 10 --visualize html --visualize-open
```

### With Metrics Server

```bash
pi-autoresearch --question "Optimize" --max-iterations 10 --metrics-enabled --metrics-port 9090
```

### Combined Features

```bash
pi-autoresearch --question "Optimize" --max-iterations 10 \
  --export json --export-path results.json \
  --visualize html --visualize-open \
  --notify-provider slack --notify-url "https://hooks.slack.com/services/xxx" \
  --audit-log-path audit.log
```

## Notes

- The `--max-iterations` flag must be explicitly set via CLI to run iterations. Config file values provide defaults but do not trigger iteration execution.
- When using `--verify-baseline`, both `--metric` and `--measure` are required (or set in config).
- Branch cleanup will not delete branches that have unmerged changes (git safety feature).
