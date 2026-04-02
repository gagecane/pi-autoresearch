# Migration Guide

This guide helps you migrate pi-autoresearch configurations and workflows between versions.

## Version History

- [0.1.0](#010) - Initial stable release (2026-04-01)
- [Unreleased](#unreleased) - Upcoming changes

---

## Unreleased

### Planned Changes

No breaking changes are currently planned.

---

## 0.1.0

This was the initial stable release of pi-autoresearch. If you were using a pre-0.1.0 development version, see the notes below.

### Config File Changes

#### New Fields Added

The following fields were added to the config file schema:

```json
{
  "iteration_timeout_minutes": 10,
  "total_timeout_minutes": 120,
  "stall_limit": 5,
  "convergence_threshold": 0.01,
  "convergence_window": 3,
  "beads_enabled": false
}
```

**Migration**: These fields are optional. If you have an existing config file, you can add these fields or omit them to use defaults.

#### Field Validation

Version 0.1.0 added comprehensive validation for all config fields:

| Field | Validation Rule |
|-------|----------------|
| `max_variance` | Must be between 0.0 and 1.0 |
| `target_improvement` | Must be positive (> 0) |
| `max_iterations` | Must be positive (> 0) |
| `iteration_timeout_minutes` | Must be positive (> 0) |
| `total_timeout_minutes` | Must be positive (> 0) |
| `stall_limit` | Must be positive (> 0) |
| `convergence_window` | Must be positive (> 0) |
| `session_file` | Must be a valid, writable path |

**Migration**: If your existing config has invalid values, update them to meet the validation rules.

**Example**:

```json
// Before (invalid)
{
  "max_variance": 1.5,
  "target_improvement": -0.1
}

// After (valid)
{
  "max_variance": 0.1,
  "target_improvement": 0.1
}
```

### CLI Changes

#### New Flags

The following CLI flags were added in 0.1.0:

| Flag | Description |
|------|-------------|
| `--version` / `-V` | Display version information |
| `--quiet` | Only show errors |
| `--verbose` | Show debug-level logs |
| `--dry-run` | Simulate without applying changes |
| `--list-branches` | List autoresearch branches |
| `--cleanup-branches` | Clean up old branches |
| `--cleanup-days N` | Days threshold for cleanup (default: 7) |
| `--compare-id1 ID` | First experiment ID for comparison |
| `--compare-id2 ID` | Second experiment ID for comparison |
| `--history` | Show experiment history |
| `--resume` | Resume last experiment |
| `--config PATH` | Use custom config file |
| `--beads-enabled` | Enable beads integration |
| `--auto-approve` | Skip interactive prompts |
| `--skip-git` | Skip git operations |

**Migration**: These flags are optional and backward compatible. Existing scripts will continue to work.

#### Removed Flags

No flags were removed in 0.1.0.

### Session File Format

#### Format Changes

Version 0.1.0 standardized the session file format to JSONL (JSON Lines):

```jsonl
{"type":"baseline","session_id":"abc123","timestamp":"2026-04-01T00:00:00Z","metric":"execution_time_ms","value":100.0}
{"type":"iteration","session_id":"abc123","iteration":1,"timestamp":"2026-04-01T00:01:00Z","improvement":0.10,"accepted":true}
```

**Migration**: The session file parser is backward compatible with:
- Compact JSONL format (one JSON object per line)
- Pretty-printed JSON format (multi-line JSON objects)
- Older session files will be read correctly

### Git Branch Naming

#### Format Changes

Version 0.1.0 updated the git branch naming format to include a unique identifier:

```
Before: autoresearch/20260401-120000
After:  autoresearch/20260401-120000-{uuid}
```

**Migration**: 
- Old branches are not automatically cleaned up
- Use `--list-branches` to see all autoresearch branches
- Use `--cleanup-branches` to remove old branches

### Logging

#### Changes

Version 0.1.0 replaced `eprintln!` with structured logging using the `tracing` crate:

- Log level can be controlled via `--quiet`, `--verbose`, or `RUST_LOG` environment variable
- Output still goes to stderr for backward compatibility

**Migration**: Existing scripts that parse stderr output will continue to work. To change log verbosity:

```bash
# Only show errors
pi-autoresearch --quiet

# Show all logs including debug
pi-autoresearch --verbose

# Or use RUST_LOG environment variable
RUST_LOG=debug pi-autoresearch
```

---

## Pre-0.1.0 Development Versions

If you were using a development version before 0.1.0, note the following:

### Breaking Changes

1. **Config file validation**: Invalid config values now cause the tool to exit with an error. Previously, invalid values might have been silently ignored.

2. **Session file format**: Session files now use a standardized JSONL format. Old session files should be migrated manually if they don't follow the new format.

3. **Git branch naming**: Branch names now include a UUID to prevent collisions. Old branches will not be automatically cleaned up.

### Migration Steps

1. **Update config file**:
   ```bash
   # Check if your config file has invalid values
   pi-autoresearch --config ~/.config/pi-autoresearch/config.json --help
   
   # Fix any validation errors reported
   ```

2. **Migrate session files** (if needed):
   ```bash
   # Backup old session file
   cp autoresearch.jsonl autoresearch.jsonl.backup
   
   # Use --resume to continue with existing session
   # Or start fresh with a new session file
   ```

3. **Clean up old git branches**:
   ```bash
   # List all autoresearch branches
   pi-autoresearch --list-branches
   
   # Clean up old branches (older than 7 days by default)
   pi-autoresearch --cleanup-branches
   ```

---

## General Migration Tips

### Config File

1. **Backup before upgrading**:
   ```bash
   cp ~/.config/pi-autoresearch/config.json ~/.config/pi-autoresearch/config.json.backup
   ```

2. **Validate config after upgrade**:
   ```bash
   pi-autoresearch --help  # Will validate config and show errors
   ```

3. **Use project-specific configs**:
   ```bash
   # Store config in project directory
   pi-autoresearch --config ./config.json --question "Optimize performance"
   ```

### Session Files

1. **Backup session files**:
   ```bash
   cp autoresearch.jsonl autoresearch.jsonl.backup
   ```

2. **View experiment history**:
   ```bash
   pi-autoresearch --history
   ```

3. **Compare experiments**:
   ```bash
   pi-autoresearch --compare-id1 abc123 --compare-id2 def456
   ```

### Git Branches

1. **List all branches**:
   ```bash
   pi-autoresearch --list-branches
   ```

2. **Clean up old branches**:
   ```bash
   # Clean branches older than 7 days
   pi-autoresearch --cleanup-branches
   
   # Clean branches older than 30 days
   pi-autoresearch --cleanup-branches --cleanup-days 30
   ```

---

## Troubleshooting

### Config Validation Errors

If you see validation errors after upgrading:

```
Invalid config file ~/.config/pi-autoresearch/config.json:
  max_variance must be between 0.0 and 1.0, got 1.50
```

**Solution**: Update your config file to meet validation rules. See [docs/CONFIG.md](CONFIG.md) for valid values.

### Session File Not Found

If you see "session file not found" errors:

```
Session file autoresearch.jsonl not found
```

**Solution**: 
- Start a new experiment to create a session file
- Or specify a valid session file with `--session-file PATH`

### Git Branch Conflicts

If you see branch name collision errors:

```
Branch autoresearch/20260401-120000 already exists
```

**Solution**: 
- Clean up old branches: `pi-autoresearch --cleanup-branches`
- Or use `--skip-git` to skip git operations

---

## Related Documentation

- [docs/CONFIG.md](CONFIG.md) - Configuration file reference
- [docs/USAGE.md](USAGE.md) - Usage guide
- [docs/TROUBLESHOOTING.md](TROUBLESHOOTING.md) - Troubleshooting guide
- [specs/SESSION.md](../specs/SESSION.md) - Session file format specification
- [CHANGELOG.md](../CHANGELOG.md) - Version history

---

## Feedback

If you encounter issues during migration, please:

1. Check [docs/TROUBLESHOOTING.md](TROUBLESHOOTING.md)
2. Review the [CHANGELOG.md](../CHANGELOG.md) for version-specific changes
3. Open an issue with details about your migration problem

</content>} </tool_call></arguments> <tool_call></tool_call>{