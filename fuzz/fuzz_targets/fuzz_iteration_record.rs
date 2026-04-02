//! Fuzz target for IterationRecord JSON parsing
//!
//! This fuzz target tests the parsing of IterationRecord JSON objects.
//! It helps identify edge cases and potential crashes in the JSON deserialization.

#![no_main]

use libfuzzer_sys::fuzz_target;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct IterationRecord {
    iteration: usize,
    timestamp: String,
    agent_action: String,
    metric_value: f64,
    improvement: f64,
    kept: bool,
}

fuzz_target!(|data: &[u8]| {
    // Try to parse as UTF-8 and then as JSON
    if let Ok(utf8_str) = std::str::from_utf8(data) {
        // Try to parse as IterationRecord
        let _ = serde_json::from_str::<IterationRecord>(utf8_str);
        
        // Also try to parse as a JSON value to catch malformed JSON
        let _ = serde_json::from_str::<serde_json::Value>(utf8_str);
    }
});
