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
