use anyhow::Result;
use std::io::Write;
use std::process::Command;
use chrono::Utc;
use colored::Colorize;

/// Error type for metric evaluation failures.
///
/// `MetricError` provides detailed error messages with suggestions for resolving
/// measurement issues. It implements `Display` with contextual suggestions and
/// `Error` for integration with Rust's error handling.
///
/// # Examples
///
/// Creating a metric error:
///
/// ```
/// use pi_autoresearch::metric_evaluator::MetricError;
///
/// let error = MetricError { 
///     message: "Could not parse measurement output: 'abc'".to_string() 
/// };
/// assert!(error.message.contains("Could not parse"));
/// ```
///
/// Displaying error with suggestions:
///
/// ```
/// use pi_autoresearch::metric_evaluator::MetricError;
///
/// let error = MetricError { 
///     message: "Empty measurement command".to_string() 
/// };
/// let output = format!("{}", error);
/// assert!(output.contains("SUGGESTION"));
/// assert!(output.contains("Provide a valid measurement command"));
/// ```
#[derive(Debug)]
pub struct MetricError { pub message: String }

impl std::fmt::Display for MetricError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.message)?;
        writeln!(f)?;
        writeln!(f, "  {}", "SUGGESTION:".yellow().bold())?;
        if self.message.contains("Empty measurement command") {
            writeln!(f, "  - Provide a valid measurement command")?;
            writeln!(f, "  - Example: --measure \"cargo bench --bench performance\"")?;
        } else if self.message.contains("Measurement command failed") {
            writeln!(f, "  - Check that the command exists and is executable")?;
            writeln!(f, "  - Run the command manually to verify it works")?;
        } else if self.message.contains("Could not parse measurement output") {
            writeln!(f, "  - Ensure the command outputs a numeric value")?;
            writeln!(f, "  - Example output: 42.5 or 100")?;
        }
        writeln!(f, "  - See: docs/TROUBLESHOOTING.md#measurement-issues")?;
        Ok(())
    }
}
impl std::error::Error for MetricError { fn source(&self) -> Option<&(dyn std::error::Error + 'static)> { None } }

/// Metric evaluator for running measurement commands and verifying baselines.
///
/// `MetricEvaluator` handles executing measurement commands, parsing numeric output,
/// and verifying baseline measurements with variance checking. It's a core component
/// of the experiment workflow.
///
/// # Examples
///
/// Creating a metric evaluator with default variance:
///
/// ```
/// use pi_autoresearch::metric_evaluator::MetricEvaluator;
///
/// let evaluator = MetricEvaluator::default();
/// // Default max_variance is 0.05 (5%)
/// ```
///
/// Creating a metric evaluator with custom variance threshold:
///
/// ```
/// use pi_autoresearch::metric_evaluator::MetricEvaluator;
///
/// let evaluator = MetricEvaluator::new(0.10);
/// // max_variance is 0.10 (10%)
/// ```
///
/// Executing a measurement command:
///
/// ```
/// use pi_autoresearch::metric_evaluator::MetricEvaluator;
///
/// let evaluator = MetricEvaluator::default();
/// let result = evaluator.execute_measurement("echo 42.5");
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap(), 42.5);
/// ```
///
/// Verifying a baseline measurement:
///
/// ```no_run
/// use pi_autoresearch::metric_evaluator::MetricEvaluator;
///
/// let evaluator = MetricEvaluator::new(0.05);
/// let result = evaluator.verify_baseline("memory_usage", "./measure_memory.sh");
/// match result {
///     Ok(baseline_result) => {
///         if baseline_result.success {
///             println!("Baseline verified successfully");
///         }
///     }
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
#[derive(Debug, Clone)]
pub struct MetricEvaluator { max_variance: f64 }

impl MetricEvaluator {
    /// Creates a new `MetricEvaluator` with the specified maximum variance threshold.
    ///
    /// The `max_variance` parameter specifies the maximum allowed variance between
    /// baseline measurement runs as a decimal (e.g., 0.05 for 5%).
    ///
    /// # Arguments
    ///
    /// * `max_variance` - Maximum allowed variance between measurement runs (0.0 to 1.0)
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::metric_evaluator::MetricEvaluator;
    ///
    /// let evaluator = MetricEvaluator::new(0.10);
    /// // Creates evaluator with 10% max variance threshold
    /// ```
    pub fn new(max_variance: f64) -> Self { Self { max_variance } }
    
    /// Executes a measurement command and parses the numeric output.
    ///
    /// Runs the specified shell command and expects a single numeric value as output.
    /// The command output is trimmed and parsed as an f64.
    ///
    /// # Arguments
    ///
    /// * `command` - Shell command to execute (e.g., "echo 42.5" or "./measure.sh")
    ///
    /// # Returns
    ///
    /// * `Ok(f64)` - The parsed numeric value from command output
    /// * `Err(MetricError)` - If command is empty, fails, or output is not numeric
    ///
    /// # Errors
    ///
    /// Returns `MetricError` if:
    /// - The command is empty
    /// - The command fails to execute
    /// - The output cannot be parsed as a number
    ///
    /// # Examples
    ///
    /// Executing a simple command:
    ///
    /// ```
    /// use pi_autoresearch::metric_evaluator::MetricEvaluator;
    ///
    /// let evaluator = MetricEvaluator::default();
    /// let result = evaluator.execute_measurement("echo 100");
    /// assert!(result.is_ok());
    /// assert_eq!(result.unwrap(), 100.0);
    /// ```
    ///
    /// Handling errors:
    ///
    /// ```
    /// use pi_autoresearch::metric_evaluator::MetricEvaluator;
    ///
    /// let evaluator = MetricEvaluator::default();
    /// let result = evaluator.execute_measurement("");
    /// assert!(result.is_err());
    /// ```
    pub fn execute_measurement(&self, command: &str) -> Result<f64> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() { return Err(anyhow::anyhow!(MetricError { message: "Empty measurement command".to_string() })); }
        let output = Command::new(parts[0]).args(&parts[1..]).output()?;
        if !output.status.success() { return Err(anyhow::anyhow!(MetricError { message: format!("Measurement command failed: {}", String::from_utf8_lossy(&output.stderr)) })); }
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.trim().parse().map_err(|_| anyhow::anyhow!(MetricError { message: format!("Could not parse measurement output as number: '{}'", stdout.trim()) }))
    }
    
    /// Gets the current git commit hash.
    ///
    /// Executes `git rev-parse HEAD` to retrieve the current commit hash.
    /// Returns "unknown" if the command fails or not in a git repository.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The git commit hash or "unknown"
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::metric_evaluator::MetricEvaluator;
    ///
    /// let evaluator = MetricEvaluator::default();
    /// let result = evaluator.get_git_commit_hash();
    /// assert!(result.is_ok());
    /// let hash = result.unwrap();
    /// assert!(!hash.is_empty());
    /// ```
    pub fn get_git_commit_hash(&self) -> Result<String> {
        let output = Command::new("git").args(["rev-parse", "HEAD"]).output()?;
        if output.status.success() { Ok(String::from_utf8_lossy(&output.stdout).trim().to_string()) } else { Ok("unknown".to_string()) }
    }
    
    /// Verifies a baseline measurement by running it twice and checking variance.
    ///
    /// Executes the measurement command twice and calculates the variance between
    /// the two runs. If the variance exceeds `max_variance`, the verification fails.
    /// Creates a `BaselineRecord` with the measurement details.
    ///
    /// # Arguments
    ///
    /// * `metric` - Name of the metric being measured (e.g., "memory_usage", "performance")
    /// * `measurement_command` - Shell command to execute for measurement
    ///
    /// # Returns
    ///
    /// * `Ok(BaselineVerificationResult)` - Contains success status and baseline record
    ///
    /// # Examples
    ///
    /// Verifying a baseline with a simple command:
    ///
    /// ```
    /// use pi_autoresearch::metric_evaluator::MetricEvaluator;
    ///
    /// let evaluator = MetricEvaluator::new(0.10);
    /// let result = evaluator.verify_baseline("test_metric", "echo 100");
    /// assert!(result.is_ok());
    /// let baseline = result.unwrap();
    /// assert!(baseline.success);
    /// ```
    ///
    /// Checking verification result:
    ///
    /// ```no_run
    /// use pi_autoresearch::metric_evaluator::MetricEvaluator;
    ///
    /// let evaluator = MetricEvaluator::default();
    /// let result = evaluator.verify_baseline("memory", "./measure.sh");
    /// if let Ok(baseline) = result {
    ///     if baseline.success {
    ///         println!("Baseline verified");
    ///     } else if let Some(err) = baseline.error_message {
    ///         eprintln!("Verification failed: {}", err);
    ///     }
    /// }
    /// ```
    pub fn verify_baseline(&self, metric: &str, measurement_command: &str) -> Result<super::phase1_design::BaselineVerificationResult> {
        eprintln!("Verifying baseline measurement...");
        eprintln!("  Metric: {}", metric);
        eprintln!("  Command: {}", measurement_command);
        eprintln!("  Max variance: {:.1}%", self.max_variance * 100.0);
        let git_commit = self.get_git_commit_hash()?;
        let timestamp = Utc::now().to_rfc3339();
        let mut runs: Vec<f64> = Vec::new();
        for i in 1..=2 {
            eprint!("  Run {}/2... ", i);
            std::io::stdout().flush()?;
            match self.execute_measurement(measurement_command) {
                Ok(value) => { eprintln!("{:.2}", value); runs.push(value); }
                Err(e) => { eprintln!("FAILED"); return Ok(super::phase1_design::BaselineVerificationResult::failure(format!("Run {} failed: {}", i, e))); }
            }
        }
        if runs.len() < 2 { return Ok(super::phase1_design::BaselineVerificationResult::failure("Insufficient successful runs".to_string())); }
        let baseline_value = runs[0];
        let variance = (runs[1] - runs[0]).abs() / runs[0].max(1.0);
        let within_threshold = variance <= self.max_variance;
        let baseline_record = super::phase1_design::BaselineRecord::new(timestamp, git_commit, metric.to_string(), measurement_command.to_string(), baseline_value, runs.clone(), variance, within_threshold);
        if !within_threshold {
            eprintln!("\nWARNING: Baseline variance {:.2}% exceeds threshold {:.2}%", variance * 100.0, self.max_variance * 100.0);
            return Ok(super::phase1_design::BaselineVerificationResult::failure_with_data(baseline_record, format!("Variance {:.2}% exceeds threshold {:.2}%", variance * 100.0, self.max_variance * 100.0)));
        }
        eprintln!("\nBaseline verified successfully!");
        eprintln!("  Value: {:.2}", baseline_value);
        Ok(super::phase1_design::BaselineVerificationResult::success(baseline_record))
    }
}

impl Default for MetricEvaluator { fn default() -> Self { Self::new(0.05) } }

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_metric_error_display() {
        let error = MetricError { message: "Test error message".to_string() };
        let output = format!("{}", error);
        assert!(output.contains("Test error message"));
        assert!(output.contains("SUGGESTION"));
    }

    #[test]
    fn test_metric_error_source() {
        let error = MetricError { message: "Test error".to_string() };
        assert!(error.source().is_none());
    }

    #[test]
    fn test_metric_evaluator_new() {
        let evaluator = MetricEvaluator::new(0.1);
        assert_eq!(evaluator.max_variance, 0.1);
    }

    #[test]
    fn test_metric_evaluator_default() {
        let evaluator = MetricEvaluator::default();
        assert_eq!(evaluator.max_variance, 0.05);
    }

    #[test]
    fn test_execute_measurement_valid_output() {
        let evaluator = MetricEvaluator::default();
        let result = evaluator.execute_measurement("echo 42.5");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42.5);
    }

    #[test]
    fn test_execute_measurement_integer_output() {
        let evaluator = MetricEvaluator::default();
        let result = evaluator.execute_measurement("echo 100");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100.0);
    }

    #[test]
    fn test_execute_measurement_negative_output() {
        let evaluator = MetricEvaluator::default();
        let result = evaluator.execute_measurement("echo -25.75");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), -25.75);
    }

    #[test]
    fn test_execute_measurement_empty_command() {
        let evaluator = MetricEvaluator::default();
        let result = evaluator.execute_measurement("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty measurement command"));
    }

    #[test]
    fn test_execute_measurement_invalid_output() {
        let evaluator = MetricEvaluator::default();
        let result = evaluator.execute_measurement("echo not_a_number");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Could not parse measurement output"));
    }

    #[test]
    fn test_execute_measurement_command_not_found() {
        let evaluator = MetricEvaluator::default();
        let result = evaluator.execute_measurement("nonexistent_command_12345");
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_measurement_whitespace_handling() {
        let evaluator = MetricEvaluator::default();
        let result = evaluator.execute_measurement("echo   123.45   ");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 123.45);
    }

    #[test]
    fn test_execute_measurement_scientific_notation() {
        let evaluator = MetricEvaluator::default();
        let result = evaluator.execute_measurement("echo 1.5e2");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 150.0);
    }

    #[test]
    fn test_get_git_commit_hash_returns_string() {
        let evaluator = MetricEvaluator::default();
        let result = evaluator.get_git_commit_hash();
        assert!(result.is_ok());
        let hash = result.unwrap();
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_verify_baseline_successful() {
        let evaluator = MetricEvaluator::new(0.1); // 10% max variance
        let result = evaluator.verify_baseline("test_metric", "echo 100");
        assert!(result.is_ok());
        let baseline_result = result.unwrap();
        assert!(baseline_result.success);
        assert!(baseline_result.baseline_record.is_some());
        assert!(baseline_result.error_message.is_none());
        let record = baseline_result.baseline_record.unwrap();
        assert_eq!(record.metric, "test_metric");
        assert_eq!(record.value, 100.0);
        assert_eq!(record.verification_runs.len(), 2);
        assert!(record.within_threshold);
    }

    #[test]
    fn test_verify_baseline_with_metric_name() {
        let evaluator = MetricEvaluator::default();
        let result = evaluator.verify_baseline("performance_metric", "echo 50");
        assert!(result.is_ok());
        let baseline_result = result.unwrap();
        assert!(baseline_result.success);
        let record = baseline_result.baseline_record.unwrap();
        assert_eq!(record.metric, "performance_metric");
    }

    #[test]
    fn test_verify_baseline_failed_command() {
        let evaluator = MetricEvaluator::default();
        let result = evaluator.verify_baseline("test_metric", "nonexistent_command_xyz");
        assert!(result.is_ok()); // Returns Ok with failure result
        let baseline_result = result.unwrap();
        assert!(!baseline_result.success);
        assert!(baseline_result.error_message.is_some());
    }

    #[test]
    fn test_verify_baseline_empty_command() {
        let evaluator = MetricEvaluator::default();
        let result = evaluator.verify_baseline("test_metric", "");
        assert!(result.is_ok()); // Returns Ok with failure result
        let baseline_result = result.unwrap();
        assert!(!baseline_result.success);
        assert!(baseline_result.error_message.is_some());
    }

    #[test]
    fn test_verify_baseline_record_has_timestamp() {
        let evaluator = MetricEvaluator::default();
        let result = evaluator.verify_baseline("test", "echo 100");
        assert!(result.is_ok());
        let record = result.unwrap().baseline_record.unwrap();
        assert!(!record.timestamp.is_empty());
    }

    #[test]
    fn test_verify_baseline_record_has_git_commit() {
        let evaluator = MetricEvaluator::default();
        let result = evaluator.verify_baseline("test", "echo 100");
        assert!(result.is_ok());
        let record = result.unwrap().baseline_record.unwrap();
        assert!(!record.git_commit.is_empty());
    }

    #[test]
    fn test_verify_baseline_record_has_command() {
        let evaluator = MetricEvaluator::default();
        let result = evaluator.verify_baseline("test", "echo 100");
        assert!(result.is_ok());
        let record = result.unwrap().baseline_record.unwrap();
        assert_eq!(record.measurement_command, "echo 100");
    }

    #[test]
    fn test_verify_baseline_variance_calculation() {
        let evaluator = MetricEvaluator::default();
        let result = evaluator.verify_baseline("test", "echo 100");
        assert!(result.is_ok());
        let record = result.unwrap().baseline_record.unwrap();
        // With echo 100, both runs should return 100, so variance should be 0
        assert_eq!(record.variance, 0.0);
    }

    #[test]
    fn test_verify_baseline_within_threshold_is_true_for_zero_variance() {
        let evaluator = MetricEvaluator::default();
        let result = evaluator.verify_baseline("test", "echo 100");
        assert!(result.is_ok());
        let record = result.unwrap().baseline_record.unwrap();
        assert!(record.within_threshold);
    }

    #[test]
    fn test_metric_evaluator_debug() {
        let evaluator = MetricEvaluator::new(0.15);
        let debug_str = format!("{:?}", evaluator);
        assert!(debug_str.contains("MetricEvaluator"));
    }

    #[test]
    fn test_metric_evaluator_clone() {
        let evaluator1 = MetricEvaluator::new(0.2);
        let evaluator2 = evaluator1.clone();
        assert_eq!(evaluator2.max_variance, 0.2);
    }
}
