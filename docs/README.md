# pi-autoresearch Documentation

pi-autoresearch is an autonomous research experiment orchestrator that helps optimize metrics through iterative experimentation.

> **Note**: This is the documentation for pi-autoresearch. For a quick overview, see the [main README](../README.md).

## Quick Start

### Installation from Source

```bash
# Clone the repository
git clone <repository-url>
cd pi-autoresearch

# Build the project
cargo build --release

# Install locally
cargo install --path .
```

### Development

To run in development mode without installing:

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run in development mode
cargo run -- <arguments>
```

### Basic Usage

```bash
# Run an experiment with a research question
pi-autoresearch --question "How can I reduce memory usage?"

# Specify metric and measurement command
pi-autoresearch \
  --question "How can I improve performance?" \
  --metric "execution_time_ms" \
  --measure "cargo bench --bench my_bench"

# Set baseline and target improvement
pi-autoresearch \
  --question "How can I reduce memory usage?" \
  --baseline 100.0 \
  --target-improvement 0.20
```

### Configuration

Create a config file at `~/.config/pi-autoresearch/config.json`:

```json
{
  "metric": "execution_time_ms",
  "measure": "cargo bench --bench my_bench",
  "baseline": 100.0,
  "target_improvement": 0.20,
  "max_iterations": 20,
  "max_variance": 0.05
}
```

### Features

- **Auto-approve**: Skip confirmation prompts with `--auto-approve`
- **Verify baseline**: Run baseline verification with `--verify-baseline`
- **Resume experiments**: Continue previous experiments with `--resume <SESSION_ID>`
- **View history**: List previous experiments with `--history`
- **Compare experiments**: Compare two experiments with `--compare-id1 <ID1> --compare-id2 <ID2>`
- **Dry run**: Preview changes without applying with `--dry-run`
- **Branch cleanup**: Clean up old branches with `--cleanup-branches`

## See Also

- [Usage Guide](USAGE.md) - Detailed usage instructions
- [Configuration](CONFIG.md) - Configuration file options
- [Examples](EXAMPLES.md) - Example use cases
