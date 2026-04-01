# Memories

Important learnings and context about the pi-autoresearch project.

## Project Overview

**pi-autoresearch** is an autonomous research experiment orchestrator that helps optimize metrics through iterative experimentation. It's designed to work with AI coding agents to automate the process of:

1. Defining research questions and metrics
2. Running experiments to measure improvements
3. Tracking results in session files
4. Creating git branches for successful experiments

## Architecture

### Core Components
- **CLI Interface**: Accepts commands via command-line arguments
- **Session Management**: Tracks experiments in `autoresearch.jsonl` files
- **Git Integration**: Creates branches for successful experiments
- **Beads Integration**: Optional issue tracking via `bd` (beads)

### Key Functions
- `finalize_experiment()`: Processes experiment results and creates git branches
- `get_git_commit_hash()`: Retrieves current commit hash for baseline tracking
- `generate_failure_recommendations()`: Provides suggestions when experiments don't meet targets

## Testing Strategy

### Integration Tests
- 19 total tests in `tests/integration_tests.rs`
- Tests use `--skip-git` flag to avoid git branch conflicts
- All tests should pass consistently when run in parallel

### Test Parallelization
- **Issue**: Branch names using timestamp format could collide in parallel tests
- **Solution**: Added UUID to branch names + `--skip-git` flag for tests
- **Result**: All 19 tests pass consistently across multiple parallel runs

## Design Decisions

### Why `--skip-git` Flag?
- Allows testing without git repository
- Prevents branch name collisions in parallel test runs
- Enables dry-run mode for future implementation
- Maintains separation of concerns (testing vs. production)

### Branch Naming Convention
- Format: `autoresearch/YYYYMMDD-HHMMSS-{uuid}`
- Includes timestamp for chronological ordering
- Includes UUID for uniqueness (prevents collisions)
- Prefix allows easy identification and cleanup

## Known Limitations

1. No config file support (Priority 2 task)
2. No branch cleanup functionality (Priority 3 task)
3. No dry-run mode for previewing changes (Priority 4 task)

## Integration Points

### Beads (bd)
- Issue tracking system
- Commands: `bd ready`, `bd show`, `bd update`, `bd close`
- Session file: `.beads/`

### Git
- Creates branches for successful experiments
- Commits with detailed metadata (metric, improvement, iterations)
- Pushes to remote if available

## Development Notes

### Session File Format
- JSONL (JSON Lines) format
- Each line is a complete JSON object
- Contains: design, baseline_record, iterations, status

### Metric Detection
- Auto-detects metric based on question keywords:
  - "memory" → `peak_memory_mb`
  - "speed/performance" → `execution_time_ms`
  - "accuracy" → `accuracy_percent`
  - Default: `metric_value`

### Baseline Verification
- Runs measurement command twice
- Checks variance between runs
- Default max variance: 5%
- Records git commit hash for reproducibility

