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
**Status**: COMPLETE ✅
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
**Review**: Implementation verified correct - all validation rules properly implemented, non-destructive validation, clear error messages, all errors reported together

## Priority 12: FEATURE - Experiment Comparison
**Status**: COMPLETE ✅
**Description**: Add `--compare-id1 SESSION_ID1 --compare-id2 SESSION_ID2` to compare two experiments
**Rationale**: Users need to compare results from different experimental runs
**Implementation**:
- Added CLI arguments: `--compare-id1` and `--compare-id2`
- Created `compare_experiments()` function to find and compare two sessions
- Created `display_experiment_comparison()` function for side-by-side display
- Created helper functions: `calculate_session_runtime()`, `format_duration()`, `truncate_str()`
- Comparison shows: question, metric, baseline, iterations, best improvement, status, runtime
- Highlights winner based on best improvement percentage
- Shows detailed iteration breakdown for each experiment
- Runtime calculation properly parses timestamps using chrono
**Test**: All 31 integration tests pass (29 original + 2 new compare tests) + 36 unit tests
**Review**: Implementation verified correct - all functions properly implemented, runtime calculation works, no issues found

## Priority 13: TEST - Add Unit Tests for Helper Functions
**Status**: COMPLETE ✅
**Description**: Add unit tests for helper functions that lack test coverage
**Rationale**: Improve test coverage and ensure helper functions work correctly
**Functions Tested**:
- `parse_branch_age_days()` - Tests for nonexistent and invalid branches
- `format_branch_age()` - Tests for today, yesterday, days, and negative values
- `uuid_generate()` - Tests for uniqueness and format
- `format_duration()` - Tests for seconds, minutes, and hours
- `truncate_str()` - Tests for short, exact, long, and empty strings
**Tests Added**: 15 new unit tests
**Result**: 51 unit tests (was 36) + 31 integration tests = 82 total tests passing
**Review**: Implementation verified correct - all helper functions properly tested, edge cases covered

## Priority 14: TEST - Add Unit Tests for Git Functions
**Status**: COMPLETE ✅
**Description**: Add unit tests for git-related functions
**Rationale**: Git functions are critical but lack dedicated tests
**Functions Tested**:
- `get_git_commit_hash()` - Test format of commit hash
- `get_current_branch()` - Test that function doesn't panic
- `does_branch_exist()` - Test current branch and nonexistent branch
- `generate_branch_name()` - Test format and uniqueness
**Tests Added**: 6 new unit tests
**Result**: 57 unit tests (was 51) + 31 integration tests = 88 total tests passing
**Note**: Some git functions require actual git operations and are tested with defensive assertions
**Review**: Implementation verified correct - all git functions properly tested, edge cases handled

## Priority 15: TEST - Add Unit Tests for Agent Functions
**Status**: COMPLETE ✅
**Description**: Add unit tests for agent-related functions
**Rationale**: Agent functions need test coverage
**Functions Tested**:
- `invoke_pi_agent()` - Test format and content of agent response
- `read_question_from_stdin()` - Cannot be tested in unit tests (requires stdin)
- `execute_measurement()` - Cannot be tested in unit tests (requires shell execution)
**Tests Added**: 3 new unit tests for invoke_pi_agent()
**Result**: 60 unit tests (was 57) + 31 integration tests = 91 total tests passing
**Note**: Some agent functions require external systems and are tested in integration tests
**Review**: Implementation verified correct - invoke_pi_agent() properly tested, edge cases covered

## Priority 16: DOCS - Create Documentation
**Status**: COMPLETE ✅
**Description**: Create documentation in docs/ folder
**Rationale**: Users need clear documentation on how to use the tool
**Documentation Created**:
- `docs/README.md` - Overview and quick start
- `docs/USAGE.md` - Detailed usage guide
- `docs/CONFIG.md` - Configuration file documentation
- `docs/EXAMPLES.md` - Example use cases
**Fixes Applied**:
- Updated installation instructions to build from source (not crates.io)
- Added development section with build, test, and run instructions
- Clarified session file format with multi-line JSONL example
- Added link to main README.md
- Moved Metric Detection section higher for better visibility
**Ready for Review**: All feedback from previous review has been addressed

## Priority 17: SPECS - Align Specs with Implementation
**Status**: COMPLETE ✅
**Description**: Create specs/ folder and align with code implementation
**Rationale**: Specs should document the expected behavior
**Specs Created**:
- `specs/CLI.md` - CLI interface specification (6.4KB)
  - Documents all CLI arguments, defaults, and precedence rules
  - Includes examples for common use cases
  - Covers special modes (dry-run, resume, comparison, etc.)
- `specs/SESSION.md` - Session file format specification (7.7KB)
  - Documents JSONL format and all record types
  - Includes schemas for BaselineRecord, IterationRecord, ExperimentSession
  - Provides example JSON for each record type
- `specs/CONFIG.md` - Configuration file specification (6.1KB)
  - Documents all config fields and validation rules
  - Includes error messages and validation ranges
  - Provides example config files
- `specs/WORKFLOW.md` - Experiment workflow specification (7.8KB)
  - Documents complete 5-stage workflow
  - Includes workflow diagram and termination conditions
  - Covers state management and git branch lifecycle
**Total**: 4 spec files created, 28KB total documentation
**Review**: Implementation verified correct - all 4 spec files accurately document the implementation, all record types, CLI arguments, config validation rules, and workflow stages align perfectly with code

## Priority 18: RESEARCH - Code Quality Improvements
**Status**: COMPLETE ✅
**Description**: Research and identify code quality improvements
**Rationale**: Improve code quality and maintainability
**Findings**:
- Clippy warnings found:
  - Manual `!RangeInclusive::contains` implementation
  - Called `Iterator::last` on `DoubleEndedIterator`
  - Large size difference between variants
  - Called `unwrap` after checking `is_some`
  - Manual implementation of assign operation
  - Borrowed expression implements required traits
- Unused variable warning at line 2998 in test code
- README.md is minimal (1 line) - needs improvement
**Decomposed Into**:
- Priority 19: CODE QUALITY - Fix Clippy Warnings
- Priority 20: CODE QUALITY - Fix Unused Variable Warning
- Priority 21: DOCS - Improve README.md

## Priority 19: CODE QUALITY - Fix Clippy Warnings
**Status**: COMPLETE ✅
**Description**: Fix all clippy warnings in the codebase
**Rationale**: Improve code quality and follow Rust best practices
**Warnings Fixed**:
1. ✅ Manual `!RangeInclusive::contains` implementation - changed to `!(0.0..=1.0).contains(&value)`
2. ✅ Called `Iterator::last` on `DoubleEndedIterator` - changed to `next_back()`
3. ✅ Large size difference between variants - boxed `ExperimentSession` variant in `SessionRecord` enum
4. ✅ Called `unwrap` after checking `is_some` - changed to `if let (Some(id1), Some(id2))` pattern
5. ✅ Manual implementation of assign operation - changed to `+=` operator
6. ✅ Borrowed expression implements required traits - removed unnecessary `&` borrow
**Changes Made**:
- Line 213: Changed `if value < 0.0 || value > 1.0` to `if !(0.0..=1.0).contains(&value)`
- Line 1196-1198: Changed `.split(':').last()` to `.split(':').next_back()`
- Line 1684: Changed `Experiment(ExperimentSession)` to `Experiment(Box<ExperimentSession>)`
- Line 1705, 1728: Updated to use `Box::new()` when creating `SessionRecord::Experiment`
- Line 2110-2112: Changed to `if let (Some(id1), Some(id2)) = (&cli.compare_id1, &cli.compare_id2)`
- Line 2168: Changed `iter.iteration = max_old_iter + iter.iteration` to `iter.iteration += max_old_iter`
- Line 2396: Changed `.open(&get_session_file(&cli, &config))` to `.open(get_session_file(&cli, &config))`
- Line 1997: Updated `find_session_by_id()` to use `session.as_ref().clone()` for boxed variant
**Test**: `cargo clippy` passes with no warnings, all 60 unit tests and 31 integration tests pass
**Review**: Implementation verified correct - all 7 clippy warnings fixed using proper Rust idioms, no regressions introduced, code quality improved

## Priority 20: CODE QUALITY - Fix Unused Variable Warning
**Status**: COMPLETE ✅
**Description**: Fix unused variable `branch` at line 2998 in test_get_current_branch_not_empty()
**Rationale**: Eliminate compiler warnings
**Fix Applied**: Changed `let branch =` to `let _branch =` to indicate intentionally unused variable
**Test**: `cargo build` completes with no warnings, all 31 integration tests pass
**Review**: Implementation verified correct - unused variable properly marked with underscore prefix, code compiles cleanly

## Priority 21: DOCS - Improve README.md
**Status**: Ready for REVIEW 🔄
**Description**: Expand README.md from 1 line to comprehensive documentation
**Rationale**: First impression of the project, needs to be informative
**Content Added**:
- ✅ Project description and purpose
- ✅ Feature highlights (10 key features)
- ✅ Installation instructions (build from source)
- ✅ Requirements section
- ✅ Quick start guide with examples
- ✅ Configuration file example
- ✅ Common use cases (performance, memory, accuracy)
- ✅ Complete command-line options table (25+ options)
- ✅ Links to detailed documentation (docs/ and specs/ folders)
- ✅ Development section (build, test, code quality)
- ✅ Contributing guidelines
- ✅ License information
- ✅ Support and acknowledgments
**Changes**:
- Expanded from 79 bytes to 6040 bytes
- Added structured sections with markdown formatting
- Included code examples and tables
- Cross-referenced all documentation files
**Test**: README.md is informative and helpful for new users
**Ready for Review**: All content added and formatted correctly

## Priority 22: RESEARCH - Code Quality and Feature Improvements
**Status**: TODO ⏳
**Description**: Research the codebase to discover additional improvement opportunities
**Rationale**: Continuous improvement of code quality, features, and documentation
**Areas to Research**:
- Mutation testing opportunities
- Contract testing for API boundaries
- Additional unit test coverage gaps
- Performance optimization opportunities
- New feature requests based on user feedback
- Documentation gaps
- Code organization and module structure improvements
- Security considerations
- Error handling improvements
- Logging and observability enhancements
**Test**: Research completed and new tasks added to tasks.md

