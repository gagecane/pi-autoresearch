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
    /// The research question
    question: String,
    
    /// Metric being optimized
    metric: String,
    
    /// Measurement command
    measure: String,
    
    /// Baseline value
    baseline: f64,
    
    /// Target improvement percentage
    target_improvement: f64,
    
    /// Maximum number of iterations
    max_iterations: usize,
    
    /// Generated design/approach
    design: String,
}
```

### BaselineRecord

Record of baseline measurement verification.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
struct BaselineRecord {
    /// Session ID
    session_id: String,
    /// Question being explored
    question: String,
    /// Metric name
    metric: String,
    /// Measurement command
    measure: String,
    /// Baseline value
    baseline: f64,
    /// Target improvement
    target_improvement: f64,
    /// Maximum iterations
    max_iterations: usize,
    /// Timestamp
    timestamp: String,
}
```

### IterationRecord

Record of a single iteration.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
struct IterationRecord {
    /// Session ID
    session_id: String,
    /// Iteration number
    iteration: usize,
    /// Measured value
    value: f64,
    /// Improvement percentage
    improvement: f64,
    /// Whether this was the best iteration
    is_best: bool,
    /// Timestamp
    timestamp: String,
    /// Agent action taken
    agent_action: String,
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
    /// Metric name
    metric: String,
    /// Measurement command
    measure: String,
    /// Baseline value
    baseline: f64,
    /// Target improvement
    target_improvement: f64,
    /// Maximum iterations
    max_iterations: usize,
    /// All iterations
    iterations: Vec<IterationRecord>,
    /// Best improvement achieved
    best_improvement: f64,
    /// Final status
    status: String,
    /// Start timestamp
    started_at: String,
    /// End timestamp
    ended_at: Option<String>,
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
    /// Final improvement percentage
    final_improvement: f64,
    /// Number of iterations completed
    iterations_completed: usize,
    /// Git branch name (if created)
    branch_name: Option<String>,
    /// Recommendations for next steps
    recommendations: Vec<String>,
}
```

### FailureReport

Report generated when experiment fails.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
struct FailureReport {
    /// Research question
    question: String,
    /// Metric name
    metric: String,
    /// Baseline value
    baseline: f64,
    /// Target improvement
    target_improvement: f64,
    /// Best improvement achieved
    best_improvement: f64,
    /// Number of iterations attempted
    iterations_completed: usize,
    /// Reason for failure
    stuck_reason: StuckReason,
    /// Recommendations
    recommendations: Vec<String>,
}
```

### StuckReason

Reason why an experiment got stuck.

```rust
#[derive(Debug, Clone)]
enum StuckReason {
    /// Maximum iterations reached
    MaxIterationsReached,
    /// Iteration timeout exceeded
    IterationTimeout,
    /// Total timeout exceeded
    TotalTimeout,
    /// Stall limit exceeded
    StallLimitReached,
    /// Convergence detected
    Convergence,
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
    measure: &str,
    expected_baseline: f64,
    max_variance: f64,
    max_retries: usize,
) -> Result<BaselineVerificationResult>
```

**Parameters:**
- `measure`: Measurement command to execute
- `expected_baseline`: Expected baseline value
- `max_variance`: Maximum allowed variance between measurements
- `max_retries`: Maximum number of measurement attempts

**Returns:**
- `Result<BaselineVerificationResult>`: Verification result with measured values

**Description:**
Runs measurement command multiple times to verify baseline is stable and reproducible.

### run_iteration

Run a single iteration of the experiment.

```rust
fn run_iteration(
    iteration: usize,
    session_id: &str,
    measure: &str,
    baseline: f64,
    current_state: &str,
    metric_feedback: &str,
    session_file: &str,
    iteration_timeout: Duration,
    verbose: bool,
) -> Result<IterationState>
```

**Parameters:**
- `iteration`: Current iteration number
- `session_id`: Unique session identifier
- `measure`: Measurement command
- `baseline`: Baseline value for comparison
- `current_state`: Current experiment state
- `metric_feedback`: Feedback about metric trends
- `session_file`: Path to session file
- `iteration_timeout`: Maximum time for this iteration
- `verbose`: Whether to show verbose output

**Returns:**
- `Result<IterationState>`: State after iteration completion

### run_iterative_loop

Run the main iterative optimization loop.

```rust
fn run_iterative_loop(
    design: ExperimentDesign,
    session_id: String,
    session_file: String,
    iteration_timeout: Duration,
    total_timeout: Duration,
    stall_limit: usize,
    convergence_threshold: f64,
    convergence_window: usize,
    verbose: bool,
    dry_run: bool,
    skip_git: bool,
) -> Result<ExperimentSession>
```

**Parameters:**
- `design`: Experiment design parameters
- `session_id`: Unique session identifier
- `session_file`: Path to session file
- `iteration_timeout`: Maximum time per iteration
- `total_timeout`: Maximum total experiment time
- `stall_limit`: Number of stalled iterations before backoff
- `convergence_threshold`: Threshold for convergence detection
- `convergence_window`: Window size for convergence detection
- `verbose`: Whether to show verbose output
- `dry_run`: Whether to skip actual changes
- `skip_git`: Whether to skip git operations

**Returns:**
- `Result<ExperimentSession>`: Complete session with all iterations

**Termination Conditions:**
- Target improvement reached
- Maximum iterations reached
- Total timeout exceeded
- Stall limit exceeded
- Convergence detected

### finalize_experiment

Finalize experiment and create git branch.

```rust
fn finalize_experiment(
    session: &ExperimentSession,
    target_improvement: f64,
    skip_git: bool,
    dry_run: bool,
) -> Result<FinalizationResult>
```

**Parameters:**
- `session`: Completed experiment session
- `target_improvement`: Target improvement percentage
- `skip_git`: Whether to skip git operations
- `dry_run`: Whether to skip actual changes

**Returns:**
- `Result<FinalizationResult>`: Finalization result with recommendations

## Session Management API

### save_to_session_file

Save a record to the session file.

```rust
fn save_to_session_file(session_file: &str, record: SessionRecord) -> Result<()>
```

**Parameters:**
- `session_file`: Path to session file
- `record`: Record to save (Baseline, Iteration, or Experiment)

**Returns:**
- `Result<()>`: Ok on success

### read_session_file

Read all records from session file.

```rust
fn read_session_file(session_file: &str) -> Result<Vec<SessionRecord>>
```

**Parameters:**
- `session_file`: Path to session file

**Returns:**
- `Result<Vec<SessionRecord>>`: All records in the file

**Format:**
Supports both compact JSONL and pretty-printed JSON formats.

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
- `Result<Vec<String>>`: List of branch names with ages

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
    InvalidMaxVariance(f64),
    /// target_improvement must be positive
    InvalidTargetImprovement(f64),
    /// max_iterations must be positive
    InvalidMaxIterations(usize),
    /// iteration_timeout must be positive
    InvalidIterationTimeout(usize),
    /// total_timeout must be positive
    InvalidTotalTimeout(usize),
    /// stall_limit must be positive
    InvalidStallLimit(usize),
    /// convergence_window must be positive
    InvalidConvergenceWindow(usize),
    /// session_file must be a valid path
    InvalidSessionPath(String),
}
```

### BaselineError

Errors from baseline verification.

```rust
#[derive(Debug)]
struct BaselineError {
    /// Expected baseline value
    expected: f64,
    /// Measured value
    measured: f64,
    /// Variance percentage
    variance: f64,
}
```

## Examples

### Example 1: Load Config and Run Experiment

```rust
use pi_autoresearch::{Cli, Config, load_config, validate_config, run_experiment};

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
    
    // Run experiment
    let result = run_experiment(&cli, &config)?;
    
    Ok(())
}
```

### Example 2: Custom Experiment Loop

```rust
use pi_autoresearch::{
    ExperimentDesign, run_iteration, run_iterative_loop,
    get_iteration_timeout, get_total_timeout,
};

fn custom_experiment(design: ExperimentDesign) -> Result<ExperimentSession> {
    let session_id = uuid_generate();
    let session_file = "custom_session.jsonl";
    
    let iteration_timeout = Duration::from_secs(300); // 5 minutes
    let total_timeout = Duration::from_secs(3600); // 1 hour
    
    let session = run_iterative_loop(
        design,
        session_id,
        session_file.to_string(),
        iteration_timeout,
        total_timeout,
        5, // stall_limit
        0.01, // convergence_threshold
        3, // convergence_window
        true, // verbose
        false, // dry_run
        false, // skip_git
    )?;
    
    Ok(session)
}
```

### Example 3: Session File Analysis

```rust
use pi_autoresearch::{read_session_file, SessionRecord, ExperimentSession};

fn analyze_sessions(session_file: &str) -> Result<()> {
    let records = read_session_file(session_file)?;
    
    for record in records {
        match record {
            SessionRecord::Experiment(session) => {
                println!("Session: {}", session.session_id);
                println!("Question: {}", session.question);
                println!("Best Improvement: {}%", session.best_improvement);
                println!("Status: {}", session.status);
            }
            SessionRecord::Baseline(baseline) => {
                println!("Baseline: {}", baseline.baseline);
            }
            SessionRecord::Iteration(iter) => {
                println!("Iteration {}: {}%", iter.iteration, iter.improvement);
            }
        }
    }
    
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

