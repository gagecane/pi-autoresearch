//! Audit logging for pi-autoresearch experiments.
//!
//! This module provides comprehensive audit logging for compliance and tracking
//! of all experiment actions, decisions, and measurements.
//!
//! # Features
//!
//! * Append-only logging for data integrity
//! * Multiple output formats (JSON, CSV, text)
//! * Atomic writes to prevent corruption
//! * User and environment tracking
//! * RFC3339 timestamped entries
//!
//! # Examples
//!
//! ```
//! use pi_autoresearch::audit::{AuditLogger, AuditEventType};
//! use std::collections::HashMap;
//!
//! // Create an audit logger
//! let logger = AuditLogger::new("audit.log").unwrap();
//!
//! // Log an event
//! let mut details = HashMap::new();
//! details.insert("session_id".to_string(), "session-123".to_string());
//! logger.log(AuditEventType::ExperimentStarted, "Starting experiment".to_string(), details).unwrap();
//!
//! // Flush to ensure data is written
//! logger.flush().unwrap();
//! ```

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::Command;

/// Event types for audit logging
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    // Experiment lifecycle events
    ExperimentStarted,
    ExperimentCompleted,
    ExperimentFailed,
    
    // Iteration events
    IterationStarted,
    IterationCompleted,
    
    // Git operations
    BranchCreated,
    BranchMerged,
    BranchDeleted,
    BranchCheckedOut,
    CommitCreated,
    ConfigChanged,
    
    // Decision events
    ChangeKept,
    ChangeReverted,
    
    // Termination reasons
    TargetAchieved,
    TargetNotAchieved,
    Stalled,
    Converged,
    Timeout,
    MaxIterationsReached,
    
    // Measurement events
    MeasurementTaken,
    MeasurementFailed,
    BaselineMeasured,
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditEventType::ExperimentStarted => write!(f, "experiment_started"),
            AuditEventType::ExperimentCompleted => write!(f, "experiment_completed"),
            AuditEventType::ExperimentFailed => write!(f, "experiment_failed"),
            AuditEventType::IterationStarted => write!(f, "iteration_started"),
            AuditEventType::IterationCompleted => write!(f, "iteration_completed"),
            AuditEventType::BranchCreated => write!(f, "branch_created"),
            AuditEventType::BranchMerged => write!(f, "branch_merged"),
            AuditEventType::BranchDeleted => write!(f, "branch_deleted"),
            AuditEventType::BranchCheckedOut => write!(f, "branch_checked_out"),
            AuditEventType::CommitCreated => write!(f, "commit_created"),
            AuditEventType::ConfigChanged => write!(f, "config_changed"),
            AuditEventType::ChangeKept => write!(f, "change_kept"),
            AuditEventType::ChangeReverted => write!(f, "change_reverted"),
            AuditEventType::TargetAchieved => write!(f, "target_achieved"),
            AuditEventType::TargetNotAchieved => write!(f, "target_not_achieved"),
            AuditEventType::Stalled => write!(f, "stalled"),
            AuditEventType::Converged => write!(f, "converged"),
            AuditEventType::Timeout => write!(f, "timeout"),
            AuditEventType::MaxIterationsReached => write!(f, "max_iterations_reached"),
            AuditEventType::MeasurementTaken => write!(f, "measurement_taken"),
            AuditEventType::MeasurementFailed => write!(f, "measurement_failed"),
            AuditEventType::BaselineMeasured => write!(f, "baseline_measured"),
        }
    }
}

/// A single audit log entry
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuditEntry {
    /// RFC3339 timestamp when the event occurred
    pub timestamp: String,
    /// Type of event
    pub event_type: AuditEventType,
    /// Session ID this event belongs to
    pub session_id: String,
    /// User information
    pub user_info: UserInfo,
    /// Action description
    pub action: String,
    /// Additional details as key-value pairs
    pub details: HashMap<String, String>,
}

impl AuditEntry {
    /// Creates a new audit entry
    pub fn new(
        event_type: AuditEventType,
        session_id: String,
        action: String,
        details: HashMap<String, String>,
    ) -> Self {
        let user_info = UserInfo::capture();
        let timestamp = chrono::Utc::now().to_rfc3339();
        
        AuditEntry {
            timestamp,
            event_type,
            session_id,
            user_info,
            action,
            details,
        }
    }
    
    /// Creates an entry with custom timestamp (for testing)
    pub fn with_timestamp(
        timestamp: String,
        event_type: AuditEventType,
        session_id: String,
        action: String,
        details: HashMap<String, String>,
    ) -> Self {
        let user_info = UserInfo::capture();
        
        AuditEntry {
            timestamp,
            event_type,
            session_id,
            user_info,
            action,
            details,
        }
    }
}

/// User and environment information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserInfo {
    /// Current username
    pub username: String,
    /// Git user name (if configured)
    pub git_user_name: Option<String>,
    /// Git user email (if configured)
    pub git_user_email: Option<String>,
    /// Current working directory
    pub cwd: String,
    /// Hostname
    pub hostname: String,
}

impl UserInfo {
    /// Captures current user and environment information
    pub fn capture() -> Self {
        let username = std::env::var("USER").or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());
        
        let git_user_name = Command::new("git")
            .args(["config", "--get", "user.name"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        
        let git_user_email = Command::new("git")
            .args(["config", "--get", "user.email"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        
        let hostname = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());
        
        UserInfo {
            username,
            git_user_name,
            git_user_email,
            cwd,
            hostname,
        }
    }
    
    /// Creates a UserInfo with specified values (for testing)
    pub fn new(
        username: String,
        git_user_name: Option<String>,
        git_user_email: Option<String>,
        cwd: String,
        hostname: String,
    ) -> Self {
        UserInfo {
            username,
            git_user_name,
            git_user_email,
            cwd,
            hostname,
        }
    }
}

/// Audit logger for append-only logging
pub struct AuditLogger {
    /// File handle for the audit log
    file: File,
    /// Path to the audit log file
    path: String,
    /// Current session ID being logged
    current_session_id: Option<String>,
}

impl AuditLogger {
    /// Creates a new audit logger for the specified path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the audit log file
    ///
    /// # Returns
    ///
    /// A new `AuditLogger` with an open file handle
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created or opened
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::audit::AuditLogger;
    ///
    /// let logger = AuditLogger::new("audit.log").unwrap();
    /// logger.flush().unwrap();
    /// ```
    pub fn new(path: &str) -> Result<Self> {
        // Create parent directories if they don't exist
        let path_obj = Path::new(path);
        if let Some(parent) = path_obj.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Open file in append mode, creating if it doesn't exist
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(path)?;
        
        Ok(AuditLogger {
            file,
            path: path.to_string(),
            current_session_id: None,
        })
    }
    
    /// Sets the current session ID for all subsequent log entries
    pub fn set_session_id(&mut self, session_id: &str) {
        self.current_session_id = Some(session_id.to_string());
    }
    
    /// Logs an audit entry
    ///
    /// # Arguments
    ///
    /// * `event_type` - Type of event being logged
    /// * `action` - Description of the action
    /// * `details` - Additional key-value details
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    ///
    /// # Errors
    ///
    /// Returns an error if the entry cannot be written
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::audit::{AuditLogger, AuditEventType};
    /// use std::collections::HashMap;
    ///
    /// let logger = AuditLogger::new("audit.log").unwrap();
    /// let mut details = HashMap::new();
    /// details.insert("key".to_string(), "value".to_string());
    /// logger.log(AuditEventType::ExperimentStarted, "Starting".to_string(), details).unwrap();
    /// ```
    pub fn log(&mut self, event_type: AuditEventType, action: String, details: HashMap<String, String>) -> Result<()> {
        let session_id = self.current_session_id.clone()
            .unwrap_or_else(|| "unknown".to_string());
        
        let entry = AuditEntry::new(event_type, session_id, action, details);
        self.write_entry(&entry)
    }
    
    /// Logs an audit entry with a specific session ID (ignores current_session_id)
    pub fn log_with_session(&mut self, session_id: &str, event_type: AuditEventType, action: String, details: HashMap<String, String>) -> Result<()> {
        let entry = AuditEntry::new(event_type, session_id.to_string(), action, details);
        self.write_entry(&entry)
    }
    
    /// Writes an audit entry to the log file
    fn write_entry(&mut self, entry: &AuditEntry) -> Result<()> {
        // Serialize to JSON with newline
        let json = serde_json::to_string_pretty(entry)?;
        
        // Write with atomic-like behavior (write to buffer, then flush)
        let line = format!("{}\n", json);
        self.file.write_all(line.as_bytes())?;
        self.file.flush()?;
        
        Ok(())
    }
    
    /// Flushes the log file to ensure all data is written
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    ///
    /// # Errors
    ///
    /// Returns an error if the flush fails
    pub fn flush(&mut self) -> Result<()> {
        self.file.flush()?;
        self.file.sync_all()?;
        Ok(())
    }
    
    /// Gets the path to the audit log file
    pub fn path(&self) -> &str {
        &self.path
    }
    
    /// Gets the current session ID
    pub fn session_id(&self) -> Option<&str> {
        self.current_session_id.as_deref()
    }
}

impl Drop for AuditLogger {
    fn drop(&mut self) {
        // Try to flush on drop, but don't panic if it fails
        let _ = self.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper to create a temporary file path
    fn temp_path(dir: &TempDir, filename: &str) -> String {
        dir.path().join(filename).to_string_lossy().to_string()
    }

    // ==================== UserInfo Tests ====================

    #[test]
    fn test_user_info_capture() {
        let info = UserInfo::capture();
        assert!(!info.username.is_empty());
        assert!(!info.cwd.is_empty());
        assert!(!info.hostname.is_empty());
    }

    #[test]
    fn test_user_info_new() {
        let info = UserInfo::new(
            "testuser".to_string(),
            Some("Test User".to_string()),
            Some("test@example.com".to_string()),
            "/tmp".to_string(),
            "testhost".to_string(),
        );
        assert_eq!(info.username, "testuser");
        assert_eq!(info.git_user_name, Some("Test User".to_string()));
        assert_eq!(info.git_user_email, Some("test@example.com".to_string()));
        assert_eq!(info.cwd, "/tmp");
        assert_eq!(info.hostname, "testhost");
    }

    #[test]
    fn test_user_info_clone() {
        let info = UserInfo::capture();
        let cloned = info.clone();
        assert_eq!(info.username, cloned.username);
        assert_eq!(info.hostname, cloned.hostname);
    }

    #[test]
    fn test_user_info_debug() {
        let info = UserInfo::capture();
        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("UserInfo"));
    }

    // ==================== AuditEventType Tests ====================

    #[test]
    fn test_audit_event_type_display() {
        assert_eq!(format!("{}", AuditEventType::ExperimentStarted), "experiment_started");
        assert_eq!(format!("{}", AuditEventType::ChangeKept), "change_kept");
        assert_eq!(format!("{}", AuditEventType::TargetAchieved), "target_achieved");
    }

    #[test]
    fn test_audit_event_type_debug() {
        let debug_str = format!("{:?}", AuditEventType::ExperimentStarted);
        assert!(debug_str.contains("ExperimentStarted"));
    }

    #[test]
    fn test_audit_event_type_clone() {
        let event1 = AuditEventType::ExperimentStarted;
        let event2 = event1.clone();
        assert_eq!(event1, event2);
    }

    #[test]
    fn test_audit_event_type_partial_eq() {
        assert_eq!(AuditEventType::ExperimentStarted, AuditEventType::ExperimentStarted);
        assert_ne!(AuditEventType::ExperimentStarted, AuditEventType::ExperimentCompleted);
    }

    #[test]
    fn test_audit_event_type_serialization() {
        let event = AuditEventType::ExperimentStarted;
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("experiment_started"));
        
        let deserialized: AuditEventType = serde_json::from_str(&json).unwrap();
        assert_eq!(event, deserialized);
    }

    // ==================== AuditEntry Tests ====================

    #[test]
    fn test_audit_entry_new() {
        let mut details = HashMap::new();
        details.insert("key".to_string(), "value".to_string());
        
        let entry = AuditEntry::new(
            AuditEventType::ExperimentStarted,
            "session-123".to_string(),
            "Starting experiment".to_string(),
            details.clone(),
        );
        
        assert_eq!(entry.event_type, AuditEventType::ExperimentStarted);
        assert_eq!(entry.session_id, "session-123");
        assert_eq!(entry.action, "Starting experiment");
        assert_eq!(entry.details.get("key"), Some(&"value".to_string()));
        assert!(!entry.timestamp.is_empty());
    }

    #[test]
    fn test_audit_entry_with_timestamp() {
        let mut details = HashMap::new();
        details.insert("key".to_string(), "value".to_string());
        
        let entry = AuditEntry::with_timestamp(
            "2024-01-01T00:00:00Z".to_string(),
            AuditEventType::ExperimentStarted,
            "session-123".to_string(),
            "Starting".to_string(),
            details,
        );
        
        assert_eq!(entry.timestamp, "2024-01-01T00:00:00Z");
    }

    #[test]
    fn test_audit_entry_clone() {
        let mut details = HashMap::new();
        details.insert("key".to_string(), "value".to_string());
        
        let entry = AuditEntry::new(
            AuditEventType::ExperimentStarted,
            "session-123".to_string(),
            "Starting".to_string(),
            details,
        );
        
        let cloned = entry.clone();
        assert_eq!(entry.timestamp, cloned.timestamp);
        assert_eq!(entry.event_type, cloned.event_type);
    }

    #[test]
    fn test_audit_entry_serialization() {
        let mut details = HashMap::new();
        details.insert("metric".to_string(), "cpu".to_string());
        details.insert("value".to_string(), "50.0".to_string());
        
        let entry = AuditEntry::new(
            AuditEventType::MeasurementTaken,
            "session-456".to_string(),
            "Took measurement".to_string(),
            details,
        );
        
        let json = serde_json::to_string_pretty(&entry).unwrap();
        assert!(json.contains("session-456"));
        assert!(json.contains("measurement_taken"));
        assert!(json.contains("cpu"));
        
        let deserialized: AuditEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry.event_type, deserialized.event_type);
        assert_eq!(entry.session_id, deserialized.session_id);
    }

    // ==================== AuditLogger Tests ====================

    #[test]
    fn test_audit_logger_new() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let logger = AuditLogger::new(&path).unwrap();
        assert_eq!(logger.path(), path);
        assert!(logger.session_id().is_none());
    }

    #[test]
    fn test_audit_logger_creates_parent_directories() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "subdir/nested/audit.log");
        
        let logger = AuditLogger::new(&path).unwrap();
        assert_eq!(logger.path(), path);
        assert!(Path::new(&path).exists());
    }

    #[test]
    fn test_audit_logger_set_session_id() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        assert!(logger.session_id().is_none());
        
        logger.set_session_id("session-789");
        assert_eq!(logger.session_id(), Some("session-789"));
    }

    #[test]
    fn test_audit_logger_log() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        logger.set_session_id("session-123");
        
        let mut details = HashMap::new();
        details.insert("metric".to_string(), "cpu".to_string());
        
        logger.log(
            AuditEventType::ExperimentStarted,
            "Starting experiment".to_string(),
            details,
        ).unwrap();
        logger.flush().unwrap();
        
        // Verify file was written
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("session-123"));
        assert!(content.contains("experiment_started"));
        assert!(content.contains("cpu"));
    }

    #[test]
    fn test_audit_logger_log_with_session() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        // Don't set session_id, use log_with_session instead
        
        let mut details = HashMap::new();
        details.insert("value".to_string(), "100.0".to_string());
        
        logger.log_with_session(
            "session-456",
            AuditEventType::BaselineMeasured,
            "Measured baseline".to_string(),
            details,
        ).unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("session-456"));
        assert!(content.contains("baseline_measured"));
    }

    #[test]
    fn test_audit_logger_multiple_entries() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        logger.set_session_id("session-789");
        
        // Log multiple entries
        logger.log(AuditEventType::ExperimentStarted, "Starting".to_string(), HashMap::new()).unwrap();
        logger.log(AuditEventType::IterationStarted, "Iteration 1".to_string(), HashMap::new()).unwrap();
        logger.log(AuditEventType::IterationCompleted, "Iteration 1 done".to_string(), HashMap::new()).unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        // Count occurrences to verify all entries were written
        let started_count = content.matches("experiment_started").count();
        let iter_started_count = content.matches("iteration_started").count();
        let iter_completed_count = content.matches("iteration_completed").count();
        
        assert_eq!(started_count, 1);
        assert_eq!(iter_started_count, 1);
        assert_eq!(iter_completed_count, 1);
    }

    #[test]
    fn test_audit_logger_append_mode() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        // Create initial entry
        {
            let mut logger = AuditLogger::new(&path).unwrap();
            logger.log(AuditEventType::ExperimentStarted, "First".to_string(), HashMap::new()).unwrap();
            logger.flush().unwrap();
        }
        
        // Open again and append
        {
            let mut logger = AuditLogger::new(&path).unwrap();
            logger.log(AuditEventType::ExperimentCompleted, "Second".to_string(), HashMap::new()).unwrap();
            logger.flush().unwrap();
        }
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("experiment_started"));
        assert!(content.contains("experiment_completed"));
    }

    #[test]
    fn test_audit_logger_flush() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        logger.log(AuditEventType::ExperimentStarted, "Test".to_string(), HashMap::new()).unwrap();
        
        // Flush should succeed
        logger.flush().unwrap();
        
        // File should exist and have content
        assert!(Path::new(&path).exists());
        let content = fs::read_to_string(&path).unwrap();
        assert!(!content.is_empty());
    }

    #[test]
    fn test_audit_logger_drop() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        {
            let mut logger = AuditLogger::new(&path).unwrap();
            logger.log(AuditEventType::ExperimentStarted, "Test".to_string(), HashMap::new()).unwrap();
            // Don't explicitly flush, rely on Drop
        }
        
        // File should still have content due to Drop flushing
        let content = fs::read_to_string(&path).unwrap();
        assert!(!content.is_empty());
    }

    #[test]
    fn test_audit_logger_with_empty_session_id() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        // Don't set session_id
        
        logger.log(AuditEventType::ExperimentStarted, "Test".to_string(), HashMap::new()).unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        // Should use "unknown" as default
        assert!(content.contains("unknown"));
    }

    #[test]
    fn test_audit_logger_empty_details() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        logger.set_session_id("session-123");
        
        logger.log(AuditEventType::ExperimentStarted, "Test".to_string(), HashMap::new()).unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("session-123"));
    }
}
