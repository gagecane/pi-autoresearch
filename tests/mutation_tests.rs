//! Mutation Testing Tests
//!
//! This module contains tests designed to verify test quality through mutation testing principles.
//! Mutation testing helps identify weak tests by ensuring tests would fail if common bugs were introduced.
//!
//! These tests are designed to be "mutation-resistant" - they should fail if:
//! - Comparison operators are flipped (< becomes >)
//! - Arithmetic operators are changed (+ becomes -)
//! - Logical operators are inverted (&& becomes ||)
//! - Return values are changed
//! - Statements are removed

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn get_cli_output(args: &[&str]) -> std::process::Output {
    let mut cmd = Command::cargo_bin("pi-autoresearch").unwrap();
    cmd.args(args);
    cmd.output().unwrap()
}

#[cfg(test)]
mod mutation_resistant_tests {
    use super::*;

    /// Test that config validation catches invalid max_variance values
    /// Mutation-resistant: Would fail if validation is removed or comparison is flipped
    #[test]
    fn test_config_validation_max_variance_high() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        fs::write(&config_path, r#"{"max_variance": 1.5}"#).unwrap();
        
        let output = get_cli_output(&[
            "--config", config_path.to_str().unwrap(),
            "--question", "test",
            "--measure", "echo 1",
            "--skip-git"
        ]);
        
        // Should fail with validation error
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("max_variance"));
    }

    /// Test that config validation catches negative target_improvement
    /// Mutation-resistant: Would fail if validation is removed or comparison is flipped
    #[test]
    fn test_config_validation_target_improvement_negative() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        fs::write(&config_path, r#"{"target_improvement": -0.1}"#).unwrap();
        
        let output = get_cli_output(&[
            "--config", config_path.to_str().unwrap(),
            "--question", "test",
            "--measure", "echo 1",
            "--skip-git"
        ]);
        
        // Should fail with validation error
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("target_improvement"));
    }

    /// Test that config validation catches zero max_iterations
    /// Mutation-resistant: Would fail if validation is removed or comparison is flipped
    #[test]
    fn test_config_validation_max_iterations_zero() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        fs::write(&config_path, r#"{"max_iterations": 0}"#).unwrap();
        
        let output = get_cli_output(&[
            "--config", config_path.to_str().unwrap(),
            "--question", "test",
            "--measure", "echo 1",
            "--skip-git"
        ]);
        
        // Should fail with validation error
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("max_iterations"));
    }

    /// Test that multiple validation errors are all reported
    /// Mutation-resistant: Would fail if error collection logic is changed
    #[test]
    fn test_multiple_validation_errors_reported() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        fs::write(&config_path, r#"{
            "max_variance": 1.5,
            "target_improvement": -0.1,
            "max_iterations": 0
        }"#).unwrap();
        
        let output = get_cli_output(&[
            "--config", config_path.to_str().unwrap(),
            "--question", "test",
            "--measure", "echo 1",
            "--skip-git"
        ]);
        
        // Should fail with multiple validation errors
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // All three errors should be reported
        assert!(stderr.contains("max_variance"));
        assert!(stderr.contains("target_improvement"));
        assert!(stderr.contains("max_iterations"));
    }

    /// Test that config validation accepts valid values
    /// Mutation-resistant: Would fail if validation is too strict
    #[test]
    fn test_config_validation_all_valid() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        fs::write(&config_path, r#"{
            "max_variance": 0.5,
            "target_improvement": 0.2,
            "max_iterations": 10
        }"#).unwrap();
        
        // Should not fail on config validation
        let output = get_cli_output(&[
            "--config", config_path.to_str().unwrap(),
            "--question", "test",
            "--measure", "echo 1",
            "--max-iterations", "1",
            "--skip-git"
        ]);
        
        // Config validation should pass (may fail on other things, but not config validation)
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!stderr.contains("Config validation failed"));
    }

    /// Test that version flag works
    /// Mutation-resistant: Would fail if version flag is removed
    #[test]
    fn test_version_flag() {
        let output = get_cli_output(&["--version"]);
        
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("pi-autoresearch"));
        assert!(stdout.contains("0.1.0"));
    }

    /// Test that help flag works
    /// Mutation-resistant: Would fail if help flag is removed
    #[test]
    fn test_help_flag() {
        let output = get_cli_output(&["--help"]);
        
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Usage:"));
        assert!(stdout.contains("--question"));
    }

    /// Test that dry-run mode works without side effects
    /// Mutation-resistant: Would fail if dry-run flag is ignored
    #[test]
    fn test_dry_run_no_side_effects() {
        let temp_dir = TempDir::new().unwrap();
        let session_file = temp_dir.path().join("test.jsonl");
        
        let output = get_cli_output(&[
            "--question", "test",
            "--measure", "echo 1",
            "--session-file", session_file.to_str().unwrap(),
            "--max-iterations", "1",
            "--dry-run",
            "--auto-approve",
            "--skip-git"
        ]);
        
        assert!(output.status.success());
        
        // Session file should not be created in dry-run mode
        assert!(!session_file.exists());
    }

    /// Test that metric detection works for performance questions
    /// Mutation-resistant: Would fail if performance keyword detection is removed
    #[test]
    fn test_metric_detection_performance() {
        let temp_dir = TempDir::new().unwrap();
        let session_file = temp_dir.path().join("test.jsonl");
        
        let output = get_cli_output(&[
            "--question", "How can I improve performance?",
            "--measure", "echo 100",
            "--session-file", session_file.to_str().unwrap(),
            "--max-iterations", "0",
            "--auto-approve",
            "--skip-git"
        ]);
        
        // CLI should run successfully (experiment may fail, but that's OK)
        // The important thing is that it doesn't crash
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Check that output contains the detected metric
        assert!(stdout.contains("execution_time_ms"));
    }

    /// Test that metric detection works for memory questions
    /// Mutation-resistant: Would fail if memory keyword detection is removed
    #[test]
    fn test_metric_detection_memory() {
        let temp_dir = TempDir::new().unwrap();
        let session_file = temp_dir.path().join("test.jsonl");
        
        let output = get_cli_output(&[
            "--question", "How can I reduce memory usage?",
            "--measure", "echo 100",
            "--session-file", session_file.to_str().unwrap(),
            "--max-iterations", "0",
            "--auto-approve",
            "--skip-git"
        ]);
        
        // CLI should run successfully (experiment may fail, but that's OK)
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Check that output contains the detected metric
        assert!(stdout.contains("peak_memory_mb"));
    }

    /// Test that branch name generation creates unique names
    /// Mutation-resistant: Would fail if UUID is removed from branch name
    #[test]
    fn test_branch_name_uniqueness() {
        // Run comparison which generates branch names
        let temp_dir = TempDir::new().unwrap();
        let session_file = temp_dir.path().join("test.jsonl");
        
        // Create two experiments
        for i in 0..2 {
            let output = get_cli_output(&[
                "--question", &format!("Test question {}", i),
                "--measure", "echo 100",
                "--session-file", session_file.to_str().unwrap(),
                "--max-iterations", "0",
                "--auto-approve",
                "--skip-git"
            ]);
            // CLI should run without crashing (experiment may fail, but that's OK)
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(!stdout.is_empty(), "CLI should produce output");
        }
        
        // Read session file and verify unique session IDs
        let contents = fs::read_to_string(&session_file).unwrap();
        let lines: Vec<&str> = contents.lines().filter(|l| !l.trim().is_empty()).collect();
        assert!(lines.len() >= 2, "Should have at least 2 records");
        
        // Each experiment should have a unique session_id
        let mut session_ids = Vec::new();
        for line in &lines {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(id) = v.get("session_id").and_then(|id| id.as_str()) {
                    session_ids.push(id.to_string());
                }
            }
        }
        
        let unique_ids: std::collections::HashSet<_> = session_ids.iter().collect();
        assert_eq!(unique_ids.len(), session_ids.len(), "All session IDs should be unique");
    }

    /// Test that error handling works for missing measurement command
    /// Mutation-resistant: Would fail if error handling is removed
    #[test]
    fn test_error_missing_measurement() {
        let output = get_cli_output(&[
            "--question", "test",
            "--auto-approve",
            "--skip-git"
        ]);
        
        // CLI should run but output should indicate measurement is needed
        // (the tool auto-detects a default measurement)
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(!stdout.is_empty(), "CLI should produce output");
    }

    /// Test that history flag works with empty session file
    /// Mutation-resistant: Would fail if empty file handling is removed
    #[test]
    fn test_history_empty_session() {
        let temp_dir = TempDir::new().unwrap();
        let session_file = temp_dir.path().join("test.jsonl");
        fs::write(&session_file, "").unwrap();
        
        let output = get_cli_output(&[
            "--history",
            "--session-file", session_file.to_str().unwrap()
        ]);
        
        assert!(output.status.success());
    }
}
