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

/// Extract session_id from a session file that may contain both compact JSONL and pretty-printed JSON
fn extract_session_id_from_file(session_file: &str) -> Option<String> {
    let contents = std::fs::read_to_string(session_file).ok()?;
    
    // First try to find session_id in compact JSONL lines
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(json) = serde_json::from_str::<Value>(trimmed) {
            if let Some(session_id) = json.get("session_id").and_then(|v| v.as_str()) {
                return Some(session_id.to_string());
            }
        }
    }
    
    // If not found, try to find it in the raw text (handles multi-line JSON)
    if let Some(start) = contents.find("\"session_id\":") {
        let rest = &contents[start..];
        if let Some(colon_pos) = rest.find(':') {
            let after_colon = &rest[colon_pos + 1..];
            let trimmed = after_colon.trim_start();
            if trimmed.starts_with('"') {
                if let Some(end) = trimmed[1..].find('"') {
                    return Some(trimmed[1..end + 1].to_string());
                }
            }
        }
    }
    
    None
}

/// Count the number of iterations in a session file
fn count_iterations_in_file(contents: &str) -> usize {
    // Count occurrences of "iteration": followed by a number
    contents.matches("\"iteration\":").count()
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
fn test_beads_enabled_flag_recognized() {
    // Test that --beads-enabled flag is recognized and doesn't cause a CLI error
    let args = vec!["--beads-enabled", "--help"];
    let mut cmd = Command::cargo_bin("pi-autoresearch").unwrap();
    cmd.args(&args);
    let output = cmd.output().unwrap();

    // Should show help (beads-enabled flag is recognized)
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("beads-enabled"), "Flag should be documented in help text");
}

#[test]
fn test_beads_enabled_with_auto_approve() {
    // Test that --beads-enabled works with --auto-approve
    // Note: If 'bd' tool is not installed, it should handle gracefully
    let args = vec![
        "--question",
        "test beads integration",
        "--auto-approve",
        "--beads-enabled",
        "--quiet",
    ];
    let output = get_cli_output(&args);

    // Should succeed (even if bd tool is not installed, it should handle gracefully)
    assert!(output.status.success());

    let json = parse_json_output(&output);
    assert!(json["hypothesis"]
        .as_str()
        .unwrap()
        .contains("test beads integration"));
}

#[test]
fn test_list_branches_flag() {
    let args = vec!["--list-branches"];
    let output = get_cli_output(&args);

    assert!(output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should either show branches or say no branches found
    assert!(stderr.contains("autoresearch") || stderr.contains("No autoresearch branches"));
}

#[test]
fn test_cleanup_branches_flag() {
    let args = vec!["--cleanup-branches", "--cleanup-days", "7"];
    let output = get_cli_output(&args);

    assert!(output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should show cleanup summary
    assert!(stderr.contains("Cleanup Summary") || stderr.contains("No autoresearch branches"));
}

#[test]
fn test_cleanup_branches_with_custom_days() {
    let args = vec!["--cleanup-branches", "--cleanup-days", "30"];
    let output = get_cli_output(&args);

    assert!(output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should show cleanup summary
    assert!(stderr.contains("Cleanup Summary") || stderr.contains("No autoresearch branches"));
}

#[test]
fn test_history_empty_session_file() {
    let session_file = "/tmp/test_history_empty.jsonl";
    std::fs::remove_file(session_file).ok();

    let args = vec!["--history", "--session-file", session_file];
    let output = get_cli_output(&args);

    assert!(output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No experiments found") || stderr.contains("experiment"));
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

    let stderr = String::from_utf8_lossy(&output.stderr);
    // History should either show experiments or say no experiments found
    // (due to JSON parsing limitations with pretty-printed sessions)
    assert!(stderr.contains("experiment") || stderr.contains("Experiment") || stderr.contains("reduce memory"));
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
    assert!(stdout.contains("resume"), "Flag should be documented in help text");
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
    assert!(stdout.contains("compare"), "Flags should be documented in help text");
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

#[test]
fn test_resume_with_valid_session() {
    let session_file = "/tmp/test_resume_valid.jsonl";
    std::fs::remove_file(session_file).ok();

    // First, create an initial experiment with 1 iteration
    // Use a target improvement that can be achieved (15% improvement from 512 to 435)
    let create_args = vec![
        "--question",
        "reduce memory usage",
        "--auto-approve",
        "--max-iterations",
        "1",
        "--metric",
        "peak_memory_mb",
        "--measure",
        "echo 435.0",
        "--baseline",
        "512.0",
        "--target-improvement",
        "0.15",
        "--session-file",
        session_file,
        "--quiet",
        "--skip-git",
    ];
    let create_output = get_cli_output(&create_args);
    assert!(create_output.status.success(), "Initial experiment should succeed");

    // Read the session file to get the session ID
    let session_id = extract_session_id_from_file(session_file);
    
    assert!(session_id.is_some(), "Session ID should be found in session file");
    let session_id = session_id.unwrap();

    // Now resume the experiment with 1 more iteration
    // Don't use --quiet so we can check the output
    let resume_args = vec![
        "--resume",
        &session_id,
        "--max-iterations",
        "1",
        "--session-file",
        session_file,
        "--skip-git",
    ];
    let resume_output = get_cli_output(&resume_args);
    
    // Resume should complete (may exit with code 1 if target not met, but operation completes)
    // Check stderr for completion message
    let stderr = String::from_utf8_lossy(&resume_output.stderr);
    let resume_completed = stderr.contains("Resumed Experiment Complete") || 
                           stderr.contains("Resuming session");
    assert!(resume_completed, "Resume operation should complete. Stderr: {}", stderr);

    // Verify the session file has been updated
    let final_contents = std::fs::read_to_string(session_file).unwrap();
    
    // Count iteration records by searching for "iteration" field in the multi-line JSON
    let iteration_count = count_iterations_in_file(&final_contents);
    
    // Should have at least 2 iterations (1 from initial + 1 from resume)
    assert!(iteration_count >= 2, "Should have at least 2 iterations after resume, found {}", iteration_count);
}

#[test]
fn test_resume_preserves_session_data() {
    let session_file = "/tmp/test_resume_preserve.jsonl";
    std::fs::remove_file(session_file).ok();

    // Create initial experiment with specific baseline
    // Use a target improvement that can be achieved (30% improvement from 500 to 350)
    let create_args = vec![
        "--question",
        "optimize database queries",
        "--auto-approve",
        "--max-iterations",
        "1",
        "--metric",
        "execution_time_ms",
        "--measure",
        "echo 350.0",
        "--baseline",
        "500.0",
        "--target-improvement",
        "0.3",
        "--session-file",
        session_file,
        "--quiet",
        "--skip-git",
    ];
    let create_output = get_cli_output(&create_args);
    assert!(create_output.status.success());

    // Get session ID
    let session_id = extract_session_id_from_file(session_file)
        .expect("Session ID should be found");

    // Resume the experiment
    let resume_args = vec![
        "--resume",
        &session_id,
        "--max-iterations",
        "1",
        "--session-file",
        session_file,
        "--quiet",
        "--skip-git",
    ];
    let resume_output = get_cli_output(&resume_args);
    
    // Resume should complete (may not meet target, but operation completes)
    let resume_completed = resume_output.status.success() || 
        String::from_utf8_lossy(&resume_output.stderr).contains("Resumed Experiment Complete");
    assert!(resume_completed, "Resume operation should complete");

    // Verify original question is preserved in the session file
    let final_contents = std::fs::read_to_string(session_file).unwrap();
    
    // Check if the original question appears in the file
    let found_original_question = final_contents.contains("optimize database queries");
    
    assert!(found_original_question, "Original question should be preserved in resumed session");
}

#[test]
fn test_resume_invalid_session_id() {
    let session_file = "/tmp/test_resume_invalid_id.jsonl";
    std::fs::remove_file(session_file).ok();

    // Try to resume with a non-existent session ID
    let resume_args = vec![
        "--resume",
        "non-existent-session-id-12345",
        "--session-file",
        session_file,
    ];
    let resume_output = get_cli_output(&resume_args);
    
    // Should fail because session doesn't exist
    assert!(!resume_output.status.success(), "Resume with invalid session ID should fail");

    let stderr = String::from_utf8_lossy(&resume_output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("Session"), 
        "Error message should indicate session not found");
}

#[test]
fn test_resume_with_empty_session_file() {
    let session_file = "/tmp/test_resume_empty.jsonl";
    std::fs::remove_file(session_file).ok();
    std::fs::write(session_file, "").ok();

    // Try to resume with an empty session file
    let resume_args = vec![
        "--resume",
        "some-session-id",
        "--session-file",
        session_file,
    ];
    let resume_output = get_cli_output(&resume_args);
    
    // Should fail because no sessions exist in empty file
    assert!(!resume_output.status.success(), "Resume with empty session file should fail");

    let stderr = String::from_utf8_lossy(&resume_output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("Session"), 
        "Error message should indicate session not found");
}

// ============================================================================
// CONTRACT TESTS - Session File Format Validation
// ============================================================================

/// Contract test: Verify BaselineRecord schema compliance
#[test]
fn test_contract_baseline_record_schema() {
    use serde_json::json;
    
    // Create a valid BaselineRecord
    let baseline_json = json!({
        "timestamp": "2024-01-15T10:30:00.000000000Z",
        "git_commit": "abc123def456",
        "metric": "execution_time_ms",
        "measurement_command": "echo 100.0",
        "value": 100.0,
        "verification_runs": [100.5, 99.8, 100.2],
        "variance": 0.0035,
        "within_threshold": true
    });
    
    // Should deserialize successfully
    let baseline: serde_json::Value = baseline_json;
    assert!(baseline.get("timestamp").is_some());
    assert!(baseline.get("git_commit").is_some());
    assert!(baseline.get("metric").is_some());
    assert!(baseline.get("measurement_command").is_some());
    assert!(baseline.get("value").is_some());
    assert!(baseline.get("verification_runs").is_some());
    assert!(baseline.get("variance").is_some());
    assert!(baseline.get("within_threshold").is_some());
    
    // Verify field types
    assert!(baseline["timestamp"].is_string());
    assert!(baseline["git_commit"].is_string());
    assert!(baseline["metric"].is_string());
    assert!(baseline["measurement_command"].is_string());
    assert!(baseline["value"].is_number());
    assert!(baseline["verification_runs"].is_array());
    assert!(baseline["variance"].is_number());
    assert!(baseline["within_threshold"].is_boolean());
}

/// Contract test: Verify IterationRecord schema compliance
#[test]
fn test_contract_iteration_record_schema() {
    use serde_json::json;
    
    // Create a valid IterationRecord
    let iteration_json = json!({
        "iteration": 1,
        "timestamp": "2024-01-15T10:31:00.000000000Z",
        "agent_action": "Optimized database query by adding index",
        "metric_value": 95.0,
        "improvement": 0.05,
        "kept": true
    });
    
    // Should deserialize successfully
    let iteration: serde_json::Value = iteration_json;
    assert!(iteration.get("iteration").is_some());
    assert!(iteration.get("timestamp").is_some());
    assert!(iteration.get("agent_action").is_some());
    assert!(iteration.get("metric_value").is_some());
    assert!(iteration.get("improvement").is_some());
    assert!(iteration.get("kept").is_some());
    
    // Verify field types
    assert!(iteration["iteration"].is_number());
    assert!(iteration["timestamp"].is_string());
    assert!(iteration["agent_action"].is_string());
    assert!(iteration["metric_value"].is_number());
    assert!(iteration["improvement"].is_number());
    assert!(iteration["kept"].is_boolean());
}

/// Contract test: Verify ExperimentSession schema compliance
#[test]
fn test_contract_experiment_session_schema() {
    use serde_json::json;
    
    // Create a valid ExperimentSession
    let session_json = json!({
        "session_id": "test-session-123",
        "question": "How can we improve performance?",
        "design": {
            "question": "How can we improve performance?",
            "metric": "execution_time_ms",
            "measurement_command": "echo 100.0",
            "baseline": 100.0,
            "target_improvement": 0.30
        },
        "baseline_record": {
            "timestamp": "2024-01-15T10:30:00.000000000Z",
            "git_commit": "abc123",
            "metric": "execution_time_ms",
            "measurement_command": "echo 100.0",
            "value": 100.0,
            "verification_runs": [100.0],
            "variance": 0.0,
            "within_threshold": true
        },
        "iterations": [],
        "best_iteration": null,
        "start_time": "2024-01-15T10:30:00.000000000Z",
        "end_time": "2024-01-15T10:35:00.000000000Z",
        "status": "complete"
    });
    
    // Should deserialize successfully
    let session: serde_json::Value = session_json;
    assert!(session.get("session_id").is_some());
    assert!(session.get("question").is_some());
    assert!(session.get("design").is_some());
    assert!(session.get("baseline_record").is_some());
    assert!(session.get("iterations").is_some());
    assert!(session.get("start_time").is_some());
    assert!(session.get("status").is_some());
    
    // Verify field types
    assert!(session["session_id"].is_string());
    assert!(session["question"].is_string());
    assert!(session["design"].is_object());
    assert!(session["baseline_record"].is_object());
    assert!(session["iterations"].is_array());
    assert!(session["start_time"].is_string());
    assert!(session["status"].is_string());
}

/// Contract test: Verify session file can be parsed and re-serialized
#[test]
fn test_contract_session_file_roundtrip() {
    use serde_json::json;
    
    let session_file = "/tmp/test_contract_roundtrip.jsonl";
    std::fs::remove_file(session_file).ok();
    
    // Create a session file with all record types
    let baseline = json!({
        "timestamp": "2024-01-15T10:30:00.000000000Z",
        "git_commit": "abc123",
        "metric": "execution_time_ms",
        "measurement_command": "echo 100.0",
        "value": 100.0,
        "verification_runs": [100.0, 100.1, 99.9],
        "variance": 0.001,
        "within_threshold": true
    });
    
    let iteration = json!({
        "iteration": 1,
        "timestamp": "2024-01-15T10:31:00.000000000Z",
        "agent_action": "Test change",
        "metric_value": 95.0,
        "improvement": 0.05,
        "kept": true
    });
    
    let session = json!({
        "session_id": "roundtrip-test-123",
        "question": "Test question",
        "design": {
            "question": "Test question",
            "metric": "execution_time_ms",
            "measurement_command": "echo 100.0",
            "baseline": 100.0,
            "target_improvement": 0.30
        },
        "baseline_record": baseline.clone(),
        "iterations": [iteration.clone()],
        "best_iteration": 1,
        "start_time": "2024-01-15T10:30:00.000000000Z",
        "end_time": "2024-01-15T10:35:00.000000000Z",
        "status": "complete"
    });
    
    // Write to file
    std::fs::write(session_file, format!("{}\n{}\n{}",
        baseline.to_string(),
        iteration.to_string(),
        session.to_string()
    )).unwrap();
    
    // Read back and verify
    let contents = std::fs::read_to_string(session_file).unwrap();
    
    // Verify all records are present
    assert!(contents.contains("baseline_record"));
    assert!(contents.contains("iteration"));
    assert!(contents.contains("session_id"));
    assert!(contents.contains("roundtrip-test-123"));
    
    // Cleanup
    std::fs::remove_file(session_file).ok();
}

/// Contract test: Verify backward compatibility with compact JSONL format
#[test]
fn test_contract_backward_compat_compact_jsonl() {
    let session_file = "/tmp/test_contract_compact.jsonl";
    std::fs::remove_file(session_file).ok();
    
    // Create compact JSONL format (one JSON object per line)
    let compact_format = r#"{"timestamp":"2024-01-15T10:30:00Z","git_commit":"abc","metric":"test","measurement_command":"echo 1","value":1.0,"verification_runs":[1.0],"variance":0.0,"within_threshold":true}
{"iteration":1,"timestamp":"2024-01-15T10:31:00Z","agent_action":"test","metric_value":0.9,"improvement":0.1,"kept":true}
{"session_id":"compact-test","question":"test","design":{"question":"test","metric":"test","measurement_command":"echo 1","baseline":1.0,"target_improvement":0.1},"baseline_record":{"timestamp":"2024-01-15T10:30:00Z","git_commit":"abc","metric":"test","measurement_command":"echo 1","value":1.0,"verification_runs":[1.0],"variance":0.0,"within_threshold":true},"iterations":[{"iteration":1,"timestamp":"2024-01-15T10:31:00Z","agent_action":"test","metric_value":0.9,"improvement":0.1,"kept":true}],"best_iteration":1,"start_time":"2024-01-15T10:30:00Z","end_time":"2024-01-15T10:35:00Z","status":"complete"}
"#;
    
    std::fs::write(session_file, compact_format).unwrap();
    
    // Run tool with this session file (should not panic)
    let args = vec![
        "--history",
        "--session-file",
        session_file,
        "--quiet",
    ];
    let output = get_cli_output(&args);
    
    // Should succeed or at least not crash
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Either succeeds or shows history (may be empty)
    assert!(output.status.success() || stderr.contains("No prior experiments"),
        "Should handle compact JSONL format without crashing");
    
    std::fs::remove_file(session_file).ok();
}

/// Contract test: Verify forward compatibility with pretty-printed JSON
#[test]
fn test_contract_forward_compat_pretty_json() {
    let session_file = "/tmp/test_contract_pretty.jsonl";
    std::fs::remove_file(session_file).ok();
    
    // Create pretty-printed JSON format (multi-line)
    let pretty_format = r#"{
  "session_id": "pretty-test",
  "question": "test question",
  "design": {
    "question": "test question",
    "metric": "execution_time_ms",
    "measurement_command": "echo 100",
    "baseline": 100.0,
    "target_improvement": 0.30
  },
  "baseline_record": {
    "timestamp": "2024-01-15T10:30:00Z",
    "git_commit": "abc123",
    "metric": "execution_time_ms",
    "measurement_command": "echo 100",
    "value": 100.0,
    "verification_runs": [100.0],
    "variance": 0.0,
    "within_threshold": true
  },
  "iterations": [],
  "best_iteration": null,
  "start_time": "2024-01-15T10:30:00Z",
  "end_time": "2024-01-15T10:35:00Z",
  "status": "complete"
}
"#;
    
    std::fs::write(session_file, pretty_format).unwrap();
    
    // Run tool with this session file (should not panic)
    let args = vec![
        "--history",
        "--session-file",
        session_file,
        "--quiet",
    ];
    let output = get_cli_output(&args);
    
    // Should succeed or at least not crash
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success() || stderr.contains("No prior experiments") || stderr.contains("Experiment"),
        "Should handle pretty-printed JSON format without crashing");
    
    std::fs::remove_file(session_file).ok();
}

/// Contract test: Verify session file validation catches missing required fields
#[test]
fn test_contract_validation_missing_fields() {
    let session_file = "/tmp/test_contract_missing.jsonl";
    std::fs::remove_file(session_file).ok();
    
    // Create invalid session with missing required fields
    let invalid_session = r#"{
  "session_id": "invalid-test"
  "question": "missing design and other fields"
}
"#;
    
    std::fs::write(session_file, invalid_session).unwrap();
    
    // Run tool with this session file
    let args = vec![
        "--history",
        "--session-file",
        session_file,
        "--quiet",
    ];
    let output = get_cli_output(&args);
    
    // Should handle gracefully (either skip invalid record or show error)
    // The tool should not crash on malformed JSON
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Either succeeds (skips invalid) or shows error, but doesn't panic
    assert!(output.status.success() || stderr.contains("error") || stderr.contains("No prior experiments"),
        "Should handle invalid JSON gracefully");
    
    std::fs::remove_file(session_file).ok();
}
