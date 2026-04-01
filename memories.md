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
- 31 total tests in `tests/integration_tests.rs`
- Tests use `--skip-git` flag to avoid git branch conflicts
- All tests should pass consistently when run in parallel

### Unit Tests
- 60 unit tests in `src/main.rs`
- Tests cover:
  - Config helper functions (13 functions)
  - Validation functions (8 rules)
  - Git functions (6 functions)
  - Agent functions (3 functions)
  - Utility functions (15 functions)

### Total Test Coverage
- 91 total tests (60 unit + 31 integration)
- All tests pass consistently

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
- UUID ensures uniqueness in parallel test runs

### Documentation Structure
- `docs/` folder contains user-facing documentation:
  - README.md: Overview and quick start
  - USAGE.md: Detailed usage guide
  - CONFIG.md: Configuration documentation
  - EXAMPLES.md: Example use cases
- `specs/` folder contains technical specifications:
  - CLI.md: CLI interface specification
  - SESSION.md: Session file format specification
  - CONFIG.md: Configuration file specification
  - WORKFLOW.md: Experiment workflow specification

### Code Quality
- All helper functions properly implemented with single responsibility
- Config validation is non-destructive (does not create directories or files)
- All 5 termination conditions properly documented and implemented
- Minor issues found:
  - Unused variable warning at line 2998 in test code
  - 7 clippy warnings identified for future improvement
  - README.md needs expansion (currently only 1 line)
- Includes UUID for uniqueness (prevents collisions)
- Prefix allows easy identification and cleanup

## Known Limitations

1. No experiment comparison feature (Priority 12 task)

## Code Review Findings

### Destructive Validation Issue (FIXED)
- **Issue**: `is_valid_session_path()` creates parent directories and truncates/deletes files during validation
- **Impact**: Validation had side effects and was destructive
- **Fix Applied**: 
  - No longer creates parent directories during validation
  - No longer truncates existing files at the actual path
  - Creates temp file in parent directory only (`.validation_temp`)
  - Handles empty parent path (just filename) by treating as current directory
- **Design Principle**: Validation should be non-destructive - check writeability without side effects

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
- `is_valid_session_path()` creates a test file in parent directory to verify writability, then cleans it up
- Validation is non-destructive (does not create directories or modify existing files)

### ConfigValidationError Enum
- Provides specific error types for each validation rule
- Clear, actionable error messages for users
- Implements Display and Error traits for proper error handling

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

## Documentation Review Findings

### Documentation Structure
- `docs/README.md` - Overview and quick start guide
- `docs/USAGE.md` - Detailed usage guide with all CLI options
- `docs/CONFIG.md` - Configuration file documentation with tables
- `docs/EXAMPLES.md` - 12 example use cases

### Documentation Quality Issues
1. **Installation Instructions**: Package not on crates.io, needs build-from-source instructions
2. **Build Instructions**: Missing development/build instructions
3. **Session File Format**: JSONL format needs clearer multi-line example
4. **Cross-References**: Missing link to main README.md
5. **Section Organization**: Metric detection section should be more prominent

### Documentation Best Practices
- Installation instructions should match actual distribution method
- Build instructions essential for unpublished packages
- Examples should show actual file formats clearly
- Documentation should link to related documents
- Important features (like auto-detection) should be prominently placed

### Documentation Fixes Applied (2026-04-01)
- Installation instructions updated to build from source (not crates.io)
- Development section added with build, test, and run instructions
- Link to main README.md added at top of docs/README.md
- Session file format clarified with multi-line JSONL example
- Metric Detection section moved higher for better visibility


### Documentation Final Review (2026-04-01)
- All 4 documentation files verified complete and accurate
- docs/README.md: Correct installation (build from source), development instructions, links to main README
- docs/USAGE.md: Comprehensive CLI options, prominent Metric Detection section, clear JSONL format example
- docs/CONFIG.md: Complete config options table, validation rules, CLI vs config precedence explained
- docs/EXAMPLES.md: 12 detailed examples covering all features and use cases
- Documentation aligns perfectly with current implementation
- All feedback from previous review successfully addressed

## Documentation and Specifications

### Spec Files Created (Priority 17)

Created comprehensive specifications in specs/ folder aligned with implementation:

**specs/CLI.md**:
- Documents all 25+ CLI arguments organized by category
- Clarifies argument precedence: CLI > config > default
- Notes exceptions: max_variance and session_file are CLI-only
- Documents special modes: dry-run, resume, comparison, history, branch management
- Includes 12 practical usage examples
- Documents exit codes (0=success, 1=failure)

**specs/SESSION.md**:
- Documents JSONL format with 3 record types
- BaselineRecord: 8 fields, verification_runs array, variance calculation
- IterationRecord: 6 fields, improvement ratio formula
- ExperimentSession: 9 fields, nested ExperimentDesign object
- Provides complete JSON examples for each record type
- Documents file operations: append-only, line-by-line parsing
- Notes session_file is CLI-only (config ignored)

**specs/CONFIG.md**:
- Documents 13 optional config fields
- Validation rules for 8 numeric fields with specific ranges
- Error messages for all validation failures
- Multiple errors reported together
- Non-destructive validation (doesn't create directories)
- Provides 3 example configs: minimal, comprehensive, project-specific

**specs/WORKFLOW.md**:
- Documents 5-stage workflow: Initialization → Design → Baseline → Iteration → Finalization
- Includes ASCII workflow diagram
- Documents 5 termination conditions with StuckReason enum
- Convergence detection: variance of recent metrics < threshold
- Stall detection: consecutive no-improvement count with backoff
- Commit message format with metadata
- Failure report structure with recommendations

### Spec Writing Principles

1. **Align with Implementation**: Specs reflect actual code behavior, not ideal behavior
2. **Include Examples**: Every record type has a complete JSON example
3. **Document Edge Cases**: Validation errors, special modes, failure conditions
4. **Clarify Precedence**: Explicitly state when CLI overrides config
5. **Be Complete**: All fields, all modes, all error conditions documented

## Task Decomposition Strategy

### When to Decompose Tasks
- When a task is too abstract (e.g., "improve code quality")
- When a task has multiple distinct subtasks
- When subtasks can be completed independently
- When each subtask is small enough to complete in one session

### Decomposition Example: Priority 18 → 19-21
- **Original Task**: RESEARCH - Code Quality Improvements (too abstract)
- **Decomposed Into**:
  - Priority 19: CODE QUALITY - Fix Clippy Warnings (specific, actionable)
  - Priority 20: CODE QUALITY - Fix Unused Variable Warning (specific, actionable)
  - Priority 21: DOCS - Improve README.md (specific, actionable)

### Benefits of Decomposition
- Each task is focused and achievable
- Progress can be tracked more granularly
- Tasks can be reviewed and completed independently
- Reduces risk of incomplete work
