//! Fuzz target for BaselineRecord JSON parsing
//!
//! This fuzz target tests the parsing of BaselineRecord JSON objects.
//! It helps identify edge cases and potential crashes in the JSON deserialization.

#![no_main]

use libfuzzer_sys::fuzz_target;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
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

fuzz_target!(|data: &[u8]| {
    // Try to parse as UTF-8 and then as JSON
    if let Ok(utf8_str) = std::str::from_utf8(data) {
        // Try to parse as BaselineRecord
        let _ = serde_json::from_str::<BaselineRecord>(utf8_str);
        
        // Also try to parse as a JSON value to catch malformed JSON
        let _ = serde_json::from_str::<serde_json::Value>(utf8_str);
    }
});
