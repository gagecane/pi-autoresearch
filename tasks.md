# Tasks

## Priority 1: FIX - Test Parallelization Issue
**Status**: COMPLETE ✅
**Description**: Fix flaky integration tests that fail when run in parallel due to git branch name collisions
**Root Cause**: Branch names use timestamp format `autoresearch/YYYYMMDD-HHMMSS` which can collide when multiple tests run within the same second
**Solution**: 
1. Added unique identifier to branch names: `autoresearch/YYYYMMDD-HHMMSS-{uuid}`
2. Added `--skip-git` flag to skip git operations during testing
3. Updated all integration tests to use `--skip-git`
**Test**: `cargo test` passes consistently (19/19 tests) across multiple runs
**Review**: Implementation verified correct - all git operations properly guarded, tests pass consistently

## Priority 2: FEATURE - Config File Support
**Status**: COMPLETE ✅
**Description**: Implement User Story 4.2 from PRD - read defaults from `~/.config/pi-autoresearch/config.json`
**Acceptance Criteria**:
- ✅ Read defaults from config file
- ✅ CLI args override config file
- ✅ Support `--config PATH` for project-specific configs
**Changes Made**:
- Fixed default-value detection bugs in `get_max_variance()` and `get_session_file()` - CLI now always takes precedence
- Added missing helper functions: `get_iteration_timeout()`, `get_total_timeout()`, `get_stall_limit()`, `get_convergence_threshold()`, `get_convergence_window()`
- Added error handling for missing explicit config files (--config PATH errors when file doesn't exist)
- Updated `run_iterative_loop()` to use helper functions consistently
- Fixed inconsistent use of helper functions in `verify_baseline` section
- Updated tests to use temporary HOME directory for tests that require no config file
- Fixed two remaining code paths that bypassed config: `target_improvement` and `session_file` in finalization section
- All 19 integration tests pass consistently across 10+ runs
**Review**: Implementation verified correct - all helper functions properly implemented, config precedence correct (CLI > config > default), all code paths use helper functions consistently

## Priority 3: FEATURE - Branch Cleanup
**Status**: COMPLETE ✅
**Description**: Add functionality to clean up autoresearch branches after completion
**Rationale**: Prevents accumulation of autoresearch branches in git repository
**Implementation**:
- Added `--list-branches` flag to list all autoresearch branches with their ages
- Added `--cleanup-branches` flag to remove old autoresearch branches
- Added `--cleanup-days N` option (default: 7) to only remove branches older than N days
- Filters out remote tracking branches (only cleans local branches)
- Gracefully handles unmerged branches (git safety feature)
- All 19 integration tests still pass
**Review**: Implementation verified correct - branch listing and cleanup work as expected, handles edge cases properly

## Priority 4: FEATURE - Dry Run Mode
**Status**: COMPLETE ✅
**Description**: Add `--dry-run` flag to simulate experiments without applying changes
**Rationale**: Allows users to preview what changes would be made
**Implementation**:
- Added `--dry-run` flag to CLI
- Shows "DRY RUN MODE" banner at start
- Skips actual code changes (keep/revert)
- Skips git branch creation
- Skips session file writes
- All 19 integration tests still pass
**Review**: Implementation verified correct - dry-run mode properly skips all side effects

## Priority 5: BUG FIX - Session File Config Precedence
**Status**: COMPLETE ✅
**Description**: Fix bugs where `cli.session_file` is used directly instead of `get_session_file(&cli, &config)`
**Root Cause**: Two code paths bypass config file support:
1. Line 748 in `run_iterative_loop()`: Used `&cli.session_file` instead of `get_session_file(&cli, &config)`
2. Line 1751 in `--resume` section: Used `&cli.session_file` instead of `get_session_file(&cli, &config)`
**Impact**: Config file `session_file` setting was ignored in these code paths
**Fix Applied**: 
- Line 748: Now uses `get_session_file(cli, config)` before calling `run_iteration()`
- Line 1751: Now uses `get_session_file(&cli, &config)` before calling `save_to_session_file()`
**Test**: All 19 integration tests pass, code compiles without warnings
**Review**: Implementation verified correct - both code paths now use helper function consistently, maintaining config precedence (CLI > config > default)

## Priority 6: TEST - Add Unit Tests for Helper Functions
**Status**: COMPLETE ✅
**Description**: Add unit tests for all 13 config helper functions
**Rationale**: Currently only integration tests exist; helper functions lack dedicated unit tests
**Functions Tested**:
- `get_metric()` - 4 tests (CLI, config, default, CLI overrides config)
- `get_measure()` - 2 tests (CLI, config)
- `get_baseline()` - 2 tests (CLI, config)
- `get_target_improvement()` - 1 test (CLI)
- `get_max_iterations()` - 1 test (CLI)
- `get_max_variance()` - 1 test (CLI always wins)
- `get_session_file()` - 1 test (CLI always wins)
- `get_beads_enabled()` - 3 tests (CLI true, config true, default false)
- `get_iteration_timeout()` - 1 test (CLI)
- `get_total_timeout()` - 1 test (CLI)
- `get_stall_limit()` - 1 test (CLI)
- `get_convergence_threshold()` - 1 test (CLI)
- `get_convergence_window()` - 1 test (CLI)
- Plus 1 comprehensive test for all functions together
**Test Scenarios**:
- CLI value provided (should use CLI)
- Config value provided, no CLI (should use config)
- Neither CLI nor config (should use default)
- CLI overrides config
**Result**: 21 unit tests added, all pass + 19 integration tests still pass

## Priority 7: TEST - Add Tests for New Features
**Status**: COMPLETE ✅
**Description**: Add integration tests for branch cleanup and dry-run mode
**Rationale**: Priority 3 and 4 features were implemented without dedicated tests
**Tests Added**:
- `test_dry_run_shows_banner`: Verifies "DRY RUN MODE" banner is displayed
- `test_dry_run_no_session_file_created`: Verifies session file not created in dry-run
- `test_dry_run_with_iterations`: Verifies dry-run completes without side effects
- `test_list_branches_flag`: Verifies `--list-branches` works correctly
- `test_cleanup_branches_flag`: Verifies `--cleanup-branches` shows summary
- `test_cleanup_branches_with_custom_days`: Verifies custom cleanup days works
**Result**: 6 new integration tests added, all pass (25 total integration tests + 21 unit tests)

## Priority 8: TEST - Add Tests for History and Resume
**Status**: COMPLETE ✅
**Description**: Add integration tests for `--history` and `--resume` flags
**Rationale**: These features lack test coverage
**Tests Added**:
- `test_history_empty_session_file`: Verifies `--history` handles empty session file
- `test_history_with_experiments`: Verifies `--history` recognizes experiments
- `test_resume_invalid_session`: Verifies `--resume` errors on missing session
- `test_resume_flag_recognized`: Verifies `--resume` flag is recognized by CLI
**Note**: Full resume functionality testing limited by session file JSON format (pretty-printed vs compact)
**Result**: 4 new integration tests added, all pass (29 total integration tests + 21 unit tests)

## Priority 9: REFACTOR - Extract Date Parsing Logic
**Status**: COMPLETE ✅
**Description**: Refactor duplicate date parsing logic into helper functions
**Root Cause**: Date parsing code was duplicated in:
1. `cleanup_autoresearch_branches()` - 25 lines of date parsing logic
2. `list_autoresearch_branches()` in main run() - 25 lines of identical logic
**Fix Applied**:
- Created `parse_branch_age_days(branch: &str) -> i64` helper function
- Created `format_branch_age(days: i64) -> String` helper function
- Both locations now use the helper functions (reduced from 50 lines to 2 lines each)
**Benefits**:
- Eliminates code duplication
- Easier to maintain and test
- Consistent date parsing across the codebase
**Test**: All 29 integration tests and 21 unit tests pass

## Priority 10: REFACTOR - Split finalize_experiment Function
**Status**: COMPLETE ✅
**Description**: Break down the 200+ line `finalize_experiment()` function into smaller functions
**Rationale**: Function is too long and complex, making it hard to test and maintain
**Implementation**:
- Created `calculate_final_improvement()` - extracts final improvement calculation
- Created `extract_key_changes()` - extracts key changes from iterations for commit message
- Created `extract_change_summary()` - helper to extract short summary from agent action
- Created `generate_commit_message()` - formats commit message with all metadata
- Created `calculate_runtime_seconds()` - calculates runtime from session timestamps
- Created `generate_branch_name()` - generates unique branch name with timestamp and UUID
- Created `execute_git_operations()` - orchestrates all git operations
- Created `get_current_branch()` - gets current git branch name
- Created `does_branch_exist()` - checks if branch exists
- Created `create_or_checkout_branch()` - creates new branch or checks out existing
- Created `stage_all_changes()` - stages all changes with git add
- Created `commit_changes()` - commits staged changes with message
- Created `has_remote_origin()` - checks if remote origin exists
- Created `push_branch()` - pushes branch to remote origin
- Created `checkout_branch()` - checks out a branch
- Created `create_failure_result()` - creates failure result with recommendations
- Refactored `finalize_experiment()` to use all helper functions (reduced from 200+ lines to ~50 lines)
**Benefits**:
- Improved code readability and maintainability
- Each function has a single responsibility
- Easier to test individual components
- Better separation of concerns
- Proper error handling with rollback on git failures
**Test**: All 21 unit tests and 29 integration tests pass
**Review**: Implementation verified correct - all 17 helper functions properly implemented, code quality excellent, proper separation of concerns, no regressions

## Priority 11: FEATURE - Config File Validation
**Status**: Ready for REVIEW
**Description**: Add validation for config file schema and values
**Rationale**: Invalid config values (negative numbers, invalid paths) should be caught early
**Validation Implemented**:
- ✅ `max_variance`: Must be between 0.0 and 1.0
- ✅ `target_improvement`: Must be positive
- ✅ `max_iterations`: Must be positive
- ✅ `iteration_timeout_minutes`: Must be positive
- ✅ `total_timeout_minutes`: Must be positive
- ✅ `stall_limit`: Must be positive
- ✅ `convergence_window`: Must be positive
- ✅ `session_file`: Must be a valid writable path
**Implementation**:
- Created `ConfigValidationError` enum with specific error types for each validation rule
- Created `validate_config()` function that checks all config values and reports all errors at once
- Created `is_valid_session_path()` helper function to check if session file path is writable
- Updated `load_config()` to call `validate_config()` after loading config
- Added 15 unit tests for validation functions
**Test**: All 36 unit tests and 29 integration tests pass

## Priority 12: FEATURE - Experiment Comparison
**Status**: TODO
**Description**: Add `--compare SESSION_ID1 SESSION_ID2` to compare two experiments
**Rationale**: Users need to compare results from different experimental runs
**Implementation**:
- Read both sessions from session file
- Compare metrics: baseline, best value, improvement %, iterations, runtime
- Display side-by-side comparison
- Highlight which experiment performed better

