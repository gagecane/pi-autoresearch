# Migration Guide

This guide helps you migrate between versions of pi-autoresearch.

## Table of Contents

- [Version History](#version-history)
- [Migrating to v0.2.0](#migrating-to-v020)
- [Migrating to v0.1.0](#migrating-to-v010)
- [Config File Changes](#config-file-changes)
- [CLI Argument Changes](#cli-argument-changes)
- [Session File Format Changes](#session-file-format-changes)

---

## Version History

| Version | Release Date | Notes |
|---------|-------------|-------|
| 0.2.0 | TBD | Progress bars, improved error messages |
| 0.1.0 | 2026-04-01 | Initial release with full feature set |
| 0.0.0 | 2026-03-30 | Initial development |

---

## Migrating to v0.2.0

### New Features

- **Progress Bars**: Long-running operations now display progress bars (requires `indicatif` crate)
- **Improved Error Messages**: Error messages now include suggestions and links to documentation
- **Colored Output**: Error and warning messages are now colorized for better readability

### Breaking Changes

None.

### Migration Steps

1. Update to v0.2.0
2. Review new error message format
3. Optional: Configure progress bar visibility with `--no-progress` flag

---

## Migrating to v0.1.0

### New Features

This was the initial stable release with the following features:

- **Config File Support**: Read defaults from `~/.config/pi-autoresearch/config.json`
- **Branch Cleanup**: `--cleanup-branches` and `--list-branches` flags
- **Dry Run Mode**: `--dry-run` flag to simulate experiments
- **Structured Logging**: Using `tracing` crate with `--quiet` and `--verbose` flags
- **Version Flag**: `--version` and `-V` flags
- **Experiment Comparison**: `--compare-id1` and `--compare-id2` flags
- **Beads Integration**: `--beads-enabled` flag for issue tracking
- **Auto-Approve**: `--auto-approve` flag for CI/CD

### Config File Schema

```json
{
  "metric": "performance",
  "measure": "echo 100",
  "baseline": 100.0,
  "target_improvement": 10.0,
  "max_iterations": 10,
  "max_variance": 0.1,
  "iteration_timeout_minutes": 30,
  "total_timeout_minutes": 120,
  "stall_limit": 3,
  "convergence_threshold": 0.01,
  "convergence_window": 3,
  "session_file": "~/.local/share/pi-autoresearch/sessions.jsonl",
  "beads_enabled": false
}
```

### Validation Rules

All config values are validated:

- `max_variance`: Must be between 0.0 and 1.0
- `target_improvement`: Must be positive
- `max_iterations`: Must be positive
- `iteration_timeout_minutes`: Must be positive
- `total_timeout_minutes`: Must be positive
- `stall_limit`: Must be positive
- `convergence_window`: Must be positive
- `session_file`: Must be a valid writable path

### CLI Argument Precedence

CLI arguments always take precedence over config file values:

```
CLI > Config File > Default Values
```

### Session File Format

Session files use JSONL format with three record types:

1. **BaselineRecord**: Initial baseline measurement
2. **IterationRecord**: Each iteration's results
3. **ExperimentSession**: Complete experiment summary

Example:

```jsonl
{"type":"baseline","session_id":"abc123","timestamp":"2026-04-01T12:00:00Z","question":"Improve performance","metric":"performance","baseline_value":100.0,"baseline_command":"echo 100"}
{"type":"iteration","session_id":"abc123","iteration":1,"timestamp":"2026-04-01T12:05:00Z","improvement":5.0,"value":105.0,"agent_action":"Added caching","branch":"autoresearch/20260401-120000-abc123"}
{"type":"experiment","session_id":"abc123","status":"success","question":"Improve performance","metric":"performance","baseline":100.0,"best_improvement":15.0,"best_value":115.0,"iterations":5,"start_time":"2026-04-01T12:00:00Z","end_time":"2026-04-01T12:30:00Z"}
```

---

## Config File Changes

### v0.1.0 → v0.2.0

No breaking changes to config file format.

### v0.0.0 → v0.1.0

**Added**: Config file support

Before v0.1.0, all settings were CLI-only. In v0.1.0, you can now:

1. Create `~/.config/pi-autoresearch/config.json` for global defaults
2. Use `--config PATH` for project-specific configs
3. CLI args still override config file values

**Migration**: No action required. Config file is optional.

---

## CLI Argument Changes

### v0.1.0 → v0.2.0

**Added**:
- `--no-progress`: Disable progress bars

**Removed**: None

**Changed**: None

### v0.0.0 → v0.1.0

**Added**:
- `--config PATH`: Specify config file path
- `--cleanup-branches`: Clean up old autoresearch branches
- `--list-branches`: List all autoresearch branches
- `--cleanup-days N`: Only remove branches older than N days (default: 7)
- `--dry-run`: Simulate experiment without applying changes
- `--quiet`: Only show errors
- `--verbose`: Show debug logs
- `--version`, `-V`: Show version
- `--compare-id1 SESSION_ID`: First session to compare
- `--compare-id2 SESSION_ID`: Second session to compare
- `--beads-enabled`: Enable beads integration
- `--auto-approve`: Auto-approve changes (CI/CD mode)

**Removed**: None

**Changed**: None

---

## Session File Format Changes

### v0.1.0 → v0.2.0

No breaking changes to session file format.

### v0.0.0 → v0.1.0

**Added**: Support for pretty-printed JSON (in addition to compact JSONL)

Before v0.1.0, session files only supported compact JSONL format. In v0.1.0, both formats are supported:

**Compact JSONL** (backward compatible):
```jsonl
{"type":"baseline",...}
{"type":"iteration",...}
```

**Pretty-printed JSON** (new):
```json
{
  "type": "baseline",
  ...
}
{
  "type": "iteration",
  ...
}
```

**Migration**: No action required. Both formats are automatically detected.

---

## Common Migration Issues

### Issue: Config file not loaded

**Symptom**: Settings from config file are ignored

**Solution**:
1. Verify config file location: `~/.config/pi-autoresearch/config.json`
2. Check file permissions: `ls -la ~/.config/pi-autoresearch/config.json`
3. Validate JSON syntax: `jq . ~/.config/pi-autoresearch/config.json`
4. Check for validation errors: `pi-autoresearch --verbose --help`

### Issue: Session file format error

**Symptom**: "Invalid session file format" error

**Solution**:
1. Verify session file is valid JSONL or JSON
2. Check that all required fields are present
3. Use `--history` to list valid sessions
4. Delete corrupted session file if needed

### Issue: CLI args not overriding config

**Symptom**: Config file values are used instead of CLI args

**Solution**: This should not happen. CLI args always take precedence. If you experience this:
1. Verify you're using the correct flag name
2. Check for typos in flag names
3. Use `--verbose` to see which values are being used
4. Report as a bug if issue persists

---

## Rollback Guide

If you need to rollback to a previous version:

1. **Save current session files**:
   ```bash
   cp ~/.local/share/pi-autoresearch/sessions.jsonl ~/sessions-backup.jsonl
   ```

2. **Save current config**:
   ```bash
   cp ~/.config/pi-autoresearch/config.json ~/config-backup.json
   ```

3. **Rebuild previous version**:
   ```bash
   git checkout <previous-version-tag>
   cargo build --release
   ```

4. **Restore if needed**:
   ```bash
   cp ~/sessions-backup.jsonl ~/.local/share/pi-autoresearch/sessions.jsonl
   cp ~/config-backup.json ~/.config/pi-autoresearch/config.json
   ```

---

## Support

For migration issues:

1. Check [Troubleshooting Guide](TROUBLESHOOTING.md)
2. Review [CHANGELOG.md](../CHANGELOG.md) for version-specific changes
3. See [USAGE.md](USAGE.md) for current usage instructions
4. Report bugs via [Beads](https://beads.dev) or GitHub issues

---

## Changelog Reference

For detailed version history, see [CHANGELOG.md](../CHANGELOG.md).

---

*Last updated: 2026-04-02*
