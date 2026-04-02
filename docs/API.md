# pi-autoresearch API Documentation

This document provides API reference for developers who want to use pi-autoresearch as a library or extend its functionality.

## Table of Contents

- [Overview](#overview)
- [Core Data Types](#core-data-types)
- [Configuration API](#configuration-api)
- [Experiment API](#experiment-api)
- [Session Management API](#session-management-api)
- [Git Integration API](#git-integration-api)
- [Agent Integration API](#agent-integration-api)
- [Error Types](#error-types)
- [Examples](#examples)

## Overview

pi-autoresearch is primarily designed as a CLI tool, but its internal functions can be used as a library for programmatic experiment orchestration. The API is organized into several categories:

- **Configuration**: Loading and validating config files
- **Experiment**: Running experiments and iterations
- **Session**: Managing session files and history
- **Git**: Branch management and version control
- **Agent**: Integration with pi agent for code changes

## Core Data Types

### Cli

Command-line interface arguments struct.

```rust
#[derive(Parser, Debug, Default)]
struct Cli {
    /// Research question to explore
    question: Option<String>,
    
    /// Auto-approve design without confirmation
    auto_approve: bool,
    
    /// Metric name for measurement
    metric: Option<String>,
    
    /// Measurement command
    measure: Option<String>,
    
    /// Baseline value
    baseline: Option<f64>,
    
    /// Target improvement ratio (e.g., 0.30 for 30%)
    target_improvement: Option<f64>,
    
    /// Maximum iterations
    max_iterations: Option<usize>,
    
    /// Iteration timeout in minutes
    iteration_timeout_minutes: Option<usize>,
    
    /// Total timeout in minutes
    total_timeout_minutes: Option<usize>,
    
    /// Stall limit before backing off
    stall_limit: Option<usize>,
    
    /// Convergence threshold
    convergence_threshold: Option<f64>,
    
    /// Convergence window size
    convergence_window: Option<usize>,
    
    /// Verify baseline measurement
    verify_baseline: bool,
    
    /// Session file path
    session_file: String,
    
    /// Maximum variance between baseline measurements
    max_variance: f64,
    
    /// Verbose output
    verbose: bool,
    
    /// Quiet mode
    quiet: bool,
    
    /// Resume a previous experiment by session ID
    resume: Option<String>,
    
    /// List prior experiments from session file
    history: bool,
    
    /// Enable beads integration
    beads_enabled: bool,
    
    /// Skip git operations
    skip_git: bool,
    
    /// Dry run mode
    dry_run: bool,
    
    /// Config file path
    config: Option<String>,
    
    /// Compare two experiments
    compare_id1: Option<String>,
    compare_id2: Option<String>,
    
    /// List autoresearch branches
    list_branches: bool,
    
    /// Cleanup old branches
    cleanup_branches: bool,
    
    /// Days threshold for branch cleanup
    cleanup_days: Option<usize>,
}
```

### Config

Configuration file structure.

```rust
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct Config {
    /// Metric name for measurement
    metric: Option<String>,
    
    /// Measurement command
    measure: Option<String>,
    
    /// Baseline value
    baseline: Option<f64>,
    
    /// Target improvement ratio
    target_improvement: Option<f64>,
    
    /// Maximum iterations
    max_iterations: Option<usize>,
    
    /// Maximum variance between baseline measurements
    max_variance: Option<f64>,
    
    /// Session file path
    session_file: Option<String>,
    
    /// Enable beads integration
    beads_enabled: Option<bool>,
    
    /// Iteration timeout in minutes
    iteration_timeout_minutes: Option<usize>,
    
    /// Total timeout in minutes
    total_timeout_minutes: Option<usize>,
    
    /// Stall limit before backing off
    stall_limit: Option<usize>,
    
    /// Convergence threshold
    convergence_threshold: Option<f64>,
    
    /// Convergence window size
    convergence_window: Option<usize>,
}
```

### ExperimentDesign

Design parameters for an experiment.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ExperimentDesign {
    /// The hypothesis for the experiment
    hypothesis: String,
    
    /// Metric being optimized
    metric: String,
    
    /// Measurement command
    measurement: String,
    
    /// Baseline value
    baseline: f64,
    
    /// Target improvement ratio (e.g., 0.30 for 30%)
    target_improvement: f64,
}
```

### BaselineRecord

Record of baseline measurement verification.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
struct BaselineRecord {
    /// Timestamp of the measurement
    timestamp: String,
    
    /// Git commit hash at time of measurement
    git_commit: String,
    
    /// Metric name
    metric: String,
    
    /// Measurement command executed
    measurement_command: String,
    
    /// Measured baseline value
    value: f64,
    
    /// All verification run values
    verification_runs: Vec<f64>,
    
    /// Variance between runs
    variance: f64,
    
    /// Whether variance is within threshold
    within_threshold: bool,
}
```

### IterationRecord

Record of a single iteration.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
struct IterationRecord {
    /// Iteration number
    iteration: usize,
    
    /// Timestamp of the iteration
    timestamp: String,
    
    /// Agent action taken
    agent_action: String,
    
    /// Measured metric value
    metric_value: f64,
    
    /// Improvement ratio (positive = improvement)
    improvement: f64,
    
    /// Whether changes were kept
    kept: bool,
}
```

### ExperimentSession

Complete experiment session with all iterations.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ExperimentSession {
    /// Unique session ID
    session_id: String,
    
    /// Research question
    question: String,
    
    /// Experiment design
    design: ExperimentDesign,
    
    /// Baseline measurement record
    baseline_record: BaselineRecord,
    
    /// All iteration records
    iterations: Vec<IterationRecord>,
    
    /// Best iteration number (if any)
    best_iteration: Option<usize>,
    
    /// Start timestamp
    start_time: String,
    
    /// End timestamp (None if still running)
    end_time: Option<String>,
    
    /// Final status (e.g., "completed", "no_improvement")
    status: String,
}
```

### SessionRecord

Enum for different types of session file records.

```rust
#[derive(Serialize, Deserialize, Debug)]
enum SessionRecord {
    /// Baseline record
    Baseline(BaselineRecord),
    /// Iteration record
    Iteration(IterationRecord),
    /// Complete experiment session (boxed for memory efficiency)
    Experiment(Box<ExperimentSession>),
}
```

### FinalizationResult

Result of experiment finalization.

```rust
#[derive(Serialize, Deserialize, Debug)]
struct FinalizationResult {
    /// Whether the experiment succeeded
    success: bool,
    
    /// Final improvement ratio
    final_improvement: f64,
    
    /// Best metric value achieved
    best_value: Option<f64>,
    
    /// Git branch name (if created)
    branch_name: Option<String>,
    
    /// Commit message (if created)
    commit_message: Option<String>,
    
    /// Key changes made
    key_changes: Vec<String>,
    
    /// Error message (if failed)
    error_message: Option<String>,
    
    /// Failure report (if failed)
    failure_report: Option<FailureReport>,
}
```

### FailureReport

Report generated when experiment fails.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
struct FailureReport {
    /// Best improvement ratio achieved
    best_improvement: f64,
    
    /// Best metric value achieved
    best_value: Option<f64>,
    
    /// Baseline value
    baseline: f64,
    
    /// Target improvement ratio
    target_improvement: f64,
    
    /// Number of iterations completed
    iterations_completed: usize,
    
    /// Reason for getting stuck (if any)
    stuck_reason: Option<String>,
    
    /// Recommendations for retry
    recommendations: Vec<String>,
}
```

### StuckReason

Reason why an experiment stopped.

```rust
#[derive(Debug, Clone)]
enum StuckReason {
    /// Iteration timeout exceeded
    IterationTimeout,
    
    /// Stall limit reached (no improvement after multiple iterations)
    StallLimitReached,
    
    /// Total experiment timeout exceeded
    TotalTimeout,
    
    /// Metric convergence achieved
    ConvergenceAchieved,
    
    /// Maximum iterations reached
    MaxIterationsReached,
}
```

## Configuration API

### init_logging

Initialize the logging subsystem.

```rust
fn init_logging(cli: &Cli) -> Result<()>
```

**Parameters:**
- `cli`: Command-line interface struct containing verbose/quiet flags

**Returns:**
- `Result<()>`: Ok on success, Err if logging initialization fails

**Description:**
Sets up tracing subscriber with appropriate log level based on CLI flags:
- `--quiet`: Only errors
- Default: Info and above
- `--verbose`: All logs including debug

Also respects `RUST_LOG` environment variable when CLI flags are not set.

### validate_config

Validate configuration file values.

```rust
fn validate_config(config: &Config) -> Result<(), ConfigValidationError>
```

**Parameters:**
- `config`: Configuration struct to validate

**Returns:**
- `Result<(), ConfigValidationError>`: Ok if valid, Err with specific validation errors

**Validation Rules:**
- `max_variance`: Must be between 0.0 and 1.0
- `target_improvement`: Must be positive
- `max_iterations`: Must be positive
- `iteration_timeout_minutes`: Must be positive
- `total_timeout_minutes`: Must be positive
- `stall_limit`: Must be positive
- `convergence_window`: Must be positive
- `session_file`: Must be a valid writable path

### load_config

Load configuration from file.

```rust
fn load_config(config_path: Option<&str>, explicit_config: bool) -> Result<Option<Config>>
```

**Parameters:**
- `config_path`: Optional explicit config file path (from `--config` flag)
- `explicit_config`: Whether config path was explicitly provided

**Returns:**
- `Result<Option<Config>>`: Ok(Some(config)) if found, Ok(None) if not found, Err on error

**Search Order:**
1. Explicit path if provided (`--config PATH`)
2. `~/.config/pi-autoresearch/config.json`
3. Return None if not found

### Helper Functions

Get effective values with precedence: CLI > config > default.

```rust
fn get_metric(cli: &Cli, config: &Option<Config>, default: &str) -> String
fn get_measure(cli: &Cli, config: &Option<Config>, default: &str) -> String
fn get_baseline(cli: &Cli, config: &Option<Config>, default: f64) -> f64
fn get_target_improvement(cli: &Cli, config: &Option<Config>, default: f64) -> f64
fn get_max_iterations(cli: &Cli, config: &Option<Config>, default: usize) -> usize
fn get_max_variance(cli: &Cli, _config: &Option<Config>, _default: f64) -> f64
fn get_session_file(cli: &Cli, _config: &Option<Config>) -> String
fn get_beads_enabled(cli: &Cli, config: &Option<Config>) -> bool
fn get_iteration_timeout(cli: &Cli, config: &Option<Config>, default: usize) -> usize
fn get_total_timeout(cli: &Cli, config: &Option<Config>, default: usize) -> usize
fn get_stall_limit(cli: &Cli, config: &Option<Config>, default: usize) -> usize
fn get_convergence_threshold(cli: &Cli, config: &Option<Config>, default: f64) -> f64
fn get_convergence_window(cli: &Cli, config: &Option<Config>, default: usize) -> usize
```

**Note:** `get_max_variance()` and `get_session_file()` always use CLI value (config file support intentionally not implemented for these).

## Experiment API

### generate_design

Generate experiment design using pi agent.

```rust
fn generate_design(question: &str) -> ExperimentDesign
```

**Parameters:**
- `question`: Research question to explore

**Returns:**
- `ExperimentDesign`: Complete design with approach and parameters

**Description:**
Invokes pi agent to generate an experimental design based on the research question.

### verify_baseline

Verify baseline measurement.

```rust
fn verify_baseline(
    metric: &str,
    measurement_command: &str,
    max_variance: f64,
) -> Result<BaselineVerificationResult>
```

**Parameters:**
- `metric`: Metric name for the measurement
- `measurement_command`: Command to execute for measurement
- `max_variance`: Maximum allowed variance between runs (0.0 to 1.0)

**Returns:**
- `Result<BaselineVerificationResult>`: Verification result with baseline record

**Description:**
Runs measurement command twice to verify baseline is stable and reproducible within the variance threshold.

### run_iteration

Run a single iteration of the experiment.

```rust
fn run_iteration(
    iteration: usize,
    question: &str,
    baseline_value: f64,
    best_metric: f64,
    measure_command: &str,
    session_file: &str,
    dry_run: bool,
) -> Result<(f64, String, bool)>
```

**Parameters:**
- `iteration`: Current iteration number
- `question`: Research question
- `baseline_value`: Baseline metric value
- `best_metric`: Best metric value so far
- `measure_command`: Command to execute for measurement
- `session_file`: Path to session file
- `dry_run`: Whether to skip actual changes

**Returns:**
- `Result<(f64, String, bool)>`: Tuple of (metric_value, agent_action, kept)

### run_iterative_loop

Run the main iterative optimization loop.

```rust
fn run_iterative_loop(
    question: &str,
    design: &ExperimentDesign,
    baseline_record: &BaselineRecord,
    cli: &Cli,
    config: &Option<Config>,
    beads: &mut Option<BeadsIntegration>,
    dry_run: bool,
) -> Result<(ExperimentSession, Option<StuckReason>)>
```

**Parameters:**
- `question`: Research question
- `design`: Experiment design parameters
- `baseline_record`: Verified baseline measurement
- `cli`: Command-line interface arguments
- `config`: Configuration file (if loaded)
- `beads`: Beads integration for issue tracking
- `dry_run`: Whether to skip actual changes

**Returns:**
- `Result<(ExperimentSession, Option<StuckReason>)>`: Tuple of (session, stuck_reason)

**Termination Conditions:**
- Maximum iterations reached
- Iteration timeout exceeded
- Total timeout exceeded
- Stall limit exceeded
- Convergence detected

### finalize_experiment

Finalize experiment and create git branch if successful.

```rust
fn finalize_experiment(
    session: &ExperimentSession,
    target_improvement: f64,
    stuck_reason: Option<&StuckReason>,
    skip_git: bool,
) -> Result<FinalizationResult>
```

**Parameters:**
- `session`: Completed experiment session
- `target_improvement`: Target improvement ratio
- `stuck_reason`: Reason experiment stopped (if any)
- `skip_git`: Whether to skip git operations

**Returns:**
- `Result<FinalizationResult>`: Finalization result with success status and details

## Session Management API

### save_to_session_file

Save a baseline record to the session file.

```rust
fn save_to_session_file(
    session_file: &str,
    record: &BaselineRecord,
) -> Result<()>
```

**Parameters:**
- `session_file`: Path to session file
- `record`: Baseline record to save

**Returns:**
- `Result<()>`: Ok on success

**Note:** Appends to file in JSONL format.

### read_session_file

Read all records from session file.

```rust
fn read_session_file(session_file: &str) -> Result<Vec<SessionRecord>>
```

**Parameters:**
- `session_file`: Path to session file

**Returns:**
- `Result<Vec<SessionRecord>>`: All records in the file

**Format:** Supports both compact JSONL and pretty-printed JSON formats.

### find_session_by_id

Find a specific session by ID.

```rust
fn find_session_by_id(session_file: &str, session_id: &str) -> Result<Option<ExperimentSession>>
```

**Parameters:**
- `session_file`: Path to session file
- `session_id`: Session ID to find

**Returns:**
- `Result<Option<ExperimentSession>>`: Found session or None

### list_history

List experiment history from session file.

```rust
fn list_history(session_file: &str) -> Result<()>
```

**Parameters:**
- `session_file`: Path to session file

**Returns:**
- `Result<()>: Ok on success

**Output:**
Displays all experiments with their IDs, questions, and results.

### compare_experiments

Compare two experiments.

```rust
fn compare_experiments(
    session_file: &str,
    session_id1: &str,
    session_id2: &str,
) -> Result<()>
```

**Parameters:**
- `session_file`: Path to session file
- `session_id1`: First session ID
- `session_id2`: Second session ID

**Returns:**
- `Result<()>`: Ok on success

**Output:**
Side-by-side comparison of two experiments with winner highlighted.

## Git Integration API

### generate_branch_name

Generate unique branch name for experiment.

```rust
fn generate_branch_name() -> String
```

**Returns:**
- `String`: Branch name in format `autoresearch/YYYYMMDD-HHMMSS-{uuid}`

### execute_git_operations

Execute all git operations for experiment.

```rust
fn execute_git_operations(branch_name: &str, commit_message: &str) -> Result<()>
```

**Parameters:**
- `branch_name`: Name of branch to create/checkout
- `commit_message`: Commit message

**Returns:**
- `Result<()>`: Ok on success

**Operations:**
1. Create or checkout branch
2. Stage all changes
3. Commit with message
4. Push to remote (if origin exists)

### get_current_branch

Get current git branch name.

```rust
fn get_current_branch() -> String
```

**Returns:**
- `String`: Current branch name or "HEAD" if detached

### does_branch_exist

Check if a branch exists.

```rust
fn does_branch_exist(branch_name: &str) -> bool
```

**Parameters:**
- `branch_name`: Branch name to check

**Returns:**
- `bool`: True if branch exists

### list_autoresearch_branches

List all autoresearch branches.

```rust
fn list_autoresearch_branches() -> Result<Vec<String>>
```

**Returns:**
- `Result<Vec<String>>`: List of local autoresearch branch names

### cleanup_autoresearch_branches

Cleanup old autoresearch branches.

```rust
fn cleanup_autoresearch_branches(days: usize) -> Result<usize>
```

**Parameters:**
- `days`: Only remove branches older than this many days

**Returns:**
- `Result<usize>`: Number of branches removed

## Agent Integration API

### invoke_pi_agent

Invoke pi agent with a question and context.

```rust
fn invoke_pi_agent(question: &str, current_state: &str, metric_feedback: &str) -> String
```

**Parameters:**
- `question`: Question to ask the agent
- `current_state`: Current experiment state
- `metric_feedback`: Feedback about metric trends

**Returns:**
- `String`: Agent response with recommended changes

### apply_changes_in_branch

Apply changes suggested by agent.

```rust
fn apply_changes_in_branch(action: &str) -> Result<String>
```

**Parameters:**
- `action`: Action/changes to apply

**Returns:**
- `Result<String>`: Branch name where changes were applied

### revert_changes

Revert changes in a branch.

```rust
fn revert_changes(branch_name: &str) -> Result<()>
```

**Parameters:**
- `branch_name`: Branch with changes to revert

**Returns:**
- `Result<()>`: Ok on success

### keep_changes

Keep changes in a branch.

```rust
fn keep_changes(branch_name: &str) -> Result<()>
```

**Parameters:**
- `branch_name`: Branch with changes to keep

**Returns:**
- `Result<()>`: Ok on success

## Error Types

### ConfigValidationError

Errors from config validation.

```rust
#[derive(Debug)]
enum ConfigValidationError {
    /// max_variance must be between 0.0 and 1.0
    MaxVarianceOutOfRange { value: f64 },
    
    /// target_improvement must be positive
    TargetImprovementNotPositive { value: f64 },
    
    /// max_iterations must be positive
    MaxIterationsNotPositive { value: usize },
    
    /// iteration_timeout_minutes must be positive
    IterationTimeoutNotPositive { value: usize },
    
    /// total_timeout_minutes must be positive
    TotalTimeoutNotPositive { value: usize },
    
    /// stall_limit must be positive
    StallLimitNotPositive { value: usize },
    
    /// convergence_window must be positive
    ConvergenceWindowNotPositive { value: usize },
    
    /// session_file path is not valid or writable
    SessionFileInvalidPath { path: String },
}
```

### BaselineError

Error from baseline measurement.

```rust
#[derive(Debug)]
struct BaselineError {
    /// Error message describing the issue
    message: String,
}
```

## Examples

### Example 1: Load Config and Run Experiment

```rust
use pi_autoresearch::{Cli, Config, load_config, validate_config};
use pi_autoresearch::{get_metric, get_measure, get_baseline};

fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();
    
    // Load config file
    let config = load_config(cli.config.as_deref(), cli.config.is_some())?;
    
    // Validate config if loaded
    if let Some(ref cfg) = config {
        validate_config(cfg)?;
    }
    
    // Get effective values (CLI > config > default)
    let metric = get_metric(&cli, &config, "performance");
    let measure = get_measure(&cli, &config, "cargo bench");
    let baseline = get_baseline(&cli, &config, 100.0);
    
    println!("Metric: {}, Measure: {}, Baseline: {}", metric, measure, baseline);
    
    Ok(())
}
```

### Example 2: Analyze Session File

```rust
use pi_autoresearch::{read_session_file, SessionRecord};

fn analyze_sessions(session_file: &str) -> Result<()> {
    let records = read_session_file(session_file)?;
    
    for record in records {
        match record {
            SessionRecord::Experiment(session) => {
                println!("Session: {}", session.session_id);
                println!("Question: {}", session.question);
                println!("Status: {}", session.status);
                if let Some(best_iter) = session.best_iteration {
                    println!("Best iteration: {}", best_iter);
                }
            }
            SessionRecord::Baseline(baseline) => {
                println!("Baseline value: {}", baseline.value);
                println!("Variance: {:.2}%", baseline.variance * 100.0);
            }
            SessionRecord::Iteration(iter) => {
                println!("Iteration {}: {:.2} (improvement: {:+.2}%, kept: {})", 
                    iter.iteration, iter.metric_value, iter.improvement * 100.0, iter.kept);
            }
        }
    }
    
    Ok(())
}
```

### Example 3: Generate Experiment Design

```rust
use pi_autoresearch::generate_design;

fn main() -> Result<()> {
    let question = "How can I reduce memory usage in my application?";
    
    let design = generate_design(question);
    
    println!("Hypothesis: {}", design.hypothesis);
    println!("Metric: {}", design.metric);
    println!("Measurement: {}", design.measurement);
    println!("Baseline: {}", design.baseline);
    println!("Target Improvement: {:.0}%", design.target_improvement * 100.0);
    
    Ok(())
}
```

## Related Documentation

- [USAGE.md](USAGE.md) - Detailed usage guide
- [CONFIG.md](CONFIG.md) - Configuration file documentation
- [EXAMPLES.md](EXAMPLES.md) - Example use cases
- [specs/SESSION.md](../specs/SESSION.md) - Session file format specification
- [specs/CLI.md](../specs/CLI.md) - CLI interface specification

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines on extending the API.

## Version

This documentation is for pi-autoresearch version 0.1.0.

