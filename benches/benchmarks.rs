use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;
use std::io::Write;
use tempfile::TempDir;
use chrono::Utc;

// Function to benchmark session file parsing
fn benchmark_session_file_parsing(c: &mut Criterion) {
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
    
    c.bench_function("session_file_parsing", |b| {
        b.iter(|| {
            // Actually parse JSON from each line to benchmark real parsing performance
            let mut parsed_count = 0;
            for line in content.lines() {
                if !line.trim().is_empty() {
                    let _parsed: serde_json::Value = serde_json::from_str(line).expect("Failed to parse JSON");
                    parsed_count += 1;
                }
            }
            black_box(parsed_count);
        })
    });
}

// Function to benchmark config file loading
fn benchmark_config_file_loading(c: &mut Criterion) {
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

    c.bench_function("config_file_loading", |b| {
        b.iter(|| {
            let content = fs::read_to_string(&config_file).expect("Failed to read config");
            // Simulate JSON parsing
            let _parsed: serde_json::Value = serde_json::from_str(&content).expect("Failed to parse");
            black_box(_parsed);
        })
    });
}

// Function to benchmark metric detection
fn benchmark_metric_detection(c: &mut Criterion) {
    let test_cases = vec![
        "How can I reduce memory usage in the database module?",
        "How can I speed up the image processing pipeline?",
        "How can I improve model accuracy?",
        "What is the best way to optimize performance?",
        "How to reduce peak memory consumption?",
    ];

    c.bench_function("metric_detection", |b| {
        b.iter(|| {
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
                black_box(_metric);
            }
        })
    });
}

// Function to benchmark git branch name generation
fn benchmark_git_branch_name_generation(c: &mut Criterion) {
    use chrono::Utc;

    c.bench_function("git_branch_name_generation", |b| {
        b.iter(|| {
            let now = Utc::now();
            let timestamp = now.format("%Y%m%d-%H%M%S").to_string();
            let uuid = format!("{:08x}", rand::random::<u32>());
            let _branch_name = format!("autoresearch/{}-{}", timestamp, uuid);
            black_box(_branch_name);
        })
    });
}

// Function to benchmark iteration record creation
fn benchmark_iteration_record_creation(c: &mut Criterion) {
    c.bench_function("iteration_record_creation", |b| {
        b.iter(|| {
            let record = serde_json::json!({
                "iteration": 1,
                "timestamp": Utc::now().to_rfc3339(),
                "value": 95.5,
                "improvement": 0.05,
                "agent_action": "Optimized database query",
                "kept": true
            });
            black_box(record);
        })
    });
}

criterion_group!(
    benches,
    benchmark_session_file_parsing,
    benchmark_config_file_loading,
    benchmark_metric_detection,
    benchmark_git_branch_name_generation,
    benchmark_iteration_record_creation,
);

criterion_main!(benches);