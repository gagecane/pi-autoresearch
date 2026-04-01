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
- Used in all 19 integration tests

### Branch Naming Convention
- Format: `autoresearch/YYYYMMDD-HHMMSS-{uuid}`
- Includes timestamp for chronological ordering
- Includes UUID for uniqueness (prevents collisions)
- Prefix allows easy identification and cleanup

## Known Limitations

1. No experiment comparison feature (Priority 12 task)

## Config File Validation

### Validation Rules
All config file values are validated when the config is loaded:
- `max_variance`: Must be between 0.0 and 1.0
- `target_improvement`: Must be positive (> 0)
- `max_iterations`: Must be positive (> 0)
- `iteration_timeout_minutes`: Must be positive (> 0)
- `total_timeout_minutes`: Must be positive (> 0)
- `stall_limit`: Must be positive (> 0)
- `convergence_window`: Must be positive (> 0)
- `session_file`: Must be a valid writable path

### Validation Design
- All errors are collected and reported together (not just first error)
- Validation happens in `load_config()` after parsing JSON
- Invalid config files cause the program to exit with a clear error message
- `is_valid_session_path()` creates a test file to verify writability, then cleans it up

## Config File Implementation Notes

### Default Value Detection Bug (FIXED)
**Bug Pattern**: When CLI has a default value (e.g., `--max-variance` defaults to `0.05`), checking `if cli.max_variance != 0.05` cannot distinguish between:
- User explicitly passing `--max-variance 0.05`
- User not passing the flag at all (clap provides default)

**Solution**: CLI arguments always take precedence. Don't try to detect if user explicitly set the default - just use the CLI value directly since clap already handles defaults.

**Implementation**:
- `get_max_variance()` now simply returns `cli.max_variance` (ignores config and default params)
- `get_session_file()` now simply returns `cli.session_file.clone()` (ignores config param)
- This ensures CLI always wins, which is the correct behavior

### Config Precedence Order
1. CLI arguments (highest priority) - **always used directly**
2. Config file values - **used when CLI doesn't specify**
3. Hardcoded defaults - **fallback when neither CLI nor config specify**

### Helper Functions
All config value retrieval uses helper functions:
- `get_metric()`, `get_measure()`, `get_baseline()`
- `get_target_improvement()`, `get_max_iterations()`
- `get_iteration_timeout()`, `get_total_timeout()`
- `get_stall_limit()`, `get_convergence_threshold()`, `get_convergence_window()`
- `get_max_variance()`, `get_session_file()`, `get_beads_enabled()`

### Error Handling
- When `--config PATH` is explicitly provided and file doesn't exist: **return error**
- When default config path doesn't exist: **silently use defaults** (no error)

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

