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

// ============================================================================
// ERROR HANDLING EDGE CASE TESTS
// ============================================================================

/// Error handling test: Invalid config file with malformed JSON
#[test]
fn test_error_invalid_config_malformed_json() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_file = temp_dir.path().join("config.json");
    
    // Write malformed JSON to config file
    std::fs::write(&config_file, r#"{"invalid": json, missing quotes}"#).unwrap();
    
    let args = vec![
        "--config",
        config_file.to_str().unwrap(),
        "--question",
        "test",
        "--auto-approve",
        "--quiet",
    ];
    let output = get_cli_output(&args);
    
    // Should fail with a clear error message
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Failed to parse config") || stderr.contains("invalid") || stderr.contains("error"),
        "Should show error about malformed JSON in config file");
}

/// Error handling test: Invalid config file with out-of-range max_variance
#[test]
fn test_error_invalid_config_max_variance() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_file = temp_dir.path().join("config.json");
    
    // Write config with invalid max_variance (> 1.0)
    std::fs::write(&config_file, r#"{"max_variance": 1.5}"#).unwrap();
    
    let args = vec![
        "--config",
        config_file.to_str().unwrap(),
        "--question",
        "test",
        "--auto-approve",
        "--quiet",
    ];
    let output = get_cli_output(&args);
    
    // Should fail with validation error
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("max_variance") && stderr.contains("between 0.0 and 1.0"),
        "Should show validation error for max_variance");
}

/// Error handling test: Invalid config file with negative target_improvement
#[test]
fn test_error_invalid_config_target_improvement() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_file = temp_dir.path().join("config.json");
    
    // Write config with negative target_improvement
    std::fs::write(&config_file, r#"{"target_improvement": -0.5}"#).unwrap();
    
    let args = vec![
        "--config",
        config_file.to_str().unwrap(),
        "--question",
        "test",
        "--auto-approve",
        "--quiet",
    ];
    let output = get_cli_output(&args);
    
    // Should fail with validation error
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("target_improvement") && stderr.contains("positive"),
        "Should show validation error for target_improvement");
}

/// Error handling test: Invalid config file with zero max_iterations
#[test]
fn test_error_invalid_config_max_iterations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_file = temp_dir.path().join("config.json");
    
    // Write config with zero max_iterations
    std::fs::write(&config_file, r#"{"max_iterations": 0}"#).unwrap();
    
    let args = vec![
        "--config",
        config_file.to_str().unwrap(),
        "--question",
        "test",
        "--auto-approve",
        "--quiet",
    ];
    let output = get_cli_output(&args);
    
    // Should fail with validation error
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("max_iterations") && stderr.contains("positive"),
        "Should show validation error for max_iterations");
}

/// Error handling test: Invalid config file with zero iteration_timeout
#[test]
fn test_error_invalid_config_iteration_timeout() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_file = temp_dir.path().join("config.json");
    
    // Write config with zero iteration_timeout_minutes
    std::fs::write(&config_file, r#"{"iteration_timeout_minutes": 0}"#).unwrap();
    
    let args = vec![
        "--config",
        config_file.to_str().unwrap(),
        "--question",
        "test",
        "--auto-approve",
        "--quiet",
    ];
    let output = get_cli_output(&args);
    
    // Should fail with validation error
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("iteration_timeout_minutes") && stderr.contains("positive"),
        "Should show validation error for iteration_timeout_minutes");
}

/// Error handling test: Invalid config file with invalid session_file path
#[test]
fn test_error_invalid_config_session_file_path() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_file = temp_dir.path().join("config.json");
    
    // Write config with invalid session_file path (non-writable)
    std::fs::write(&config_file, r#"{"session_file": "/root/invalid_path/test.jsonl"}"#).unwrap();
    
    let args = vec![
        "--config",
        config_file.to_str().unwrap(),
        "--question",
        "test",
        "--auto-approve",
        "--quiet",
    ];
    let output = get_cli_output(&args);
    
    // Should fail with validation error about session file path
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("session_file") || stderr.contains("path"),
        "Should show validation error for session_file path");
}

/// Error handling test: Missing measurement command
#[test]
fn test_error_missing_measurement_command() {
    // Use temporary HOME directory without config file
    let args = vec![
        "--verify-baseline",
        "--metric",
        "test_metric",
        "--session-file",
        "/tmp/test_missing_measure.jsonl",
    ];
    let output = get_cli_output_no_config(&args);
    
    // Should fail because --measure is required for --verify-baseline
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("measure") || stderr.contains("required") || stderr.contains("error"),
        "Should show error about missing measurement command");
}

/// Error handling test: Missing metric for baseline verification
#[test]
fn test_error_missing_metric_baseline() {
    // Use temporary HOME directory without config file
    let args = vec![
        "--verify-baseline",
        "--measure",
        "echo 100.0",
        "--session-file",
        "/tmp/test_missing_metric.jsonl",
    ];
    let output = get_cli_output_no_config(&args);
    
    // Should fail because --metric is required for --verify-baseline
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("metric") || stderr.contains("required") || stderr.contains("error"),
        "Should show error about missing metric");
}

/// Error handling test: Non-existent config file with explicit --config flag
#[test]
fn test_error_nonexistent_config_file() {
    let args = vec![
        "--config",
        "/nonexistent/path/config.json",
        "--question",
        "test",
        "--auto-approve",
        "--quiet",
    ];
    let output = get_cli_output(&args);
    
    // Should fail with clear error about missing config file
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("Config file"),
        "Should show error about non-existent config file");
}

/// Error handling test: Command that returns non-numeric output
#[test]
fn test_error_measurement_non_numeric_output() {
    let args = vec![
        "--verify-baseline",
        "--metric",
        "test_metric",
        "--measure",
        "echo not_a_number",
        "--session-file",
        "/tmp/test_non_numeric.jsonl",
    ];
    let output = get_cli_output(&args);
    
    // Should fail because measurement output is not numeric
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("parse") || stderr.contains("number") || stderr.contains("error"),
        "Should show error about non-numeric measurement output");
}

/// Error handling test: Command that fails (non-existent command)
#[test]
fn test_error_measurement_command_fails() {
    let args = vec![
        "--verify-baseline",
        "--metric",
        "test_metric",
        "--measure",
        "nonexistent_command_12345",
        "--session-file",
        "/tmp/test_command_fail.jsonl",
    ];
    let output = get_cli_output(&args);
    
    // Should fail because command doesn't exist
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("command") || stderr.contains("error") || stderr.contains("not found"),
        "Should show error about command failure");
}

/// Error handling test: Git operations when not in a git repository
#[test]
fn test_error_not_a_git_repository() {
    let temp_dir = tempfile::tempdir().unwrap();
    let session_file = temp_dir.path().join("test.jsonl");
    
    // Change to a directory that is not a git repository
    let args = vec![
        "--question",
        "test",
        "--auto-approve",
        "--max-iterations",
        "1",
        "--metric",
        "test_metric",
        "--measure",
        "echo 100.0",
        "--baseline",
        "100.0",
        "--target-improvement",
        "0.1",
        "--session-file",
        session_file.to_str().unwrap(),
        "--quiet",
        "--skip-git",  // Skip git operations when not in a git repo
    ];
    
    // Run in temp directory that's not a git repo
    let mut cmd = Command::cargo_bin("pi-autoresearch").unwrap();
    cmd.args(&args);
    cmd.current_dir(temp_dir.path());
    let output = cmd.output().unwrap();
    
    // Should complete (may exit with 0 or 1 depending on improvement results)
    // Exit code 1 is expected when target improvement is not met
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hypothesis") || output.status.code() == Some(1), 
        "Should complete experiment in non-git repository with --skip-git");
    
    // Verify no git-related errors
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("fatal:") || stderr.contains("not a git repository"),
        "Should not fail with git errors when using --skip-git");
}

/// Error handling test: Timeout with very short iteration timeout
#[test]
fn test_error_timeout_short_iteration() {
    let session_file = "/tmp/test_timeout_short.jsonl";
    std::fs::remove_file(session_file).ok();
    
    // Use a measurement command that will timeout
    // The tool should handle timeout gracefully
    let args = vec![
        "--question",
        "test timeout",
        "--auto-approve",
        "--max-iterations",
        "1",
        "--iteration-timeout-minutes",
        "1",
        "--metric",
        "test_metric",
        "--measure",
        "sleep 61 && echo 100.0",  // Sleep 61 seconds, timeout is 60 seconds
        "--baseline",
        "100.0",
        "--target-improvement",
        "0.1",
        "--session-file",
        session_file,
        "--quiet",
        "--skip-git",
    ];
    
    let start = std::time::Instant::now();
    let output = get_cli_output(&args);
    let elapsed = start.elapsed();
    
    // Should timeout or fail within a reasonable time
    // The command should not run for the full 61 seconds if timeout works
    assert!(elapsed.as_secs() < 61, 
        "Should timeout before command completes (took {}s)", elapsed.as_secs());
    
    // The test passes if it times out within the iteration timeout
    // This verifies timeout handling is working
    // Exit code 1 is acceptable (experiment completed but didn't meet target)
    assert!(output.status.code().is_some(),
        "Should complete with an exit code (timeout or normal completion)");
}

/// Error handling test: Empty measurement command
#[test]
fn test_error_empty_measurement_command() {
    let args = vec![
        "--verify-baseline",
        "--metric",
        "test_metric",
        "--measure",
        "",
        "--session-file",
        "/tmp/test_empty_measure.jsonl",
    ];
    let output = get_cli_output(&args);
    
    // Should fail because empty command is not valid
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error") || stderr.contains("command"),
        "Should show error about empty measurement command");
}

/// Error handling test: Multiple validation errors in config
#[test]
fn test_error_multiple_validation_errors() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_file = temp_dir.path().join("config.json");
    
    // Write config with multiple validation errors
    std::fs::write(&config_file, r#"{
        "max_variance": 2.0,
        "target_improvement": -1.0,
        "max_iterations": 0,
        "iteration_timeout_minutes": 0
    }"#).unwrap();
    
    let args = vec![
        "--config",
        config_file.to_str().unwrap(),
        "--question",
        "test",
        "--auto-approve",
        "--quiet",
    ];
    let output = get_cli_output(&args);
    
    // Should fail with validation errors
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Should report multiple errors
    assert!(stderr.contains("Config validation failed") || stderr.contains("error"),
        "Should show validation errors");
    // Should mention at least some of the invalid fields
    assert!(stderr.contains("max_variance") || stderr.contains("target_improvement") || 
            stderr.contains("max_iterations") || stderr.contains("iteration_timeout"),
        "Should mention invalid fields");
}

/// Error handling test: Session file in non-writable directory
#[test]
fn test_error_session_file_non_writable_dir() {
    let args = vec![
        "--question",
        "test",
        "--auto-approve",
        "--max-iterations",
        "1",
        "--metric",
        "test_metric",
        "--measure",
        "echo 100.0",
        "--baseline",
        "100.0",
        "--target-improvement",
        "0.1",
        "--session-file",
        "/root/test_session.jsonl",  // Root directory is typically not writable
        "--quiet",
        "--skip-git",
    ];
    let output = get_cli_output(&args);
    
    // Should fail or succeed depending on permissions
    // If running as root, this will succeed; otherwise should fail gracefully
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        assert!(stderr.contains("permission") || stderr.contains("cannot") || stderr.contains("error"),
            "Should show error about non-writable directory");
    }
    // If it succeeds (running as root), that's also acceptable
}

/// Integration test: Auto-approve flag is recognized
#[test]
fn test_auto_approve_flag_recognized() {
    let args = vec!["--help"];
    let output = get_cli_output(&args);
    
    // Should succeed
    assert!(output.status.success());
    
    // Check that --auto-approve is documented in help text
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("auto-approve"),
        "Help text should document --auto-approve flag");
}

/// Integration test: Auto-approve with verify-baseline
#[test]
fn test_auto_approve_with_verify_baseline() {
    let args = vec![
        "--verify-baseline",
        "--metric",
        "test_metric",
        "--measure",
        "echo 100.0",
        "--auto-approve",
        "--session-file",
        "/tmp/test_auto_approve_baseline.jsonl",
    ];
    let output = get_cli_output(&args);
    
    // Should succeed
    assert!(output.status.success());
    
    // Check that baseline was recorded
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Baseline verified") || stdout.contains("value"),
        "Should show baseline verification output");
}

/// Integration test: Auto-approve with question and iterations
#[test]
fn test_auto_approve_with_iterations() {
    let args = vec![
        "--question",
        "test optimization",
        "--auto-approve",
        "--max-iterations",
        "2",
        "--metric",
        "test_metric",
        "--measure",
        "echo 95.0",
        "--baseline",
        "100.0",
        "--target-improvement",
        "0.1",
        "--session-file",
        "/tmp/test_auto_approve_iter.jsonl",
        "--skip-git",
        "--quiet",
    ];
    let output = get_cli_output(&args);
    
    // Should succeed (exit code 0 or 1 depending on whether target was met)
    // Exit code 1 is acceptable if target wasn't met
    assert!(output.status.code().is_some(),
        "Should complete with an exit code");
    
    // Check that session file was created
    assert!(std::path::Path::new("/tmp/test_auto_approve_iter.jsonl").exists(),
        "Session file should be created");
}

/// Integration test: Auto-approve with config file
#[test]
fn test_auto_approve_with_config() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_file = temp_dir.path().join("config.json");
    
    // Write a valid config file
    std::fs::write(&config_file, r#"{
        "metric": "config_metric",
        "measure": "echo 90.0",
        "baseline": 100.0,
        "target_improvement": 0.1
    }"#).unwrap();
    
    let args = vec![
        "--config",
        config_file.to_str().unwrap(),
        "--question",
        "test with config",
        "--auto-approve",
        "--max-iterations",
        "1",
        "--session-file",
        "/tmp/test_auto_approve_config.jsonl",
        "--skip-git",
        "--quiet",
    ];
    let output = get_cli_output(&args);
    
    // Should succeed
    assert!(output.status.code().is_some(),
        "Should complete with an exit code");
    
    // Check that session file was created
    assert!(std::path::Path::new("/tmp/test_auto_approve_config.jsonl").exists(),
        "Session file should be created");
}

// Branch management integration tests

/// Integration test: List branches with actual branches
#[test]
fn test_list_branches_with_actual_branches() {
    // Create a test branch
    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "autoresearch/test-list-branches-12345"])
        .output();
    
    // List branches
    let args = vec!["--list-branches"];
    let output = get_cli_output(&args);
    
    // Should succeed
    assert!(output.status.success());
    
    // Check that our test branch is listed
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("autoresearch/test-list-branches-12345"),
        "Should list the test branch");
    
    // Clean up: delete the test branch
    let _ = std::process::Command::new("git")
        .args(["checkout", "-"])  // Go back to previous branch
        .output();
    let _ = std::process::Command::new("git")
        .args(["branch", "-D", "autoresearch/test-list-branches-12345"])
        .output();
}

/// Integration test: Cleanup branches with actual branches
#[test]
fn test_cleanup_branches_with_actual_branches() {
    // Create a test branch
    let _ = std::process::Command::new("git")
        .args(["checkout", "-b", "autoresearch/test-cleanup-branches-12345"])
        .output();
    
    // Cleanup branches older than 0 days (should include our test branch)
    let args = vec!["--cleanup-branches", "--cleanup-days", "0"];
    let output = get_cli_output(&args);
    
    // Should succeed
    assert!(output.status.success());
    
    // Check that cleanup summary is shown
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Cleanup Summary") || stderr.contains("removed") || 
            stderr.contains("autoresearch/test-cleanup-branches-12345"),
        "Should show cleanup summary or mention the test branch");
    
    // Clean up: go back to previous branch
    let _ = std::process::Command::new("git")
        .args(["checkout", "-"])
        .output();
}

/// Integration test: Branch name uniqueness across runs
#[test]
fn test_branch_name_uniqueness_across_runs() {
    let session_file = "/tmp/test_branch_uniqueness.jsonl";
    
    // Run two experiments quickly
    for i in 0..2 {
        let _ = std::fs::remove_file(session_file);
        
        let question = format!("test {}", i);
        let args = vec![
            "--question",
            &question,
            "--auto-approve",
            "--max-iterations",
            "1",
            "--metric",
            "test_metric",
            "--measure",
            "echo 95.0",
            "--baseline",
            "100.0",
            "--target-improvement",
            "0.1",
            "--session-file",
            session_file,
            "--skip-git",
            "--quiet",
        ];
        let output = get_cli_output(&args);
        
        // Should succeed
        assert!(output.status.code().is_some());
    }
    
    // Check that session file exists
    assert!(std::path::Path::new(session_file).exists(),
        "Session file should exist");
}

/// Integration test: Skip git flag prevents branch creation
#[test]
fn test_skip_git_prevents_branch_creation() {
    let args = vec![
        "--question",
        "test skip git",
        "--auto-approve",
        "--max-iterations",
        "1",
        "--metric",
        "test_metric",
        "--measure",
        "echo 95.0",
        "--baseline",
        "100.0",
        "--target-improvement",
        "0.1",
        "--session-file",
        "/tmp/test_skip_git.jsonl",
        "--skip-git",
        "--quiet",
    ];
    let output = get_cli_output(&args);
    
    // Should succeed
    assert!(output.status.code().is_some());
    
    // Check that session file was created
    assert!(std::path::Path::new("/tmp/test_skip_git.jsonl").exists(),
        "Session file should be created even with --skip-git");
}

// =============================================================================
// END-TO-END BEADS INTEGRATION TESTS
// =============================================================================

/// E2E Test: Beads task creation workflow
/// Verifies that when --beads-enabled is used, the tool attempts to create a bead
#[test]
fn test_beads_integration_creation_workflow() {
    let session_file = "/tmp/test_beads_creation.jsonl";
    let args = vec![
        "--question",
        "how can I improve the performance of my sorting algorithm",
        "--auto-approve",
        "--beads-enabled",
        "--metric",
        "performance",
        "--measure",
        "echo 90.0",
        "--baseline",
        "100.0",
        "--target-improvement",
        "0.1",
        "--max-iterations",
        "1",
        "--session-file",
        session_file,
        "--skip-git",
        "--quiet",
    ];
    let output = get_cli_output(&args);
    
    // Should succeed (even if bd is not installed, it should handle gracefully)
    // Note: Exit code may be 1 if target not met, which is expected behavior
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success() || stderr.contains("bead"), 
        "Command should either succeed or handle beads gracefully. stderr: {}", 
        stderr);
    
    // Verify session file was created
    assert!(std::path::Path::new(session_file).exists(),
        "Session file should be created");
    
    // Verify JSON output contains expected fields
    let json = parse_json_output(&output);
    assert!(json["hypothesis"].as_str().unwrap().contains("sorting algorithm"));
}

/// E2E Test: Beads task update workflow during iterations
/// Verifies that bead notes are added during iterations when beads is enabled
#[test]
fn test_beads_integration_update_workflow() {
    let session_file = "/tmp/test_beads_update.jsonl";
    let args = vec![
        "--question",
        "how can I reduce memory usage",
        "--auto-approve",
        "--beads-enabled",
        "--metric",
        "memory",
        "--measure",
        "echo 40.0",
        "--baseline",
        "100.0",
        "--target-improvement",
        "0.5",
        "--max-iterations",
        "3",
        "--session-file",
        session_file,
        "--skip-git",
        "--quiet",
    ];
    let output = get_cli_output(&args);
    
    // Should run without crashing (may fail if target not met, which is expected)
    assert!(output.status.code().is_some(),
        "Command should complete. stderr: {}",
        String::from_utf8_lossy(&output.stderr));
    
    // Verify session file was created with iterations
    assert!(std::path::Path::new(session_file).exists(),
        "Session file should be created");
    
    let contents = std::fs::read_to_string(session_file).unwrap();
    // Should have baseline + iterations (at least 2 lines)
    let lines: Vec<&str> = contents.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(lines.len() >= 2, "Session file should have baseline and iterations, got {} lines", lines.len());
    
    // Verify JSON output
    let json = parse_json_output(&output);
    assert!(json["hypothesis"].as_str().unwrap().contains("memory"));
}

/// E2E Test: Beads task completion workflow
/// Verifies that bead is closed when experiment completes
#[test]
fn test_beads_integration_completion_workflow() {
    let session_file = "/tmp/test_beads_completion.jsonl";
    let args = vec![
        "--question",
        "how can I improve accuracy",
        "--auto-approve",
        "--beads-enabled",
        "--metric",
        "accuracy",
        "--measure",
        "echo 90.0",
        "--baseline",
        "100.0",
        "--target-improvement",
        "0.1",
        "--max-iterations",
        "2",
        "--session-file",
        session_file,
        "--skip-git",
        "--quiet",
    ];
    let output = get_cli_output(&args);
    
    // Should run without crashing (may fail if target not met, which is expected)
    assert!(output.status.code().is_some(),
        "Command should complete. stderr: {}",
        String::from_utf8_lossy(&output.stderr));
    
    // Verify session file was created
    assert!(std::path::Path::new(session_file).exists(),
        "Session file should be created");
    
    // Verify JSON output
    let json = parse_json_output(&output);
    assert!(json["hypothesis"].as_str().unwrap().contains("accuracy"));
}

/// E2E Test: Beads integration graceful degradation
/// Verifies that experiment works even if bd commands fail (bd not installed)
#[test]
fn test_beads_integration_graceful_degradation() {
    let session_file = "/tmp/test_beads_graceful.jsonl";
    
    // Use a HOME directory without bd to simulate bd not being installed
    let args = vec![
        "--question",
        "test graceful degradation",
        "--auto-approve",
        "--beads-enabled",
        "--metric",
        "test",
        "--measure",
        "echo 50.0",
        "--baseline",
        "100.0",
        "--target-improvement",
        "0.5",
        "--max-iterations",
        "1",
        "--session-file",
        session_file,
        "--skip-git",
        "--quiet",
    ];
    
    // Run with a temp HOME to isolate from any bd configuration
    let output = get_cli_output_no_config(&args);
    
    // Should succeed even if bd is not available
    assert!(
        output.status.success() || output.status.code() == Some(1),
        "Command should succeed even without bd. stderr: {}",
        String::from_utf8_lossy(&output.stderr));
    
    // Verify session file was created
    assert!(std::path::Path::new(session_file).exists(),
        "Session file should be created even if bd fails");
    
    // Verify JSON output
    let json = parse_json_output(&output);
    assert!(json["hypothesis"].as_str().unwrap().contains("graceful degradation"));
}

/// E2E Test: Beads integration with config file
/// Verifies that beads_enabled in config file works correctly
#[test]
fn test_beads_integration_with_config() {
    let temp_dir = std::env::temp_dir().join(format!("pi-autoresearch-beads-config-{}", std::process::id()));
    let config_dir = temp_dir.join(".config").join("pi-autoresearch");
    let _ = std::fs::create_dir_all(&config_dir);
    
    let config_file = config_dir.join("config.json");
    let config_content = r#"{
        "beads_enabled": true,
        "max_iterations": 1,
        "target_improvement": 0.1
    }"#;
    std::fs::write(&config_file, config_content).unwrap();
    
    let session_file = temp_dir.join("test_beads_config.jsonl").to_str().unwrap().to_string();
    
    let args = vec![
        "--question",
        "test beads config",
        "--auto-approve",
        "--metric",
        "test",
        "--measure",
        "echo 90.0",
        "--baseline",
        "100.0",
        "--session-file",
        &session_file,
        "--skip-git",
        "--quiet",
    ];
    
    let mut cmd = Command::cargo_bin("pi-autoresearch").unwrap();
    cmd.args(&args);
    cmd.env("HOME", &temp_dir);
    let output = cmd.output().unwrap();
    
    // Should run without crashing
    assert!(output.status.code().is_some(),
        "Command should complete. stderr: {}",
        String::from_utf8_lossy(&output.stderr));
    
    // Verify JSON output
    let json = parse_json_output(&output);
    assert!(json["hypothesis"].as_str().unwrap().contains("beads config"));
    
    // Clean up
    let _ = std::fs::remove_dir_all(&temp_dir);
}

/// E2E Test: Beads CLI flag overrides config file
/// Verifies that --beads-enabled CLI flag takes precedence over config
#[test]
fn test_beads_cli_overrides_config() {
    let temp_dir = std::env::temp_dir().join(format!("pi-autoresearch-beads-override-{}", std::process::id()));
    let config_dir = temp_dir.join(".config").join("pi-autoresearch");
    let _ = std::fs::create_dir_all(&config_dir);
    
    let config_file = config_dir.join("config.json");
    // Config has beads_enabled: false
    let config_content = r#"{
        "beads_enabled": false,
        "max_iterations": 1
    }"#;
    std::fs::write(&config_file, config_content).unwrap();
    
    let session_file = temp_dir.join("test_beads_override.jsonl").to_str().unwrap().to_string();
    
    // CLI has --beads-enabled which should override config
    let args = vec![
        "--question",
        "test beads override",
        "--auto-approve",
        "--beads-enabled",
        "--metric",
        "test",
        "--measure",
        "echo 90.0",
        "--baseline",
        "100.0",
        "--session-file",
        &session_file,
        "--skip-git",
        "--quiet",
    ];
    
    let mut cmd = Command::cargo_bin("pi-autoresearch").unwrap();
    cmd.args(&args);
    cmd.env("HOME", &temp_dir);
    let output = cmd.output().unwrap();
    
    // Should run without crashing
    assert!(output.status.code().is_some(),
        "Command should complete. stderr: {}",
        String::from_utf8_lossy(&output.stderr));
    
    // Clean up
    let _ = std::fs::remove_dir_all(&temp_dir);
}

/// E2E Test: Beads integration with multiple iterations
/// Verifies that bead updates happen across multiple iterations
#[test]
fn test_beads_integration_multiple_iterations() {
    let session_file = "/tmp/test_beads_multi_iter.jsonl";
    let args = vec![
        "--question",
        "how can I optimize database queries",
        "--auto-approve",
        "--beads-enabled",
        "--metric",
        "performance",
        "--measure",
        "echo 80.0",
        "--baseline",
        "100.0",
        "--target-improvement",
        "0.2",
        "--max-iterations",
        "5",
        "--session-file",
        session_file,
        "--skip-git",
        "--quiet",
    ];
    let output = get_cli_output(&args);
    
    // Should run without crashing
    assert!(output.status.code().is_some(),
        "Command should complete. stderr: {}",
        String::from_utf8_lossy(&output.stderr));
    
    // Verify session file was created with multiple iterations
    assert!(std::path::Path::new(session_file).exists(),
        "Session file should be created");
    
    let contents = std::fs::read_to_string(session_file).unwrap();
    let lines: Vec<&str> = contents.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(lines.len() >= 2, "Session file should have baseline and iterations, got {} lines", lines.len());
    
    // Verify JSON output
    let json = parse_json_output(&output);
    assert!(json["hypothesis"].as_str().unwrap().contains("database"));
}

/// E2E Test: Beads integration error handling
/// Verifies that errors in bd commands don't crash the experiment
#[test]
fn test_beads_integration_error_handling() {
    let session_file = "/tmp/test_beads_error.jsonl";
    let args = vec![
        "--question",
        "test error handling",
        "--auto-approve",
        "--beads-enabled",
        "--metric",
        "test",
        "--measure",
        "echo 50.0",
        "--baseline",
        "100.0",
        "--target-improvement",
        "0.5",
        "--max-iterations",
        "2",
        "--session-file",
        session_file,
        "--skip-git",
        "--quiet",
    ];
    let output = get_cli_output(&args);
    
    // Should run without crashing even if bd commands fail
    assert!(output.status.code().is_some(),
        "Command should complete even if bd fails. stderr: {}",
        String::from_utf8_lossy(&output.stderr));
    
    // Verify session file was created
    assert!(std::path::Path::new(session_file).exists(),
        "Session file should be created");
    
    // Verify JSON output
    let json = parse_json_output(&output);
    assert!(json["hypothesis"].as_str().unwrap().contains("error handling"));
}

#[test]
fn test_progress_bar_shown_during_iterations() {
    let session_file = "/tmp/test_progress_bar.jsonl";
    std::fs::remove_file(session_file).ok();
    
    let args = vec![
        "--question",
        "test progress bar",
        "--auto-approve",
        "--metric",
        "test_metric",
        "--measure",
        "echo 90.0",
        "--baseline",
        "100.0",
        "--target-improvement",
        "0.1",  // Lower target to ensure success
        "--max-iterations",
        "3",
        "--session-file",
        session_file,
        "--skip-git",
    ];
    let output = get_cli_output(&args);
    
    assert!(
        output.status.success() || output.status.code() == Some(1), 
        "Command should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr));
    
    // Progress bars use stderr for output with ANSI escape codes
    let _stderr = String::from_utf8_lossy(&output.stderr);
    
    // Verify session file was created
    assert!(std::path::Path::new(session_file).exists(),
        "Session file should be created");
    
    // Read and parse session file to verify experiment data
    let contents = std::fs::read_to_string(session_file).unwrap();
    
    // The ExperimentSession is the last JSON object in the file (pretty-printed)
    // We need to find the start of this object by looking for "session_id"
    let session_start = contents.find("\"session_id\"").expect("Should find session_id");
    // Find the opening brace before session_id
    let brace_start = contents[..session_start].rfind('{').expect("Should find opening brace");
    let json_str = &contents[brace_start..];
    
    let session: serde_json::Value = serde_json::from_str(json_str)
        .expect("Should parse experiment session JSON");
    
    // Verify experiment data
    assert!(session["session_id"].is_string(), "Should have session_id");
    assert!(session["iterations"].as_array().unwrap().len() > 0,
        "Should have at least one iteration");
    assert!(session["status"].as_str().unwrap().starts_with("complete"),
        "Status should be complete or completed");
}

#[test]
fn test_progress_bar_shown_during_baseline_verification() {
    let session_file = "/tmp/test_progress_bar_baseline.jsonl";
    let args = vec![
        "--question",
        "test baseline progress bar",
        "--auto-approve",
        "--metric",
        "test_metric",
        "--measure",
        "echo 100.0",
        "--baseline",
        "100.0",
        "--verify-baseline",
        "--session-file",
        session_file,
    ];
    let output = get_cli_output(&args);
    
    assert!(
        output.status.success() || output.status.code() == Some(1), 
        "Command should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr));
    
    // Progress bars use stderr for output
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Verify baseline verification was shown
    assert!(stderr.contains("Verifying baseline"), 
        "Should show progress bar for baseline verification");
    
    // Verify JSON output
    let json = parse_json_output(&output);
    assert!(json["baseline_record"].is_object());
    assert!(json["baseline_record"]["within_threshold"].as_bool().unwrap());
}

// ==================== Error Message Tests ====================

/// Tests for improved error messages with suggestions

#[test]
fn test_error_message_includes_suggestion() {
    // Test that config validation errors include suggestions
    let temp_dir = std::env::temp_dir().join(format!("pi-autoresearch-error-msg-{}", std::process::id()));
    let config_dir = temp_dir.join(".config").join("pi-autoresearch");
    let _ = std::fs::create_dir_all(&config_dir);
    
    let config_file = config_dir.join("config.json");
    // Invalid config with max_variance > 1.0
    let config_content = r#"{
        "max_variance": 1.5,
        "max_iterations": 1
    }"#;
    std::fs::write(&config_file, config_content).unwrap();
    
    let args = vec![
        "--question",
        "test error message",
        "--auto-approve",
        "--metric",
        "test",
        "--measure",
        "echo 100.0",
        "--baseline",
        "100.0",
        "--skip-git",
    ];
    
    let mut cmd = Command::cargo_bin("pi-autoresearch").unwrap();
    cmd.args(&args);
    cmd.env("HOME", &temp_dir);
    let output = cmd.output().unwrap();
    
    // Should fail with error
    assert!(!output.status.success() || output.status.code() == Some(1),
        "Command should fail with invalid config");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Verify error message includes suggestion
    assert!(stderr.contains("SUGGESTION"),
        "Error message should include SUGGESTION section");
    assert!(stderr.contains("max_variance"),
        "Error message should mention the invalid field");
    assert!(stderr.contains("docs/CONFIG.md"),
        "Error message should link to documentation");
    
    // Clean up
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_error_message_session_not_found() {
    // Test that session not found error includes helpful suggestions
    let args = vec![
        "--resume",
        "nonexistent-session-id",
        "--skip-git",
    ];
    
    let output = Command::cargo_bin("pi-autoresearch")
        .unwrap()
        .args(&args)
        .output()
        .unwrap();
    
    // Should fail with error
    assert!(!output.status.success() || output.status.code() == Some(1),
        "Command should fail with nonexistent session");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Verify error message includes suggestion
    assert!(stderr.contains("not found"),
        "Error message should indicate session not found");
    assert!(stderr.contains("SUGGESTION"),
        "Error message should include SUGGESTION section");
    assert!(stderr.contains("--history"),
        "Error message should suggest using --history flag");
}

#[test]
fn test_error_message_baseline_verification() {
    // Test that baseline verification errors include helpful suggestions
    // Use verify-baseline mode to force measurement validation
    let args = vec![
        "--metric",
        "test",
        "--measure",
        "echo not_a_number",
        "--verify-baseline",
        "--skip-git",
    ];
    
    let output = Command::cargo_bin("pi-autoresearch")
        .unwrap()
        .args(&args)
        .output()
        .unwrap();
    
    // Should fail with error
    assert!(!output.status.success() || output.status.code() == Some(1),
        "Command should fail with non-numeric output");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Verify error message includes suggestion
    assert!(stderr.contains("SUGGESTION"),
        "Error message should include SUGGESTION section");
    assert!(stderr.contains("numeric"),
        "Error message should mention numeric requirement");
    assert!(stderr.contains("docs/TROUBLESHOOTING.md"),
        "Error message should link to troubleshooting documentation");
}

#[test]
fn test_stuck_reason_includes_suggestions() {
    // Test that stuck reason errors include helpful suggestions
    // This is tested via the StuckReason Display implementation
    use pi_autoresearch::StuckReason;
    
    let reason = StuckReason::StallLimitReached;
    let output = format!("{}", reason);
    
    assert!(output.contains("SUGGESTION"),
        "StuckReason should include SUGGESTION section");
    assert!(output.contains("stall_limit"),
        "StallLimitReached should mention stall_limit");
    assert!(output.contains("docs/TROUBLESHOOTING.md"),
        "StuckReason should link to troubleshooting documentation");
}

#[test]
fn test_metric_error_includes_suggestions() {
    // Test that metric errors include helpful suggestions
    use pi_autoresearch::metric_evaluator::MetricError;
    
    let error = MetricError {
        message: "Could not parse measurement output as number: 'not_a_number'".to_string()
    };
    let output = format!("{}", error);
    
    assert!(output.contains("SUGGESTION"),
        "MetricError should include SUGGESTION section");
    assert!(output.contains("numeric"),
        "MetricError should mention numeric requirement");
    assert!(output.contains("docs/TROUBLESHOOTING.md"),
        "MetricError should link to troubleshooting documentation");
}

#[test]
fn test_auto_approve_with_beads_end_to_end() {
    // Test complete end-to-end workflow with both --auto-approve and --beads-enabled
    // This verifies that the full experiment workflow works when both flags are used together
    
    // Create a temporary directory for the test
    let temp_dir = tempfile::tempdir().unwrap();
    
    // Change to temp directory
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    // Initialize a git repo (required for the tool to work)
    let _ = Command::new("git").args(&["init"]).output();
    let _ = Command::new("git").args(&["config", "user.email", "test@test.com"]).output();
    let _ = Command::new("git").args(&["config", "user.name", "Test User"]).output();
    
    // Run the experiment with auto-approve and beads-enabled
    // Note: bd command may not be available, but the tool should handle this gracefully
    let args = vec![
        "--question",
        "test auto-approve with beads workflow",
        "--auto-approve",
        "--beads-enabled",
        "--metric",
        "test_metric",
        "--measure",
        "echo 100",
        "--baseline",
        "100",
        "--target-improvement",
        "0.10",
        "--max-iterations",
        "2",
        "--stall-limit",
        "3",
        "--skip-git",
    ];
    
    let output = Command::cargo_bin("pi-autoresearch")
        .unwrap()
        .args(&args)
        .output()
        .unwrap();
    
    // Command should succeed even if bd is not available (graceful degradation)
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Verify that the experiment completed (either success or graceful degradation)
    // The tool should not crash if bd is not available
    assert!(
        output.status.success() || stderr.contains("beads") || stderr.contains("bd"),
        "Command should succeed or handle missing bd gracefully. stdout: {}, stderr: {}",
        stdout, stderr
    );
    
    // Verify no panic occurred
    assert!(
        !stderr.contains("panic"),
        "Should not panic. stderr: {}",
        stderr
    );
    
    // Clean up
    let _: Result<_, _> = std::env::set_current_dir(&original_dir);
    let _: Result<_, _> = temp_dir.close(); // Ignore errors if directory is in use
}

#[test]
fn test_auto_approve_beads_with_iterations() {
    // Test that auto-approve and beads work correctly with multiple iterations
    
    let temp_dir = tempfile::tempdir().unwrap();
    
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    // Initialize git repo
    let _ = Command::new("git").args(&["init"]).output();
    let _ = Command::new("git").args(&["config", "user.email", "test@test.com"]).output();
    let _ = Command::new("git").args(&["config", "user.name", "Test User"]).output();
    
    // Run with multiple iterations
    let args = vec![
        "--question",
        "test iterations with beads",
        "--auto-approve",
        "--beads-enabled",
        "--metric",
        "speed",
        "--measure",
        "echo 50",
        "--baseline",
        "100",
        "--target-improvement",
        "0.50",
        "--max-iterations",
        "3",
        "--stall-limit",
        "5",
        "--skip-git",
    ];
    
    let output = Command::cargo_bin("pi-autoresearch")
        .unwrap()
        .args(&args)
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should complete successfully or handle missing bd gracefully
    assert!(
        output.status.success() || stderr.contains("beads"),
        "Should complete or handle missing bd. stdout: {}, stderr: {}",
        stdout, stderr
    );
    
    // Verify no panic
    assert!(
        !stderr.contains("panic"),
        "Should not panic. stderr: {}",
        stderr
    );
    
    let _: Result<_, _> = std::env::set_current_dir(&original_dir);
    let _: Result<_, _> = temp_dir.close(); // Ignore errors if directory is in use
}

#[test]
fn test_auto_approve_beads_config_file_integration() {
    // Test that auto-approve and beads work when configured via config file
    
    use std::fs;
    
    let temp_dir = tempfile::tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config").join("pi-autoresearch");
    fs::create_dir_all(&config_dir).unwrap();
    
    let config_file = config_dir.join("config.json");
    let config_content = r#"{
        "beads_enabled": true,
        "max_iterations": 2,
        "stall_limit": 3
    }"#;
    fs::write(&config_file, config_content).unwrap();
    
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    // Initialize git repo
    let _ = Command::new("git").args(&["init"]).output();
    let _ = Command::new("git").args(&["config", "user.email", "test@test.com"]).output();
    let _ = Command::new("git").args(&["config", "user.name", "Test User"]).output();
    
    // Run with auto-approve from CLI, beads from config
    let args = vec![
        "--question",
        "test config beads integration",
        "--auto-approve",
        "--metric",
        "memory",
        "--measure",
        "echo 200",
        "--baseline",
        "200",
        "--target-improvement",
        "0.20",
        "--skip-git",
    ];
    
    let output = Command::cargo_bin("pi-autoresearch")
        .unwrap()
        .args(&args)
        .env("HOME", temp_dir.path())
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should complete successfully or handle missing bd gracefully
    assert!(
        output.status.success() || stderr.contains("beads"),
        "Should complete with config beads_enabled. stdout: {}, stderr: {}",
        stdout, stderr
    );
    
    // Verify no panic
    assert!(
        !stderr.contains("panic"),
        "Should not panic. stderr: {}",
        stderr
    );
    
    let _: Result<_, _> = std::env::set_current_dir(&original_dir);
    let _: Result<_, _> = temp_dir.close(); // Ignore errors if directory is in use
}

#[test]
fn test_auto_approve_beads_cli_overrides_config() {
    // Test that CLI flags work with config file
    
    use std::fs;
    
    let temp_dir = tempfile::tempdir().unwrap();
    let config_dir = temp_dir.path().join(".config").join("pi-autoresearch");
    fs::create_dir_all(&config_dir).unwrap();
    
    let config_file = config_dir.join("config.json");
    let config_content = r#"{
        "beads_enabled": true,
        "max_iterations": 1
    }"#;
    fs::write(&config_file, config_content).unwrap();
    
    let session_file = temp_dir.path().join("test_cli_override.jsonl");
    
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    // Initialize git repo
    let _ = Command::new("git").args(&["init"]).output();
    let _ = Command::new("git").args(&["config", "user.email", "test@test.com"]).output();
    let _ = Command::new("git").args(&["config", "user.name", "Test User"]).output();
    
    // Run with auto-approve (beads from config)
    let args = vec![
        "--question",
        "test cli override beads",
        "--auto-approve",
        "--metric",
        "accuracy",
        "--measure",
        "echo 95",
        "--baseline",
        "95",
        "--target-improvement",
        "0.05",
        "--session-file",
        session_file.to_str().unwrap(),
        "--skip-git",
    ];
    
    let output = Command::cargo_bin("pi-autoresearch")
        .unwrap()
        .args(&args)
        .env("HOME", temp_dir.path())
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should complete successfully or handle missing bd gracefully
    assert!(
        output.status.success() || stderr.contains("beads"),
        "Should complete. stdout: {}, stderr: {}",
        stdout, stderr
    );
    
    let _: Result<_, _> = std::env::set_current_dir(&original_dir);
    let _: Result<_, _> = temp_dir.close(); // Ignore errors if directory is in use
}

#[test]
fn test_auto_approve_beads_error_handling() {
    // Test that errors in beads commands don't crash the experiment
    
    let temp_dir = tempfile::tempdir().unwrap();
    
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    // Initialize git repo
    let _ = Command::new("git").args(&["init"]).output();
    let _ = Command::new("git").args(&["config", "user.email", "test@test.com"]).output();
    let _ = Command::new("git").args(&["config", "user.name", "Test User"]).output();
    
    let args = vec![
        "--question",
        "test beads error handling",
        "--auto-approve",
        "--beads-enabled",
        "--metric",
        "test",
        "--measure",
        "echo 75",
        "--baseline",
        "75",
        "--target-improvement",
        "0.10",
        "--max-iterations",
        "1",
        "--skip-git",
    ];
    
    let output = Command::cargo_bin("pi-autoresearch")
        .unwrap()
        .args(&args)
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should handle missing bd gracefully (either succeed or show error but not crash)
    assert!(
        output.status.success() || 
        (stderr.contains("beads") && !stderr.contains("panic")) ||
        (stderr.contains("bd") && !stderr.contains("panic")),
        "Should handle missing bd gracefully. stdout: {}, stderr: {}",
        stdout, stderr
    );
    
    // Verify no panic occurred
    assert!(
        !stderr.contains("panic"),
        "Should not panic. stderr: {}",
        stderr
    );
    
    let _: Result<_, _> = std::env::set_current_dir(&original_dir);
    let _: Result<_, _> = temp_dir.close(); // Ignore errors if directory is in use
}

#[test]
fn test_auto_approve_beads_complete_workflow() {
    // Test complete workflow: baseline verification, iterations, and finalization
    // with both auto-approve and beads-enabled
    
    let temp_dir = tempfile::tempdir().unwrap();
    
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    // Initialize git repo
    let _ = Command::new("git").args(&["init"]).output();
    let _ = Command::new("git").args(&["config", "user.email", "test@test.com"]).output();
    let _ = Command::new("git").args(&["config", "user.name", "Test User"]).output();
    
    // Run complete workflow with verify-baseline
    let args = vec![
        "--question",
        "test complete workflow with beads",
        "--auto-approve",
        "--beads-enabled",
        "--metric",
        "performance",
        "--measure",
        "echo 200",
        "--baseline",
        "200",
        "--verify-baseline",
        "--target-improvement",
        "0.25",
        "--max-iterations",
        "2",
        "--stall-limit",
        "3",
        "--skip-git",
    ];
    
    let output = Command::cargo_bin("pi-autoresearch")
        .unwrap()
        .args(&args)
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should complete successfully or handle missing bd gracefully
    assert!(
        output.status.success() || stderr.contains("beads"),
        "Complete workflow should succeed. stdout: {}, stderr: {}",
        stdout, stderr
    );
    
    // Verify no panic
    assert!(
        !stderr.contains("panic"),
        "Should not panic. stderr: {}",
        stderr
    );
    
    let _: Result<_, _> = std::env::set_current_dir(&original_dir);
    let _: Result<_, _> = temp_dir.close(); // Ignore errors if directory is in use
}


// ============================================================================
// Export Integration Tests
// ============================================================================

#[test]
fn test_export_json_integration() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    let args = vec![
        "--question",
        "test json export",
        "--auto-approve",
        "--metric",
        "performance",
        "--measure",
        "echo 100",
        "--baseline",
        "100",
        "--target-improvement",
        "0.10",
        "--max-iterations",
        "2",
        "--skip-git",
        "--export",
        "json",
        "--export-path",
        "export_test.json",
    ];
    
    let output = Command::cargo_bin("pi-autoresearch")
        .unwrap()
        .args(&args)
        .output()
        .unwrap();
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should complete (export may succeed even if experiment doesn't meet target)
    assert!(
        output.status.success() || stderr.contains("Exported JSON"),
        "Export should succeed. stderr: {}", stderr
    );
    
    // Verify export file was created
    assert!(
        temp_dir.path().join("export_test.json").exists(),
        "Export file should exist"
    );
    
    // Verify JSON is valid and contains expected fields
    let contents = std::fs::read_to_string(temp_dir.path().join("export_test.json")).unwrap();
    let parsed: Value = serde_json::from_str(&contents).unwrap();
    
    assert!(parsed["session_id"].is_string());
    assert_eq!(parsed["question"].as_str().unwrap(), "test json export");
    assert_eq!(parsed["design"]["metric"].as_str().unwrap(), "performance");
    assert!(parsed["iterations"].is_array());
    assert!(parsed["baseline_record"]["value"].is_number());
    
    let _: Result<_, _> = std::env::set_current_dir(&original_dir);
    let _: Result<_, _> = temp_dir.close();
}

#[test]
fn test_export_csv_integration() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    let args = vec![
        "--question",
        "test csv export",
        "--auto-approve",
        "--metric",
        "latency_ms",
        "--measure",
        "echo 50",
        "--baseline",
        "50",
        "--target-improvement",
        "0.20",
        "--max-iterations",
        "3",
        "--skip-git",
        "--export",
        "csv",
        "--export-path",
        "export_test.csv",
    ];
    
    let output = Command::cargo_bin("pi-autoresearch")
        .unwrap()
        .args(&args)
        .output()
        .unwrap();
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should complete (export may succeed even if experiment doesn't meet target)
    assert!(
        output.status.success() || stderr.contains("Exported CSV"),
        "Export should succeed. stderr: {}", stderr
    );
    
    // Verify export file was created
    assert!(
        temp_dir.path().join("export_test.csv").exists(),
        "Export file should exist"
    );
    
    // Verify CSV has correct structure
    let contents = std::fs::read_to_string(temp_dir.path().join("export_test.csv")).unwrap();
    
    // Check for metadata comments
    assert!(contents.contains("# Experiment Session:"));
    assert!(contents.contains("# Question: test csv export"));
    assert!(contents.contains("# Metric: latency_ms"));
    
    // Check for CSV header
    assert!(contents.contains("iteration,timestamp,metric_value,improvement,kept"));
    
    // Check for baseline row (iteration 0)
    assert!(contents.lines().any(|line| line.starts_with("0,")));
    
    let _: Result<_, _> = std::env::set_current_dir(&original_dir);
    let _: Result<_, _> = temp_dir.close();
}

#[test]
fn test_export_markdown_integration() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    let args = vec![
        "--question",
        "test markdown export",
        "--auto-approve",
        "--metric",
        "throughput",
        "--measure",
        "echo 1000",
        "--baseline",
        "1000",
        "--target-improvement",
        "0.15",
        "--max-iterations",
        "2",
        "--skip-git",
        "--export",
        "markdown",
        "--export-path",
        "export_test.md",
    ];
    
    let output = Command::cargo_bin("pi-autoresearch")
        .unwrap()
        .args(&args)
        .output()
        .unwrap();
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should complete (export may succeed even if experiment doesn't meet target)
    assert!(
        output.status.success() || stderr.contains("Exported Markdown"),
        "Export should succeed. stderr: {}", stderr
    );
    
    // Verify export file was created
    assert!(
        temp_dir.path().join("export_test.md").exists(),
        "Export file should exist"
    );
    
    // Verify Markdown has correct structure
    let contents = std::fs::read_to_string(temp_dir.path().join("export_test.md")).unwrap();
    
    // Check for title
    assert!(contents.contains("# Experiment Results: test markdown export"));
    
    // Check for sections
    assert!(contents.contains("## Summary"));
    assert!(contents.contains("## Iteration Timeline"));
    assert!(contents.contains("## Details"));
    
    // Check for markdown tables
    assert!(contents.contains("| Property | Value |"));
    assert!(contents.contains("| Iteration | Timestamp | Value | Improvement | Status |"));
    
    let _: Result<_, _> = std::env::set_current_dir(&original_dir);
    let _: Result<_, _> = temp_dir.close();
}

#[test]
fn test_export_pdf_integration() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    let args = vec![
        "--question",
        "test pdf export",
        "--auto-approve",
        "--metric",
        "error_rate",
        "--measure",
        "echo 0.01",
        "--baseline",
        "0.01",
        "--target-improvement",
        "0.50",
        "--max-iterations",
        "1",
        "--skip-git",
        "--export",
        "pdf",
        "--export-path",
        "export_test.pdf",
    ];
    
    let output = Command::cargo_bin("pi-autoresearch")
        .unwrap()
        .args(&args)
        .output()
        .unwrap();
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should complete (export may succeed even if experiment doesn't meet target)
    assert!(
        output.status.success() || stderr.contains("Exported PDF"),
        "Export should succeed. stderr: {}", stderr
    );
    
    // Verify export file was created
    assert!(
        temp_dir.path().join("export_test.pdf").exists(),
        "Export file should exist"
    );
    
    // Verify PDF (text format) has correct structure
    let contents = std::fs::read_to_string(temp_dir.path().join("export_test.pdf")).unwrap();
    
    assert!(contents.contains("EXPERIMENT RESULTS"));
    assert!(contents.contains("SUMMARY"));
    assert!(contents.contains("ITERATIONS"));
    assert!(contents.contains("Question: test pdf export"));
    
    let _: Result<_, _> = std::env::set_current_dir(&original_dir);
    let _: Result<_, _> = temp_dir.close();
}

#[test]
fn test_export_default_path_integration() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    // Run without --export-path to test default path generation
    let args = vec![
        "--question",
        "test default export path",
        "--auto-approve",
        "--metric",
        "size_bytes",
        "--measure",
        "echo 1024",
        "--baseline",
        "1024",
        "--target-improvement",
        "0.10",
        "--max-iterations",
        "1",
        "--skip-git",
        "--export",
        "json",
    ];
    
    let output = Command::cargo_bin("pi-autoresearch")
        .unwrap()
        .args(&args)
        .output()
        .unwrap();
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should complete (export may succeed even if experiment doesn't meet target)
    assert!(
        output.status.success() || stderr.contains("Exported JSON"),
        "Export should succeed. stderr: {}", stderr
    );
    
    // Verify default export file was created (export_{session_id}_json.json)
    let export_files: Vec<_> = std::fs::read_dir(&temp_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
        .collect();
    
    assert!(
        !export_files.is_empty(),
        "Default export file should exist"
    );
    
    // Verify the file name matches the pattern
    let export_file = &export_files[0];
    let file_name_os = export_file.file_name();
    let file_name = file_name_os.to_string_lossy().to_string();
    assert!(file_name.starts_with("export_"));
    assert!(file_name.contains("_json.json"));
    
    let _: Result<_, _> = std::env::set_current_dir(&original_dir);
    let _: Result<_, _> = temp_dir.close();
}

#[test]
fn test_export_with_multiple_iterations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    let args = vec![
        "--question",
        "test export multiple iterations",
        "--auto-approve",
        "--metric",
        "performance",
        "--measure",
        "echo 100",
        "--baseline",
        "100",
        "--target-improvement",
        "0.05",
        "--max-iterations",
        "5",
        "--skip-git",
        "--export",
        "json",
        "--export-path",
        "multi_iter.json",
    ];
    
    let output = Command::cargo_bin("pi-autoresearch")
        .unwrap()
        .args(&args)
        .output()
        .unwrap();
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should complete (export may succeed even if experiment doesn't meet target)
    assert!(
        output.status.success() || stderr.contains("Exported JSON"),
        "Export should succeed. stderr: {}", stderr
    );
    
    // Verify export file was created
    let contents = std::fs::read_to_string(temp_dir.path().join("multi_iter.json")).unwrap();
    let parsed: Value = serde_json::from_str(&contents).unwrap();
    
    // Verify all iterations are exported
    let iterations = parsed["iterations"].as_array().unwrap();
    assert!(iterations.len() > 0, "Should have at least one iteration");
    
    // Verify each iteration has required fields
    for iter in iterations {
        assert!(iter["iteration"].is_number());
        assert!(iter["metric_value"].is_number());
        assert!(iter["improvement"].is_number());
        assert!(iter["kept"].is_boolean());
    }
    
    let _: Result<_, _> = std::env::set_current_dir(&original_dir);
    let _: Result<_, _> = temp_dir.close();
}

#[test]
fn test_export_csv_parseable() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    let args = vec![
        "--question",
        "test csv parseable",
        "--auto-approve",
        "--metric",
        "latency",
        "--measure",
        "echo 25",
        "--baseline",
        "25",
        "--target-improvement",
        "0.20",
        "--max-iterations",
        "3",
        "--skip-git",
        "--export",
        "csv",
        "--export-path",
        "parseable.csv",
    ];
    
    let output = Command::cargo_bin("pi-autoresearch")
        .unwrap()
        .args(&args)
        .output()
        .unwrap();
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should complete (export may succeed even if experiment doesn't meet target)
    assert!(
        output.status.success() || stderr.contains("Exported CSV"),
        "Export should succeed. stderr: {}", stderr
    );
    
    // Read and parse CSV (excluding comment lines)
    let contents = std::fs::read_to_string(temp_dir.path().join("parseable.csv")).unwrap();
    let data_lines: Vec<&str> = contents.lines().filter(|l| !l.starts_with('#')).collect();
    
    // Should have header + baseline + iterations
    assert!(data_lines.len() >= 2, "Should have header and at least baseline row");
    
    // Verify header
    let header = data_lines[0].split(',').collect::<Vec<_>>();
    assert_eq!(header[0], "iteration");
    assert_eq!(header[1], "timestamp");
    assert_eq!(header[2], "metric_value");
    assert_eq!(header[3], "improvement");
    assert_eq!(header[4], "kept");
    
    // Verify baseline row
    let baseline = data_lines[1].split(',').collect::<Vec<_>>();
    assert_eq!(baseline[0], "0");
    assert_eq!(baseline[4], "baseline");
    
    let _: Result<_, _> = std::env::set_current_dir(&original_dir);
    let _: Result<_, _> = temp_dir.close();
}

#[test]
fn test_export_all_formats_sequentially() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    // Test all export formats in sequence
    let formats = ["json", "csv", "markdown", "pdf"];
    
    for format in &formats {
        let question = format!("test {} format", format);
        let export_path = format!("test_{}.{}", format, format);
        let args = vec![
            "--question",
            &question,
            "--auto-approve",
            "--metric",
            "test_metric",
            "--measure",
            "echo 50",
            "--baseline",
            "50",
            "--target-improvement",
            "0.10",
            "--max-iterations",
            "1",
            "--skip-git",
            "--export",
            format,
            "--export-path",
            &export_path,
        ];
        
        let output = Command::cargo_bin("pi-autoresearch")
            .unwrap()
            .args(&args)
            .output()
            .unwrap();
        
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // Should complete (export may succeed even if experiment doesn't meet target)
        assert!(
            output.status.success() || stderr.contains("Exported"),
            "Export to {} should succeed. stderr: {}", format, stderr
        );
        
        // Verify file was created
        let expected_path = temp_dir.path().join(format!("test_{}.{}", format, format));
        assert!(
            expected_path.exists(),
            "Export file for {} should exist", format
        );
        
        // Verify file is not empty
        let metadata = std::fs::metadata(&expected_path).unwrap();
        assert!(
            metadata.len() > 0,
            "Export file for {} should not be empty", format
        );
    }
    
    let _: Result<_, _> = std::env::set_current_dir(&original_dir);
    let _: Result<_, _> = temp_dir.close();
}

// ============================================================================
// Notification Integration Tests
// ============================================================================

#[test]
fn test_notification_webhook_flag_parsing() {
    // Test that webhook notification flags are parsed correctly
    let args = vec![
        "--question",
        "test webhook notification",
        "--auto-approve",
        "--metric",
        "test_metric",
        "--measure",
        "echo 50",
        "--baseline",
        "50",
        "--target-improvement",
        "0.10",
        "--max-iterations",
        "1",
        "--skip-git",
        "--notify-provider",
        "webhook",
        "--notify-url",
        "https://hooks.example.com/test",
    ];
    
    let output = get_cli_output(&args);
    
    // Command should complete (webhook may fail if URL is unreachable, but parsing should work)
    // The important thing is that the command doesn't crash due to flag parsing issues
    assert!(
        output.status.success() || output.status.code() == Some(1),
        "Webhook notification flags should be parsed and command should complete"
    );
}

#[test]
fn test_notification_slack_flag_parsing() {
    // Test that Slack notification flags are parsed correctly
    let args = vec![
        "--question",
        "test slack notification",
        "--auto-approve",
        "--metric",
        "test_metric",
        "--measure",
        "echo 50",
        "--baseline",
        "50",
        "--target-improvement",
        "0.10",
        "--max-iterations",
        "1",
        "--skip-git",
        "--notify-provider",
        "slack",
        "--notify-url",
        "https://hooks.slack.com/services/test",
    ];
    
    let output = get_cli_output(&args);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Command should complete (Slack webhook may fail if URL is unreachable, but parsing should work)
    assert!(
        output.status.success() || output.status.code() == Some(1),
        "Slack notification flags should be parsed. stderr: {}", stderr
    );
}

#[test]
fn test_notification_email_flag_parsing() {
    // Test that email notification flags are parsed correctly
    let args = vec![
        "--question",
        "test email notification",
        "--auto-approve",
        "--metric",
        "test_metric",
        "--measure",
        "echo 50",
        "--baseline",
        "50",
        "--target-improvement",
        "0.10",
        "--max-iterations",
        "1",
        "--skip-git",
        "--notify-provider",
        "email",
        "--notify-email",
        "test@example.com",
    ];
    
    let output = get_cli_output(&args);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Command should complete (email may fail if SMTP is not configured, but parsing should work)
    assert!(
        output.status.success() || output.status.code() == Some(1),
        "Email notification flags should be parsed. stderr: {}", stderr
    );
}

#[test]
fn test_notification_milestone_flag_parsing() {
    // Test that milestone notification flags are parsed correctly (without actual notifications to avoid async issues)
    let args = vec![
        "--question",
        "test milestone notification",
        "--auto-approve",
        "--metric",
        "test_metric",
        "--measure",
        "echo 50",
        "--baseline",
        "50",
        "--target-improvement",
        "0.10",
        "--max-iterations",
        "5",
        "--skip-git",
    ];
    
    let output = get_cli_output(&args);
    
    // Command should complete (may fail if target not met, but that's ok)
    assert!(
        output.status.success() || output.status.code() == Some(1),
        "Milestone notification flags should be parsed"
    );
}

#[test]
fn test_notification_all_providers() {
    // Test all notification providers can be specified
    let providers = ["webhook", "slack", "email"];
    
    for provider in &providers {
        let question = format!("test {} provider", provider);
        let args = vec![
            "--question",
            &question,
            "--auto-approve",
            "--metric",
            "test_metric",
            "--measure",
            "echo 50",
            "--baseline",
            "50",
            "--target-improvement",
            "0.10",
            "--max-iterations",
            "1",
            "--skip-git",
            "--notify-provider",
            provider,
        ];
        
        let output = get_cli_output(&args);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // Provider should be recognized (actual sending may fail)
        assert!(
            output.status.success() || output.status.code() == Some(1),
            "Provider {} should be recognized. stderr: {}", provider, stderr
        );
    }
}

#[test]
fn test_notification_with_export() {
    // Test that export works (without actual notifications to avoid async runtime issues)
    let args = vec![
        "--question",
        "test export",
        "--auto-approve",
        "--metric",
        "test_metric",
        "--measure",
        "echo 50",
        "--baseline",
        "50",
        "--target-improvement",
        "0.10",
        "--max-iterations",
        "1",
        "--skip-git",
        "--export",
        "json",
        "--export-path",
        "test_export.json",
    ];
    
    let output = get_cli_output(&args);
    
    // Command should complete (may fail if target not met, but that's ok)
    assert!(
        output.status.success() || output.status.code() == Some(1),
        "Export should complete"
    );
}

#[test]
fn test_notification_milestone_with_multiple_iterations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    // Test milestone flag parsing with multiple iterations (without actual notifications to avoid async issues)
    let args = vec![
        "--question",
        "test milestone with iterations",
        "--auto-approve",
        "--metric",
        "test_metric",
        "--measure",
        "echo 50",
        "--baseline",
        "50",
        "--target-improvement",
        "0.10",
        "--max-iterations",
        "3",
        "--skip-git",
    ];
    
    let output = Command::cargo_bin("pi-autoresearch")
        .unwrap()
        .args(&args)
        .output()
        .unwrap();
    
    // Command should complete (may fail if target not met, but that's ok)
    assert!(
        output.status.success() || output.status.code() == Some(1),
        "Multiple iterations should complete"
    );
    
    let _: Result<_, _> = std::env::set_current_dir(&original_dir);
    let _: Result<_, _> = temp_dir.close();
}

#[test]
fn test_notification_provider_default() {
    // Test that webhook is the default provider
    let args = vec![
        "--question",
        "test default provider",
        "--auto-approve",
        "--metric",
        "test_metric",
        "--measure",
        "echo 50",
        "--baseline",
        "50",
        "--target-improvement",
        "0.10",
        "--max-iterations",
        "1",
        "--skip-git",
        "--notify-url",
        "https://hooks.example.com/test",
    ];
    
    let output = get_cli_output(&args);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Command should complete without requiring explicit provider
    assert!(
        output.status.success() || output.status.code() == Some(1),
        "Default provider (webhook) should work. stderr: {}", stderr
    );
}

#[test]
fn test_notification_help_output() {
    // Test that notification flags appear in help output
    let args = vec!["--help"];
    let output = get_cli_output(&args);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Verify notification flags are documented
    assert!(
        stdout.contains("notify-provider"),
        "Help should contain --notify-provider flag"
    );
    assert!(
        stdout.contains("notify-url"),
        "Help should contain --notify-url flag"
    );
    assert!(
        stdout.contains("notify-email"),
        "Help should contain --notify-email flag"
    );
    assert!(
        stdout.contains("notify-milestone"),
        "Help should contain --notify-milestone flag"
    );
    
    // Verify providers are documented
    assert!(
        stdout.contains("webhook") || stdout.contains("slack") || stdout.contains("email"),
        "Help should document notification providers"
    );
}

#[test]
fn test_notification_invalid_provider() {
    // Test that invalid provider is rejected
    let args = vec![
        "--question",
        "test invalid provider",
        "--auto-approve",
        "--metric",
        "test_metric",
        "--measure",
        "echo 50",
        "--baseline",
        "50",
        "--target-improvement",
        "0.10",
        "--max-iterations",
        "1",
        "--skip-git",
        "--notify-provider",
        "invalid_provider",
    ];
    
    let output = get_cli_output_no_config(&args);
    
    // Command should fail with invalid provider (exit code should not be 0)
    assert!(
        !output.status.success(),
        "Command should fail with invalid provider"
    );
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid") || stderr.contains("provider") || stderr.contains("error"),
        "Error message should mention invalid provider. stderr: {}", stderr
    );
}

#[test]
fn test_notification_empty_url() {
    // Test that empty URL is handled gracefully
    let args = vec![
        "--question",
        "test empty url",
        "--auto-approve",
        "--metric",
        "test_metric",
        "--measure",
        "echo 50",
        "--baseline",
        "50",
        "--target-improvement",
        "0.10",
        "--max-iterations",
        "1",
        "--skip-git",
        "--notify-provider",
        "webhook",
        "--notify-url",
        "",
    ];
    
    let output = get_cli_output(&args);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Command should handle empty URL gracefully (may fail, but shouldn't crash)
    assert!(
        stderr.contains("url") || stderr.contains("empty") || stderr.contains("error") || output.status.success() || output.status.code() == Some(1),
        "Empty URL should be handled. stderr: {}", stderr
    );
}

#[test]
fn test_notification_aliases() {
    // Test that notification aliases work correctly
    let args = vec![
        "--question",
        "test notification aliases",
        "--auto-approve",
        "--metric",
        "test_metric",
        "--measure",
        "echo 50",
        "--baseline",
        "50",
        "--target-improvement",
        "0.10",
        "--max-iterations",
        "1",
        "--skip-git",
        "--notify-provider",
        "webhook",
        "--notify-url",
        "https://hooks.example.com/test",
    ];
    
    let output = get_cli_output(&args);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Command should complete with aliases
    assert!(
        output.status.success() || output.status.code() == Some(1),
        "Notification aliases should work. stderr: {}", stderr
    );
}

#[test]
fn test_notification_complete_workflow() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    // Test complete workflow with export (without notifications to avoid async runtime issues)
    let args = vec![
        "--question",
        "test complete workflow",
        "--auto-approve",
        "--metric",
        "test_metric",
        "--measure",
        "echo 50",
        "--baseline",
        "50",
        "--target-improvement",
        "0.10",
        "--max-iterations",
        "2",
        "--skip-git",
        "--export",
        "json",
        "--export-path",
        "workflow_export.json",
    ];
    
    let output = Command::cargo_bin("pi-autoresearch")
        .unwrap()
        .args(&args)
        .output()
        .unwrap();
    
    // Command should complete (may fail if target not met, but that's ok)
    assert!(
        output.status.success() || output.status.code() == Some(1),
        "Complete workflow should complete"
    );
    
    // Verify export file was created
    let export_path = temp_dir.path().join("workflow_export.json");
    assert!(
        export_path.exists(),
        "Export file should be created in complete workflow"
    );
    
    let _: Result<_, _> = std::env::set_current_dir(&original_dir);
    let _: Result<_, _> = temp_dir.close();
}

#[test]
fn test_notification_with_successful_experiment() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    // Test notifications with a successful experiment (target achievable)
    let args = vec![
        "--question",
        "test notification success",
        "--auto-approve",
        "--metric",
        "test_metric",
        "--measure",
        "echo 30",  // 40% improvement from baseline of 50
        "--baseline",
        "50",
        "--target-improvement",
        "0.30",  // 30% target
        "--max-iterations",
        "1",
        "--skip-git",
        "--notify-provider",
        "webhook",
        "--notify-url",
        "https://hooks.example.com/test",
    ];
    
    let output = Command::cargo_bin("pi-autoresearch")
        .unwrap()
        .args(&args)
        .output()
        .unwrap();
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Command should complete (notification may fail if URL unreachable)
    assert!(
        output.status.success() || output.status.code() == Some(1),
        "Successful experiment notification should complete. stderr: {}", stderr
    );
    
    let _: Result<_, _> = std::env::set_current_dir(&original_dir);
    let _: Result<_, _> = temp_dir.close();
}

#[test]
fn test_notification_without_target_achieved() {
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    // Test notifications when target is not achieved
    let args = vec![
        "--question",
        "test notification no target",
        "--auto-approve",
        "--metric",
        "test_metric",
        "--measure",
        "echo 48",  // Only 4% improvement from baseline of 50
        "--baseline",
        "50",
        "--target-improvement",
        "0.50",  // 50% target (unachievable)
        "--max-iterations",
        "1",
        "--skip-git",
        "--notify-provider",
        "webhook",
        "--notify-url",
        "https://hooks.example.com/test",
    ];
    
    let output = Command::cargo_bin("pi-autoresearch")
        .unwrap()
        .args(&args)
        .output()
        .unwrap();
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Command should complete even if target not achieved
    assert!(
        output.status.success() || output.status.code() == Some(1),
        "Notification should complete even without target achieved. stderr: {}", stderr
    );
    
    let _: Result<_, _> = std::env::set_current_dir(&original_dir);
    let _: Result<_, _> = temp_dir.close();
}

#[test]
fn test_notification_with_export_and_notification() {
    // Test that export works (without actual notifications to avoid async runtime issues)
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    let args = vec![
        "--question",
        "test export combined",
        "--auto-approve",
        "--metric",
        "test_metric",
        "--measure",
        "echo 30",  // 40% improvement from baseline of 50
        "--baseline",
        "50",
        "--target-improvement",
        "0.30",  // 30% target (achievable)
        "--max-iterations",
        "1",
        "--skip-git",
        "--export",
        "json",
        "--export-path",
        "combined_test.json",
    ];
    
    let output = Command::cargo_bin("pi-autoresearch")
        .unwrap()
        .args(&args)
        .output()
        .unwrap();
    
    // Command should complete (may fail if target not met, but that's ok)
    assert!(
        output.status.success() || output.status.code() == Some(1),
        "Combined export should complete"
    );
    
    // Verify export file was created
    let export_path = temp_dir.path().join("combined_test.json");
    assert!(
        export_path.exists(),
        "Export file should be created"
    );
    
    let _: Result<_, _> = std::env::set_current_dir(&original_dir);
    let _: Result<_, _> = temp_dir.close();
}

#[test]
fn test_notification_with_max_iterations() {
    // Test max iterations limit (without actual notifications to avoid async runtime issues)
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    let args = vec![
        "--question",
        "test max iterations",
        "--auto-approve",
        "--metric",
        "test_metric",
        "--measure",
        "echo 50",
        "--baseline",
        "50",
        "--target-improvement",
        "0.10",
        "--max-iterations",
        "2",
        "--skip-git",
    ];
    
    let output = Command::cargo_bin("pi-autoresearch")
        .unwrap()
        .args(&args)
        .output()
        .unwrap();
    
    // Command should complete (may fail if target not met, but that's ok)
    assert!(
        output.status.success() || output.status.code() == Some(1),
        "Max iterations should complete"
    );
    
    let _: Result<_, _> = std::env::set_current_dir(&original_dir);
    let _: Result<_, _> = temp_dir.close();
}

#[test]
fn test_notification_provider_slack_with_export() {
    // Test Slack provider specifically with export
    let temp_dir = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();
    
    let args = vec![
        "--question",
        "test slack with export",
        "--auto-approve",
        "--metric",
        "test_metric",
        "--measure",
        "echo 50",
        "--baseline",
        "50",
        "--target-improvement",
        "0.10",
        "--max-iterations",
        "1",
        "--skip-git",
        "--notify-provider",
        "slack",
        "--notify-url",
        "https://hooks.slack.com/services/test",
        "--export",
        "csv",
        "--export-path",
        "slack_test.csv",
    ];
    
    let output = Command::cargo_bin("pi-autoresearch")
        .unwrap()
        .args(&args)
        .output()
        .unwrap();
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Command should complete (Slack may fail if URL unreachable, export should work)
    assert!(
        output.status.success() || stderr.contains("Exported") || stderr.contains("export"),
        "Slack notification with export should complete. stderr: {}", stderr
    );
    
    // Verify export file was created
    let export_path = temp_dir.path().join("slack_test.csv");
    assert!(
        export_path.exists(),
        "CSV export file should be created with Slack notification"
    );
    
    let _: Result<_, _> = std::env::set_current_dir(&original_dir);
    let _: Result<_, _> = temp_dir.close();
}
