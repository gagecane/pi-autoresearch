# pi-autoresearch API Documentation

This document provides a comprehensive reference for the pi-autoresearch library API. The library is designed to be used both as a CLI tool and as a Rust library for programmatic access to autonomous research experimentation.

## Table of Contents

- [Overview](#overview)
- [Modules](#modules)
- [Core Types](#core-types)
- [Usage Examples](#usage-examples)
- [Error Handling](#error-handling)

## Overview

The pi-autoresearch library provides a modular architecture for autonomous research experimentation:

```rust
use pi_autoresearch::*;

// Core modules
use pi_autoresearch::cli::Cli;
use pi_autoresearch::phase1_design::{ExperimentDesign, generate_design, BaselineRecord, BaselineRecordBuilder};
use pi_autoresearch::phase2_iterate::{IterationRecord, IterationConfig, IterationExecutor, IterationResult};
use pi_autoresearch::stuck_detector::{StuckReason, StuckDetector, StuckDetectorConfig, IterationState};
use pi_autoresearch::metric_evaluator::MetricEvaluator;
use pi_autoresearch::pi_agent::PiAgent;
use pi_autoresearch::session::{ExperimentSession, SessionManager, SessionRecord};
```

## Modules

### `cli`

Command-line interface definitions.

#### `Cli`

Struct representing the CLI arguments.

```rust
pub struct Cli {
    /// Research question to explore
    pub question: Option<String>,
    
    /// Auto-approve design without confirmation
    pub auto_approve: bool,
    
    /// Metric name for measurement
    pub metric: Option<String>,
    
    /// Measurement command
    pub measure: Option<String>,
    
    /// Baseline value
    pub baseline: Option<f64>,
    
    /// Target improvement ratio (e.g., 0.30 for 30%)
    pub target_improvement: Option<f64>,
    
    /// Maximum iterations
    pub max_iterations: Option<usize>,
    
    /// Iteration timeout in minutes
    pub iteration_timeout_minutes: Option<usize>,
    
    /// Total timeout in minutes
    pub total_timeout_minutes: Option<usize>,
    
    /// Stall limit before backing off
    pub stall_limit: Option<usize>,
    
    /// Convergence threshold
    pub convergence_threshold: Option<f64>,
    
    /// Convergence window size
    pub convergence_window: Option<usize>,
    
    /// Verify baseline measurement
    pub verify_baseline: bool,
    
    /// Session file path
    pub session_file: String,
    
    /// Maximum variance between baseline measurements
    pub max_variance: f64,
    
    /// Verbose output
    pub verbose: bool,
    
    /// Quiet mode
    pub quiet: bool,
    
    /// Resume a previous experiment by session ID
    pub resume: Option<String>,
    
    /// List prior experiments
    pub history: bool,
    
    /// Enable beads integration
    pub beads_enabled: bool,
    
    /// Skip git operations
    pub skip_git: bool,
    
    /// List all autoresearch branches
    pub list_branches: bool,
    
    /// Clean up old autoresearch branches
    pub cleanup_branches: bool,
    
    /// Only remove branches older than N days
    pub cleanup_days: usize,
    
    /// Dry run mode
    pub dry_run: bool,
    
    /// Path to config file
    pub config: Option<String>,
    
    /// First session ID for comparison
    pub compare_id1: Option<String>,
    
    /// Second session ID for comparison
    pub compare_id2: Option<String>,
}
```

**Traits Implemented:** `Parser`, `Debug`, `Default`

---

### `phase1_design`

Design and baseline verification for experiments.

#### `ExperimentDesign`

Represents the design of an experiment.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExperimentDesign {
    pub hypothesis: String,
    pub metric: String,
    pub measurement: String,
    pub baseline: f64,
    pub target_improvement: f64,
}
```

**Methods:**

```rust
impl ExperimentDesign {
    /// Create a new experiment design
    pub fn new(
        hypothesis: String,
        metric: String,
        measurement: String,
        baseline: f64,
        target_improvement: f64,
    ) -> Self
}
```

**Example:**

```rust
let design = ExperimentDesign::new(
    "Reduce memory usage in database module".to_string(),
    "peak_memory_mb".to_string(),
    "Run benchmark suite, capture peak RSS".to_string(),
    512.0,
    0.25, // 25% improvement target
);
```

#### `BaselineRecord`

Records baseline measurement data.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaselineRecord {
    pub timestamp: String,
    pub git_commit: String,
    pub metric: String,
    pub measurement_command: String,
    pub value: f64,
    pub verification_runs: Vec<f64>,
    pub variance: f64,
    pub within_threshold: bool,
}
```

**Methods:**

```rust
impl BaselineRecord {
    /// Create using builder pattern (recommended)
    pub fn builder() -> BaselineRecordBuilder
    
    /// Create directly (use with caution - many arguments)
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        timestamp: String,
        git_commit: String,
        metric: String,
        measurement_command: String,
        value: f64,
        verification_runs: Vec<f64>,
        variance: f64,
        within_threshold: bool,
    ) -> Self
}
```

**Example (Builder Pattern):**

```rust
let baseline = BaselineRecord::builder()
    .timestamp("2024-01-01T00:00:00Z".to_string())
    .git_commit("abc123".to_string())
    .metric("peak_memory_mb".to_string())
    .measurement_command("./benchmark".to_string())
    .value(512.0)
    .verification_runs(vec![512.0, 515.0])
    .variance(0.006)
    .within_threshold(true)
    .build();
```

**Example (Direct Construction):**

```rust
let baseline = BaselineRecord::new(
    "2024-01-01T00:00:00Z".to_string(),
    "abc123".to_string(),
    "peak_memory_mb".to_string(),
    "./benchmark".to_string(),
    512.0,
    vec![512.0, 515.0],
    0.006,
    true,
);
```

#### `BaselineRecordBuilder`

Builder for constructing `BaselineRecord` instances.

```rust
#[derive(Default)]
pub struct BaselineRecordBuilder
```

**Methods:**

```rust
impl BaselineRecordBuilder {
    pub fn timestamp(mut self, timestamp: String) -> Self
    pub fn git_commit(mut self, git_commit: String) -> Self
    pub fn metric(mut self, metric: String) -> Self
    pub fn measurement_command(mut self, measurement_command: String) -> Self
    pub fn value(mut self, value: f64) -> Self
    pub fn verification_runs(mut self, verification_runs: Vec<f64>) -> Self
    pub fn variance(mut self, variance: f64) -> Self
    pub fn within_threshold(mut self, within_threshold: bool) -> Self
    pub fn build(self) -> BaselineRecord
}
```

**Note:** All fields are required. Calling `build()` without setting all fields will panic with a descriptive error message.

#### `BaselineVerificationResult`

Result of baseline verification.

```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct BaselineVerificationResult {
    pub success: bool,
    pub baseline_record: Option<BaselineRecord>,
    pub error_message: Option<String>,
}
```

**Methods:**

```rust
impl BaselineVerificationResult {
    pub fn success(baseline_record: BaselineRecord) -> Self
    pub fn failure(error_message: String) -> Self
    pub fn failure_with_data(baseline_record: BaselineRecord, error_message: String) -> Self
}
```

#### `generate_design()`

Generates an experiment design based on a research question.

```rust
pub fn generate_design(question: &str) -> ExperimentDesign
```

**Parameters:**
- `question`: The research question to explore

**Returns:** An `ExperimentDesign` with auto-detected metric, measurement command, and baseline

**Metric Detection:**
- Contains "memory" → `peak_memory_mb` (baseline: 512.0)
- Contains "speed" or "performance" → `execution_time_ms` (baseline: 1000.0)
- Contains "accuracy" → `accuracy_percent` (baseline: 85.0)
- Default → `metric_value` (baseline: 100.0)

**Example:**

```rust
let design = generate_design("How can I reduce memory usage?");
assert_eq!(design.metric, "peak_memory_mb");
assert_eq!(design.baseline, 512.0);
assert_eq!(design.target_improvement, 0.30); // Default 30%
```

---

### `phase2_iterate`

Iteration execution and management.

#### `IterationRecord`

Records a single iteration's results.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IterationRecord {
    pub iteration: usize,
    pub timestamp: String,
    pub agent_action: String,
    pub metric_value: f64,
    pub improvement: f64,
    pub kept: bool,
}
```

**Methods:**

```rust
impl IterationRecord {
    pub fn new(
        iteration: usize,
        agent_action: String,
        metric_value: f64,
        improvement: f64,
        kept: bool,
    ) -> Self
}
```

#### `IterationConfig`

Configuration for the iteration loop.

```rust
#[derive(Debug, Clone)]
pub struct IterationConfig {
    pub max_iterations: usize,
    pub iteration_timeout_secs: u64,
    pub total_timeout_secs: u64,
    pub stall_limit: usize,
    pub convergence_threshold: f64,
    pub convergence_window: usize,
    pub verbose: bool,
    pub quiet: bool,
}
```

**Default Values:**

```rust
impl Default for IterationConfig {
    fn default() -> Self {
        Self {
            max_iterations: 20,
            iteration_timeout_secs: 600,    // 10 minutes
            total_timeout_secs: 7200,       // 2 hours
            stall_limit: 5,
            convergence_threshold: 0.01,    // 1%
            convergence_window: 3,
            verbose: false,
            quiet: false,
        }
    }
}
```

#### `IterationResult`

Result of running the iteration loop.

```rust
#[derive(Debug)]
pub struct IterationResult {
    pub iterations: Vec<IterationRecord>,
    pub best_iteration: Option<usize>,
    pub best_metric: f64,
    pub stuck_reason: Option<StuckReason>,
}
```

#### `IterationExecutor`

Executes the iteration loop.

```rust
pub struct IterationExecutor
```

**Methods:**

```rust
impl IterationExecutor {
    /// Create a new executor with the given configuration
    pub fn new(config: IterationConfig, max_variance: f64) -> Self
    
    /// Run a single iteration
    pub fn run_iteration(
        &self,
        iteration: usize,
        question: &str,
        baseline_value: f64,
        best_metric: f64,
        measure_command: &str,
    ) -> Result<IterationRecord>
    
    /// Run the full iteration loop
    pub fn run_loop(
        &self,
        question: &str,
        baseline_value: f64,
        measure_command: &str,
    ) -> Result<IterationResult>
}
```

**Example:**

```rust
let config = IterationConfig::default();
let executor = IterationExecutor::new(config, 0.05);

let result = executor.run_loop(
    "How can I improve performance?",
    1000.0,  // baseline
    "./benchmark",  // measurement command
)?;

println!("Best iteration: {:?}", result.best_iteration);
println!("Best metric: {:.2}", result.best_metric);
println!("Stuck reason: {:?}", result.stuck_reason);
```

---

### `stuck_detector`

Detects when experiments are stuck or should terminate.

#### `StuckReason`

Reasons for experiment termination.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StuckReason {
    IterationTimeout,
    StallLimitReached,
    TotalTimeout,
    ConvergenceAchieved,
    MaxIterationsReached,
}
```

**Display Implementations:**
- `IterationTimeout` → "Iteration timeout exceeded"
- `StallLimitReached` → "No improvement after multiple iterations (stall limit reached)"
- `TotalTimeout` → "Total experiment timeout exceeded"
- `ConvergenceAchieved` → "Metric convergence achieved"
- `MaxIterationsReached` → "Maximum iterations reached"

#### `IterationState`

Tracks the state of iterations.

```rust
#[derive(Debug, Clone)]
pub struct IterationState {
    pub current_iteration: usize,
    pub best_metric: f64,
    pub best_iteration: usize,
    pub consecutive_no_improvement: usize,
    pub backoff_count: usize,
    pub start_time: Instant,
    pub recent_metrics: Vec<f64>,
}
```

**Methods:**

```rust
impl IterationState {
    pub fn new(baseline_metric: f64) -> Self
    pub fn record_improvement(&mut self, iteration: usize, metric_value: f64)
    pub fn record_no_improvement(&mut self, iteration: usize, metric_value: f64)
    pub fn apply_backoff(&mut self)
    pub fn elapsed(&self) -> Duration
}
```

**Example:**

```rust
let mut state = IterationState::new(100.0);

// Record an improvement
state.record_improvement(1, 90.0);
assert_eq!(state.best_metric, 90.0);
assert_eq!(state.best_iteration, 1);

// Record no improvement
state.record_no_improvement(2, 95.0);
assert_eq!(state.consecutive_no_improvement, 1);

// Apply backoff
state.apply_backoff();
assert_eq!(state.consecutive_no_improvement, 0);
assert_eq!(state.backoff_count, 1);
```

#### `StuckDetectorConfig`

Configuration for stuck detection.

```rust
#[derive(Debug, Clone)]
pub struct StuckDetectorConfig {
    pub max_iterations: usize,
    pub iteration_timeout_secs: u64,
    pub total_timeout_secs: u64,
    pub stall_limit: usize,
    pub convergence_threshold: f64,
    pub convergence_window: usize,
}
```

**Default Values:**

```rust
impl Default for StuckDetectorConfig {
    fn default() -> Self {
        Self {
            max_iterations: 20,
            iteration_timeout_secs: 600,
            total_timeout_secs: 7200,
            stall_limit: 5,
            convergence_threshold: 0.01,
            convergence_window: 3,
        }
    }
}
```

#### `StuckDetector`

Detects when an experiment should stop.

```rust
pub struct StuckDetector { config: StuckDetectorConfig }
```

**Methods:**

```rust
impl StuckDetector {
    pub fn new(config: StuckDetectorConfig) -> Self
    
    pub fn check_total_timeout(&self, elapsed: Duration) -> Option<StuckReason>
    
    pub fn check_iteration_timeout(&self, elapsed: Duration) -> Option<StuckReason>
    
    pub fn check_max_iterations(&self, current: usize) -> Option<StuckReason>
    
    pub fn check_convergence(&self, recent_metrics: &[f64]) -> Option<StuckReason>
    
    pub fn check_stall_limit(
        &self,
        consecutive_no_improvement: usize,
        backoff_count: usize,
    ) -> Option<StuckReason>
    
    pub fn should_backoff(
        &self,
        consecutive_no_improvement: usize,
        backoff_count: usize,
    ) -> bool
}
```

**Example:**

```rust
let config = StuckDetectorConfig::default();
let detector = StuckDetector::new(config);

// Check for convergence
let metrics = vec![100.0, 100.5, 100.3];
match detector.check_convergence(&metrics) {
    Some(StuckReason::ConvergenceAchieved) => println!("Converged!"),
    None => println!("Still improving..."),
}

// Check if should backoff
if detector.should_backoff(5, 1) {
    println!("Should backoff");
}
```

---

### `metric_evaluator`

Evaluates metrics and executes measurements.

#### `MetricEvaluator`

Executes measurement commands and verifies baselines.

```rust
pub struct MetricEvaluator { max_variance: f64 }
```

**Methods:**

```rust
impl MetricEvaluator {
    pub fn new(max_variance: f64) -> Self
    
    pub fn execute_measurement(&self, command: &str) -> Result<f64>
    
    pub fn verify_baseline(
        &self,
        metric: &str,
        measurement_command: &str,
    ) -> Result<BaselineVerificationResult>
}
```

---

### `pi_agent`

AI agent for proposing changes.

#### `PiAgent`

Simulated AI agent that proposes changes.

```rust
pub struct PiAgent { _simulated: bool }
```

**Methods:**

```rust
impl PiAgent {
    pub fn new(_simulated: bool) -> Self
    
    pub fn propose_change(
        &self,
        question: &str,
        current_state: &str,
        metric_feedback: &str,
    ) -> String
}
```

**Default:**

```rust
impl Default for PiAgent {
    fn default() -> Self {
        Self::new(true)
    }
}
```

---

### `session`

Session management and persistence.

#### `ExperimentSession`

Represents a complete experiment session.

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExperimentSession {
    pub session_id: String,
    pub question: String,
    pub design: ExperimentDesign,
    pub baseline_record: BaselineRecord,
    pub iterations: Vec<IterationRecord>,
    pub best_iteration: Option<usize>,
    pub start_time: String,
    pub end_time: Option<String>,
    pub status: String,
}
```

**Methods:**

```rust
impl ExperimentSession {
    pub fn new(
        session_id: String,
        question: String,
        design: ExperimentDesign,
        baseline_record: BaselineRecord,
    ) -> Self
    
    pub fn add_iteration(&mut self, iteration: IterationRecord)
    
    pub fn finalize(&mut self, best_iteration: Option<usize>, status: String)
    
    pub fn calculate_final_improvement(&self) -> f64
}
```

**Example:**

```rust
let session = ExperimentSession::new(
    "session-123".to_string(),
    "Improve performance".to_string(),
    design,
    baseline,
);

// Add iterations
session.add_iteration(iteration1);
session.add_iteration(iteration2);

// Finalize
session.finalize(Some(2), "completed".to_string());

// Calculate improvement
let improvement = session.calculate_final_improvement();
println!("Final improvement: {:+.2}%", improvement * 100.0);
```

#### `SessionRecord`

Enum for different types of session records.

```rust
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum SessionRecord {
    Baseline(BaselineRecord),
    Iteration(IterationRecord),
    Experiment(Box<ExperimentSession>),
}
```

#### `SessionManager`

Manages session file persistence.

```rust
pub struct SessionManager { session_file: String }
```

**Methods:**

```rust
impl SessionManager {
    pub fn new(session_file: String) -> Self
    
    pub fn save_baseline(&self, record: &BaselineRecord) -> Result<()>
    
    pub fn save_iteration(&self, record: &IterationRecord) -> Result<()>
    
    pub fn save_session(&self, session: &ExperimentSession) -> Result<()>
    
    pub fn read_all(&self) -> Result<Vec<SessionRecord>>
    
    pub fn find_session(&self, session_id: &str) -> Result<Option<ExperimentSession>>
    
    pub fn list_history(&self) -> Result<String>
}
```

**Example:**

```rust
let manager = SessionManager::new("experiments.jsonl".to_string());

// Save baseline
manager.save_baseline(&baseline)?;

// Save iterations
for iteration in &iterations {
    manager.save_iteration(iteration)?;
}

// Save final session
manager.save_session(&session)?;

// List history
let history = manager.list_history()?;
println!("{}", history);
```

#### `generate_session_id()`

Generates a unique session ID.

```rust
pub fn generate_session_id() -> String
```

**Returns:** A unique hexadecimal string

---

## Usage Examples

### Complete Experiment Workflow

```rust
use pi_autoresearch::*;
use pi_autoresearch::phase1_design::{generate_design, BaselineRecord};
use pi_autoresearch::phase2_iterate::{IterationConfig, IterationExecutor};
use pi_autoresearch::session::{ExperimentSession, SessionManager};
use anyhow::Result;

fn run_experiment(question: &str, measure_command: &str) -> Result<()> {
    // Phase 1: Design
    let design = generate_design(question);
    println!("Design: {:?}", design);
    
    // Verify baseline
    let evaluator = MetricEvaluator::new(0.05);
    let baseline_result = evaluator.verify_baseline(&design.metric, measure_command)?;
    
    if !baseline_result.success {
        anyhow::bail!("Baseline verification failed: {:?}", baseline_result.error_message);
    }
    
    let baseline = baseline_result.baseline_record.unwrap();
    
    // Create session
    let session_id = generate_session_id();
    let mut session = ExperimentSession::new(
        session_id,
        question.to_string(),
        design.clone(),
        baseline.clone(),
    );
    
    // Save baseline
    let manager = SessionManager::new("experiments.jsonl".to_string());
    manager.save_baseline(&baseline)?;
    
    // Phase 2: Iterate
    let config = IterationConfig::default();
    let executor = IterationExecutor::new(config, 0.05);
    
    let iteration_result = executor.run_loop(question, baseline.value, measure_command)?;
    
    // Save iterations
    for iteration in &iteration_result.iterations {
        session.add_iteration(iteration.clone());
        manager.save_iteration(iteration)?;
    }
    
    // Finalize
    session.finalize(
        iteration_result.best_iteration,
        if iteration_result.best_metric < baseline.value {
            "completed".to_string()
        } else {
            "no_improvement".to_string()
        },
    );
    
    manager.save_session(&session)?;
    
    // Report results
    let improvement = session.calculate_final_improvement();
    println!("Experiment complete!");
    println!("Final improvement: {:+.2}%", improvement * 100.0);
    println!("Best iteration: {:?}", iteration_result.best_iteration);
    println!("Stuck reason: {:?}", iteration_result.stuck_reason);
    
    Ok(())
}
```

### Using the Stuck Detector Directly

```rust
use pi_autoresearch::stuck_detector::*;
use std::time::Duration;

fn main() {
    let config = StuckDetectorConfig {
        max_iterations: 10,
        iteration_timeout_secs: 60,
        total_timeout_secs: 300,
        stall_limit: 3,
        convergence_threshold: 0.01,
        convergence_window: 3,
    };
    
    let detector = StuckDetector::new(config);
    let mut state = IterationState::new(100.0);
    
    // Simulate iterations
    for i in 1..=10 {
        // Check total timeout
        if let Some(reason) = detector.check_total_timeout(state.elapsed()) {
            println!("Stopped: {}", reason);
            break;
        }
        
        // Record improvement or no improvement
        if i % 2 == 0 {
            state.record_improvement(i, 100.0 - (i as f64));
        } else {
            state.record_no_improvement(i, 100.0 + (i as f64));
        }
        
        // Check convergence
        if let Some(reason) = detector.check_convergence(&state.recent_metrics) {
            println!("Stopped: {}", reason);
            break;
        }
        
        // Check stall limit
        if detector.should_backoff(state.consecutive_no_improvement, state.backoff_count) {
            state.apply_backoff();
            println!("Backing off...");
        }
        
        if let Some(reason) = detector.check_stall_limit(
            state.consecutive_no_improvement,
            state.backoff_count,
        ) {
            println!("Stopped: {}", reason);
            break;
        }
    }
}
```

## Error Handling

The library uses `anyhow::Result` for error handling:

```rust
use anyhow::Result;

fn example() -> Result<()> {
    // Operations that can fail
    let value = execute_measurement("./benchmark")?;
    
    // Handle specific errors
    let result = verify_baseline("metric", "./benchmark");
    match result {
        Ok(baseline) => println!("Baseline: {:.2}", baseline.value),
        Err(e) => eprintln!("Failed: {}", e),
    }
    
    Ok(())
}
```

## See Also

- [Usage Guide](USAGE.md) - How to use the CLI
- [Configuration](CONFIG.md) - Config file options
- [Examples](EXAMPLES.md) - Example use cases
- [CLI Specification](../specs/CLI.md) - Complete CLI reference
- [Session Format](../specs/SESSION.md) - Session file format
- [Workflow](../specs/WORKFLOW.md) - Experiment workflow

