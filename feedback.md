# Feedback

## Issues Found During Review of Config File Support

### Bug 1: target_improvement not using config file (Line 1796)
**Location**: `src/main.rs`, line 1796
**Current Code**:
```rust
let target_improvement = cli.target_improvement.unwrap_or(design.target_improvement);
```
**Issue**: This bypasses the config file value. If a user sets `target_improvement` in the config file but doesn't specify it via CLI, the config value is ignored.
**Fix**: Use the helper function:
```rust
let target_improvement = get_target_improvement(&cli, &config, design.target_improvement);
```

### Bug 2: session_file not using config file (Line 1793)
**Location**: `src/main.rs`, line 1793
**Current Code**:
```rust
.open(&cli.session_file)?;
```
**Issue**: This bypasses the config file value. If a user sets `session_file` in the config file but doesn't specify it via CLI, the config value is ignored.
**Fix**: Use the helper function:
```rust
.open(&get_session_file(&cli, &config))?;
```

### Impact
These bugs mean that config file values for `target_improvement` and `session_file` (in this specific code path) are never used, violating the acceptance criteria:
- ✅ CLI args override config file
- ❌ Config file values are ignored in some code paths

### Task Status
Mark as **REVISE** - implement the fixes above.
