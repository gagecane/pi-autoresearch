use std::time::Instant;

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
            StuckReason::IterationTimeout => write!(f, "Iteration timeout exceeded"),
            StuckReason::StallLimitReached => write!(f, "No improvement after multiple iterations (stall limit reached)"),
            StuckReason::TotalTimeout => write!(f, "Total experiment timeout exceeded"),
            StuckReason::ConvergenceAchieved => write!(f, "Metric convergence achieved"),
            StuckReason::MaxIterationsReached => write!(f, "Maximum iterations reached"),
        }
    }
}

#[derive(Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_total_timeout_detection() {
        let detector = StuckDetector::new(StuckDetectorConfig { total_timeout_secs: 10, ..Default::default() });
        assert!(detector.check_total_timeout(std::time::Duration::from_secs(15)).is_some());
    }
    #[test] fn test_convergence_detection() {
        let detector = StuckDetector::new(StuckDetectorConfig { convergence_threshold: 0.01, convergence_window: 3, ..Default::default() });
        assert_eq!(detector.check_convergence(&[100.0, 100.1, 100.05]), Some(StuckReason::ConvergenceAchieved));
    }
    #[test] fn test_stall_detection() {
        let detector = StuckDetector::new(StuckDetectorConfig { stall_limit: 5, ..Default::default() });
        assert_eq!(detector.check_stall_limit(5, 2), Some(StuckReason::StallLimitReached));
    }
}
