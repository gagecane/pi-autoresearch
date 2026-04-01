# Contributing to pi-autoresearch

Thank you for your interest in contributing to pi-autoresearch! This document provides guidelines and instructions for contributing.

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on what's best for the community
- Welcome newcomers and answer questions

## Getting Started

### Prerequisites

- Rust toolchain (1.70+ recommended)
- Cargo package manager
- Git
- LLVM tools (for coverage): `rustup component add llvm-tools-preview`
- cargo-llvm-cov: `cargo install cargo-llvm-cov`

### Development Setup

```bash
# Clone the repository
git clone https://github.com/sikashep/pi-autoresearch.git
cd pi-autoresearch

# Build the project
cargo build

# Run tests
cargo test

# Run clippy
cargo clippy

# Run coverage
cargo llvm-cov
```

### Project Structure

```
pi-autoresearch/
├── src/
│   └── main.rs              # Main application code
├── tests/
│   ├── integration_tests.rs # Integration tests
│   ├── mutation_tests.rs    # Mutation-resistant tests
│   └── README.md           # Test documentation
├── benches/
│   └── benchmarks.rs       # Performance benchmarks
├── docs/
│   ├── README.md           # Documentation overview
│   ├── USAGE.md           # Usage guide
│   ├── CONFIG.md          # Configuration guide
│   ├── EXAMPLES.md        # Example use cases
│   ├── COVERAGE.md        # Coverage testing guide
│   └── TROUBLESHOOTING.md # Troubleshooting guide
├── specs/
│   ├── CLI.md             # CLI specification
│   ├── SESSION.md         # Session file specification
│   ├── CONFIG.md          # Config specification
│   └── WORKFLOW.md        # Workflow specification
├── scripts/
│   └── run-coverage.sh    # Coverage script
├── Cargo.toml             # Project manifest
├── CHANGELOG.md           # Version history
├── CONTRIBUTING.md        # This file
├── README.md              # Project overview
└── AGENTS.md              # Agent instructions
```

## How to Contribute

### Reporting Bugs

1. Check existing issues first
2. Create a new issue with:
   - Clear title and description
   - Steps to reproduce
   - Expected vs actual behavior
   - Environment details (OS, Rust version)
   - Error messages (full output)
   - pi-autoresearch version (`--version`)

### Suggesting Features

1. Check existing issues and proposals
2. Create an issue with:
   - Feature description
   - Use case and motivation
   - Proposed implementation (optional)
   - Potential impact

### Submitting Pull Requests

1. Fork the repository
2. Create a branch from main:
   ```bash
   git checkout -b feature/your-feature-name
   ```
3. Make your changes
4. Run tests:
   ```bash
   cargo test
   ```
5. Run clippy:
   ```bash
   cargo clippy -- -D warnings
   ```
6. Format code:
   ```bash
   cargo fmt
   ```
7. Commit with clear messages:
   ```bash
   git commit -m "feat: add new feature"
   ```
8. Push and create PR:
   ```bash
   git push origin feature/your-feature-name
   ```

### Commit Message Format

Follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` New features
- `fix:` Bug fixes
- `docs:` Documentation changes
- `test:` Test additions/changes
- `refactor:` Code refactoring
- `perf:` Performance improvements
- `chore:` Maintenance tasks

Example:
```bash
git commit -m "feat: add new --optimize flag for auto-tuning"
```

## Development Guidelines

### Code Style

- Follow Rust style guide
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Write descriptive variable and function names
- Add comments for complex logic

### Testing

- Write tests for new features
- Maintain 75%+ line coverage
- Add both unit and integration tests
- Run all tests before submitting PR

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run coverage
cargo llvm-cov
```

### Documentation

- Update README.md for user-facing changes
- Update docs/ for detailed documentation
- Update specs/ for API changes
- Add examples for new features
- Document breaking changes in CHANGELOG.md

### Code Review

- All PRs require review
- Address review feedback promptly
- Keep PRs focused and small
- Provide context for complex changes

## Areas Needing Contribution

### High Priority

- [ ] Performance regression tests (Priority 41)
- [ ] More examples in docs/EXAMPLES.md
- [ ] Migration guide for config changes
- [ ] API documentation for library users
- [ ] Fuzzing tests for session file parsing

### Medium Priority

- [ ] End-to-end beads workflow tests
- [ ] Progress bars for long operations
- [ ] Better error messages with suggestions
- [ ] Additional metrics for detection
- [ ] More optimization strategies

### Low Priority

- [ ] GUI frontend
- [ ] Cloud integration
- [ ] Plugin system
- [ ] Advanced analytics
- [ ] Multi-language support

## Getting Help

- Check documentation in docs/ folder
- Review specs in specs/ folder
- Read troubleshooting guide
- Ask in GitHub Discussions
- Create an issue for questions

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust By Example](https://doc.rust-lang.org/rust-by-example/)
- [Clippy Documentation](https://rust-lang.github.io/rust-clippy/)
- [Keep a Changelog](https://keepachangelog.com/)
- [Conventional Commits](https://www.conventionalcommits.org/)

## License

By contributing, you agree that your contributions will be licensed under the project's license.

## Acknowledgments

- Thanks to all contributors!
- Special thanks to users who report bugs and suggest features
- Community feedback drives this project forward

## Contact

- Issues: GitHub Issues
- Discussions: GitHub Discussions
- Email: (project maintainer contact)

---

Thank you for contributing to pi-autoresearch! 🚀

