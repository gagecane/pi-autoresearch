# pi-autoresearch Configuration

This document describes the configuration file options for pi-autoresearch.

## Config File Location

The config file is located at:
- Default: `~/.config/pi-autoresearch/config.json`
- Custom: Specify with `--config <PATH>`

## Config File Format

```json
{
  "metric": "execution_time_ms",
  "measure": "cargo bench --bench my_bench",
  "baseline": 100.0,
  "target_improvement": 0.20,
  "max_iterations": 20,
  "iteration_timeout_minutes": 10,
  "total_timeout_minutes": 120,
  "stall_limit": 5,
  "convergence_threshold": 0.01,
  "convergence_window": 3,
  "max_variance": 0.05,
  "session_file": "autoresearch.jsonl",
  "beads_enabled": false
}
```

## Configuration Options

### Metric Options

| Option | Type | Required | Default | Description |
|--------|------|----------|---------|-------------|
| `metric` | string | No | auto-detected | Metric name for measurement |
| `measure` | string | No | - | Measurement command to run |
| `baseline` | number | No | auto-measured | Baseline value |
| `target_improvement` | number | No | 0.20 | Target improvement ratio (e.g., 0.20 for 20%) |

### Control Options

| Option | Type | Required | Default | Description |
|--------|------|----------|---------|-------------|
| `max_iterations` | integer | No | 20 | Maximum number of iterations |
| `iteration_timeout_minutes` | integer | No | 10 | Timeout per iteration in minutes |
| `total_timeout_minutes` | integer | No | 120 | Total timeout in minutes |
| `stall_limit` | integer | No | 5 | Number of stalled iterations before backing off |
| `convergence_threshold` | number | No | 0.01 | Threshold for convergence detection |
| `convergence_window` | integer | No | 3 | Window size for convergence detection |

### Verification Options

| Option | Type | Required | Default | Description |
|--------|------|----------|---------|-------------|
| `max_variance` | number | No | 0.05 | Maximum variance between baseline measurements (0.0 to 1.0) |

### Session Options

| Option | Type | Required | Default | Description |
|--------|------|----------|---------|-------------|
| `session_file` | string | No | autoresearch.jsonl | Path to session file |

### Integration Options

| Option | Type | Required | Default | Description |
|--------|------|----------|---------|-------------|
| `beads_enabled` | boolean | No | false | Enable beads (bd) integration for issue tracking |

## Validation Rules

The following validation rules are enforced:

- `max_variance`: Must be between 0.0 and 1.0
- `target_improvement`: Must be positive (> 0)
- `max_iterations`: Must be positive (> 0)
- `iteration_timeout_minutes`: Must be positive (> 0)
- `total_timeout_minutes`: Must be positive (> 0)
- `stall_limit`: Must be positive (> 0)
- `convergence_window`: Must be positive (> 0)
- `session_file`: Must be a valid writable path

## CLI vs Config Precedence

Command-line arguments always take precedence over config file values:

1. CLI arguments (highest priority)
2. Config file values
3. Hardcoded defaults (lowest priority)

Example:

```bash
# Config file sets max_iterations to 20
# CLI overrides it to 10
pi-autoresearch --max-iterations 10
```

## Example Config Files

### Performance Optimization

```json
{
  "metric": "execution_time_ms",
  "measure": "cargo bench --bench performance",
  "target_improvement": 0.30,
  "max_iterations": 20,
  "max_variance": 0.02
}
```

### Memory Optimization

```json
{
  "metric": "peak_memory_mb",
  "measure": "cargo run --release -- --benchmark-memory",
  "target_improvement": 0.25,
  "max_iterations": 15,
  "max_variance": 0.05
}
```

### Accuracy Optimization

```json
{
  "metric": "accuracy_percent",
  "measure": "cargo test --test accuracy_test",
  "target_improvement": 0.05,
  "max_iterations": 30,
  "convergence_threshold": 0.001
}
```

## Troubleshooting

### Config file not found

If the default config file doesn't exist, pi-autoresearch will use hardcoded defaults without error.

### Invalid config values

Invalid config values will cause pi-autoresearch to exit with a clear error message listing all validation failures.

### Session file not writable

If the session file path is not writable, pi-autoresearch will exit with an error. Use a valid path or change permissions.
