# Experiment Workflow Specification

## Overview

This specification documents the complete workflow of an autoresearch experiment, from initialization to finalization.

## Workflow Stages

The experiment workflow consists of the following stages:

1. **Initialization**
2. **Design Generation**
3. **Baseline Verification**
4. **Iterative Exploration**
5. **Finalization**

---

## Stage 1: Initialization

### Purpose

Set up the experiment environment and load configuration.

### Steps

1. Parse CLI arguments
2. Load config file (if exists)
3. Apply value precedence (CLI > config > default)
4. Check for special modes (dry-run, history, comparison, cleanup)
5. Initialize beads integration (if enabled)

### Special Modes (Exit Early)

- `--list-branches`: List branches and exit
- `--cleanup-branches`: Clean branches and exit
- `--history`: Show history and exit
- `--compare-id1 --compare-id2`: Compare experiments and exit
- `--resume <ID>`: Resume from saved session

### Outputs

- Parsed CLI configuration
- Loaded config file values
- Merged effective configuration

---

## Stage 2: Design Generation

### Purpose

Generate an experiment design based on the research question.

### Steps

1. Get or prompt for research question
2. Auto-detect metric type from question keywords:
   - "memory" → `peak_memory_mb`
   - "speed"/"performance" → `execution_time_ms`
   - "accuracy" → `accuracy_percent`
   - Default → `metric_value`
3. Generate hypothesis
4. Set default measurement command
5. Set default baseline value
6. Apply config/CLI overrides for metric, measure, baseline

### Design Object

```json
{
  "hypothesis": "Optimizing based on: <question>",
  "metric": "<metric name>",
  "measurement": "<measurement command>",
  "baseline": <baseline value>,
  "target_improvement": 0.30
}
```

### Outputs

- ExperimentDesign object
- JSON output to stdout
- Bead issue created (if beads enabled)

### User Confirmation

Unless `--auto-approve` is set, prompt user:
```
Approve this design? [y/N]:
```

---

## Stage 3: Baseline Verification

### Purpose

Establish a reliable baseline measurement before iterations.

### Steps

1. Get effective metric, measure, and max_variance values
2. Execute measurement command twice
3. Calculate variance: `abs(run2 - run1) / run1`
4. Check if variance <= max_variance threshold
5. Record baseline with git commit hash
6. Save BaselineRecord to session file

### Verification Result

```json
{
  "success": true/false,
  "baseline_record": {
    "timestamp": "<RFC3339>",
    "git_commit": "<hash>",
    "metric": "<name>",
    "measurement_command": "<command>",
    "value": <float>,
    "verification_runs": [<float>, <float>],
    "variance": <float>,
    "within_threshold": true/false
  },
  "error_message": "<optional error>"
}
```

### Failure Conditions

- Measurement command fails
- Variance exceeds threshold
- Insufficient successful runs

### Outputs

- Verified BaselineRecord
- BaselineRecord saved to session file (unless dry-run)

---

## Stage 4: Iterative Exploration

### Purpose

Run multiple iterations to find improvements.

### Precondition

`--max-iterations` must be set via CLI.

### Iteration Loop

For each iteration (1 to max_iterations):

1. **Timeout Check**: Check total timeout, break if exceeded
2. **Agent Invocation**: Call pi agent with current state
3. **Branch Creation**: Create git branch for changes
4. **Apply Changes**: Apply agent's proposed changes
5. **Measure**: Execute measurement command
6. **Calculate Improvement**: `(best_metric - metric_value) / baseline`
7. **Keep or Revert**:
   - If improvement > 0: Keep changes, update best_metric
   - If improvement <= 0: Revert changes
8. **Log Iteration**: Save IterationRecord to session file
9. **Update Bead**: Update bead progress (if enabled)
10. **Convergence Check**: Check if recent metrics converged
11. **Stall Check**: Check if stall limit reached

### Termination Conditions

The loop terminates when any of these conditions is met:

| Condition | Reason |
|-----------|--------|
| max_iterations reached | `MaxIterationsReached` |
| total timeout exceeded | `TotalTimeout` |
| iteration timeout exceeded | `IterationTimeout` |
| stall limit reached (2 backoffs) | `StallLimitReached` |
| convergence achieved | `ConvergenceAchieved` |

### Convergence Detection

- Track last N metrics (convergence_window)
- Calculate variance: `(max - min) / min`
- If variance < convergence_threshold: Converged

### Stall Detection

- Count consecutive iterations without improvement
- If count >= stall_limit: Back off (reset counter)
- If backoffs >= 2: Abort

### Outputs

- Vector of IterationRecord objects
- Best iteration number
- Stuck reason (termination cause)

---

## Stage 5: Finalization

### Purpose

Evaluate results and create git branch if successful.

### Steps

1. Calculate final improvement from baseline to best kept value
2. Compare against target improvement
3. If successful:
   - Generate branch name: `autoresearch/YYYYMMDD-HHMMSS-{uuid}`
   - Generate commit message with metadata
   - Create branch, stage changes, commit, push
   - Save ExperimentSession to session file
4. If unsuccessful:
   - Generate failure report with recommendations
   - No changes applied to codebase

### Success Criteria

```
final_improvement >= target_improvement
```

Where:
```
final_improvement = (baseline - best_kept_value) / baseline
```

### Commit Message Format

```
[autoresearch] Reduce <metric> by <X>% (<baseline> → <best>)

Iterations: <N>/<M> | Runtime: <S>s | Convergence: <status>

Key changes:
Iteration <N>: <change summary> (-<X>%)
...

Metric: <name> | Baseline: <value> | Best: <value>
```

### Failure Report

```json
{
  "best_improvement": <float>,
  "best_value": <float or null>,
  "baseline": <float>,
  "target_improvement": <float>,
  "iterations_completed": <int>,
  "stuck_reason": "<reason string>",
  "recommendations": ["<string>", ...]
}
```

### Outputs

- Success: Git branch created, changes committed
- Failure: Recommendations for retry
- ExperimentSession saved to session file (unless dry-run)

---

## Complete Workflow Diagram

```
┌─────────────────┐
│  Initialization │
└────────┬────────┘
         │
         v
┌─────────────────┐     Special modes
│  Check modes    │────► Exit early
└────────┬────────┘
         │
         v
┌─────────────────┐
│ Design          │
│ Generation      │
└────────┬────────┘
         │
         v
┌─────────────────┐
│ User Approval   │
└────────┬────────┘
         │
         v
┌─────────────────┐
│ Baseline        │
│ Verification    │
└────────┬────────┘
         │
         v
┌─────────────────┐
│ Iterative       │
│ Exploration     │◄────┐
│ (loop)          │     │
└────────┬────────┘     │
         │              │
         v              │
┌─────────────────┐     │
│ Termination     │─────┘
│ check           │
└────────┬────────┘
         │
         v
┌─────────────────┐
│ Finalization    │
└────────┬────────┘
         │
         ├──────────┬──────────┐
         v          v          v
    Success    Failure   Dry-run
    (branch)   (report)  (none)
```

---

## State Management

### Session File Updates

1. **BaselineRecord**: Written after baseline verification
2. **IterationRecord**: Written after each iteration
3. **ExperimentSession**: Written after finalization

### Git Branch Lifecycle

1. Created per iteration (temporary)
2. Merged if changes kept
3. Final branch created on success
4. Cleanup via `--cleanup-branches`

---

## Notes

- Dry-run mode skips all side effects (git, session file, code changes)
- Beads integration is optional and non-blocking (warnings on failure)
- All timestamps use RFC3339 format
- Session IDs and branch names include UUIDs for uniqueness
