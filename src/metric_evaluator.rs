use anyhow::Result;
use std::io::Write;
use std::process::Command;
use chrono::Utc;

/// Error type for metric evaluation failures
#[derive(Debug)]
pub struct MetricError {
    pub message: String,
}

impl std::fmt::Display for MetricError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for MetricError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

/// Metric evaluator for executing measurements
#[derive(Debug, Clone)]
pub struct MetricEvaluator {
    max_variance: f64,
}

impl MetricEvaluator {
    pub fn new(max_variance: f64) -> Self {
        Self { max_variance }
    }

    /// Execute a measurement command and return the numeric result
    pub fn execute_measurement(&self, command: &str) -> Result<f64> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow::anyhow!(MetricError {
                message: "Empty measurement command".to_string(),
            }));
        }

        let output = Command::new(parts[0])
            .args(&parts[1..])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(MetricError {
                message: format!("Measurement command failed: {}", stderr),
            }));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let value: f64 = stdout.trim().parse().map_err(|_| {
            anyhow::anyhow!(MetricError {
                message: format!(
                    "Could not parse measurement output as number: '{}'",
                    stdout.trim()
                ),
            })
        })?;

        Ok(value)
    }

    /// Get the current git commit hash
    pub fn get_git_commit_hash(&self) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()?;

        if output.status.success() {
            let hash = String::from_utf8_lossy(&output.stdout);
            Ok(hash.trim().to_string())
        } else {
            Ok("unknown".to_string())
        }
    }

    /// Verify baseline by running measurement multiple times
    pub fn verify_baseline(
        &self,
        metric: &str,
        measurement_command: &str,
    ) -> Result<super::phase1_design::BaselineVerificationResult> {
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
                Ok(value) => {
                    eprintln!("{:.2}", value);
                    runs.push(value);
                }
                Err(e) => {
                    eprintln!("FAILED");
                    return Ok(
                        super::phase1_design::BaselineVerificationResult::failure(format!(
                            "Run {} failed: {}",
                            i, e
                        ))
                    );
                }
            }
        }

        if runs.len() < 2 {
            return Ok(
                super::phase1_design::BaselineVerificationResult::failure(
                    "Insufficient successful runs".to_string()
                )
            );
        }

        let baseline_value = runs[0];
        let variance = (runs[1] - runs[0]).abs() / runs[0].max(1.0);
        let within_threshold = variance <= self.max_variance;

        let baseline_record = super::phase1_design::BaselineRecord::new(
            timestamp,
            git_commit,
            metric.to_string(),
            measurement_command.to_string(),
            baseline_value,
            runs.clone(),
            variance,
            within_threshold,
        );

        if !within_threshold {
            eprintln!(
                "\nWARNING: Baseline variance {:.2}% exceeds threshold {:.2}%",
                variance * 100.0,
                self.max_variance * 100.0
            );
            eprintln!("  Run 1: {:.2}", runs[0]);
            eprintln!("  Run 2: {:.2}", runs[1]);

            return Ok(super::phase1_design::BaselineVerificationResult::failure_with_data(
                baseline_record,
                format!(
                    "Variance {:.2}% exceeds threshold {:.2}%",
                    variance * 100.0,
                    self.max_variance * 100.0
                ),
            ));
        }

        eprintln!("\nBaseline verified successfully!");
        eprintln!("  Value: {:.2}", baseline_value);
        eprintln!("  Variance: {:.2}%", variance * 100.0);
        eprintln!("  Git commit: {}", baseline_record.git_commit);

        Ok(super::phase1_design::BaselineVerificationResult::success(baseline_record))
    }
}

impl Default for MetricEvaluator {
    fn default() -> Self {
        Self::new(0.05) // 5% max variance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_measurement_echo() {
        let evaluator = MetricEvaluator::default();
        let result = evaluator.execute_measurement("echo 42.5");
        assert!(result.is_ok());
        assert!((result.unwrap() - 42.5).abs() < 0.001);
    }

    #[test]
    fn test_execute_measurement_invalid() {
        let evaluator = MetricEvaluator::default();
        let result = evaluator.execute_measurement("echo not_a_number");
        assert!(result.is_err());
    }
}
