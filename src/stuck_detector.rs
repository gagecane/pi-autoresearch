use std::time::Instant;
use colored::Colorize;

/// Reason why the iteration loop stopped.
///
/// Each variant includes helpful suggestions for users when displayed.
///
/// # Variants
///
/// * `IterationTimeout` - A single iteration took too long
/// * `StallLimitReached` - No improvement after multiple iterations
/// * `TotalTimeout` - The entire experiment exceeded the time limit
/// * `ConvergenceAchieved` - The metric has stabilized
/// * `MaxIterationsReached` - Reached the maximum number of iterations
///
/// # Examples
///
/// ```
/// use pi_autoresearch::stuck_detector::StuckReason;
///
/// // Create a stuck reason for timeout
/// let reason = StuckReason::TotalTimeout;
///
/// // Display includes helpful suggestions
/// let message = format!("{}", reason);
/// assert!(message.contains("timeout"));
/// ```
///
/// ```
/// use pi_autoresearch::stuck_detector::StuckReason;
///
/// // Compare stuck reasons
/// let reason1 = StuckReason::ConvergenceAchieved;
/// let reason2 = StuckReason::MaxIterationsReached;
///
/// assert_ne!(reason1, reason2);
/// ```
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

/// State tracker for the iteration loop.
///
/// Maintains information about the current state of the experiment,
/// including metrics, iteration counts, and timing information.
///
/// # Fields
///
/// * `current_iteration` - The current iteration number
/// * `best_metric` - The best metric value seen so far
/// * `best_iteration` - The iteration number with the best metric
/// * `consecutive_no_improvement` - Number of iterations without improvement
/// * `backoff_count` - Number of times backoff has been applied
/// * `start_time` - When the experiment started
/// * `recent_metrics` - Recent metric values for convergence detection
///
/// # Examples
///
/// ```
/// use pi_autoresearch::stuck_detector::IterationState;
///
/// // Create a new state with baseline metric
/// let mut state = IterationState::new(100.0);
///
/// assert_eq!(state.current_iteration, 0);
/// assert_eq!(state.best_metric, 100.0);
/// ```
///
/// ```
/// use pi_autoresearch::stuck_detector::IterationState;
///
/// // Record improvements and track state
/// let mut state = IterationState::new(100.0);
///
/// state.record_improvement(1, 95.0);
/// assert_eq!(state.best_metric, 95.0);
/// assert_eq!(state.best_iteration, 1);
/// assert_eq!(state.consecutive_no_improvement, 0);
///
/// state.record_no_improvement(2, 96.0);
/// assert_eq!(state.consecutive_no_improvement, 1);
/// ```
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
    /// Create a new `IterationState` with the given baseline metric.
    ///
    /// # Arguments
    ///
    /// * `baseline_metric` - The initial metric value to start from
    ///
    /// # Returns
    ///
    /// A new `IterationState` with all counters reset
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::stuck_detector::IterationState;
    ///
    /// let state = IterationState::new(100.0);
    ///
    /// assert_eq!(state.current_iteration, 0);
    /// assert_eq!(state.best_metric, 100.0);
    /// assert_eq!(state.best_iteration, 0);
    /// assert_eq!(state.consecutive_no_improvement, 0);
    /// ```
    pub fn new(baseline_metric: f64) -> Self {
        Self { current_iteration: 0, best_metric: baseline_metric, best_iteration: 0, consecutive_no_improvement: 0, backoff_count: 0, start_time: Instant::now(), recent_metrics: Vec::new() }
    }
    /// Record an improvement in the metric.
    ///
    /// Resets the consecutive no-improvement counter and backoff count.
    ///
    /// # Arguments
    ///
    /// * `iteration` - The current iteration number
    /// * `metric_value` - The new improved metric value
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::stuck_detector::IterationState;
    ///
    /// let mut state = IterationState::new(100.0);
    /// state.record_improvement(1, 95.0);
    ///
    /// assert_eq!(state.best_metric, 95.0);
    /// assert_eq!(state.consecutive_no_improvement, 0);
    /// assert_eq!(state.backoff_count, 0);
    /// ```
    pub fn record_improvement(&mut self, iteration: usize, metric_value: f64) {
        self.current_iteration = iteration; self.best_metric = metric_value; self.best_iteration = iteration; self.consecutive_no_improvement = 0; self.backoff_count = 0; self.recent_metrics.push(metric_value);
    }
    /// Record an iteration without improvement.
    ///
    /// Increments the consecutive no-improvement counter.
    ///
    /// # Arguments
    ///
    /// * `iteration` - The current iteration number
    /// * `metric_value` - The metric value (not an improvement)
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::stuck_detector::IterationState;
    ///
    /// let mut state = IterationState::new(100.0);
    /// state.record_no_improvement(1, 102.0);
    ///
    /// assert_eq!(state.consecutive_no_improvement, 1);
    /// ```
    pub fn record_no_improvement(&mut self, iteration: usize, metric_value: f64) {
        self.current_iteration = iteration; self.consecutive_no_improvement += 1; self.recent_metrics.push(metric_value);
    }
    /// Apply backoff by resetting the no-improvement counter.
    ///
    /// Increments the backoff count. Used when stall limit is reached
    /// but we want to try a few more times with adjusted parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::stuck_detector::IterationState;
    ///
    /// let mut state = IterationState::new(100.0);
    /// state.consecutive_no_improvement = 5;
    ///
    /// state.apply_backoff();
    ///
    /// assert_eq!(state.consecutive_no_improvement, 0);
    /// assert_eq!(state.backoff_count, 1);
    /// ```
    pub fn apply_backoff(&mut self) { self.consecutive_no_improvement = 0; self.backoff_count += 1; }
    /// Get the elapsed time since the experiment started.
    ///
    /// # Returns
    ///
    /// The duration since `start_time`
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::stuck_detector::IterationState;
    ///
    /// let state = IterationState::new(100.0);
    /// let elapsed = state.elapsed();
    ///
    /// assert!(elapsed.as_secs() >= 0);
    /// ```
    pub fn elapsed(&self) -> std::time::Duration { self.start_time.elapsed() }
}

/// Configuration for the stuck detector.
///
/// Controls when the iteration loop should stop due to timeouts,
/// convergence, or lack of progress.
///
/// # Fields
///
/// * `max_iterations` - Maximum number of iterations to run
/// * `iteration_timeout_secs` - Timeout for a single iteration
/// * `total_timeout_secs` - Total timeout for the entire experiment
/// * `stall_limit` - Consecutive non-improving iterations before backoff
/// * `convergence_threshold` - Minimum variance to consider as progress
/// * `convergence_window` - Number of recent iterations to check for convergence
///
/// # Examples
///
/// ```
/// use pi_autoresearch::stuck_detector::StuckDetectorConfig;
///
/// // Use default configuration
/// let config = StuckDetectorConfig::default();
///
/// assert_eq!(config.max_iterations, 20);
/// assert_eq!(config.stall_limit, 5);
/// ```
///
/// ```
/// use pi_autoresearch::stuck_detector::StuckDetectorConfig;
///
/// // Custom configuration for faster experiments
/// let config = StuckDetectorConfig {
///     max_iterations: 10,
///     iteration_timeout_secs: 300,
///     total_timeout_secs: 3600,
///     stall_limit: 3,
///     convergence_threshold: 0.01,
///     convergence_window: 3,
/// };
///
/// assert_eq!(config.max_iterations, 10);
/// assert_eq!(config.stall_limit, 3);
/// ```
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
    /// Create a default `StuckDetectorConfig` with reasonable defaults.
    ///
    /// # Defaults
    ///
    /// * `max_iterations`: 20
    /// * `iteration_timeout_secs`: 600 (10 minutes)
    /// * `total_timeout_secs`: 7200 (2 hours)
    /// * `stall_limit`: 5
    /// * `convergence_threshold`: 0.01 (1%)
    /// * `convergence_window`: 3
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::stuck_detector::StuckDetectorConfig;
    ///
    /// let config = StuckDetectorConfig::default();
    ///
    /// assert_eq!(config.max_iterations, 20);
    /// assert_eq!(config.iteration_timeout_secs, 600);
    /// assert_eq!(config.total_timeout_secs, 7200);
    /// assert_eq!(config.stall_limit, 5);
    /// assert_eq!(config.convergence_threshold, 0.01);
    /// assert_eq!(config.convergence_window, 3);
    /// ```
    fn default() -> Self {
        Self { max_iterations: 20, iteration_timeout_secs: 600, total_timeout_secs: 7200, stall_limit: 5, convergence_threshold: 0.01, convergence_window: 3 }
    }
}

/// Detector for determining when the iteration loop should stop.
///
/// Provides methods to check various stopping conditions including
/// timeouts, convergence, and stall limits.
///
/// # Examples
///
/// ```
/// use pi_autoresearch::stuck_detector::{StuckDetector, StuckDetectorConfig};
///
/// // Create a detector with default configuration
/// let config = StuckDetectorConfig::default();
/// let detector = StuckDetector::new(config);
///
/// // The detector is ready to check stopping conditions
/// ```
///
/// ```
/// use pi_autoresearch::stuck_detector::{StuckDetector, StuckDetectorConfig};
///
/// // Create a detector with custom configuration
/// let config = StuckDetectorConfig {
///     max_iterations: 10,
///     iteration_timeout_secs: 300,
///     total_timeout_secs: 3600,
///     stall_limit: 3,
///     convergence_threshold: 0.01,
///     convergence_window: 3,
/// };
///
/// let detector = StuckDetector::new(config);
/// ```
#[derive(Debug)]
pub struct StuckDetector { config: StuckDetectorConfig }

impl StuckDetector {
    /// Create a new `StuckDetector` with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The detector configuration
    ///
    /// # Returns
    ///
    /// A new `StuckDetector` ready to check stopping conditions
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::stuck_detector::{StuckDetector, StuckDetectorConfig};
    ///
    /// let config = StuckDetectorConfig::default();
    /// let detector = StuckDetector::new(config);
    ///
    /// // Detector is ready to use
    /// ```
    pub fn new(config: StuckDetectorConfig) -> Self { Self { config } }
    /// Check if the total experiment timeout has been exceeded.
    ///
    /// # Arguments
    ///
    /// * `elapsed` - The time elapsed since the experiment started
    ///
    /// # Returns
    ///
    /// * `Some(StuckReason::TotalTimeout)` if timeout exceeded
    /// * `None` if still within the time limit
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::stuck_detector::{StuckDetector, StuckDetectorConfig, StuckReason};
    /// use std::time::Duration;
    ///
    /// let config = StuckDetectorConfig { total_timeout_secs: 100, ..StuckDetectorConfig::default() };
    /// let detector = StuckDetector::new(config);
    ///
    /// let result = detector.check_total_timeout(Duration::from_secs(50));
    /// assert!(result.is_none());
    ///
    /// let result = detector.check_total_timeout(Duration::from_secs(150));
    /// assert_eq!(result, Some(StuckReason::TotalTimeout));
    /// ```
    pub fn check_total_timeout(&self, elapsed: std::time::Duration) -> Option<StuckReason> { if elapsed.as_secs() >= self.config.total_timeout_secs { Some(StuckReason::TotalTimeout) } else { None } }
    /// Check if a single iteration has exceeded its timeout.
    ///
    /// # Arguments
    ///
    /// * `elapsed` - The time elapsed for the current iteration
    ///
    /// # Returns
    ///
    /// * `Some(StuckReason::IterationTimeout)` if timeout exceeded
    /// * `None` if still within the time limit
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::stuck_detector::{StuckDetector, StuckDetectorConfig, StuckReason};
    /// use std::time::Duration;
    ///
    /// let config = StuckDetectorConfig { iteration_timeout_secs: 60, ..StuckDetectorConfig::default() };
    /// let detector = StuckDetector::new(config);
    ///
    /// let result = detector.check_iteration_timeout(Duration::from_secs(30));
    /// assert!(result.is_none());
    ///
    /// let result = detector.check_iteration_timeout(Duration::from_secs(90));
    /// assert_eq!(result, Some(StuckReason::IterationTimeout));
    /// ```
    pub fn check_iteration_timeout(&self, elapsed: std::time::Duration) -> Option<StuckReason> { if elapsed.as_secs() >= self.config.iteration_timeout_secs { Some(StuckReason::IterationTimeout) } else { None } }
    /// Check if the maximum number of iterations has been reached.
    ///
    /// # Arguments
    ///
    /// * `current` - The current iteration number
    ///
    /// # Returns
    ///
    /// * `Some(StuckReason::MaxIterationsReached)` if limit reached
    /// * `None` if more iterations allowed
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::stuck_detector::{StuckDetector, StuckDetectorConfig, StuckReason};
    ///
    /// let config = StuckDetectorConfig { max_iterations: 5, ..StuckDetectorConfig::default() };
    /// let detector = StuckDetector::new(config);
    ///
    /// let result = detector.check_max_iterations(3);
    /// assert!(result.is_none());
    ///
    /// let result = detector.check_max_iterations(5);
    /// assert_eq!(result, Some(StuckReason::MaxIterationsReached));
    /// ```
    pub fn check_max_iterations(&self, current: usize) -> Option<StuckReason> { if current >= self.config.max_iterations { Some(StuckReason::MaxIterationsReached) } else { None } }
    /// Check if the metric has converged (no significant improvement).
    ///
    /// # Arguments
    ///
    /// * `recent_metrics` - Recent metric values to analyze
    ///
    /// # Returns
    ///
    /// * `Some(StuckReason::ConvergenceAchieved)` if converged
    /// * `None` if still improving or not enough data
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::stuck_detector::{StuckDetector, StuckDetectorConfig, StuckReason};
    ///
    /// let config = StuckDetectorConfig {
    ///     convergence_threshold: 0.01,
    ///     convergence_window: 3,
    ///     ..StuckDetectorConfig::default()
    /// };
    /// let detector = StuckDetector::new(config);
    ///
    /// // Not enough data points
    /// let result = detector.check_convergence(&[100.0, 99.5]);
    /// assert!(result.is_none());
    ///
    /// // Converged (very similar values)
    /// let result = detector.check_convergence(&[100.0, 99.9, 99.95]);
    /// assert_eq!(result, Some(StuckReason::ConvergenceAchieved));
    /// ```
    pub fn check_convergence(&self, recent_metrics: &[f64]) -> Option<StuckReason> {
        if recent_metrics.len() < self.config.convergence_window { return None; }
        let min_recent = *recent_metrics.iter().min_by(|a, b| a.total_cmp(b)).unwrap();
        let max_recent = *recent_metrics.iter().max_by(|a, b| a.total_cmp(b)).unwrap();
        let variance = if min_recent.abs() > 1e-10 { (max_recent - min_recent) / min_recent.abs() } else { 0.0 };
        if variance.abs() < self.config.convergence_threshold { Some(StuckReason::ConvergenceAchieved) } else { None }
    }
    /// Check if the stall limit has been reached.
    ///
    /// # Arguments
    ///
    /// * `consecutive_no_improvement` - Number of iterations without improvement
    /// * `backoff_count` - Number of times backoff has been applied
    ///
    /// # Returns
    ///
    /// * `Some(StuckReason::StallLimitReached)` if stall limit reached after 2 backoffs
    /// * `None` if still within limits
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::stuck_detector::{StuckDetector, StuckDetectorConfig, StuckReason};
    ///
    /// let config = StuckDetectorConfig { stall_limit: 3, ..StuckDetectorConfig::default() };
    /// let detector = StuckDetector::new(config);
    ///
    /// // Stall limit reached but not enough backoffs
    /// let result = detector.check_stall_limit(5, 1);
    /// assert!(result.is_none());
    ///
    /// // Stall limit reached with enough backoffs
    /// let result = detector.check_stall_limit(5, 2);
    /// assert_eq!(result, Some(StuckReason::StallLimitReached));
    /// ```
    pub fn check_stall_limit(&self, consecutive_no_improvement: usize, backoff_count: usize) -> Option<StuckReason> {
        if consecutive_no_improvement >= self.config.stall_limit { if backoff_count >= 2 { Some(StuckReason::StallLimitReached) } else { None } } else { None }
    }
    /// Check if backoff should be applied.
    ///
    /// # Arguments
    ///
    /// * `consecutive_no_improvement` - Number of iterations without improvement
    /// * `backoff_count` - Number of times backoff has been applied
    ///
    /// # Returns
    ///
    /// `true` if backoff should be applied (stall limit reached but less than 2 backoffs)
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::stuck_detector::{StuckDetector, StuckDetectorConfig};
    ///
    /// let config = StuckDetectorConfig { stall_limit: 3, ..StuckDetectorConfig::default() };
    /// let detector = StuckDetector::new(config);
    ///
    /// // Not at stall limit yet
    /// assert!(!detector.should_backoff(2, 0));
    ///
    /// // At stall limit, should backoff
    /// assert!(detector.should_backoff(5, 0));
    ///
    /// // Already backed off twice, should not backoff again
    /// assert!(!detector.should_backoff(5, 2));
    /// ```
    pub fn should_backoff(&self, consecutive_no_improvement: usize, backoff_count: usize) -> bool { consecutive_no_improvement >= self.config.stall_limit && backoff_count < 2 }
}
