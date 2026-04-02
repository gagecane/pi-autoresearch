//! Fuzz target for full session file parsing
//!
//! This fuzz target tests the parsing of entire session files containing
//! multiple JSON objects (JSONL format with potential multi-line JSON).
//! It helps identify edge cases and potential crashes in the session file parsing.

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

#[derive(Debug)]
enum SessionRecord {
    Baseline(BaselineRecord),
    Iteration(IterationRecord),
    Experiment(Box<ExperimentSession>),
}

/// Parse session file content (simplified version of the actual implementation)
fn parse_session_file(contents: &str) -> Vec<SessionRecord> {
    let mut records = Vec::new();
    
    // Parse multi-line JSON objects
    let mut remaining = contents;
    while let Some(start_pos) = remaining.find('{') {
        let start_idx = start_pos;
        let mut brace_count = 0;
        let mut end_idx = start_idx;
        let mut found_complete = false;
        
        for (i, ch) in remaining[start_idx..].char_indices() {
            match ch {
                '{' => brace_count += 1,
                '}' => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        end_idx = start_idx + i;
                        found_complete = true;
                        break;
                    }
                }
                _ => {}
            }
        }
        
        if found_complete {
            let json_str = &remaining[start_idx..=end_idx];
            
            // Try to parse as ExperimentSession first
            if json_str.contains("\"session_id\"") {
                if let Ok(session) = serde_json::from_str::<ExperimentSession>(json_str) {
                    records.push(SessionRecord::Experiment(Box::new(session)));
                    remaining = &remaining[end_idx + 1..];
                    continue;
                }
            }
            
            // Try other types
            if let Ok(iteration) = serde_json::from_str::<IterationRecord>(json_str) {
                records.push(SessionRecord::Iteration(iteration));
            } else if let Ok(baseline) = serde_json::from_str::<BaselineRecord>(json_str) {
                records.push(SessionRecord::Baseline(baseline));
            }
            
            remaining = &remaining[end_idx + 1..];
        } else {
            break;
        }
    }
    
    // Parse compact JSONL lines
    for line in contents.lines() {
        if line.trim().is_empty() {
            continue;
        }
        
        let trimmed = line.trim();
        
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            // Try to parse as ExperimentSession first
            if trimmed.contains("\"session_id\"") {
                if let Ok(session) = serde_json::from_str::<ExperimentSession>(trimmed) {
                    if !records.iter().any(|r| {
                        if let SessionRecord::Experiment(s) = r {
                            s.session_id == session.session_id
                        } else {
                            false
                        }
                    }) {
                        records.push(SessionRecord::Experiment(Box::new(session)));
                    }
                    continue;
                }
            }
            
            // Try to parse as IterationRecord
            if trimmed.contains("\"iteration\"") && trimmed.contains("\"agent_action\"") {
                if let Ok(iteration) = serde_json::from_str::<IterationRecord>(trimmed) {
                    records.push(SessionRecord::Iteration(iteration));
                    continue;
                }
            }
            
            // Try to parse as BaselineRecord
            if trimmed.contains("\"verification_runs\"") {
                if let Ok(baseline) = serde_json::from_str::<BaselineRecord>(trimmed) {
                    records.push(SessionRecord::Baseline(baseline));
                    continue;
                }
            }
        }
    }
    
    records
}

fuzz_target!(|data: &[u8]| {
    // Try to parse as UTF-8
    if let Ok(utf8_str) = std::str::from_utf8(data) {
        // Parse as session file
        let _ = parse_session_file(utf8_str);
        
        // Also try parsing each line as JSON
        for line in utf8_str.lines() {
            let _ = serde_json::from_str::<serde_json::Value>(line);
        }
    }
});
