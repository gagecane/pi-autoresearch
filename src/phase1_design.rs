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

    // Tests for ExperimentDesign::new()
    #[test]
    fn test_experiment_design_new() {
        let design = ExperimentDesign::new(
            "Test hypothesis".to_string(),
            "test_metric".to_string(),
            "test_command".to_string(),
            100.0,
            0.25
        );
        assert_eq!(design.hypothesis, "Test hypothesis");
        assert_eq!(design.metric, "test_metric");
        assert_eq!(design.measurement, "test_command");
        assert_eq!(design.baseline, 100.0);
        assert_eq!(design.target_improvement, 0.25);
    }

    #[test]
    fn test_experiment_design_clone() {
        let design1 = ExperimentDesign::new(
            "Hypothesis".to_string(),
            "metric".to_string(),
            "command".to_string(),
            50.0,
            0.30
        );
        let design2 = design1.clone();
        assert_eq!(design2.hypothesis, design1.hypothesis);
        assert_eq!(design2.metric, design1.metric);
    }

    // Tests for BaselineRecord::new()
    #[test]
    fn test_baseline_record_new() {
        let record = BaselineRecord::new(
            "2024-01-01T00:00:00Z".to_string(),
            "abc123".to_string(),
            "test_metric".to_string(),
            "test_command".to_string(),
            100.0,
            vec![100.0, 101.0],
            0.01,
            true
        );
        assert_eq!(record.timestamp, "2024-01-01T00:00:00Z");
        assert_eq!(record.git_commit, "abc123");
        assert_eq!(record.metric, "test_metric");
        assert_eq!(record.measurement_command, "test_command");
        assert_eq!(record.value, 100.0);
        assert_eq!(record.verification_runs, vec![100.0, 101.0]);
        assert_eq!(record.variance, 0.01);
        assert!(record.within_threshold);
    }

    #[test]
    fn test_baseline_record_clone() {
        let record1 = BaselineRecord::new(
            "timestamp".to_string(),
            "commit".to_string(),
            "metric".to_string(),
            "command".to_string(),
            50.0,
            vec![50.0],
            0.0,
            true
        );
        let record2 = record1.clone();
        assert_eq!(record2.metric, record1.metric);
        assert_eq!(record2.value, record1.value);
    }

    #[test]
    fn test_baseline_record_debug() {
        let record = BaselineRecord::new(
            "timestamp".to_string(),
            "commit".to_string(),
            "test_metric".to_string(),
            "command".to_string(),
            100.0,
            vec![100.0],
            0.0,
            true
        );
        let debug_str = format!("{:?}", record);
        assert!(debug_str.contains("BaselineRecord"));
        assert!(debug_str.contains("test_metric"));
    }

    // Tests for BaselineVerificationResult
    #[test]
    fn test_baseline_verification_result_success() {
        let record = BaselineRecord::new(
            "timestamp".to_string(),
            "commit".to_string(),
            "metric".to_string(),
            "command".to_string(),
            100.0,
            vec![100.0],
            0.0,
            true
        );
        let result = BaselineVerificationResult::success(record);
        assert!(result.success);
        assert!(result.baseline_record.is_some());
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_baseline_verification_result_failure() {
        let result = BaselineVerificationResult::failure("Test error".to_string());
        assert!(!result.success);
        assert!(result.baseline_record.is_none());
        assert_eq!(result.error_message, Some("Test error".to_string()));
    }

    #[test]
    fn test_baseline_verification_result_failure_with_data() {
        let record = BaselineRecord::new(
            "timestamp".to_string(),
            "commit".to_string(),
            "metric".to_string(),
            "command".to_string(),
            100.0,
            vec![100.0],
            0.0,
            false
        );
        let result = BaselineVerificationResult::failure_with_data(record, "Variance too high".to_string());
        assert!(!result.success);
        assert!(result.baseline_record.is_some());
        assert_eq!(result.error_message, Some("Variance too high".to_string()));
    }

    #[test]
    fn test_baseline_verification_result_debug() {
        let result = BaselineVerificationResult::failure("error".to_string());
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("BaselineVerificationResult"));
    }

    // Tests for generate_design()
    #[test]
    fn test_generate_design_memory_question() {
        let design = generate_design("How can I reduce memory usage?");
        assert_eq!(design.metric, "peak_memory_mb");
        assert_eq!(design.baseline, 512.0);
        assert!(design.hypothesis.contains("How can I reduce memory usage?"));
    }

    #[test]
    fn test_generate_design_speed_question() {
        let design = generate_design("How can I improve speed?");
        assert_eq!(design.metric, "execution_time_ms");
        assert_eq!(design.baseline, 1000.0);
    }

    #[test]
    fn test_generate_design_performance_question() {
        let design = generate_design("Optimize performance of this function");
        assert_eq!(design.metric, "execution_time_ms");
        assert_eq!(design.baseline, 1000.0);
    }

    #[test]
    fn test_generate_design_accuracy_question() {
        let design = generate_design("Improve model accuracy");
        assert_eq!(design.metric, "accuracy_percent");
        assert_eq!(design.baseline, 85.0);
    }

    #[test]
    fn test_generate_design_default_question() {
        let design = generate_design("Make this better");
        assert_eq!(design.metric, "metric_value");
        assert_eq!(design.baseline, 100.0);
        assert_eq!(design.measurement, "Run evaluation script");
    }

    #[test]
    fn test_generate_design_case_insensitive() {
        let design1 = generate_design("MEMORY optimization");
        let design2 = generate_design("memory optimization");
        assert_eq!(design1.metric, design2.metric);
    }

    #[test]
    fn test_generate_design_target_improvement_default() {
        let design = generate_design("Test question");
        assert_eq!(design.target_improvement, 0.30);
    }

    #[test]
    fn test_generate_design_hypothesis_format() {
        let question = "Improve cache performance";
        let design = generate_design(question);
        assert_eq!(design.hypothesis, format!("Optimizing based on: {}", question));
    }
}
