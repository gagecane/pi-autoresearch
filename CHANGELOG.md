# Changelog

All notable changes to pi-autoresearch will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Line and branch coverage testing infrastructure (Priority 34)
- Integration tests for `--auto-approve` flag (Priority 35)
- Unit tests for git-related functions (Priority 36)
- Integration tests for branch management features (Priority 37)
- Troubleshooting guide documentation (Priority 38)

### Improved
- Test coverage: 80.20% line coverage (3245 regions, 188 functions)
- Total test count: 160 tests (79 unit + 68 integration + 13 mutation)

## [0.1.0] - 2026-04-01

### Added
- Core autonomous research experiment orchestration
- CLI interface with 25+ options
- Config file support (`~/.config/pi-autoresearch/config.json`)
- Baseline verification with variance checking
- Iterative optimization loop with convergence detection
- Git branch management for experiments
- Session file persistence (JSONL format)
- Experiment comparison (`--compare-id1` and `--compare-id2`)
- History listing (`--history`)
- Resume functionality (`--resume`)
- Dry-run mode (`--dry-run`)
- Branch cleanup (`--cleanup-branches`)
- Branch listing (`--list-branches`)
- Beads (bd) integration for issue tracking
- Structured logging with tracing crate
- Version flag (`--version`)
- Performance benchmarks
- Contract tests for session file format
- Error handling edge case tests
- Mutation-resistant tests

### Features
- Automatic metric detection from research question
- Configurable convergence threshold and window
- Stall limit with backoff mechanism
- Iteration and total timeout support
- Multiple termination conditions (timeout, convergence, stall, max iterations)
- Comprehensive config validation
- Session file roundtrip serialization
- Backward compatibility with compact JSONL format
- Forward compatibility with pretty-printed JSON

### Documentation
- README.md with complete overview
- docs/USAGE.md - Detailed usage guide
- docs/CONFIG.md - Configuration documentation
- docs/EXAMPLES.md - Example use cases
- docs/COVERAGE.md - Coverage testing guide
- docs/TROUBLESHOOTING.md - Troubleshooting guide
- specs/CLI.md - CLI interface specification
- specs/SESSION.md - Session file format specification
- specs/CONFIG.md - Configuration file specification
- specs/WORKFLOW.md - Experiment workflow specification

### Testing
- 79 unit tests covering helper functions, validation, git operations
- 68 integration tests covering CLI, config, experiments, edge cases
- 13 mutation-resistant tests for test quality verification
- Performance benchmarks for key operations
- Contract tests for session file format compliance
- Error handling tests for comprehensive coverage

### Quality
- Zero clippy warnings
- Zero compiler warnings
- 80.20% line coverage
- 83.51% function coverage
- 78.37% region coverage

## Future Plans

### Planned Features
- More examples in docs/EXAMPLES.md
- Migration guide for config file changes
- API documentation for library users
- CONTRIBUTING.md for developer guidelines
- Performance regression tests
- Fuzzing tests for session file parsing
- End-to-end beads workflow integration tests

### Improvements
- Increase line coverage to 85% (target)
- Increase line coverage to 90% (ideal)
- Add more mutation testing coverage
- Improve error messages with suggestions
- Add progress bars for long operations

## [0.0.0] - Initial Development

### Added
- Initial project structure
- Basic CLI interface
- Core experiment orchestration logic

---

## Notes

- This changelog tracks all changes to pi-autoresearch
- Version 0.1.0 represents the first stable release
- All tasks from Priority 1-38 have been completed
- Test coverage exceeds minimum goal (80.20% > 75%)
- All 160 tests pass consistently

## Related Documentation

- [README.md](README.md) - Project overview
- [docs/USAGE.md](docs/USAGE.md) - Usage guide
- [docs/CONFIG.md](docs/CONFIG.md) - Configuration guide
- [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) - Troubleshooting guide
- [specs/WORKFLOW.md](specs/WORKFLOW.md) - Workflow specification

