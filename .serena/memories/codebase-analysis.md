# Codebase Analysis - pi-autoresearch

## Architecture Understanding

**Product**: CLI tool for autonomous research experiments (3-phase framework)
- Phase 1: Experiment Design & Baseline Verification
- Phase 2: Iterative Experiment Loop with Stuck Detection  
- Phase 3: Finalization & Merge

**Key Files**:
- `src/main.rs` - Single file implementation (~1600 lines)
- `tests/integration_tests.rs` - 19 integration tests
- `tasks/prd-autoresearch-cli-wrapper-for-pi-coding-agent.md` - Complete PRD

## Implementation Status

✅ **Complete**:
- CLI argument parsing (all PRD requirements)
- Experiment design generation
- Baseline verification with variance checking
- Iterative loop with stuck detection (4 layers)
- Live progress display
- Git branch creation and commit generation
- Failed experiment reporting
- Beads (bd) integration
- Session persistence (JSONL format)

⚠️ **Issues Found**:
1. **Test flakiness**: 3 tests fail in parallel due to branch name collision
2. **Missing features**: Config file support, branch cleanup, dry-run mode

## Technical Details

**Branch Naming Issue**:
```rust
// Current (line ~1300):
let branch_name = format!("autoresearch/{}", timestamp);
// Problem: Same timestamp = branch collision
// Fix: Add unique identifier
let branch_name = format!("autoresearch/{}-{}", timestamp, uuid_generate());
```

**Test Isolation**:
- Tests use shared git repository state
- Branch creation conflicts when run in parallel
- Solution: Run with `--test-threads=1` (workaround) or fix branch naming

## Next Steps

1. **FIX**: Test parallelization (Priority 1)
2. **FEATURE**: Config file support (User Story 4.2)
3. **FEATURE**: Branch cleanup
4. **FEATURE**: Dry-run mode

## Session Notes

- All 19 tests pass with `--test-threads=1`
- 16 tests pass in parallel, 3 fail (iterative loop tests)
- Implementation is functionally complete and working
- Code quality is good with clear separation of concerns

