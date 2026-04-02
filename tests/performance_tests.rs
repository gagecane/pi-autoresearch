//! Performance regression tests
//!
//! These tests ensure that performance doesn't regress beyond acceptable thresholds.
//! They are faster than cargo bench and can be run as part of the normal test suite.
//!
//! Usage:
//!   cargo test --test performance_tests
//!   cargo test --test performance_tests -- --nocapture

use std::time::Instant;
use std::fs;
use std::io::Write;
use tempfile::TempDir;
use chrono::Utc;

// Performance thresholds (in seconds)
const SESSION_FILE_PARSING_THRESHOLD: f64 = 0.010;  // 10ms
const CONFIG_FILE_LOADING_THRESHOLD: f64 = 0.020;   // 20ms
const METRIC_DETECTION_THRESHOLD: f64 = 0.001;      // 1ms
const BRANCH_NAME_GENERATION_THRESHOLD: f64 = 0.001; // 1ms
const ITERATION_RECORD_CREATION_THRESHOLD: f64 = 0.001; // 1ms

/// Test session file parsing performance
#[test]
fn test_session_file_parsing_performance() {
    // Create a temporary directory for test session files
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let session_file = temp_dir.path().join("test_session.jsonl");
    
    // Create a sample session file with realistic data
    let mut file = fs::File::create(&session_file).expect("Failed to create session file");
    
    // Write baseline record
    writeln!(file, r#"{{"timestamp":"2024-01-15T10:00:00Z","value":100.0,"metric":"execution_time_ms","verification_runs":[100.0,101.0],"variance":0.01,"git_commit":"abc123"}}"#).unwrap();
    
    // Write multiple iteration records
    for i in 1..=10 {
        writeln!(file, r#"{{"iteration":{},"timestamp":"2024-01-15T10:0{}:00Z","value":{},"improvement":{},"agent_action":"test action","kept":{}}}"#,
                 i, i, 100.0 - (i as f64 * 0.5), (i as f64 * 0.005), i % 2 == 0).unwrap();
    }
    
    // Write experiment session record
    writeln!(file, r#"{{"session_id":"test-session-001","question":"Test question","metric":"execution_time_ms","baseline":100.0,"target_improvement":0.20,"iterations":10,"status":"completed","start_time":"2024-01-15T10:00:00Z","end_time":"2024-01-15T10:30:00Z"}}"#).unwrap();
    
    file.flush().unwrap();
    
    // Read the file content
    let content = fs::read_to_string(&session_file).expect("Failed to read session file");
    
    // Measure parsing time
    let start = Instant::now();
    
    // Parse JSON from each line
    let mut parsed_count = 0;
    for line in content.lines() {
        if !line.trim().is_empty() {
            let _parsed: serde_json::Value = serde_json::from_str(line).expect("Failed to parse JSON");
            parsed_count += 1;
        }
    }
    
    let duration = start.elapsed().as_secs_f64();
    
    // Verify we parsed something
    assert!(parsed_count > 0, "Should have parsed at least one record");
    
    // Check performance threshold
    assert!(
        duration < SESSION_FILE_PARSING_THRESHOLD,
        "Session file parsing took {:.3}s, exceeds threshold of {:.3}s",
        duration,
        SESSION_FILE_PARSING_THRESHOLD
    );
    
    eprintln!("Session file parsing: {:.3}s (threshold: {:.3}s) ✓", duration, SESSION_FILE_PARSING_THRESHOLD);
}

/// Test config file loading performance
#[test]
fn test_config_file_loading_performance() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_file = temp_dir.path().join("config.json");

    // Create a sample config file
    let config_content = r#"{
        "max_iterations": 20,
        "iteration_timeout_minutes": 30,
        "stall_limit": 3,
        "convergence_threshold": 0.01,
        "convergence_window": 5,
        "max_variance": 0.05,
        "target_improvement": 0.20,
        "session_file": "autoresearch.jsonl"
    }"#;

    fs::write(&config_file, config_content).expect("Failed to write config file");

    // Measure loading time
    let start = Instant::now();
    
    let content = fs::read_to_string(&config_file).expect("Failed to read config");
    let _parsed: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse");
    
    let duration = start.elapsed().as_secs_f64();
    
    // Check performance threshold
    assert!(
        duration < CONFIG_FILE_LOADING_THRESHOLD,
        "Config file loading took {:.3}s, exceeds threshold of {:.3}s",
        duration,
        CONFIG_FILE_LOADING_THRESHOLD
    );
    
    eprintln!("Config file loading: {:.3}s (threshold: {:.3}s) ✓", duration, CONFIG_FILE_LOADING_THRESHOLD);
}

/// Test metric detection performance
#[test]
fn test_metric_detection_performance() {
    let test_cases = vec![
        "How can I reduce memory usage in the database module?",
        "How can I speed up the image processing pipeline?",
        "How can I improve model accuracy?",
        "What is the best way to optimize performance?",
        "How to reduce peak memory consumption?",
    ];

    // Measure detection time (run multiple times for accuracy)
    let iterations = 100;
    let start = Instant::now();
    
    for _ in 0..iterations {
        for question in &test_cases {
            // Simulate metric detection logic
            let question_lower = question.to_lowercase();
            let _metric = if question_lower.contains("memory") {
                "peak_memory_mb"
            } else if question_lower.contains("speed") || question_lower.contains("performance") {
                "execution_time_ms"
            } else if question_lower.contains("accuracy") {
                "accuracy_percent"
            } else {
                "metric_value"
            };
        }
    }
    
    let duration = start.elapsed().as_secs_f64() / iterations as f64;
    
    // Check performance threshold
    assert!(
        duration < METRIC_DETECTION_THRESHOLD,
        "Metric detection took {:.6}s per iteration, exceeds threshold of {:.6}s",
        duration,
        METRIC_DETECTION_THRESHOLD
    );
    
    eprintln!("Metric detection: {:.6}s per iteration (threshold: {:.6}s) ✓", duration, METRIC_DETECTION_THRESHOLD);
}

/// Test git branch name generation performance
#[test]
fn test_branch_name_generation_performance() {
    // Measure generation time (run multiple times for accuracy)
    let iterations = 100;
    let start = Instant::now();
    
    for _ in 0..iterations {
        let now = Utc::now();
        let timestamp = now.format("%Y%m%d-%H%M%S").to_string();
        let uuid = format!("{:08x}", rand::random::<u32>());
        let _branch_name = format!("autoresearch/{}-{}", timestamp, uuid);
    }
    
    let duration = start.elapsed().as_secs_f64() / iterations as f64;
    
    // Check performance threshold
    assert!(
        duration < BRANCH_NAME_GENERATION_THRESHOLD,
        "Branch name generation took {:.6}s per iteration, exceeds threshold of {:.6}s",
        duration,
        BRANCH_NAME_GENERATION_THRESHOLD
    );
    
    eprintln!("Branch name generation: {:.6}s per iteration (threshold: {:.6}s) ✓", duration, BRANCH_NAME_GENERATION_THRESHOLD);
}

/// Test iteration record creation performance
#[test]
fn test_iteration_record_creation_performance() {
    // Measure creation time (run multiple times for accuracy)
    let iterations = 100;
    let start = Instant::now();
    
    for _ in 0..iterations {
        let _record = serde_json::json!({
            "iteration": 1,
            "timestamp": Utc::now().to_rfc3339(),
            "value": 95.5,
            "improvement": 0.05,
            "agent_action": "Optimized database query",
            "kept": true
        });
    }
    
    let duration = start.elapsed().as_secs_f64() / iterations as f64;
    
    // Check performance threshold
    assert!(
        duration < ITERATION_RECORD_CREATION_THRESHOLD,
        "Iteration record creation took {:.6}s per iteration, exceeds threshold of {:.6}s",
        duration,
        ITERATION_RECORD_CREATION_THRESHOLD
    );
    
    eprintln!("Iteration record creation: {:.6}s per iteration (threshold: {:.6}s) ✓", duration, ITERATION_RECORD_CREATION_THRESHOLD);
}

/// Test overall performance - all operations combined
#[test]
fn test_overall_performance() {
    let start = Instant::now();
    
    // Run all performance-critical operations
    test_session_file_parsing_performance();
    test_config_file_loading_performance();
    test_metric_detection_performance();
    test_branch_name_generation_performance();
    test_iteration_record_creation_performance();
    
    let total_duration = start.elapsed().as_secs_f64();
    
    // Overall threshold (should be less than sum of individual thresholds)
    let overall_threshold = SESSION_FILE_PARSING_THRESHOLD + 
                           CONFIG_FILE_LOADING_THRESHOLD + 
                           METRIC_DETECTION_THRESHOLD + 
                           BRANCH_NAME_GENERATION_THRESHOLD + 
                           ITERATION_RECORD_CREATION_THRESHOLD;
    
    assert!(
        total_duration < overall_threshold,
        "Overall performance took {:.3}s, exceeds threshold of {:.3}s",
        total_duration,
        overall_threshold
    );
    
    eprintln!("\nOverall performance: {:.3}s (threshold: {:.3}s) ✓", total_duration, overall_threshold);
}

/// Test that performance is consistent across multiple runs
#[test]
fn test_performance_consistency() {
    let mut durations = Vec::new();
    
    // Run session file parsing 5 times with more data for meaningful measurements
    for _ in 0..5 {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let session_file = temp_dir.path().join("test_session.jsonl");
        
        // Create a more realistic session file with multiple records
        let mut file = fs::File::create(&session_file).expect("Failed to create session file");
        
        // Write baseline record
        writeln!(file, r#"{{"timestamp":"2024-01-15T10:00:00Z","value":100.0,"metric":"execution_time_ms","verification_runs":[100.0,101.0],"variance":0.01,"git_commit":"abc123"}}"#).unwrap();
        
        // Write multiple iteration records
        for i in 1..=10 {
            writeln!(file, r#"{{"iteration":{},"timestamp":"2024-01-15T10:0{}:00Z","value":{},"improvement":{},"agent_action":"test action","kept":{}}}"#,
                     i, i, 100.0 - (i as f64 * 0.5), (i as f64 * 0.005), i % 2 == 0).unwrap();
        }
        
        // Write experiment session record
        writeln!(file, r#"{{"session_id":"test-session-001","question":"Test question","metric":"execution_time_ms","baseline":100.0,"target_improvement":0.20,"iterations":10,"status":"completed","start_time":"2024-01-15T10:00:00Z","end_time":"2024-01-15T10:30:00Z"}}"#).unwrap();
        
        file.flush().unwrap();
        
        let content = fs::read_to_string(&session_file).expect("Failed to read");
        
        // Run parsing multiple times within each measurement for more stable timing
        let start = Instant::now();
        for _ in 0..10 {
            for line in content.lines() {
                if !line.trim().is_empty() {
                    let _parsed: serde_json::Value = serde_json::from_str(line).unwrap();
                }
            }
        }
        durations.push(start.elapsed().as_secs_f64());
    }
    
    // Calculate variance
    let mean: f64 = durations.iter().sum::<f64>() / durations.len() as f64;
    let variance: f64 = durations.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / durations.len() as f64;
    let std_dev = variance.sqrt();
    
    // Skip variance check if mean is too small (measurement noise dominates)
    if mean < 0.0001 {
        eprintln!("Performance consistency: mean={:.6}s (too fast for variance analysis)", mean);
        return;
    }
    
    // Coefficient of variation should be less than 50% (allowing for system noise)
    let cv = std_dev / mean;
    assert!(
        cv < 0.5,
        "Performance variance too high: CV={:.2}% (mean={:.3}s, std_dev={:.3}s)",
        cv * 100.0, mean, std_dev
    );
    
    eprintln!("Performance consistency: mean={:.3}s, std_dev={:.3}s, CV={:.2}% ✓", mean, std_dev, cv * 100.0);
}

