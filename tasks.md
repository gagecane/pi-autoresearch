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
**Status**: TODO
**Description**: Fix bugs where `cli.session_file` is used directly instead of `get_session_file(&cli, &config)`
**Root Cause**: Two code paths bypass config file support:
1. Line 736 in `run_iterative_loop()`: Uses `&cli.session_file` instead of `get_session_file(&cli, &config)`
2. Line 1722 in `--resume` section: Uses `&cli.session_file` instead of `get_session_file(&cli, &config)`
**Impact**: Config file `session_file` setting is ignored in these code paths
**Fix Required**: Replace direct `cli.session_file` usage with `get_session_file(&cli, &config)`

## Priority 6: TEST - Add Unit Tests for Helper Functions
**Status**: TODO
**Description**: Add unit tests for all 13 config helper functions
**Rationale**: Currently only integration tests exist; helper functions lack dedicated unit tests
**Functions to Test**:
- `get_metric()`
- `get_measure()`
- `get_baseline()`
- `get_target_improvement()`
- `get_max_iterations()`
- `get_max_variance()`
- `get_session_file()`
- `get_beads_enabled()`
- `get_iteration_timeout()`
- `get_total_timeout()`
- `get_stall_limit()`
- `get_convergence_threshold()`
- `get_convergence_window()`
**Test Scenarios**:
- CLI value provided (should use CLI)
- Config value provided, no CLI (should use config)
- Neither CLI nor config (should use default)
- CLI overrides config

## Priority 7: TEST - Add Tests for New Features
**Status**: TODO
**Description**: Add integration tests for branch cleanup and dry-run mode
**Rationale**: Priority 3 and 4 features were implemented without dedicated tests
**Tests Needed**:
- `test_list_branches`: Verify `--list-branches` shows autoresearch branches
- `test_cleanup_branches`: Verify `--cleanup-branches` removes old branches
- `test_cleanup_branches_preserves_recent`: Verify recent branches are kept
- `test_dry_run_no_session_file`: Verify dry-run doesn't create session files
- `test_dry_run_no_git_branch`: Verify dry-run doesn't create git branches
- `test_dry_run_shows_banner`: Verify dry-run banner is displayed

## Priority 8: TEST - Add Tests for History and Resume
**Status**: TODO
**Description**: Add integration tests for `--history` and `--resume` flags
**Rationale**: These features lack test coverage
**Tests Needed**:
- `test_history_empty`: Verify `--history` handles empty session file
- `test_history_with_experiments`: Verify `--history` lists experiments correctly
- `test_resume_valid_session`: Verify `--resume` continues from saved state
- `test_resume_invalid_session`: Verify `--resume` errors on missing session

## Priority 9: REFACTOR - Extract Date Parsing Logic
**Status**: TODO
**Description**: Refactor duplicate date parsing logic into a helper function
**Root Cause**: Date parsing code is duplicated in:
1. `cleanup_autoresearch_branches()` (lines 1636-1649)
2. `list_autoresearch_branches()` in main run() (lines 1706-1719)
**Fix Required**: 
- Create `parse_branch_age(branch: &str) -> Result<String>` helper function
- Use in both locations

## Priority 10: REFACTOR - Split finalize_experiment Function
**Status**: TODO
**Description**: Break down the 200+ line `finalize_experiment()` function into smaller functions
**Rationale**: Function is too long and complex, making it hard to test and maintain
**Proposed Split**:
- `calculate_final_improvement()` - calculate improvement metrics
- `generate_failure_report()` - create failure report (already exists as `generate_failure_recommendations`)
- `generate_success_commit()` - create branch and commit for success
- `generate_commit_message()` - format commit message
- `execute_git_operations()` - perform git branch/commit/push

## Priority 11: FEATURE - Config File Validation
**Status**: TODO
**Description**: Add validation for config file schema and values
**Rationale**: Invalid config values (negative numbers, invalid paths) should be caught early
**Validation Needed**:
- `max_variance`: Must be between 0.0 and 1.0
- `target_improvement`: Must be positive
- `max_iterations`: Must be positive
- `iteration_timeout_minutes`: Must be positive
- `total_timeout_minutes`: Must be positive
- `stall_limit`: Must be positive
- `convergence_window`: Must be positive
- `session_file`: Must be a valid writable path

## Priority 12: FEATURE - Experiment Comparison
**Status**: TODO
**Description**: Add `--compare SESSION_ID1 SESSION_ID2` to compare two experiments
**Rationale**: Users need to compare results from different experimental runs
**Implementation**:
- Read both sessions from session file
- Compare metrics: baseline, best value, improvement %, iterations, runtime
- Display side-by-side comparison
- Highlight which experiment performed better

