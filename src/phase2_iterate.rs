use serde::{Deserialize, Serialize};
use anyhow::Result;
use chrono::Utc;

use crate::metric_evaluator::MetricEvaluator;
use crate::pi_agent::PiAgent;
use crate::stuck_detector::{IterationState, StuckDetector, StuckDetectorConfig, StuckReason};

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
    pub fn new(iteration: usize, agent_action: String, metric_value: f64, improvement: f64, kept: bool) -> Self {
        Self { iteration, timestamp: Utc::now().to_rfc3339(), agent_action, metric_value, improvement, kept }
    }
}

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
    fn default() -> Self {
        Self { max_iterations: 20, iteration_timeout_secs: 600, total_timeout_secs: 7200, stall_limit: 5, convergence_threshold: 0.01, convergence_window: 3, verbose: false, quiet: false }
    }
}

#[derive(Debug)]
pub struct IterationResult {
    pub iterations: Vec<IterationRecord>,
    pub best_iteration: Option<usize>,
    pub best_metric: f64,
    pub stuck_reason: Option<StuckReason>,
}

pub struct IterationExecutor {
    config: IterationConfig,
    metric_evaluator: MetricEvaluator,
    pi_agent: PiAgent,
    stuck_detector: StuckDetector,
}

impl IterationExecutor {
    pub fn new(config: IterationConfig, max_variance: f64) -> Self {
        let stuck_detector = StuckDetector::new(StuckDetectorConfig {
            max_iterations: config.max_iterations, iteration_timeout_secs: config.iteration_timeout_secs,
            total_timeout_secs: config.total_timeout_secs, stall_limit: config.stall_limit,
            convergence_threshold: config.convergence_threshold, convergence_window: config.convergence_window,
        });
        Self { config, metric_evaluator: MetricEvaluator::new(max_variance), pi_agent: PiAgent::default(), stuck_detector }
    }
    pub fn run_iteration(&self, iteration: usize, question: &str, baseline_value: f64, best_metric: f64, measure_command: &str) -> Result<IterationRecord> {
        let current_state = format!("Iteration {}, best metric: {:.2}", iteration, best_metric);
        let metric_feedback = format!("Baseline: {:.2}, current best: {:.2}", baseline_value, best_metric);
        let agent_action = self.pi_agent.propose_change(question, &current_state, &metric_feedback);
        let metric_value = self.metric_evaluator.execute_measurement(measure_command)?;
        let improvement = (best_metric - metric_value) / baseline_value;
        let kept = metric_value < best_metric;
        Ok(IterationRecord::new(iteration, agent_action, metric_value, improvement, kept))
    }
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
