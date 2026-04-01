# Tasks

## Priority 1: FIX - Test Parallelization Issue
**Status**: TODO
**Description**: Fix flaky integration tests that fail when run in parallel due to git branch name collisions
**Root Cause**: Branch names use timestamp format `autoresearch/YYYYMMDD-HHMMSS` which can collide when multiple tests run within the same second
**Solution**: Add unique identifier to branch names (UUID or nanosecond precision)
**Test**: `cargo test` should pass consistently without `--test-threads=1`

## Priority 2: FEATURE - Config File Support
**Status**: TODO  
**Description**: Implement User Story 4.2 from PRD - read defaults from `~/.config/pi-autoresearch/config.json`
**Acceptance Criteria**:
- Read defaults from config file
- CLI args override config file
- Support `--config PATH` for project-specific configs

## Priority 3: FEATURE - Branch Cleanup
**Status**: TODO
**Description**: Add functionality to clean up autoresearch branches after completion
**Rationale**: Prevents accumulation of autoresearch branches in git repository

## Priority 4: FEATURE - Dry Run Mode
**Status**: TODO
**Description**: Add `--dry-run` flag to simulate experiments without applying changes
**Rationale**: Allows users to preview what changes would be made

