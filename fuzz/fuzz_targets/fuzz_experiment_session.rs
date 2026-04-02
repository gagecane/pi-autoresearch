//! Fuzz target for ExperimentSession JSON parsing
//!
//! This fuzz target tests the parsing of ExperimentSession JSON objects.
//! It helps identify edge cases and potential crashes in the JSON deserialization.

#![no_main]

use libfuzzer_sys::fuzz_target;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct BaselineRecord {
    timestamp: String,
    git_commit: String,
    metric: String,
    measurement_command: String,
    value: f64,
    verification_runs: Vec<f64>,
    variance: f64,
    within_threshold: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct IterationRecord {
    iteration: usize,
    timestamp: String,
    agent_action: String,
    metric_value: f64,
    improvement: f64,
    kept: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ExperimentDesign {
    question: String,
    metric: String,
    measurement_command: String,
    baseline: f64,
    target_improvement: f64,
    max_iterations: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ExperimentSession {
    session_id: String,
    question: String,
    design: ExperimentDesign,
    baseline_record: BaselineRecord,
    iterations: Vec<IterationRecord>,
    best_iteration: Option<usize>,
    start_time: String,
    end_time: Option<String>,
    status: String,
}

fuzz_target!(|data: &[u8]| {
    // Try to parse as UTF-8 and then as JSON
    if let Ok(utf8_str) = std::str::from_utf8(data) {
        // Try to parse as ExperimentSession
        let _ = serde_json::from_str::<ExperimentSession>(utf8_str);
        
        // Also try to parse as a JSON value to catch malformed JSON
        let _ = serde_json::from_str::<serde_json::Value>(utf8_str);
    }
});
