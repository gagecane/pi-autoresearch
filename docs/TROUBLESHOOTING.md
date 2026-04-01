# Troubleshooting Guide

This document provides solutions to common issues encountered when using pi-autoresearch.

## Installation Issues

### "pi-autoresearch: command not found"

**Solution**: Build from source:
```bash
cargo build --release
cp target/release/pi-autoresearch ~/.local/bin/
```

### "cargo: command not found"

**Solution**: Install Rust toolchain:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### "llvm-tools-preview not found" (coverage)

**Solution**:
```bash
rustup component add llvm-tools-preview
```

## Configuration Issues

### "Config validation failed"

**Common causes**:
- `max_variance` must be between 0.0 and 1.0
- `target_improvement` must be positive
- `max_iterations` must be positive
- `iteration_timeout_minutes` must be positive
- `total_timeout_minutes` must be positive
- `stall_limit` must be positive
- `convergence_window` must be positive
- `session_file` must be a valid writable path

**Solution**: Check your config file at `~/.config/pi-autoresearch/config.json`:
```bash
cat ~/.config/pi-autoresearch/config.json
```

### "Config file not found"

**Solution**: Create config file:
```bash
mkdir -p ~/.config/pi-autoresearch
cat > ~/.config/pi-autoresearch/config.json << 'EOF'
{
    "metric": "execution_time_ms",
    "measure": "hyperfine --warmup 3 'your-command'",
    "baseline": 100.0,
    "target_improvement": 0.1
}
EOF
```

## Measurement Issues

### "Could not parse measurement output as number"

**Cause**: Measurement command doesn't output a numeric value.

**Solution**: Ensure your measurement command outputs only a number:
```bash
# Bad: Outputs text
echo "Result: 42.5"

# Good: Outputs only number
echo "42.5"
```

### "Measurement command failed"

**Cause**: Command execution failed (non-zero exit code).

**Solution**: Test your measurement command manually:
```bash
# Test the command
echo 100.0

# Check exit code
echo $?
```

### "Baseline variance exceeds threshold"

**Cause**: Baseline measurements vary too much between runs.

**Solutions**:
1. Increase `--max-variance` threshold:
   ```bash
   --max-variance 0.1  # Allow 10% variance
   ```
2. Improve measurement stability (more warmup runs, etc.)
3. Use a more stable measurement command

## Git Issues

### "Not a git repository"

**Solution**: Initialize git repo or use `--skip-git`:
```bash
# Option 1: Initialize git
git init

# Option 2: Skip git operations
--skip-git
```

### "Failed to create branch"

**Cause**: Branch name conflict or git permission issues.

**Solutions**:
1. Clean up old branches:
   ```bash
   pi-autoresearch --list-branches
   pi-autoresearch --cleanup-branches --cleanup-days 7
   ```
2. Use `--skip-git` to bypass branch creation

### "Failed to push branch"

**Cause**: No remote origin or network issues.

**Solutions**:
1. Add remote origin:
   ```bash
   git remote add origin <url>
   ```
2. Use `--skip-git` to skip push

## Experiment Issues

### "Stall limit reached"

**Cause**: No improvement after multiple iterations.

**Solutions**:
1. Increase `--stall-limit`:
   ```bash
   --stall-limit 10
   ```
2. Try a different optimization approach
3. Adjust `--target-improvement` to be more achievable

### "Iteration timeout exceeded"

**Cause**: Individual iteration takes too long.

**Solutions**:
1. Increase `--iteration-timeout-minutes`:
   ```bash
   --iteration-timeout-minutes 30
   ```
2. Optimize measurement command for faster feedback
3. Simplify the agent task

### "Total timeout exceeded"

**Cause**: Experiment runs too long overall.

**Solutions**:
1. Increase `--total-timeout-minutes`:
   ```bash
   --total-timeout-minutes 240
   ```
2. Reduce `--max-iterations`:
   ```bash
   --max-iterations 10
   ```
3. Use faster measurement command

### "Convergence achieved" (premature)

**Cause**: Metric converged before reaching target.

**Solutions**:
1. Decrease `--convergence-threshold`:
   ```bash
   --convergence-threshold 0.001
   ```
2. Increase `--convergence-window`:
   ```bash
   --convergence-window 5
   ```
3. Use a different metric

## Session File Issues

### "Session file not found"

**Cause**: Session file path doesn't exist or is empty.

**Solutions**:
1. Check session file path:
   ```bash
   ls -la autoresearch.jsonl
   ```
2. Use `--session-file` to specify custom path
3. Use `--history` to list existing experiments

### "Invalid session ID" (resume)

**Cause**: Session ID doesn't exist in session file.

**Solution**: List available sessions:
```bash
pi-autoresearch --history
```

## Performance Issues

### "Experiments run slowly"

**Solutions**:
1. Use faster measurement command
2. Reduce `--max-iterations`
3. Use `--quiet` to reduce output overhead
4. Check system resources (CPU, memory, disk I/O)

### "High memory usage"

**Solutions**:
1. Reduce `--convergence-window` size
2. Use smaller session files
3. Clean up old session files:
   ```bash
   rm -f autoresearch.jsonl
   ```

## Beads Integration Issues

### "bd: command not found"

**Solution**: Install beads or disable integration:
```bash
# Option 1: Install beads
# (follow beads installation instructions)

# Option 2: Disable beads integration
# Don't use --beads-enabled flag
```

### "Failed to create bead"

**Cause**: Beads server unavailable or authentication issues.

**Solutions**:
1. Check beads server status
2. Verify beads authentication
3. Disable beads integration temporarily

## Logging Issues

### "Too much output"

**Solution**: Use `--quiet` flag:
```bash
pi-autoresearch --quiet ...
```

### "Not enough output"

**Solution**: Use `--verbose` flag or set RUST_LOG:
```bash
# Option 1: Use verbose flag
pi-autoresearch --verbose ...

# Option 2: Set RUST_LOG environment variable
RUST_LOG=debug pi-autoresearch ...
```

## Common Error Messages

### "Design not approved"

**Cause**: Interactive prompt declined (not using `--auto-approve`).

**Solution**: Use `--auto-approve` for non-interactive mode:
```bash
pi-autoresearch --auto-approve ...
```

### "Question cannot be empty"

**Cause**: No question provided.

**Solution**: Provide a research question:
```bash
pi-autoresearch --question "How to optimize X?" ...
```

### "Empty measurement command"

**Cause**: `--measure` flag has empty value.

**Solution**: Provide valid measurement command:
```bash
pi-autoresearch --measure "echo 100.0" ...
```

## Getting Help

1. Check this troubleshooting guide first
2. Review documentation in `docs/` folder
3. Check specs in `specs/` folder
4. Run with `--help` for CLI options
5. Run with `--verbose` for detailed logs
6. Check GitHub issues for known problems

## Debug Mode

For detailed debugging, use:
```bash
RUST_LOG=debug pi-autoresearch --verbose ...
```

This shows all internal logging including:
- Configuration loading
- Metric detection
- Baseline verification
- Iteration details
- Git operations
- Session file operations

## Reporting Bugs

When reporting bugs, include:
1. pi-autoresearch version (`--version`)
2. Operating system and version
3. Rust version (`rustc --version`)
4. Command line used
5. Error message (full output)
6. Config file (if applicable)
7. Session file (if applicable)

## Related Documentation

- [Usage Guide](USAGE.md)
- [Configuration Guide](CONFIG.md)
- [Examples](EXAMPLES.md)
- [Coverage Guide](COVERAGE.md)
- [CLI Specification](../specs/CLI.md)
- [Session File Specification](../specs/SESSION.md)
- [Workflow Specification](../specs/WORKFLOW.md)

