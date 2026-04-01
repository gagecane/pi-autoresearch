# Feedback for Priority 2: Config File Support

## Review Status: REVISE

The config file support has been implemented but has several issues that need to be fixed:

## Issues Found

### 1. **Bug in `get_max_variance()` function** (Line ~147-154)
```rust
fn get_max_variance(cli: &Cli, config: &Option<Config>, default: f64) -> f64 {
    if cli.max_variance != 0.05 {
        // CLI explicitly set (not default)
        cli.max_variance
    } else {
        config.as_ref()
            .and_then(|c| c.max_variance)
            .unwrap_or(default)
    }
}
```
**Problem**: This logic is flawed. If a user explicitly passes `--max-variance 0.05` (the default), it will ignore the CLI value and use the config instead. The function cannot distinguish between "user explicitly set the default" vs "user didn't set anything".

**Fix**: CLI arguments should always take precedence over config. Remove the default-value check and simply use `cli.max_variance` directly since clap already provides the default.

### 2. **Bug in `get_session_file()` function** (Line ~156-164)
```rust
fn get_session_file(cli: &Cli, config: &Option<Config>) -> String {
    if cli.session_file != "autoresearch.jsonl" {
        // CLI explicitly set (not default)
        cli.session_file.clone()
    } else {
        config.as_ref()
            .and_then(|c| c.session_file.clone())
            .unwrap_or_else(|| "autoresearch.jsonl".to_string())
    }
}
```
**Problem**: Same issue as above. If user explicitly passes `--session-file autoresearch.jsonl`, it will be ignored in favor of config.

**Fix**: CLI should always win. Simply use `cli.session_file.clone()`.

### 3. **Inconsistent use of helper functions**
In the `run()` function, some places use the helper functions correctly, but others bypass them:

- Line ~1673-1678: Uses direct `.or()` pattern instead of `get_metric()`
- Line ~1673: `let metric = cli.metric.clone().or(config.as_ref().and_then(|c| c.metric.clone()))`

**Fix**: Use the helper functions consistently throughout.

### 4. **Missing config file validation**
When `--config PATH` is explicitly provided but the file doesn't exist, the code silently falls back to defaults. This is confusing for users.

**Fix**: If `--config` is explicitly provided and the file doesn't exist, return an error.

### 5. **Missing iteration timeout config helpers**
The `Config` struct has `iteration_timeout_minutes` and `total_timeout_minutes`, but there are no helper functions to retrieve them (unlike other config values).

**Fix**: Add `get_iteration_timeout()` and `get_total_timeout()` helper functions.

### 6. **Missing config for other CLI options**
The `Config` struct is missing:
- `auto_approve`
- `verify_baseline`
- `verbose`
- `quiet`
- `beads_enabled` (actually present but not consistently used)

**Recommendation**: Either add these to config or document why they're excluded.

## Acceptance Criteria Status

- ✅ Read defaults from config file - **Partially implemented** (has bugs)
- ✅ CLI args override config file - **Broken** (due to default-value detection bugs)
- ✅ Support `--config PATH` for project-specific configs - **Partially implemented** (should error on missing file)

## Recommended Actions

1. Fix the default-value detection bugs in `get_max_variance()` and `get_session_file()`
2. Add missing helper functions for timeout values
3. Add error handling for missing explicit config files
4. Use helper functions consistently throughout `run()`
5. Add integration tests for config file functionality
6. Add example config file to repository

## Test Cases to Add

```rust
#[test]
fn test_config_file_defaults() {
    // Test that config values are used when CLI doesn't specify
}

#[test]
fn test_cli_overrides_config() {
    // Test that CLI args take precedence over config
}

#[test]
fn test_missing_config_file_error() {
    // Test that --config PATH errors when file doesn't exist
}
```