# pi-autoresearch API Documentation

This document provides API reference for developers who want to use pi-autoresearch as a library.

## Table of Contents

- [Overview](#overview)
- [Public Modules](#public-modules)
- [Core Types](#core-types)
- [CLI Interface](#cli-interface)
- [Experiment Design](#experiment-design)
- [Iteration Management](#iteration-management)
- [Stuck Detection](#stuck-detection)
- [Session Management](#session-management)
- [Metric Evaluation](#metric-evaluation)
- [PI Agent Integration](#pi-agent-integration)
- [Usage Examples](#usage-examples)

---

## Overview

The pi-autoresearch library provides a modular API for autonomous research experiments. You can use individual components or the full experiment workflow.

```rust
use pi_autoresearch::*;
```

---

## Public Modules

### `cli`

Command-line interface definitions and utilities.

### `phase1_design`

Experiment design and baseline verification.

### `phase2_iterate`

Iteration management and record keeping.

### `stuck_detector`

Detection of stalled or stuck experiments.

### `metric_evaluator`

Metric measurement and evaluation.

### `pi_agent`

PI agent integration for code modifications.

### `session`

Session file management and persistence.

---

## Core Types

### `Cli`

Command-line interface struct with all configuration options.

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
    
    /// Enable beads integration
    pub beads_enabled: bool,
    
    /// Skip git operations
    pub skip_git: bool,
    
    /// Dry run mode
    pub dry_run: bool,
}
```

**Methods:**

- `impl Default for Cli` - Creates a default CLI configuration
- `fn parse()` - Parses command-line arguments (via clap)

---

## Experiment Design

### `ExperimentDesign`

Represents the design of an experiment.

```rust
pub struct ExperimentDesign {
    pub question: String,
    pub metric: String,
    pub measure: String,
    pub baseline: f64,
    pub target_improvement: f64,
    pub max_iterations: usize,
    pub strategies: Vec<String>,
    pub risks: Vec<String>,
    pub validation_steps: Vec<String>,
}
```

### `generate_design()`

Generates an experiment design based on the research question.

```rust
pub fn generate_design(
    question: &str,
    metric: &str,
    baseline: f64,
    target_improvement: f64,
) -> Result<ExperimentDesign>
```

**Parameters:**
- `question` - The research question to explore
- `metric` - The metric to optimize
- `baseline` - The baseline value
- `target_improvement` - Target improvement ratio (0.0 to 1.0)

**Returns:**
- `Ok(ExperimentDesign)` - The generated experiment design
- `Err(anyhow::Error)` - If design generation fails

**Example:**

```rust
use pi_autoresearch::{generate_design, ExperimentDesign};

let design: ExperimentDesign = generate_design(
    "How can I reduce memory usage?",
    "peak_memory_mb",
    100.0,
    0.20,
)?;

println!("Strategies: {:?}", design.strategies);
```

---

## Iteration Management

### `IterationRecord`

Represents a single iteration in an experiment.

```rust
pub struct IterationRecord {
    pub iteration: usize,
    pub timestamp: String,
    pub metric_value: f64,
    pub improvement: f64,
    pub is_improvement: bool,
    pub agent_action: String,
    pub changes_applied: bool,
    pub changes_kept: bool,
    pub git_commit: Option<String>,
    pub duration_seconds: f64,
    pub notes: Option<String>,
}
```

**Fields:**
- `iteration` - Iteration number (1-indexed)
- `timestamp` - ISO 8601 timestamp
- `metric_value` - Measured metric value
- `improvement` - Improvement percentage from baseline
- `is_improvement` - Whether this iteration improved the metric
- `agent_action` - Description of what the agent did
- `changes_applied` - Whether changes were applied
- `changes_kept` - Whether changes were kept (not reverted)
- `git_commit` - Git commit hash if changes were committed
- `duration_seconds` - Duration of the iteration
- `notes` - Additional notes about the iteration

---

## Stuck Detection

### `StuckDetector`

Detects when an experiment is stuck or should terminate.

```rust
pub struct StuckDetector {
    pub config: StuckDetectorConfig,
    pub state: IterationState,
}
```

### `StuckDetectorConfig`

Configuration for the stuck detector.

```rust
pub struct StuckDetectorConfig {
    pub max_iterations: usize,
    pub iteration_timeout: Duration,
    pub total_timeout: Duration,
    pub stall_limit: usize,
    pub convergence_threshold: f64,
    pub convergence_window: usize,
}
```

### `IterationState`

Current state of the iteration process.

```rust
pub struct IterationState {
    pub current_iteration: usize,
    pub best_metric: f64,
    pub best_iteration: usize,
    pub consecutive_no_improvement: usize,
    pub backoff_count: usize,
    pub recent_metrics: Vec<f64>,
}
```

**Methods:**

```rust
impl StuckDetector {
    pub fn new(config: StuckDetectorConfig) -> Self
    
    pub fn update(&mut self, metric_value: f64, iteration: usize)
    
    pub fn is_stuck(&self) -> bool
    
    pub fn get_stuck_reason(&self) -> Option<StuckReason>
    
    pub fn should_backoff(&self) -> bool
}
```

### `StuckReason`

Reason why an experiment is stuck.

```rust
pub enum StuckReason {
    /// Maximum iterations reached
    MaxIterationsReached,
    
    /// Iteration timeout exceeded
    IterationTimeout,
    
    /// Total experiment timeout exceeded
    TotalTimeout,
    
    /// No improvement after multiple iterations
    StallLimitReached,
    
    /// Metric convergence achieved
    ConvergenceAchieved,
}
```

**Example:**

```rust
use pi_autoresearch::{StuckDetector, StuckDetectorConfig, IterationState};
use std::time::Duration;

let config = StuckDetectorConfig {
    max_iterations: 20,
    iteration_timeout: Duration::from_secs(600),
    total_timeout: Duration::from_secs(3600),
    stall_limit: 5,
    convergence_threshold: 0.01,
    convergence_window: 3,
};

let mut detector = StuckDetector::new(config);

// Update with each iteration
for (i, metric) in metrics.iter().enumerate() {
    detector.update(*metric, i + 1);
    
    if detector.is_stuck() {
        println!("Experiment stuck: {:?}", detector.get_stuck_reason());
        break;
    }
}
```

---

## Session Management

### `ExperimentSession`

Represents a complete experiment session.

```rust
pub struct ExperimentSession {
    pub session_id: String,
    pub question: String,
    pub metric: String,
    pub measure: String,
    pub baseline: f64,
    pub target_improvement: f64,
    pub start_time: String,
    pub end_time: Option<String>,
    pub status: String,
    pub iterations: Vec<IterationRecord>,
    pub best_improvement: f64,
    pub best_iteration: usize,
    pub final_metric: f64,
    pub stuck_reason: Option<StuckReason>,
    pub git_branch: Option<String>,
    pub git_commit: Option<String>,
}
```

**Methods:**

```rust
impl ExperimentSession {
    pub fn new(
        session_id: String,
        question: String,
        metric: String,
        measure: String,
        baseline: f64,
        target_improvement: f64,
    ) -> Self
    
    pub fn add_iteration(&mut self, record: IterationRecord)
    
    pub fn get_best_iteration(&self) -> Option<&IterationRecord>
    
    pub fn is_complete(&self) -> bool
}
```

---

## Metric Evaluation

### `MetricEvaluator`

Evaluates metrics and verifies baselines.

```rust
pub struct MetricEvaluator {
    pub metric: String,
    pub measure: String,
    pub baseline: f64,
    pub max_variance: f64,
}
```

**Methods:**

```rust
impl MetricEvaluator {
    pub fn new(metric: String, measure: String, baseline: f64) -> Self
    
    pub fn default() -> Self
    
    pub fn execute_measurement(&self) -> Result<f64>
    
    pub fn verify_baseline(&self) -> Result<BaselineRecord>
    
    pub fn get_git_commit_hash(&self) -> Result<String>
}
```

### `MetricError`

Error type for metric evaluation.

```rust
pub enum MetricError {
    /// Command execution failed
    CommandFailed(String),
    
    /// Output could not be parsed as a number
    ParseError(String),
    
    /// Baseline variance exceeded threshold
    VarianceExceeded { expected: f64, actual: f64, threshold: f64 },
    
    /// Measurement command is empty
    EmptyCommand,
}

impl std::fmt::Display for MetricError
impl std::error::Error for MetricError
```

**Example:**

```rust
use pi_autoresearch::MetricEvaluator;

let evaluator = MetricEvaluator::new(
    "execution_time_ms".to_string(),
    "cargo bench --bench my_bench".to_string(),
    500.0,
);

let baseline_record = evaluator.verify_baseline()?;
println!("Verified baseline: {}", baseline_record.metric_value);
```

---

## PI Agent Integration

### `invoke_pi_agent()`

Invokes the PI agent to generate code modifications.

```rust
pub fn invoke_pi_agent(
    question: &str,
    current_code: &str,
    strategies: &[String],
) -> Result<String>
```

**Parameters:**
- `question` - The research question
- `current_code` - Current code to modify
- `strategies` - Optimization strategies to consider

**Returns:**
- `Ok(String)` - Agent response with proposed changes
- `Err(anyhow::Error)` - If agent invocation fails

---

## Usage Examples

### Example 1: Simple Experiment

```rust
use pi_autoresearch::{Cli, ExperimentSession, generate_design};

fn main() -> anyhow::Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();
    
    // Generate experiment design
    let design = generate_design(
        &cli.question.unwrap_or_default(),
        &cli.metric.unwrap_or_default(),
        cli.baseline.unwrap_or(100.0),
        cli.target_improvement.unwrap_or(0.20),
    )?;
    
    println!("Experiment Design:");
    println!("  Question: {}", design.question);
    println!("  Metric: {}", design.metric);
    println!("  Baseline: {}", design.baseline);
    println!("  Target: {:.1}%", design.target_improvement * 100.0);
    
    Ok(())
}
```

### Example 2: Custom Stuck Detection

```rust
use pi_autoresearch::{StuckDetector, StuckDetectorConfig, StuckReason};
use std::time::Duration;

fn run_custom_experiment() -> anyhow::Result<()> {
    let config = StuckDetectorConfig {
        max_iterations: 15,
        iteration_timeout: Duration::from_secs(300),
        total_timeout: Duration::from_secs(1800),
        stall_limit: 4,
        convergence_threshold: 0.005,
        convergence_window: 3,
    };
    
    let mut detector = StuckDetector::new(config);
    let mut metrics = vec![100.0, 95.0, 92.0, 90.0, 88.0, 87.0, 86.5];
    
    for (i, metric) in metrics.iter().enumerate() {
        detector.update(*metric, i + 1);
        
        println!("Iteration {}: metric = {}", i + 1, metric);
        
        if detector.is_stuck() {
            match detector.get_stuck_reason() {
                Some(StuckReason::ConvergenceAchieved) => {
                    println!("✓ Converged at iteration {}", i + 1);
                }
                Some(StuckReason::StallLimitReached) => {
                    println!("⚠ Stalled at iteration {}", i + 1);
                }
                _ => {
                    println!("✗ Stopped at iteration {}", i + 1);
                }
            }
            break;
        }
    }
    
    Ok(())
}
```

### Example 3: Session Management

```rust
use pi_autoresearch::{ExperimentSession, IterationRecord};

fn manage_session() -> anyhow::Result<()> {
    let mut session = ExperimentSession::new(
        "session-001".to_string(),
        "How to improve performance?".to_string(),
        "execution_time_ms".to_string(),
        "cargo bench".to_string(),
        500.0,
        0.30,
    );
    
    // Add iterations
    for i in 1..=5 {
        let record = IterationRecord {
            iteration: i,
            timestamp: chrono::Utc::now().to_rfc3339(),
            metric_value: 500.0 - (i as f64 * 10.0),
            improvement: (i as f64 * 2.0),
            is_improvement: true,
            agent_action: format!("Optimized iteration {}", i),
            changes_applied: true,
            changes_kept: true,
            git_commit: None,
            duration_seconds: 10.0,
            notes: None,
        };
        
        session.add_iteration(record);
    }
    
    println!("Best improvement: {:.1}%", session.best_improvement);
    println!("Best iteration: {}", session.best_iteration);
    
    Ok(())
}
```

### Example 4: Metric Evaluation

```rust
use pi_autoresearch::MetricEvaluator;

fn evaluate_metric() -> anyhow::Result<()> {
    let evaluator = MetricEvaluator::new(
        "throughput_rps".to_string(),
        "./benchmark.sh".to_string(),
        1000.0,
    );
    
    // Execute measurement
    let value = evaluator.execute_measurement()?;
    println!("Measured value: {}", value);
    
    // Verify baseline
    let baseline = evaluator.verify_baseline()?;
    println!("Baseline verified: {}", baseline.metric_value);
    
    Ok(())
}
```

---

## Error Handling

All public functions return `Result<T, anyhow::Error>` for consistent error handling.

```rust
use anyhow::Result;

fn my_function() -> Result<()> {
    // Handle errors with ? operator
    let design = generate_design("question", "metric", 100.0, 0.20)?;
    
    // Or match on errors
    match metric_evaluator.verify_baseline() {
        Ok(baseline) => println!("Baseline: {}", baseline.metric_value),
        Err(e) => eprintln!("Failed to verify baseline: {}", e),
    }
    
    Ok(())
}
```

---

## See Also

- [USAGE.md](./USAGE.md) - Complete usage guide
- [CONFIG.md](./CONFIG.md) - Configuration file reference
- [EXAMPLES.md](./EXAMPLES.md) - Example use cases
- [TROUBLESHOOTING.md](./TROUBLESHOOTING.md) - Common issues and solutions
- [specs/](../specs/) - Technical specifications
- [README.md](../README.md) - Project overview

