# Session File Format Specification

## Overview

The session file stores experiment data in JSON Lines (JSONL) format, where each line is a valid JSON object. This specification documents the structure of all record types.

## File Format

- **Format**: JSON Lines (JSONL)
- **Default Location**: `autoresearch.jsonl`
- **Configurable**: Yes, via `--session-file` CLI argument
- **Encoding**: UTF-8

## Record Types

The session file contains three types of records:

1. **BaselineRecord** - Baseline measurement data
2. **IterationRecord** - Individual iteration data
3. **ExperimentSession** - Complete experiment summary

### Record Identification

Records are identified by their fields:
- `ExperimentSession`: Contains `session_id` field
- `IterationRecord`: Contains `iteration` and `agent_action` fields
- `BaselineRecord`: Contains `verification_runs` field

---

## BaselineRecord

Records the initial baseline measurement and verification.

### Schema

```json
{
  "timestamp": "RFC3339 timestamp",
  "git_commit": "string (git commit hash)",
  "metric": "string (metric name)",
  "measurement_command": "string (command used)",
  "value": "float (baseline value)",
  "verification_runs": ["array of floats (measurement values)"],
  "variance": "float (variance between runs)",
  "within_threshold": "boolean (variance within acceptable range)"
}
```

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `timestamp` | String | Yes | RFC3339 formatted timestamp of measurement |
| `git_commit` | String | Yes | Git commit hash at time of measurement |
| `metric` | String | Yes | Name of the metric being measured |
| `measurement_command` | String | Yes | Shell command used for measurement |
| `value` | Float | Yes | Primary baseline value (first run) |
| `verification_runs` | Array<Float> | Yes | All measurement run values (minimum 2) |
| `variance` | Float | Yes | Calculated variance: `abs(run2 - run1) / run1` |
| `within_threshold` | Boolean | Yes | True if variance <= max_variance threshold |

### Example

```json
{
  "timestamp": "2024-01-15T10:30:00+00:00",
  "git_commit": "a1b2c3d4e5f6",
  "metric": "execution_time_ms",
  "measurement_command": "hyperfine ./benchmark",
  "value": 1000.0,
  "verification_runs": [1000.0, 1002.5],
  "variance": 0.0025,
  "within_threshold": true
}
```

---

## IterationRecord

Records data from a single iteration of the experiment.

### Schema

```json
{
  "iteration": "integer (iteration number, 1-indexed)",
  "timestamp": "RFC3339 timestamp",
  "agent_action": "string (description of agent action)",
  "metric_value": "float (measured value)",
  "improvement": "float (improvement ratio)",
  "kept": "boolean (whether changes were kept)"
}
```

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `iteration` | Integer | Yes | Iteration number (1-indexed) |
| `timestamp` | String | Yes | RFC3339 formatted timestamp |
| `agent_action` | String | Yes | Description of the agent's proposed change |
| `metric_value` | Float | Yes | Measured metric value after change |
| `improvement` | Float | Yes | Improvement ratio: `(best_metric - metric_value) / baseline` |
| `kept` | Boolean | Yes | True if metric improved and changes were kept |

### Example

```json
{
  "iteration": 1,
  "timestamp": "2024-01-15T10:35:00+00:00",
  "agent_action": "Proposed change for 'Optimize performance': Implemented cache optimization",
  "metric_value": 950.0,
  "improvement": 0.05,
  "kept": true
}
```

---

## ExperimentSession

Records the complete experiment summary with all iterations.

### Schema

```json
{
  "session_id": "string (unique identifier)",
  "question": "string (research question)",
  "design": "ExperimentDesign object",
  "baseline_record": "BaselineRecord object",
  "iterations": "array of IterationRecord objects",
  "best_iteration": "integer or null (best iteration number)",
  "start_time": "RFC3339 timestamp",
  "end_time": "RFC3339 timestamp or null",
  "status": "string (completed, no_improvement, etc.)"
}
```

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `session_id` | String | Yes | Unique identifier for this experiment session |
| `question` | String | Yes | Research question being explored |
| `design` | Object | Yes | ExperimentDesign object (see below) |
| `baseline_record` | Object | Yes | BaselineRecord object |
| `iterations` | Array | Yes | All IterationRecord objects from this experiment |
| `best_iteration` | Integer\|null | Yes | Iteration number with best metric, or null if none |
| `start_time` | String | Yes | RFC3339 timestamp when experiment started |
| `end_time` | String\|null | Yes | RFC3339 timestamp when experiment ended |
| `status` | String | Yes | "completed" if target met, "no_improvement" otherwise |

### ExperimentDesign Object

Nested within `ExperimentSession`:

```json
{
  "hypothesis": "string",
  "metric": "string",
  "measurement": "string",
  "baseline": "float",
  "target_improvement": "float"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `hypothesis` | String | Yes | Generated hypothesis based on question |
| `metric` | String | Yes | Metric name |
| `measurement` | String | Yes | Measurement command |
| `baseline` | Float | Yes | Baseline value |
| `target_improvement` | Float | Yes | Target improvement ratio (e.g., 0.30 for 30%) |

### Example

```json
{
  "session_id": "abc123def456",
  "question": "How can we reduce memory usage?",
  "design": {
    "hypothesis": "Optimizing based on: How can we reduce memory usage?",
    "metric": "peak_memory_mb",
    "measurement": "Run benchmark suite, capture peak RSS",
    "baseline": 512.0,
    "target_improvement": 0.30
  },
  "baseline_record": {
    "timestamp": "2024-01-15T10:30:00+00:00",
    "git_commit": "a1b2c3d4e5f6",
    "metric": "peak_memory_mb",
    "measurement_command": "Run benchmark suite, capture peak RSS",
    "value": 512.0,
    "verification_runs": [512.0, 515.0],
    "variance": 0.0059,
    "within_threshold": true
  },
  "iterations": [
    {
      "iteration": 1,
      "timestamp": "2024-01-15T10:35:00+00:00",
      "agent_action": "Implemented memory pooling",
      "metric_value": 480.0,
      "improvement": 0.0625,
      "kept": true
    }
  ],
  "best_iteration": 1,
  "start_time": "2024-01-15T10:30:00+00:00",
  "end_time": "2024-01-15T10:45:00+00:00",
  "status": "completed"
}
```

---

## File Operations

### Reading

- File is read line-by-line
- Empty lines are skipped
- Each line is parsed as JSON
- Records are identified by their fields
- If parsing fails for a line, it is skipped

### Writing

- New records are appended to the file
- Each record is written as a single line
- Pretty-printed JSON is used for ExperimentSession records
- Compact JSON is used for BaselineRecord and IterationRecord

### Session File Location

1. CLI `--session-file` argument (always takes precedence)
2. Default: `autoresearch.jsonl` in current directory

**Note**: The config file `session_file` setting is NOT used (CLI always wins).

---

## Status Values

| Status | Description |
|--------|-------------|
| `completed` | Target improvement was achieved |
| `no_improvement` | No improvement was achieved over baseline |

---

## Notes

- The session file is append-only; records are never modified or deleted
- Each experiment creates one BaselineRecord, multiple IterationRecords, and one ExperimentSession record
- Session IDs are generated using a hash-based UUID generator
- Timestamps use RFC3339 format for consistency and parsing
