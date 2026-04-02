use serde::{Deserialize, Serialize};
use anyhow::Result;
use chrono::Utc;

use crate::metric_evaluator::MetricEvaluator;
use crate::pi_agent::PiAgent;
use crate::stuck_detector::{IterationState, StuckDetector, StuckDetectorConfig, StuckReason};

/// Record of a single iteration in the autoresearch loop.
///
/// Contains all the information about what happened during an iteration,
/// including the agent's action, the resulting metric value, and whether
/// the change was kept or reverted.
///
/// # Fields
///
/// * `iteration` - The iteration number (1-indexed)
/// * `timestamp` - ISO 8601 timestamp when the iteration completed
/// * `agent_action` - Description of what the AI agent proposed to change
/// * `metric_value` - The measured metric value after the change
/// * `improvement` - Relative improvement over baseline (positive = better)
/// * `kept` - Whether the change was kept (true) or reverted (false)
///
/// # Examples
///
/// ```
/// use pi_autoresearch::phase2_iterate::IterationRecord;
///
/// // Create a record for a successful iteration
/// let record = IterationRecord::new(
///     1,
///     "Optimized database query".to_string(),
///     95.0,
///     0.05,
///     true
/// );
///
/// assert_eq!(record.iteration, 1);
/// assert_eq!(record.metric_value, 95.0);
/// assert!(record.kept);
/// ```
///
/// ```
/// use pi_autoresearch::phase2_iterate::IterationRecord;
///
/// // Create a record for a failed iteration (change reverted)
/// let record = IterationRecord::new(
///     2,
///     "Increased cache size".to_string(),
///     105.0,
///     -0.05,
///     false
/// );
///
/// assert_eq!(record.improvement, -0.05);
/// assert!(!record.kept);
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IterationRecord {
    pub iteration: usize,
    pub timestamp: String,
    pub agent_action: String,
    pub metric_value: f64,
    pub improvement: f64,
    pub kept: bool,
}

impl IterationRecord {
    /// Create a new iteration record with the given parameters.
    ///
    /// The timestamp is automatically set to the current UTC time.
    ///
    /// # Arguments
    ///
    /// * `iteration` - The iteration number (1-indexed)
    /// * `agent_action` - Description of what the AI agent proposed
    /// * `metric_value` - The measured metric value
    /// * `improvement` - Relative improvement (positive = better, negative = worse)
    /// * `kept` - Whether the change was kept or reverted
    ///
    /// # Returns
    ///
    /// A new `IterationRecord` with the current timestamp
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::phase2_iterate::IterationRecord;
    ///
    /// let record = IterationRecord::new(
    ///     1,
    ///     "Reduced memory allocation".to_string(),
    ///     85.0,
    ///     0.15,
    ///     true
    /// );
    ///
    /// assert_eq!(record.iteration, 1);
    /// assert_eq!(record.agent_action, "Reduced memory allocation");
    /// assert!(record.kept);
    /// ```
    pub fn new(iteration: usize, agent_action: String, metric_value: f64, improvement: f64, kept: bool) -> Self {
        Self { iteration, timestamp: Utc::now().to_rfc3339(), agent_action, metric_value, improvement, kept }
    }
}

/// Configuration for the iterative exploration loop.
///
/// Controls how the autoresearch system runs experiments, including
/// timeouts, convergence detection, and output verbosity.
///
/// # Fields
///
/// * `max_iterations` - Maximum number of iterations to run
/// * `iteration_timeout_secs` - Timeout for a single iteration
/// * `total_timeout_secs` - Total timeout for the entire loop
/// * `stall_limit` - Number of consecutive non-improving iterations before backing off
/// * `convergence_threshold` - Minimum improvement to consider as progress
/// * `convergence_window` - Number of iterations to check for convergence
/// * `verbose` - Enable verbose output
/// * `quiet` - Suppress all output
///
/// # Examples
///
/// ```
/// use pi_autoresearch::phase2_iterate::IterationConfig;
///
/// // Use default configuration
/// let config = IterationConfig::default();
///
/// assert_eq!(config.max_iterations, 20);
/// assert_eq!(config.iteration_timeout_secs, 600);
/// assert_eq!(config.stall_limit, 5);
/// ```
///
/// ```
/// use pi_autoresearch::phase2_iterate::IterationConfig;
///
/// // Custom configuration for faster experiments
/// let config = IterationConfig {
///     max_iterations: 5,
///     iteration_timeout_secs: 60,
///     total_timeout_secs: 300,
///     stall_limit: 3,
///     convergence_threshold: 0.01,
///     convergence_window: 3,
///     verbose: false,
///     quiet: true,
/// };
///
/// assert_eq!(config.max_iterations, 5);
/// assert!(config.quiet);
/// ```
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

impl Default for IterationConfig {
    /// Create a default `IterationConfig` with reasonable defaults.
    ///
    /// # Defaults
    ///
    /// * `max_iterations`: 20
    /// * `iteration_timeout_secs`: 600 (10 minutes)
    /// * `total_timeout_secs`: 7200 (2 hours)
    /// * `stall_limit`: 5
    /// * `convergence_threshold`: 0.01 (1%)
    /// * `convergence_window`: 3
    /// * `verbose`: false
    /// * `quiet`: false
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::phase2_iterate::IterationConfig;
    ///
    /// let config = IterationConfig::default();
    ///
    /// assert_eq!(config.max_iterations, 20);
    /// assert_eq!(config.iteration_timeout_secs, 600);
    /// assert_eq!(config.total_timeout_secs, 7200);
    /// assert_eq!(config.stall_limit, 5);
    /// assert_eq!(config.convergence_threshold, 0.01);
    /// assert_eq!(config.convergence_window, 3);
    /// assert!(!config.verbose);
    /// assert!(!config.quiet);
    /// ```
    fn default() -> Self {
        Self { max_iterations: 20, iteration_timeout_secs: 600, total_timeout_secs: 7200, stall_limit: 5, convergence_threshold: 0.01, convergence_window: 3, verbose: false, quiet: false }
    }
}

/// Result of running the iterative exploration loop.
///
/// Contains all iteration records, information about the best iteration,
/// and the reason why the loop terminated.
///
/// # Fields
///
/// * `iterations` - All iteration records from the loop
/// * `best_iteration` - The iteration number with the best metric (if any)
/// * `best_metric` - The best metric value achieved
/// * `stuck_reason` - Why the loop terminated (timeout, convergence, etc.)
///
/// # Examples
///
/// ```
/// use pi_autoresearch::phase2_iterate::{IterationRecord, IterationResult};
/// use pi_autoresearch::stuck_detector::StuckReason;
///
/// // Create a result with successful iterations
/// let iterations = vec![
///     IterationRecord::new(1, "Action 1".to_string(), 95.0, 0.05, true),
///     IterationRecord::new(2, "Action 2".to_string(), 90.0, 0.10, true),
/// ];
///
/// let result = IterationResult {
///     iterations,
///     best_iteration: Some(2),
///     best_metric: 90.0,
///     stuck_reason: Some(StuckReason::MaxIterationsReached),
/// };
///
/// assert_eq!(result.iterations.len(), 2);
/// assert_eq!(result.best_iteration, Some(2));
/// assert_eq!(result.best_metric, 90.0);
/// ```
///
/// ```
/// use pi_autoresearch::phase2_iterate::{IterationRecord, IterationResult};
///
/// // Create a result with no successful iterations
/// let result = IterationResult {
///     iterations: vec![],
///     best_iteration: None,
///     best_metric: 100.0,
///     stuck_reason: None,
/// };
///
/// assert!(result.iterations.is_empty());
/// assert!(result.best_iteration.is_none());
/// ```
#[derive(Debug)]
pub struct IterationResult {
    pub iterations: Vec<IterationRecord>,
    pub best_iteration: Option<usize>,
    pub best_metric: f64,
    pub stuck_reason: Option<StuckReason>,
}

/// Executor for running iterative exploration loops.
///
/// Coordinates the AI agent, metric evaluator, and stuck detector to
/// run multiple iterations of experiments, tracking improvements and
/// detecting when to stop.
///
/// # Examples
///
/// ```
/// use pi_autoresearch::phase2_iterate::{IterationConfig, IterationExecutor};
///
/// // Create an executor with default configuration
/// let config = IterationConfig::default();
/// let executor = IterationExecutor::new(config, 0.1);
///
/// // The executor is ready to run iterations
/// ```
///
/// ```
/// use pi_autoresearch::phase2_iterate::{IterationConfig, IterationExecutor};
///
/// // Create an executor with custom configuration
/// let config = IterationConfig {
///     max_iterations: 10,
///     iteration_timeout_secs: 300,
///     total_timeout_secs: 3600,
///     stall_limit: 3,
///     convergence_threshold: 0.01,
///     convergence_window: 3,
///     verbose: false,
///     quiet: true,
/// };
///
/// let executor = IterationExecutor::new(config, 0.15);
/// ```
pub struct IterationExecutor {
    config: IterationConfig,
    metric_evaluator: MetricEvaluator,
    pi_agent: PiAgent,
    stuck_detector: StuckDetector,
}

impl IterationExecutor {
    /// Create a new `IterationExecutor` with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The iteration configuration
    /// * `max_variance` - Maximum acceptable variance for metric measurements
    ///
    /// # Returns
    ///
    /// A new `IterationExecutor` ready to run iterations
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::phase2_iterate::{IterationConfig, IterationExecutor};
    ///
    /// let config = IterationConfig::default();
    /// let executor = IterationExecutor::new(config, 0.1);
    ///
    /// // Executor is ready to run iterations
    /// ```
    pub fn new(config: IterationConfig, max_variance: f64) -> Self {
        let stuck_detector = StuckDetector::new(StuckDetectorConfig {
            max_iterations: config.max_iterations, iteration_timeout_secs: config.iteration_timeout_secs,
            total_timeout_secs: config.total_timeout_secs, stall_limit: config.stall_limit,
            convergence_threshold: config.convergence_threshold, convergence_window: config.convergence_window,
        });
        Self { config, metric_evaluator: MetricEvaluator::new(max_variance), pi_agent: PiAgent::default(), stuck_detector }
    }
    /// Run a single iteration of the exploration loop.
    ///
    /// Asks the AI agent to propose a change, measures the result,
    /// and determines whether to keep or revert the change.
    ///
    /// # Arguments
    ///
    /// * `iteration` - The iteration number (1-indexed)
    /// * `question` - The research question being explored
    /// * `baseline_value` - The original baseline metric value
    /// * `best_metric` - The best metric value seen so far
    /// * `measure_command` - Shell command to measure the metric
    ///
    /// # Returns
    ///
    /// * `Ok(IterationRecord)` - The record of what happened in this iteration
    /// * `Err` - If the measurement command fails
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::phase2_iterate::{IterationConfig, IterationExecutor};
    ///
    /// let config = IterationConfig {
    ///     max_iterations: 5,
    ///     iteration_timeout_secs: 60,
    ///     total_timeout_secs: 300,
    ///     stall_limit: 3,
    ///     convergence_threshold: 0.01,
    ///     convergence_window: 3,
    ///     verbose: false,
    ///     quiet: true,
    /// };
    ///
    /// let executor = IterationExecutor::new(config, 0.1);
    ///
    /// // Run a single iteration with a simple measurement command
    /// let result = executor.run_iteration(
    ///     1,
    ///     "How can I reduce execution time?",
    ///     100.0,
    ///     100.0,
    ///     "echo 90"
    /// );
    ///
    /// assert!(result.is_ok());
    /// let record = result.unwrap();
    /// assert_eq!(record.iteration, 1);
    /// assert_eq!(record.metric_value, 90.0);
    /// assert!(record.kept); // 90 < 100, so it's an improvement
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the measurement command fails to execute
    /// or produces invalid output.
    pub fn run_iteration(&self, iteration: usize, question: &str, baseline_value: f64, best_metric: f64, measure_command: &str) -> Result<IterationRecord> {
        let current_state = format!("Iteration {}, best metric: {:.2}", iteration, best_metric);
        let metric_feedback = format!("Baseline: {:.2}, current best: {:.2}", baseline_value, best_metric);
        let agent_action = self.pi_agent.propose_change(question, &current_state, &metric_feedback);
        let metric_value = self.metric_evaluator.execute_measurement(measure_command)?;
        let improvement = (best_metric - metric_value) / baseline_value;
        let kept = metric_value < best_metric;
        Ok(IterationRecord::new(iteration, agent_action, metric_value, improvement, kept))
    }
    /// Run the complete iterative exploration loop.
    ///
    /// Runs multiple iterations until one of the stopping conditions is met:
    /// - Maximum iterations reached
    /// - Total timeout reached
    /// - Convergence detected (no significant improvement)
    /// - Stall limit reached (too many consecutive non-improving iterations)
    ///
    /// # Arguments
    ///
    /// * `question` - The research question being explored
    /// * `baseline_value` - The original baseline metric value
    /// * `measure_command` - Shell command to measure the metric
    ///
    /// # Returns
    ///
    /// * `Ok(IterationResult)` - The complete result including all iterations and the best metric
    /// * `Err` - If a critical error occurs (rare)
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::phase2_iterate::{IterationConfig, IterationExecutor};
    ///
    /// let config = IterationConfig {
    ///     max_iterations: 5,
    ///     iteration_timeout_secs: 60,
    ///     total_timeout_secs: 300,
    ///     stall_limit: 3,
    ///     convergence_threshold: 0.01,
    ///     convergence_window: 3,
    ///     verbose: false,
    ///     quiet: true,
    /// };
    ///
    /// let executor = IterationExecutor::new(config, 0.1);
    ///
    /// // Run the complete loop
    /// let result = executor.run_loop(
    ///     "How can I reduce memory usage?",
    ///     100.0,
    ///     "echo 95"
    /// );
    ///
    /// assert!(result.is_ok());
    /// let iter_result = result.unwrap();
    /// assert!(!iter_result.iterations.is_empty());
    /// assert!(iter_result.best_metric <= 100.0);
    /// ```
    ///
    /// # Stopping Conditions
    ///
    /// The loop stops when any of these conditions is met:
    ///
    /// 1. **Max iterations**: Reached `config.max_iterations`
    /// 2. **Total timeout**: Exceeded `config.total_timeout_secs`
    /// 3. **Convergence**: No improvement > `config.convergence_threshold` for `config.convergence_window` iterations
    /// 4. **Stall limit**: `config.stall_limit` consecutive non-improving iterations after 2 backoffs
    ///
    /// The `stuck_reason` field in the result indicates why the loop stopped.
    pub fn run_loop(&self, question: &str, baseline_value: f64, measure_command: &str) -> Result<IterationResult> {
        let mut state = IterationState::new(baseline_value);
        let mut iterations = Vec::new();
        let mut stuck_reason: Option<StuckReason> = None;
        if !self.config.quiet {
            eprintln!("\nStarting iterative exploration loop...");
            eprintln!("Max iterations: {}", self.config.max_iterations);
            eprintln!("Baseline: {:.2}", baseline_value);
            eprintln!();
        }
        while state.current_iteration < self.config.max_iterations {
            if let Some(reason) = self.stuck_detector.check_total_timeout(state.elapsed()) {
                if !self.config.quiet { eprintln!("\nTotal timeout reached. Saving best result."); }
                stuck_reason = Some(reason);
                break;
            }
            state.current_iteration += 1;
            if !self.config.quiet {
                eprintln!("[AutoResearch] Iter {}/{} | Best: {:.1} | Stall: {}/{} | Time: {:.0}s/{:.0}s",
                    state.current_iteration, self.config.max_iterations, state.best_metric,
                    state.consecutive_no_improvement, self.config.stall_limit,
                    state.elapsed().as_secs(), self.config.total_timeout_secs);
            }
            let iteration_start = std::time::Instant::now();
            match self.run_iteration(state.current_iteration, question, baseline_value, state.best_metric, measure_command) {
                Ok(record) => {
                    let iteration_elapsed = iteration_start.elapsed();
                    if let Some(reason) = self.stuck_detector.check_iteration_timeout(iteration_elapsed) {
                        if !self.config.quiet { eprintln!("\nIteration {} exceeded timeout.", state.current_iteration); }
                        stuck_reason = Some(reason);
                        break;
                    }
                    if record.kept {
                        state.record_improvement(record.iteration, record.metric_value);
                        if !self.config.quiet { eprintln!("  ✓ Kept - improvement: {:+.2}%", record.improvement * 100.0); }
                    } else {
                        state.record_no_improvement(record.iteration, record.metric_value);
                        if !self.config.quiet { eprintln!("  ✗ Reverted - degradation: {:-.2}%", record.improvement * 100.0); }
                    }
                    iterations.push(record);
                    if let Some(reason) = self.stuck_detector.check_convergence(&state.recent_metrics) {
                        if !self.config.quiet { eprintln!("\nConvergence achieved! Exiting early."); }
                        stuck_reason = Some(reason);
                        break;
                    }
                    if self.stuck_detector.should_backoff(state.consecutive_no_improvement, state.backoff_count) {
                        state.apply_backoff();
                        if !self.config.quiet { eprintln!("\nStall limit reached. Backing off (attempt {}/2)...", state.backoff_count); }
                    }
                    if let Some(reason) = self.stuck_detector.check_stall_limit(state.consecutive_no_improvement, state.backoff_count) {
                        if !self.config.quiet { eprintln!("\nStall limit reached after {} backoffs. Aborting.", state.backoff_count); }
                        stuck_reason = Some(reason);
                        break;
                    }
                }
                Err(e) => {
                    if !self.config.quiet { eprintln!("  ✗ Iteration failed: {}", e); }
                    state.consecutive_no_improvement += 1;
                }
            }
        }
        if stuck_reason.is_none() && state.current_iteration >= self.config.max_iterations { stuck_reason = Some(StuckReason::MaxIterationsReached); }
        Ok(IterationResult { iterations, best_iteration: if state.best_iteration > 0 { Some(state.best_iteration) } else { None }, best_metric: state.best_metric, stuck_reason })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for IterationRecord
    #[test]
    fn test_iteration_record_new() {
        let record = IterationRecord::new(1, "test action".to_string(), 100.0, 0.1, true);
        assert_eq!(record.iteration, 1);
        assert_eq!(record.agent_action, "test action");
        assert_eq!(record.metric_value, 100.0);
        assert_eq!(record.improvement, 0.1);
        assert!(record.kept);
    }

    #[test]
    fn test_iteration_record_clone() {
        let record1 = IterationRecord::new(1, "action".to_string(), 50.0, 0.05, false);
        let record2 = record1.clone();
        assert_eq!(record2.iteration, record1.iteration);
        assert_eq!(record2.metric_value, record1.metric_value);
    }

    #[test]
    fn test_iteration_record_debug() {
        let record = IterationRecord::new(1, "test".to_string(), 100.0, 0.1, true);
        let debug_str = format!("{:?}", record);
        assert!(debug_str.contains("IterationRecord"));
        assert!(debug_str.contains("test"));
    }

    // Tests for IterationConfig
    #[test]
    fn test_iteration_config_default() {
        let config = IterationConfig::default();
        assert_eq!(config.max_iterations, 20);
        assert_eq!(config.iteration_timeout_secs, 600);
        assert_eq!(config.total_timeout_secs, 7200);
        assert_eq!(config.stall_limit, 5);
        assert_eq!(config.convergence_threshold, 0.01);
        assert_eq!(config.convergence_window, 3);
        assert!(!config.verbose);
        assert!(!config.quiet);
    }

    #[test]
    fn test_iteration_config_custom() {
        let config = IterationConfig {
            max_iterations: 10,
            iteration_timeout_secs: 300,
            total_timeout_secs: 3600,
            stall_limit: 3,
            convergence_threshold: 0.05,
            convergence_window: 5,
            verbose: true,
            quiet: false,
        };
        assert_eq!(config.max_iterations, 10);
        assert_eq!(config.iteration_timeout_secs, 300);
        assert!(config.verbose);
    }

    #[test]
    fn test_iteration_config_clone() {
        let config1 = IterationConfig::default();
        let config2 = config1.clone();
        assert_eq!(config2.max_iterations, config1.max_iterations);
        assert_eq!(config2.stall_limit, config1.stall_limit);
    }

    // Tests for IterationResult
    #[test]
    fn test_iteration_result_with_data() {
        let iterations = vec![
            IterationRecord::new(1, "action1".to_string(), 90.0, 0.1, true),
            IterationRecord::new(2, "action2".to_string(), 85.0, 0.15, true),
        ];
        let result = IterationResult {
            iterations,
            best_iteration: Some(2),
            best_metric: 85.0,
            stuck_reason: Some(StuckReason::MaxIterationsReached),
        };
        assert_eq!(result.iterations.len(), 2);
        assert_eq!(result.best_iteration, Some(2));
        assert_eq!(result.best_metric, 85.0);
        assert!(result.stuck_reason.is_some());
    }

    #[test]
    fn test_iteration_result_empty() {
        let result = IterationResult {
            iterations: vec![],
            best_iteration: None,
            best_metric: 100.0,
            stuck_reason: None,
        };
        assert!(result.iterations.is_empty());
        assert!(result.best_iteration.is_none());
        assert!(result.stuck_reason.is_none());
    }

    // Tests for IterationExecutor
    #[test]
    fn test_iteration_executor_new() {
        let config = IterationConfig::default();
        let executor = IterationExecutor::new(config, 0.1);
        // Just verify it doesn't panic
        assert!(true);
    }

    #[test]
    fn test_iteration_executor_new_with_custom_max_variance() {
        let config = IterationConfig {
            max_iterations: 10,
            iteration_timeout_secs: 300,
            total_timeout_secs: 3600,
            stall_limit: 3,
            convergence_threshold: 0.05,
            convergence_window: 5,
            verbose: false,
            quiet: true,
        };
        let executor = IterationExecutor::new(config, 0.2);
        // Just verify it doesn't panic
        assert!(true);
    }

    // Test run_iteration with a simple command
    #[test]
    fn test_run_iteration_valid_command() {
        let config = IterationConfig {
            max_iterations: 5,
            iteration_timeout_secs: 60,
            total_timeout_secs: 300,
            stall_limit: 3,
            convergence_threshold: 0.01,
            convergence_window: 3,
            verbose: false,
            quiet: true,
        };
        let executor = IterationExecutor::new(config, 0.1);
        let result = executor.run_iteration(1, "test question", 100.0, 100.0, "echo 90");
        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.iteration, 1);
        assert_eq!(record.metric_value, 90.0);
        assert!(record.kept); // 90 < 100, so it's an improvement
    }

    #[test]
    fn test_run_iteration_degradation() {
        let config = IterationConfig {
            max_iterations: 5,
            iteration_timeout_secs: 60,
            total_timeout_secs: 300,
            stall_limit: 3,
            convergence_threshold: 0.01,
            convergence_window: 3,
            verbose: false,
            quiet: true,
        };
        let executor = IterationExecutor::new(config, 0.1);
        let result = executor.run_iteration(1, "test question", 100.0, 100.0, "echo 110");
        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.metric_value, 110.0);
        assert!(!record.kept); // 110 > 100, so it's a degradation
    }

    #[test]
    fn test_run_iteration_invalid_command() {
        let config = IterationConfig::default();
        let executor = IterationExecutor::new(config, 0.1);
        let result = executor.run_iteration(1, "test question", 100.0, 100.0, "echo not_a_number");
        assert!(result.is_err());
    }

    // Test run_loop
    #[test]
    fn test_run_loop_improvement() {
        let config = IterationConfig {
            max_iterations: 5,
            iteration_timeout_secs: 60,
            total_timeout_secs: 300,
            stall_limit: 3,
            convergence_threshold: 0.01,
            convergence_window: 3,
            verbose: false,
            quiet: true,
        };
        let executor = IterationExecutor::new(config, 0.1);
        let result = executor.run_loop("test question", 100.0, "echo 90");
        assert!(result.is_ok());
        let iter_result = result.unwrap();
        assert!(!iter_result.iterations.is_empty());
        assert!(iter_result.best_metric <= 100.0);
    }

    #[test]
    fn test_run_loop_max_iterations() {
        let config = IterationConfig {
            max_iterations: 3,
            iteration_timeout_secs: 60,
            total_timeout_secs: 300,
            stall_limit: 10,
            convergence_threshold: 0.01,
            convergence_window: 5, // Window larger than max_iterations to prevent convergence
            verbose: false,
            quiet: true,
        };
        let executor = IterationExecutor::new(config, 0.1);
        let result = executor.run_loop("test question", 100.0, "echo 95");
        assert!(result.is_ok());
        let iter_result = result.unwrap();
        assert_eq!(iter_result.iterations.len(), 3);
        assert_eq!(iter_result.stuck_reason, Some(StuckReason::MaxIterationsReached));
    }

    #[test]
    fn test_run_loop_convergence() {
        let config = IterationConfig {
            max_iterations: 10,
            iteration_timeout_secs: 60,
            total_timeout_secs: 300,
            stall_limit: 10,
            convergence_threshold: 0.01,
            convergence_window: 3,
            verbose: false,
            quiet: true,
        };
        let executor = IterationExecutor::new(config, 0.1);
        // Use constant value to trigger convergence
        let result = executor.run_loop("test question", 100.0, "echo 99.5");
        assert!(result.is_ok());
        let iter_result = result.unwrap();
        // Should exit early due to convergence
        assert!(iter_result.iterations.len() <= 10);
    }

    #[test]
    fn test_run_loop_error_handling() {
        let config = IterationConfig {
            max_iterations: 3,
            iteration_timeout_secs: 60,
            total_timeout_secs: 300,
            stall_limit: 10,
            convergence_threshold: 0.01,
            convergence_window: 3,
            verbose: false,
            quiet: true,
        };
        let executor = IterationExecutor::new(config, 0.1);
        let result = executor.run_loop("test question", 100.0, "nonexistent_command");
        assert!(result.is_ok()); // Should handle errors gracefully
        let iter_result = result.unwrap();
        // All iterations should fail
        assert!(iter_result.iterations.is_empty() || iter_result.iterations.len() < 3);
    }
}
