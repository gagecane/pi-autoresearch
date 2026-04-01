# pi-autoresearch

An autonomous research experiment orchestrator that helps optimize metrics through iterative experimentation. Designed to work with AI coding agents to automate the process of defining research questions, running experiments, measuring improvements, and tracking results.

## Features

- 🎯 **Automated Experimentation**: Define a research question and let the system run iterative experiments
- 📊 **Metric Optimization**: Track and optimize any measurable metric (performance, memory, accuracy, etc.)
- 🔄 **Iterative Improvement**: Automatically runs multiple iterations, keeping improvements and reverting failures
- 💾 **Session Management**: Tracks all experiments in JSONL session files for reproducibility
- 🌿 **Git Integration**: Creates branches for successful experiments with detailed commit messages
- ⚙️ **Configuration Support**: Use config files for project-specific defaults
- 🏃 **Dry-Run Mode**: Preview changes without applying them
- 📈 **Experiment Comparison**: Compare results from different experimental runs
- 🧹 **Branch Cleanup**: Automatically clean up old autoresearch branches

## Installation

### Build from Source

```bash
# Clone the repository
git clone https://github.com/your-org/pi-autoresearch.git
cd pi-autoresearch

# Build the project
cargo build --release

# Install globally (optional)
cargo install --path .
```

### Requirements

- Rust 1.70 or later
- Cargo (Rust package manager)
- Git (for branch management features)

## Quick Start

### Basic Usage

```bash
# Run an experiment with a research question
pi-autoresearch --question "How can I reduce memory usage in the database module?" \
                --measure "peak_memory_mb" \
                --baseline 100.0 \
                --target-improvement 0.20
```

### With Configuration File

Create `~/.config/pi-autoresearch/config.json`:

```json
{
  "max_iterations": 10,
  "iteration_timeout_minutes": 30,
  "stall_limit": 3,
  "convergence_threshold": 0.01,
  "convergence_window": 5
}
```

Then run:

```bash
pi-autoresearch --question "Optimize query performance" --measure "execution_time_ms"
```

## Common Use Cases

### Performance Optimization

```bash
pi-autoresearch --question "How can I speed up the image processing pipeline?" \
                --measure "execution_time_ms" \
                --baseline 500.0 \
                --target-improvement 0.30
```

### Memory Optimization

```bash
pi-autoresearch --question "How can I reduce peak memory usage?" \
                --measure "peak_memory_mb" \
                --baseline 256.0 \
                --target-improvement 0.25
```

### Accuracy Improvement

```bash
pi-autoresearch --question "How can I improve model accuracy?" \
                --measure "accuracy_percent" \
                --baseline 85.0 \
                --target-improvement 0.10
```

## Command-Line Options

| Option | Description | Default |
|--------|-------------|---------|
| `--question` | Research question to explore | Required |
| `--measure` | Measurement command | Auto-detected |
| `--metric` | Metric name | Auto-detected |
| `--baseline` | Baseline value | Auto-measured |
| `--target-improvement` | Target improvement ratio (e.g., 0.30 for 30%) | 0.20 |
| `--max-iterations` | Maximum number of iterations | 20 |
| `--iteration-timeout-minutes` | Timeout per iteration | 60 |
| `--total-timeout-minutes` | Total experiment timeout | 480 |
| `--stall-limit` | Stall limit before backing off | 3 |
| `--convergence-threshold` | Convergence threshold | 0.01 |
| `--convergence-window` | Convergence window size | 5 |
| `--verify-baseline` | Verify baseline measurement | false |
| `--session-file` | Session file path | autoresearch.jsonl |
| `--config` | Config file path | ~/.config/pi-autoresearch/config.json |
| `--dry-run` | Simulate without applying changes | false |
| `--skip-git` | Skip git operations | false |
| `--verbose` | Verbose output | false |
| `--quiet` | Quiet mode | false |
| `--resume` | Resume previous experiment by ID | - |
| `--history` | List prior experiments | false |
| `--list-branches` | List autoresearch branches | false |
| `--cleanup-branches` | Clean up old branches | false |
| `--cleanup-days` | Branch age threshold for cleanup | 7 |
| `--compare-id1` | First experiment ID to compare | - |
| `--compare-id2` | Second experiment ID to compare | - |

## Documentation

- **[Usage Guide](docs/USAGE.md)** - Detailed usage instructions
- **[Configuration](docs/CONFIG.md)** - Configuration file options
- **[Examples](docs/EXAMPLES.md)** - Example use cases and workflows
- **[CLI Specification](specs/CLI.md)** - Complete CLI reference
- **[Session Format](specs/SESSION.md)** - Session file format specification
- **[Workflow](specs/WORKFLOW.md)** - Experiment workflow documentation

## Development

### Building

```bash
# Build for development
cargo build

# Build for release
cargo build --release
```

### Testing

```bash
# Run all tests
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Code Quality

```bash
# Check for clippy warnings
cargo clippy

# Format code
cargo fmt
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

For issues, questions, or feature requests, please open an issue on the [GitHub repository](https://github.com/your-org/pi-autoresearch/issues).

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- CLI parsing with [clap](https://github.com/clap-rs/clap)
- Serialization with [serde](https://github.com/serde-rs/serde)
- Time handling with [chrono](https://github.com/chronotope/chrono)
