use std::time::Instant;

/// Reasons why an experiment might be considered stuck
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StuckReason {
    /// Per-iteration time limit exceeded
    IterationTimeout,
    /// No improvement after multiple iterations
    StallLimitReached,
    /// Total experiment runtime limit exceeded
    TotalTimeout,
    /// Metric variance below threshold for window size
    ConvergenceAchieved,
    /// Maximum iterations reached
    MaxIterationsReached,
}

impl std::fmt::Display for StuckReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StuckReason::IterationTimeout => write!(f, "Iteration timeout exceeded"),
            StuckReason::StallLimitReached => {
                write!(f, "No improvement after multiple iterations (stall limit reached)")
            }
            StuckReason::TotalTimeout => write!(f, "Total experiment timeout exceeded"),
            StuckReason::ConvergenceAchieved => write!(f, "Metric convergence achieved"),
            StuckReason::MaxIterationsReached => write!(f, "Maximum iterations reached"),
        }
    }
}

/// State tracking for iteration progress
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
        Self {
            current_iteration: 0,
            best_metric: baseline_metric,
            best_iteration: 0,
            consecutive_no_improvement: 0,
            backoff_count: 0,
            start_time: Instant::now(),
            recent_metrics: Vec::new(),
        }
    }

    /// Record a successful iteration with improvement
    pub fn record_improvement(&mut self, iteration: usize, metric_value: f64) {
        self.current_iteration = iteration;
        self.best_metric = metric_value;
        self.best_iteration = iteration;
        self.consecutive_no_improvement = 0;
        self.backoff_count = 0;
        self.recent_metrics.push(metric_value);
    }

    /// Record a failed iteration without improvement
    pub fn record_no_improvement(&mut self, iteration: usize, metric_value: f64) {
        self.current_iteration = iteration;
        self.consecutive_no_improvement += 1;
        self.recent_metrics.push(metric_value);
    }

    /// Reset stall counter after backoff
    pub fn apply_backoff(&mut self) {
        self.consecutive_no_improvement = 0;
        self.backoff_count += 1;
    }

    /// Calculate current improvement ratio
    pub fn current_improvement(&self, baseline: f64) -> f64 {
        (baseline - self.best_metric) / baseline
    }

    /// Get elapsed time since start
    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

/// Configuration for stuck detection
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
        Self {
            max_iterations: 20,
            iteration_timeout_secs: 10 * 60, // 10 minutes
            total_timeout_secs: 120 * 60,     // 120 minutes
            stall_limit: 5,
            convergence_threshold: 0.01,
            convergence_window: 3,
        }
    }
}

/// Stuck detector implementing multi-layer guards
#[derive(Debug)]
pub struct StuckDetector {
    config: StuckDetectorConfig,
}

impl StuckDetector {
    pub fn new(config: StuckDetectorConfig) -> Self {
        Self { config }
    }

    /// Check if total timeout has been reached
    pub fn check_total_timeout(&self, elapsed: std::time::Duration) -> Option<StuckReason> {
        if elapsed.as_secs() >= self.config.total_timeout_secs {
            Some(StuckReason::TotalTimeout)
        } else {
            None
        }
    }

    /// Check if iteration timeout has been reached
    pub fn check_iteration_timeout(&self, elapsed: std::time::Duration) -> Option<StuckReason> {
        if elapsed.as_secs() >= self.config.iteration_timeout_secs {
            Some(StuckReason::IterationTimeout)
        } else {
            None
        }
    }

    /// Check if max iterations has been reached
    pub fn check_max_iterations(&self, current: usize) -> Option<StuckReason> {
        if current >= self.config.max_iterations {
            Some(StuckReason::MaxIterationsReached)
        } else {
            None
        }
    }

    /// Check for convergence based on recent metrics
    pub fn check_convergence(&self, recent_metrics: &[f64]) -> Option<StuckReason> {
        if recent_metrics.len() < self.config.convergence_window {
            return None;
        }

        let min_recent = *recent_metrics.iter().min_by(|a, b| a.total_cmp(b)).unwrap();
        let max_recent = *recent_metrics.iter().max_by(|a, b| a.total_cmp(b)).unwrap();
        let variance = if min_recent.abs() > 1e-10 {
            (max_recent - min_recent) / min_recent.abs()
        } else {
            0.0
        };

        if variance.abs() < self.config.convergence_threshold {
            Some(StuckReason::ConvergenceAchieved)
        } else {
            None
        }
    }

    /// Check if stall limit has been reached
    pub fn check_stall_limit(
        &self,
        consecutive_no_improvement: usize,
        backoff_count: usize,
    ) -> Option<StuckReason> {
        if consecutive_no_improvement >= self.config.stall_limit {
            if backoff_count >= 2 {
                Some(StuckReason::StallLimitReached)
            } else {
                None // Signal to backoff instead
            }
        } else {
            None
        }
    }

    /// Get backoff signal (true if should backoff)
    pub fn should_backoff(&self, consecutive_no_improvement: usize, backoff_count: usize) -> bool {
        consecutive_no_improvement >= self.config.stall_limit && backoff_count < 2
    }

    /// Check all stuck conditions
    pub fn check_all(
        &self,
        state: &IterationState,
        iteration_elapsed: std::time::Duration,
    ) -> Option<StuckReason> {
        // Layer 1: Iteration timeout
        if let Some(reason) = self.check_iteration_timeout(iteration_elapsed) {
            return Some(reason);
        }

        // Layer 3: Total timeout
        if let Some(reason) = self.check_total_timeout(state.elapsed()) {
            return Some(reason);
        }

        // Layer 4: Convergence
        if let Some(reason) = self.check_convergence(&state.recent_metrics) {
            return Some(reason);
        }

        // Layer 2: Stall limit
        if let Some(reason) = self.check_stall_limit(
            state.consecutive_no_improvement,
            state.backoff_count,
        ) {
            return Some(reason);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_total_timeout_detection() {
        let detector = StuckDetector::new(StuckDetectorConfig {
            total_timeout_secs: 10,
            ..Default::default()
        });

        assert!(detector
            .check_total_timeout(std::time::Duration::from_secs(15))
            .is_some());
        assert!(detector
            .check_total_timeout(std::time::Duration::from_secs(5))
            .is_none());
    }

    #[test]
    fn test_convergence_detection() {
        let detector = StuckDetector::new(StuckDetectorConfig {
            convergence_threshold: 0.01,
            convergence_window: 3,
            ..Default::default()
        });

        // Converged metrics
        let converged = vec![100.0, 100.1, 100.05];
        assert_eq!(
            detector.check_convergence(&converged),
            Some(StuckReason::ConvergenceAchieved)
        );

        // Non-converged metrics
        let not_converged = vec![100.0, 150.0, 120.0];
        assert!(detector.check_convergence(&not_converged).is_none());
    }

    #[test]
    fn test_stall_detection() {
        let detector = StuckDetector::new(StuckDetectorConfig {
            stall_limit: 5,
            ..Default::default()
        });

        // Below stall limit
        assert!(detector.check_stall_limit(3, 0).is_none());

        // At stall limit, first backoff
        assert!(detector.check_stall_limit(5, 0).is_none());

        // At stall limit, after backoffs
        assert_eq!(
            detector.check_stall_limit(5, 2),
            Some(StuckReason::StallLimitReached)
        );
    }
}
