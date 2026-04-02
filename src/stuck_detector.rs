use std::time::Instant;
use colored::Colorize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StuckReason {
    IterationTimeout,
    StallLimitReached,
    TotalTimeout,
    ConvergenceAchieved,
    MaxIterationsReached,
}

impl std::fmt::Display for StuckReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StuckReason::IterationTimeout => {
                writeln!(f, "Iteration timeout exceeded")?;
                writeln!(f)?;
                writeln!(f, "  {}", "SUGGESTION:".yellow().bold())?;
                writeln!(f, "  - Increase iteration_timeout_minutes")?;
                writeln!(f, "  - Check if the measurement command is slow")?;
                writeln!(f, "  - See: docs/TROUBLESHOOTING.md#experiment-issues")?;
                Ok(())
            }
            StuckReason::StallLimitReached => {
                writeln!(f, "No improvement after multiple iterations (stall limit reached)")?;
                writeln!(f)?;
                writeln!(f, "  {}", "SUGGESTION:".yellow().bold())?;
                writeln!(f, "  - Increase stall_limit to allow more iterations")?;
                writeln!(f, "  - Adjust target_improvement to a more achievable value")?;
                writeln!(f, "  - Review the agent's proposed changes for effectiveness")?;
                writeln!(f, "  - See: docs/TROUBLESHOOTING.md#experiment-issues")?;
                Ok(())
            }
            StuckReason::TotalTimeout => {
                writeln!(f, "Total experiment timeout exceeded")?;
                writeln!(f)?;
                writeln!(f, "  {}", "SUGGESTION:".yellow().bold())?;
                writeln!(f, "  - Increase total_timeout_minutes")?;
                writeln!(f, "  - Reduce max_iterations for faster completion")?;
                writeln!(f, "  - See: docs/TROUBLESHOOTING.md#experiment-issues")?;
                Ok(())
            }
            StuckReason::ConvergenceAchieved => {
                writeln!(f, "Metric convergence achieved")?;
                writeln!(f)?;
                writeln!(f, "  {}", "NOTE:".cyan().bold())?;
                writeln!(f, "  - The metric has stabilized within the convergence threshold")?;
                writeln!(f, "  - Consider adjusting convergence_threshold if more precision is needed")?;
                Ok(())
            }
            StuckReason::MaxIterationsReached => {
                writeln!(f, "Maximum iterations reached")?;
                writeln!(f)?;
                writeln!(f, "  {}", "SUGGESTION:".yellow().bold())?;
                writeln!(f, "  - Increase max_iterations to explore more possibilities")?;
                writeln!(f, "  - Adjust target_improvement to a more achievable value")?;
                writeln!(f, "  - See: docs/TROUBLESHOOTING.md#experiment-issues")?;
                Ok(())
            }
        }
    }
}

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

impl IterationState {
    pub fn new(baseline_metric: f64) -> Self {
        Self { current_iteration: 0, best_metric: baseline_metric, best_iteration: 0, consecutive_no_improvement: 0, backoff_count: 0, start_time: Instant::now(), recent_metrics: Vec::new() }
    }
    pub fn record_improvement(&mut self, iteration: usize, metric_value: f64) {
        self.current_iteration = iteration; self.best_metric = metric_value; self.best_iteration = iteration; self.consecutive_no_improvement = 0; self.backoff_count = 0; self.recent_metrics.push(metric_value);
    }
    pub fn record_no_improvement(&mut self, iteration: usize, metric_value: f64) {
        self.current_iteration = iteration; self.consecutive_no_improvement += 1; self.recent_metrics.push(metric_value);
    }
    pub fn apply_backoff(&mut self) { self.consecutive_no_improvement = 0; self.backoff_count += 1; }
    pub fn elapsed(&self) -> std::time::Duration { self.start_time.elapsed() }
}

#[derive(Debug, Clone)]
pub struct StuckDetectorConfig {
    pub max_iterations: usize,
    pub iteration_timeout_secs: u64,
    pub total_timeout_secs: u64,
    pub stall_limit: usize,
    pub convergence_threshold: f64,
    pub convergence_window: usize,
}

impl Default for StuckDetectorConfig {
    fn default() -> Self {
        Self { max_iterations: 20, iteration_timeout_secs: 600, total_timeout_secs: 7200, stall_limit: 5, convergence_threshold: 0.01, convergence_window: 3 }
    }
}

#[derive(Debug)]
pub struct StuckDetector { config: StuckDetectorConfig }

impl StuckDetector {
    pub fn new(config: StuckDetectorConfig) -> Self { Self { config } }
    pub fn check_total_timeout(&self, elapsed: std::time::Duration) -> Option<StuckReason> { if elapsed.as_secs() >= self.config.total_timeout_secs { Some(StuckReason::TotalTimeout) } else { None } }
    pub fn check_iteration_timeout(&self, elapsed: std::time::Duration) -> Option<StuckReason> { if elapsed.as_secs() >= self.config.iteration_timeout_secs { Some(StuckReason::IterationTimeout) } else { None } }
    pub fn check_max_iterations(&self, current: usize) -> Option<StuckReason> { if current >= self.config.max_iterations { Some(StuckReason::MaxIterationsReached) } else { None } }
    pub fn check_convergence(&self, recent_metrics: &[f64]) -> Option<StuckReason> {
        if recent_metrics.len() < self.config.convergence_window { return None; }
        let min_recent = *recent_metrics.iter().min_by(|a, b| a.total_cmp(b)).unwrap();
        let max_recent = *recent_metrics.iter().max_by(|a, b| a.total_cmp(b)).unwrap();
        let variance = if min_recent.abs() > 1e-10 { (max_recent - min_recent) / min_recent.abs() } else { 0.0 };
        if variance.abs() < self.config.convergence_threshold { Some(StuckReason::ConvergenceAchieved) } else { None }
    }
    pub fn check_stall_limit(&self, consecutive_no_improvement: usize, backoff_count: usize) -> Option<StuckReason> {
        if consecutive_no_improvement >= self.config.stall_limit { if backoff_count >= 2 { Some(StuckReason::StallLimitReached) } else { None } } else { None }
    }
    pub fn should_backoff(&self, consecutive_no_improvement: usize, backoff_count: usize) -> bool { consecutive_no_improvement >= self.config.stall_limit && backoff_count < 2 }
}
