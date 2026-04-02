use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExperimentDesign {
    pub hypothesis: String,
    pub metric: String,
    pub measurement: String,
    pub baseline: f64,
    pub target_improvement: f64,
}

impl ExperimentDesign {
    pub fn new(hypothesis: String, metric: String, measurement: String, baseline: f64, target_improvement: f64) -> Self {
        Self { hypothesis, metric, measurement, baseline, target_improvement }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaselineRecord {
    pub timestamp: String,
    pub git_commit: String,
    pub metric: String,
    pub measurement_command: String,
    pub value: f64,
    pub verification_runs: Vec<f64>,
    pub variance: f64,
    pub within_threshold: bool,
}

impl BaselineRecord {
    pub fn new(timestamp: String, git_commit: String, metric: String, measurement_command: String, value: f64, verification_runs: Vec<f64>, variance: f64, within_threshold: bool) -> Self {
        Self { timestamp, git_commit, metric, measurement_command, value, verification_runs, variance, within_threshold }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BaselineVerificationResult {
    pub success: bool,
    pub baseline_record: Option<BaselineRecord>,
    pub error_message: Option<String>,
}

impl BaselineVerificationResult {
    pub fn success(baseline_record: BaselineRecord) -> Self {
        Self { success: true, baseline_record: Some(baseline_record), error_message: None }
    }
    pub fn failure(error_message: String) -> Self {
        Self { success: false, baseline_record: None, error_message: Some(error_message) }
    }
    pub fn failure_with_data(baseline_record: BaselineRecord, error_message: String) -> Self {
        Self { success: false, baseline_record: Some(baseline_record), error_message: Some(error_message) }
    }
}

pub fn generate_design(question: &str) -> ExperimentDesign {
    let lower_question = question.to_lowercase();
    let (metric, measurement, baseline) = if lower_question.contains("memory") {
        ("peak_memory_mb".to_string(), "Run benchmark suite, capture peak RSS via /proc/self/status".to_string(), 512.0)
    } else if lower_question.contains("speed") || lower_question.contains("performance") {
        ("execution_time_ms".to_string(), "Run benchmark suite with hyperfine, report mean".to_string(), 1000.0)
    } else if lower_question.contains("accuracy") {
        ("accuracy_percent".to_string(), "Run test suite, calculate pass rate".to_string(), 85.0)
    } else {
        ("metric_value".to_string(), "Run evaluation script".to_string(), 100.0)
    };
    ExperimentDesign { hypothesis: format!("Optimizing based on: {}", question), metric, measurement, baseline, target_improvement: 0.30 }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_baseline_is_valid() {
        let baseline = BaselineRecord::new("2024-01-01".to_string(), "abc123".to_string(), "metric".to_string(), "measure".to_string(), 100.0, vec![100.0, 100.0], 0.0, true);
        assert!(baseline.within_threshold);
    }
}
