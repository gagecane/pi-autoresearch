# Configuration File Specification

## Overview

The configuration file allows users to set default values for experiment parameters. This specification documents the config file format, location, and validation rules.

## File Location

### Default Location

```
~/.config/pi-autoresearch/config.json
```

### Custom Location

Specify a custom config file path using the `--config PATH` or `-c PATH` CLI argument.

### Behavior

- If the default config file exists, it is loaded automatically
- If `--config PATH` is specified, that file must exist (error if not found)
- If no config file exists and `--config` is not specified, defaults are used

## File Format

- **Format**: JSON
- **Encoding**: UTF-8
- **Validation**: All values are validated on load

## Schema

```json
{
  "metric": "string (optional)",
  "measure": "string (optional)",
  "baseline": "float (optional)",
  "target_improvement": "float (optional)",
  "max_iterations": "integer (optional)",
  "iteration_timeout_minutes": "integer (optional)",
  "total_timeout_minutes": "integer (optional)",
  "stall_limit": "integer (optional)",
  "convergence_threshold": "float (optional)",
  "convergence_window": "integer (optional)",
  "max_variance": "float (optional)",
  "session_file": "string (optional)",
  "beads_enabled": "boolean (optional)"
}
```

## Fields

### Experiment Parameters

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `metric` | String | No | (auto) | Default metric name |
| `measure` | String | No | (auto) | Default measurement command |
| `baseline` | Float | No | (auto) | Default baseline value |
| `target_improvement` | Float | No | 0.30 | Default target improvement ratio |
| `max_iterations` | Integer | No | - | Default maximum iterations |

### Timeout and Convergence Parameters

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `iteration_timeout_minutes` | Integer | No | 10 | Default iteration timeout |
| `total_timeout_minutes` | Integer | No | 120 | Default total timeout |
| `stall_limit` | Integer | No | 5 | Default stall limit |
| `convergence_threshold` | Float | No | 0.01 | Default convergence threshold |
| `convergence_window` | Integer | No | 3 | Default convergence window |

### Verification Parameters

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `max_variance` | Float | No | 0.05 | Default max variance (0.0 to 1.0) |

### Session and Integration

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `session_file` | String | No | autoresearch.jsonl | Default session file path |
| `beads_enabled` | Boolean | No | false | Enable beads integration by default |

## Validation Rules

All config values are validated on load. Invalid values cause the tool to exit with an error.

### max_variance

- **Valid Range**: 0.0 to 1.0 (inclusive)
- **Error**: "max_variance must be between 0.0 and 1.0, got {value}"

### target_improvement

- **Valid Range**: Must be positive (> 0.0)
- **Error**: "target_improvement must be positive, got {value}"

### max_iterations

- **Valid Range**: Must be positive (> 0)
- **Error**: "max_iterations must be positive, got {value}"

### iteration_timeout_minutes

- **Valid Range**: Must be positive (> 0)
- **Error**: "iteration_timeout_minutes must be positive, got {value}"

### total_timeout_minutes

- **Valid Range**: Must be positive (> 0)
- **Error**: "total_timeout_minutes must be positive, got {value}"

### stall_limit

- **Valid Range**: Must be positive (> 0)
- **Error**: "stall_limit must be positive, got {value}"

### convergence_window

- **Valid Range**: Must be positive (> 0)
- **Error**: "convergence_window must be positive, got {value}"

### session_file

- **Valid Range**: Must be a valid, writable path
- **Validation**: Parent directory must exist and be writable
- **Error**: "session_file path is not valid or writable: {path}"

## Value Precedence

When a value can be specified in multiple places:

1. **CLI arguments** (highest priority)
2. **Config file** values
3. **Hardcoded defaults** (lowest priority)

**Exceptions**:
- `max_variance`: CLI only (config value is ignored)
- `session_file`: CLI only (config value is ignored)

## Example Configuration Files

### Minimal Config

```json
{
  "metric": "execution_time_ms",
  "measure": "hyperfine ./benchmark"
}
```

### Comprehensive Config

```json
{
  "metric": "peak_memory_mb",
  "measure": "./memory-benchmark",
  "baseline": 512.0,
  "target_improvement": 0.25,
  "max_iterations": 15,
  "iteration_timeout_minutes": 15,
  "total_timeout_minutes": 180,
  "stall_limit": 5,
  "convergence_threshold": 0.01,
  "convergence_window": 3,
  "max_variance": 0.05,
  "session_file": "experiments.jsonl",
  "beads_enabled": true
}
```

### Project-Specific Config

Store project-specific configs in the project directory:

```json
{
  "metric": "test_execution_time_ms",
  "measure": "cargo test --release -- --test-threads=1",
  "baseline": 5000.0,
  "target_improvement": 0.20
}
```

Usage:
```bash
pi-autoresearch --config ./project-config.json --question "Optimize tests" --max-iterations 10
```

## Error Handling

### Config File Not Found

- **Default path**: Silently skipped, defaults used
- **Explicit `--config PATH`**: Error and exit

### Invalid JSON

```
Failed to parse config file /path/to/config.json: [JSON parse error details]
```

### Validation Errors

Multiple validation errors are reported together:

```
Invalid config file /path/to/config.json:
  max_variance must be between 0.0 and 1.0, got 1.50
  target_improvement must be positive, got -0.10
  max_iterations must be positive, got 0
```

## Notes

- All fields are optional; omitting a field uses the default value
- Config file validation is non-destructive (does not create directories or files)
- The config file does not trigger iteration execution; `--max-iterations` must be set via CLI
- Config values are merged with CLI values according to precedence rules
