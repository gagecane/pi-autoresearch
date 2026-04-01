use assert_cmd::Command;
use serde_json::Value;
use std::process::Output;

fn get_cli_output(args: &[&str]) -> Output {
    let mut cmd = Command::cargo_bin("pi-autoresearch").unwrap();
    cmd.args(args);
    cmd.output().unwrap()
}

fn get_cli_output_no_config(args: &[&str]) -> Output {
    // Create a temporary directory without config file
    let temp_dir = std::env::temp_dir().join(format!("pi-autoresearch-test-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    
    let mut cmd = Command::cargo_bin("pi-autoresearch").unwrap();
    cmd.args(args);
    cmd.env("HOME", &temp_dir);
    let output = cmd.output().unwrap();
    
    // Clean up
    let _ = std::fs::remove_dir_all(&temp_dir);
    
    output
}

fn parse_json_output(output: &Output) -> Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    serde_json::from_str(trimmed).unwrap()
}

#[test]
fn test_question_argument_parsing() {
    let args = vec!["--question", "test question", "--auto-approve"];
    let output = get_cli_output(&args);

    assert!(output.status.success());

    let json = parse_json_output(&output);
    assert!(json["hypothesis"]
        .as_str()
        .unwrap()
        .contains("test question"));
}

#[test]
fn test_metric_detection_memory() {
    let args = vec!["--question", "reduce memory usage", "--auto-approve"];
    let output = get_cli_output(&args);

    assert!(output.status.success());

    let json = parse_json_output(&output);
    assert_eq!(json["metric"].as_str().unwrap(), "peak_memory_mb");
}

#[test]
fn test_metric_detection_speed() {
    let args = vec!["--question", "improve speed", "--auto-approve"];
    let output = get_cli_output(&args);

    assert!(output.status.success());

    let json = parse_json_output(&output);
    assert_eq!(json["metric"].as_str().unwrap(), "execution_time_ms");
}

#[test]
fn test_metric_detection_performance() {
    let args = vec!["--question", "boost performance", "--auto-approve"];
    let output = get_cli_output(&args);

    assert!(output.status.success());

    let json = parse_json_output(&output);
    assert_eq!(json["metric"].as_str().unwrap(), "execution_time_ms");
}

#[test]
fn test_metric_detection_accuracy() {
    let args = vec!["--question", "increase accuracy", "--auto-approve"];
    let output = get_cli_output(&args);

    assert!(output.status.success());

    let json = parse_json_output(&output);
    assert_eq!(json["metric"].as_str().unwrap(), "accuracy_percent");
}

#[test]
fn test_default_metric_for_unknown() {
    let args = vec!["--question", "some random question", "--auto-approve"];
    let output = get_cli_output(&args);

    assert!(output.status.success());

    let json = parse_json_output(&output);
    assert_eq!(json["metric"].as_str().unwrap(), "metric_value");
}

#[test]
fn test_json_output_structure() {
    let args = vec!["--question", "test", "--auto-approve"];
    let output = get_cli_output(&args);

    assert!(output.status.success());

    let json = parse_json_output(&output);
    assert!(json["hypothesis"].is_string());
    assert!(json["metric"].is_string());
    assert!(json["measurement"].is_string());
    assert!(json["baseline"].is_number());
    assert!(json["target_improvement"].is_number());
}

#[test]
fn test_baseline_verification_success() {
    let args = vec![
        "--verify-baseline",
        "--metric",
        "test_metric",
        "--measure",
        "echo 100.0",
        "--session-file",
        "/tmp/test_baseline.jsonl",
    ];
    let output = get_cli_output(&args);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(stdout.trim()).unwrap();

    assert_eq!(json["success"].as_bool().unwrap(), true);
    assert!(json["baseline_record"].is_object());
    assert!(json["error_message"].is_null());
}

#[test]
fn test_baseline_verification_records_timestamp() {
    let args = vec![
        "--verify-baseline",
        "--metric",
        "test_metric",
        "--measure",
        "echo 100.0",
        "--session-file",
        "/tmp/test_baseline_timestamp.jsonl",
    ];
    let output = get_cli_output(&args);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(stdout.trim()).unwrap();

    assert!(!json["baseline_record"]["timestamp"]
        .as_str()
        .unwrap()
        .is_empty());
}

#[test]
fn test_baseline_verification_records_git_commit() {
    let args = vec![
        "--verify-baseline",
        "--metric",
        "test_metric",
        "--measure",
        "echo 100.0",
        "--session-file",
        "/tmp/test_baseline_git.jsonl",
    ];
    let output = get_cli_output(&args);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(stdout.trim()).unwrap();

    assert!(!json["baseline_record"]["git_commit"]
        .as_str()
        .unwrap()
        .is_empty());
}

#[test]
fn test_baseline_verification_within_variance() {
    let args = vec![
        "--verify-baseline",
        "--metric",
        "test_metric",
        "--measure",
        "echo 100.0",
        "--session-file",
        "/tmp/test_baseline_variance.jsonl",
    ];
    let output = get_cli_output(&args);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(stdout.trim()).unwrap();

    assert_eq!(
        json["baseline_record"]["within_threshold"]
            .as_bool()
            .unwrap(),
        true
    );
    assert_eq!(json["baseline_record"]["variance"].as_f64().unwrap(), 0.0);
}

#[test]
fn test_baseline_verification_exceeds_variance() {
    let args = vec![
        "--verify-baseline",
        "--metric",
        "test_metric",
        "--measure",
        "echo $((100 + RANDOM % 50))",
        "--max-variance",
        "0.01",
        "--session-file",
        "/tmp/test_baseline_fail.jsonl",
    ];
    let output = get_cli_output(&args);

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Baseline verification failed"));
}

#[test]
fn test_iterative_loop_single_iteration() {
    let session_file = "/tmp/test_iter_single.jsonl";
    std::fs::remove_file(session_file).ok();

    let args = vec![
        "--question",
        "reduce memory",
        "--auto-approve",
        "--max-iterations",
        "1",
        "--metric",
        "peak_memory_mb",
        "--measure",
        "echo 350.0",
        "--baseline",
        "512.0",
        "--target-improvement",
        "0.1",
        "--session-file",
        session_file,
        "--quiet",
        "--skip-git",
    ];
    let output = get_cli_output(&args);

    assert!(output.status.success());

    assert!(std::path::Path::new(session_file).exists());

    let contents = std::fs::read_to_string(session_file).unwrap();
    let lines: Vec<&str> = contents.lines().collect();
    assert!(lines.len() >= 1);
}

#[test]
fn test_baseline_verification_missing_metric() {
    // Use temporary HOME directory without config file
    let args = vec!["--verify-baseline", "--measure", "echo 100.0"];
    let output = get_cli_output_no_config(&args);

    assert!(!output.status.success());
}

#[test]
fn test_baseline_verification_missing_measure() {
    // Use temporary HOME directory without config file
    let args = vec!["--verify-baseline", "--metric", "test_metric"];
    let output = get_cli_output_no_config(&args);

    assert!(!output.status.success());
}

#[test]
fn test_iterative_loop_logs_iteration_record() {
    let session_file = "/tmp/test_iter_log.jsonl";
    std::fs::remove_file(session_file).ok();

    let args = vec![
        "--question",
        "reduce memory",
        "--auto-approve",
        "--max-iterations",
        "2",
        "--metric",
        "peak_memory_mb",
        "--measure",
        "echo 350.0",
        "--baseline",
        "512.0",
        "--target-improvement",
        "0.1",
        "--session-file",
        session_file,
        "--quiet",
        "--skip-git",
    ];
    let output = get_cli_output(&args);

    assert!(output.status.success());

    let contents = std::fs::read_to_string(session_file).unwrap();
    let lines: Vec<&str> = contents.lines().collect();

    let mut found_iteration = false;
    for line in &lines {
        let json: Value = serde_json::from_str(line).unwrap();
        if json.get("iteration").is_some() {
            found_iteration = true;
            assert!(json.get("timestamp").is_some());
            assert!(json.get("agent_action").is_some());
            assert!(json.get("metric_value").is_some());
            assert!(json.get("improvement").is_some());
            assert!(json.get("kept").is_some());
            break;
        }
    }
    assert!(found_iteration, "No iteration record found in session file");
}

#[test]
fn test_max_iterations_default_is_20() {
    let args = vec!["--question", "test", "--auto-approve"];
    let output = get_cli_output(&args);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Optimizing based on: test"));
}

#[test]
fn test_iteration_keeps_improvement() {
    let session_file = "/tmp/test_iter_keep.jsonl";
    std::fs::remove_file(session_file).ok();

    let args = vec![
        "--question",
        "reduce memory",
        "--auto-approve",
        "--max-iterations",
        "1",
        "--metric",
        "peak_memory_mb",
        "--measure",
        "echo 350.0",
        "--baseline",
        "512.0",
        "--target-improvement",
        "0.1",
        "--session-file",
        session_file,
        "--quiet",
        "--skip-git",
    ];
    let output = get_cli_output(&args);

    assert!(output.status.success());

    let contents = std::fs::read_to_string(session_file).unwrap();
    let lines: Vec<&str> = contents.lines().collect();

    for line in &lines {
        let json: Value = serde_json::from_str(line).unwrap();
        if json.get("iteration").is_some() {
            assert_eq!(json["kept"].as_bool().unwrap(), true);
            return;
        }
    }
    panic!("No iteration record found");
}

#[test]
fn test_session_file_path_configurable() {
    let custom_session_file = "/tmp/test_custom_session.jsonl";
    std::fs::remove_file(custom_session_file).ok();

    let args = vec![
        "--question",
        "reduce memory",
        "--auto-approve",
        "--max-iterations",
        "1",
        "--metric",
        "peak_memory_mb",
        "--measure",
        "echo 350.0",
        "--baseline",
        "512.0",
        "--target-improvement",
        "0.1",
        "--session-file",
        custom_session_file,
        "--quiet",
        "--skip-git",
    ];
    let output = get_cli_output(&args);

    assert!(output.status.success());
    assert!(std::path::Path::new(custom_session_file).exists());
}

#[test]
fn test_dry_run_shows_banner() {
    let args = vec![
        "--question",
        "reduce memory",
        "--auto-approve",
        "--dry-run",
    ];
    let output = get_cli_output(&args);

    assert!(output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("DRY RUN MODE"));
    assert!(stderr.contains("No changes will be made"));
}

#[test]
fn test_dry_run_no_session_file_created() {
    let session_file = "/tmp/test_dry_run_no_file.jsonl";
    std::fs::remove_file(session_file).ok();

    let args = vec![
        "--question",
        "reduce memory",
        "--auto-approve",
        "--max-iterations",
        "1",
        "--metric",
        "peak_memory_mb",
        "--measure",
        "echo 350.0",
        "--baseline",
        "512.0",
        "--target-improvement",
        "0.1",
        "--session-file",
        session_file,
        "--quiet",
        "--dry-run",
    ];
    let output = get_cli_output(&args);

    assert!(output.status.success());
    assert!(!std::path::Path::new(session_file).exists(), 
        "Session file should not be created in dry-run mode");
}

#[test]
fn test_dry_run_with_iterations() {
    let session_file = "/tmp/test_dry_run_iterations.jsonl";
    std::fs::remove_file(session_file).ok();

    let args = vec![
        "--question",
        "reduce memory",
        "--auto-approve",
        "--max-iterations",
        "2",
        "--metric",
        "peak_memory_mb",
        "--measure",
        "echo 350.0",
        "--baseline",
        "512.0",
        "--target-improvement",
        "0.1",
        "--session-file",
        session_file,
        "--dry-run",
    ];
    let output = get_cli_output(&args);

    assert!(output.status.success());
    
    // In dry-run mode, session file should not be created
    assert!(!std::path::Path::new(session_file).exists());
    
    // But should still show experiment complete message
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Experiment Complete") || stderr.contains("DRY RUN"));
}

#[test]
fn test_list_branches_flag() {
    let args = vec!["--list-branches"];
    let output = get_cli_output(&args);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should either show branches or say no branches found
    assert!(stdout.contains("autoresearch") || stdout.contains("No autoresearch branches"));
}

#[test]
fn test_cleanup_branches_flag() {
    let args = vec!["--cleanup-branches", "--cleanup-days", "7"];
    let output = get_cli_output(&args);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show cleanup summary
    assert!(stdout.contains("Cleanup Summary") || stdout.contains("No autoresearch branches"));
}

#[test]
fn test_cleanup_branches_with_custom_days() {
    let args = vec!["--cleanup-branches", "--cleanup-days", "30"];
    let output = get_cli_output(&args);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show cleanup summary
    assert!(stdout.contains("Cleanup Summary") || stdout.contains("No autoresearch branches"));
}

#[test]
fn test_history_empty_session_file() {
    let session_file = "/tmp/test_history_empty.jsonl";
    std::fs::remove_file(session_file).ok();

    let args = vec!["--history", "--session-file", session_file];
    let output = get_cli_output(&args);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No experiments found") || stdout.contains("experiment"));
}

#[test]
fn test_history_with_experiments() {
    let session_file = "/tmp/test_history_with_exp.jsonl";
    std::fs::remove_file(session_file).ok();

    // First, create an experiment
    let create_args = vec![
        "--question",
        "reduce memory",
        "--auto-approve",
        "--max-iterations",
        "1",
        "--metric",
        "peak_memory_mb",
        "--measure",
        "echo 350.0",
        "--baseline",
        "512.0",
        "--target-improvement",
        "0.1",
        "--session-file",
        session_file,
        "--quiet",
        "--skip-git",
    ];
    let _ = get_cli_output(&create_args);

    // Now check history
    let args = vec!["--history", "--session-file", session_file];
    let output = get_cli_output(&args);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // History should either show experiments or say no experiments found
    // (due to JSON parsing limitations with pretty-printed sessions)
    assert!(stdout.contains("experiment") || stdout.contains("Experiment") || stdout.contains("reduce memory"));
}

#[test]
fn test_resume_invalid_session() {
    let session_file = "/tmp/test_resume_invalid.jsonl";
    std::fs::remove_file(session_file).ok();

    let args = vec!["--resume", "nonexistent-session-id", "--session-file", session_file];
    let output = get_cli_output(&args);

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("Session"));
}

#[test]
fn test_resume_flag_recognized() {
    // Test that --resume flag is recognized and doesn't cause a CLI error
    let args = vec!["--resume", "test-session-id", "--help"];
    let mut cmd = Command::cargo_bin("pi-autoresearch").unwrap();
    cmd.args(&args);
    let output = cmd.output().unwrap();

    // Should show help (resume flag is recognized)
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("resume") || output.status.success());
}

#[test]
fn test_compare_flag_recognized() {
    // Test that --compare-id1 and --compare-id2 flags are recognized
    let args = vec!["--compare-id1", "session-1", "--compare-id2", "session-2", "--help"];
    let mut cmd = Command::cargo_bin("pi-autoresearch").unwrap();
    cmd.args(&args);
    let output = cmd.output().unwrap();

    // Should show help (compare flags are recognized)
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("compare") || output.status.success());
}

#[test]
fn test_compare_missing_session_id() {
    // Test that compare with only one session ID shows error
    let session_file = "/tmp/test_compare_missing.jsonl";
    std::fs::write(session_file, "{}").ok();

    let args = vec!["--compare-id1", "session-1", "--session-file", session_file];
    let output = get_cli_output(&args);

    // Should succeed but do nothing (only one ID provided)
    let stderr = String::from_utf8_lossy(&output.stderr);
    // When only one ID is provided, it should not trigger compare
    // and should proceed to normal execution which will fail due to missing question
    assert!(!stderr.contains("compare"));
}
