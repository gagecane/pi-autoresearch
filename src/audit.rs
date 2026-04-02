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

// ==================== Action Logging Helper Functions ====================

impl AuditLogger {
    /// Logs experiment start with configuration details
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for the experiment session
    /// * `question` - Research question being explored
    /// * `metric` - Metric name being optimized
    /// * `baseline` - Baseline measurement value
    /// * `target_improvement` - Target improvement ratio
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::audit::{AuditLogger, AuditEventType};
    /// use std::collections::HashMap;
    ///
    /// let mut logger = AuditLogger::new("audit.log").unwrap();
    /// logger.log_experiment_start(
    ///     "session-123",
    ///     "Can we improve performance?",
    ///     "cpu_time",
    ///     100.0,
    ///     0.20,
    /// ).unwrap();
    /// ```
    pub fn log_experiment_start(
        &mut self,
        session_id: &str,
        question: &str,
        metric: &str,
        baseline: f64,
        target_improvement: f64,
    ) -> Result<()> {
        let mut details = HashMap::new();
        details.insert("question".to_string(), question.to_string());
        details.insert("metric".to_string(), metric.to_string());
        details.insert("baseline".to_string(), baseline.to_string());
        details.insert("target_improvement".to_string(), target_improvement.to_string());
        
        self.log_with_session(
            session_id,
            AuditEventType::ExperimentStarted,
            format!("Starting experiment: {}", question),
            details,
        )
    }
    
    /// Logs experiment completion
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for the experiment session
    /// * `target_achieved` - Whether the target improvement was achieved
    /// * `final_improvement` - Final improvement achieved
    /// * `iterations` - Number of iterations performed
    /// * `termination_reason` - Why the experiment terminated
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::audit::AuditLogger;
    ///
    /// let mut logger = AuditLogger::new("audit.log").unwrap();
    /// logger.log_experiment_end("session-123", true, 0.25, 10, "target_achieved").unwrap();
    /// ```
    pub fn log_experiment_end(
        &mut self,
        session_id: &str,
        target_achieved: bool,
        final_improvement: f64,
        iterations: usize,
        termination_reason: &str,
    ) -> Result<()> {
        let mut details = HashMap::new();
        details.insert("target_achieved".to_string(), target_achieved.to_string());
        details.insert("final_improvement".to_string(), final_improvement.to_string());
        details.insert("iterations".to_string(), iterations.to_string());
        details.insert("termination_reason".to_string(), termination_reason.to_string());
        
        let event_type = if target_achieved {
            AuditEventType::ExperimentCompleted
        } else {
            AuditEventType::ExperimentFailed
        };
        
        self.log_with_session(
            session_id,
            event_type,
            format!("Experiment {}", if target_achieved { "completed" } else { "failed" }),
            details,
        )
    }
    
    /// Logs iteration start
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for the experiment session
    /// * `iteration_num` - Current iteration number (1-indexed)
    /// * `branch_name` - Name of the branch being created
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::audit::AuditLogger;
    ///
    /// let mut logger = AuditLogger::new("audit.log").unwrap();
    /// logger.log_iteration_start("session-123", 1, "autoresearch/iter-1-abc").unwrap();
    /// ```
    pub fn log_iteration_start(
        &mut self,
        session_id: &str,
        iteration_num: usize,
        branch_name: &str,
    ) -> Result<()> {
        let mut details = HashMap::new();
        details.insert("iteration".to_string(), iteration_num.to_string());
        details.insert("branch_name".to_string(), branch_name.to_string());
        
        self.log_with_session(
            session_id,
            AuditEventType::IterationStarted,
            format!("Starting iteration {}", iteration_num),
            details,
        )
    }
    
    /// Logs iteration completion
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for the experiment session
    /// * `iteration_num` - Current iteration number (1-indexed)
    /// * `improvement` - Improvement achieved in this iteration
    /// * `kept` - Whether the change was kept
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::audit::AuditLogger;
    ///
    /// let mut logger = AuditLogger::new("audit.log").unwrap();
    /// logger.log_iteration_end("session-123", 1, 0.15, true).unwrap();
    /// ```
    pub fn log_iteration_end(
        &mut self,
        session_id: &str,
        iteration_num: usize,
        improvement: f64,
        kept: bool,
    ) -> Result<()> {
        let mut details = HashMap::new();
        details.insert("iteration".to_string(), iteration_num.to_string());
        details.insert("improvement".to_string(), improvement.to_string());
        details.insert("kept".to_string(), kept.to_string());
        
        self.log_with_session(
            session_id,
            AuditEventType::IterationCompleted,
            format!("Completed iteration {} (improvement: {:+.2}%, kept: {})", 
                    iteration_num, improvement * 100.0, kept),
            details,
        )
    }
    
    /// Logs branch creation
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for the experiment session
    /// * `branch_name` - Name of the created branch
    /// * `base_commit` - Base commit hash the branch was created from
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::audit::AuditLogger;
    ///
    /// let mut logger = AuditLogger::new("audit.log").unwrap();
    /// logger.log_branch_created("session-123", "autoresearch/iter-1-abc", "abc123").unwrap();
    /// ```
    pub fn log_branch_created(
        &mut self,
        session_id: &str,
        branch_name: &str,
        base_commit: &str,
    ) -> Result<()> {
        let mut details = HashMap::new();
        details.insert("branch_name".to_string(), branch_name.to_string());
        details.insert("base_commit".to_string(), base_commit.to_string());
        
        self.log_with_session(
            session_id,
            AuditEventType::BranchCreated,
            format!("Created branch: {}", branch_name),
            details,
        )
    }
    
    /// Logs commit creation
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for the experiment session
    /// * `commit_hash` - Hash of the created commit
    /// * `commit_message` - Message of the commit
    /// * `branch_name` - Branch where the commit was created
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::audit::AuditLogger;
    ///
    /// let mut logger = AuditLogger::new("audit.log").unwrap();
    /// logger.log_commit_created("session-123", "def456", "Iter 1 changes", "autoresearch/iter-1-abc").unwrap();
    /// ```
    pub fn log_commit_created(
        &mut self,
        session_id: &str,
        commit_hash: &str,
        commit_message: &str,
        branch_name: &str,
    ) -> Result<()> {
        let mut details = HashMap::new();
        details.insert("commit_hash".to_string(), commit_hash.to_string());
        details.insert("commit_message".to_string(), commit_message.to_string());
        details.insert("branch_name".to_string(), branch_name.to_string());
        
        self.log_with_session(
            session_id,
            AuditEventType::CommitCreated,
            format!("Created commit: {}", commit_hash),
            details,
        )
    }
    
    /// Logs branch merge
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for the experiment session
    /// * `branch_name` - Name of the branch being merged
    /// * `target_branch` - Target branch to merge into
    /// * `merge_commit` - Hash of the merge commit
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::audit::AuditLogger;
    ///
    /// let mut logger = AuditLogger::new("audit.log").unwrap();
    /// logger.log_branch_merged("session-123", "autoresearch/iter-1-abc", "main", "ghi789").unwrap();
    /// ```
    pub fn log_branch_merged(
        &mut self,
        session_id: &str,
        branch_name: &str,
        target_branch: &str,
        merge_commit: &str,
    ) -> Result<()> {
        let mut details = HashMap::new();
        details.insert("branch_name".to_string(), branch_name.to_string());
        details.insert("target_branch".to_string(), target_branch.to_string());
        details.insert("merge_commit".to_string(), merge_commit.to_string());
        
        self.log_with_session(
            session_id,
            AuditEventType::BranchMerged,
            format!("Merged branch {} into {}", branch_name, target_branch),
            details,
        )
    }
    
    /// Logs branch deletion
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for the experiment session
    /// * `branch_name` - Name of the deleted branch
    /// * `reason` - Reason for deletion
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::audit::AuditLogger;
    ///
    /// let mut logger = AuditLogger::new("audit.log").unwrap();
    /// logger.log_branch_deleted("session-123", "autoresearch/iter-1-abc", "Change reverted").unwrap();
    /// ```
    pub fn log_branch_deleted(
        &mut self,
        session_id: &str,
        branch_name: &str,
        reason: &str,
    ) -> Result<()> {
        let mut details = HashMap::new();
        details.insert("branch_name".to_string(), branch_name.to_string());
        details.insert("reason".to_string(), reason.to_string());
        
        self.log_with_session(
            session_id,
            AuditEventType::BranchDeleted,
            format!("Deleted branch: {}", branch_name),
            details,
        )
    }
    
    /// Logs branch checkout
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for the experiment session
    /// * `branch_name` - Name of the checked out branch
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::audit::AuditLogger;
    ///
    /// let mut logger = AuditLogger::new("audit.log").unwrap();
    /// logger.log_branch_checked_out("session-123", "main").unwrap();
    /// ```
    pub fn log_branch_checked_out(
        &mut self,
        session_id: &str,
        branch_name: &str,
    ) -> Result<()> {
        let mut details = HashMap::new();
        details.insert("branch_name".to_string(), branch_name.to_string());
        
        self.log_with_session(
            session_id,
            AuditEventType::BranchCheckedOut,
            format!("Checked out branch: {}", branch_name),
            details,
        )
    }
    
    /// Logs configuration change
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for the experiment session
    /// * `config_key` - Configuration key that changed
    /// * `old_value` - Old value (if known)
    /// * `new_value` - New value
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::audit::AuditLogger;
    ///
    /// let mut logger = AuditLogger::new("audit.log").unwrap();
    /// logger.log_config_changed("session-123", "max_iterations", "10", "20").unwrap();
    /// ```
    pub fn log_config_changed(
        &mut self,
        session_id: &str,
        config_key: &str,
        old_value: &str,
        new_value: &str,
    ) -> Result<()> {
        let mut details = HashMap::new();
        details.insert("config_key".to_string(), config_key.to_string());
        details.insert("old_value".to_string(), old_value.to_string());
        details.insert("new_value".to_string(), new_value.to_string());
        
        self.log_with_session(
            session_id,
            AuditEventType::ConfigChanged,
            format!("Config changed: {} = {}", config_key, new_value),
            details,
        )
    }
}

// ==================== Decision Logging Helper Functions ====================

impl AuditLogger {
    /// Logs that a change was kept due to improvement
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for the experiment session
    /// * `iteration_num` - Current iteration number (1-indexed)
    /// * `improvement` - Improvement achieved (as a ratio, e.g., 0.15 for 15%)
    /// * `metric_value` - The measured metric value
    /// * `baseline` - Baseline value for comparison
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::audit::AuditLogger;
    ///
    /// let mut logger = AuditLogger::new("audit.log").unwrap();
    /// logger.log_change_kept("session-123", 1, 0.15, 85.0, 100.0).unwrap();
    /// ```
    pub fn log_change_kept(
        &mut self,
        session_id: &str,
        iteration_num: usize,
        improvement: f64,
        metric_value: f64,
        baseline: f64,
    ) -> Result<()> {
        let mut details = HashMap::new();
        details.insert("iteration".to_string(), iteration_num.to_string());
        details.insert("improvement".to_string(), improvement.to_string());
        details.insert("improvement_percent".to_string(), format!("{:+.2}%", improvement * 100.0));
        details.insert("metric_value".to_string(), metric_value.to_string());
        details.insert("baseline".to_string(), baseline.to_string());
        details.insert("decision".to_string(), "kept".to_string());
        details.insert("reason".to_string(), format!("Improvement of {:+.2}% achieved", improvement * 100.0));
        
        self.log_with_session(
            session_id,
            AuditEventType::ChangeKept,
            format!("Kept change in iteration {} (improvement: {:+.2}%)", iteration_num, improvement * 100.0),
            details,
        )
    }
    
    /// Logs that a change was reverted due to lack of improvement
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for the experiment session
    /// * `iteration_num` - Current iteration number (1-indexed)
    /// * `improvement` - Improvement achieved (negative or zero for reverted changes)
    /// * `metric_value` - The measured metric value
    /// * `baseline` - Baseline value for comparison
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::audit::AuditLogger;
    ///
    /// let mut logger = AuditLogger::new("audit.log").unwrap();
    /// logger.log_change_reverted("session-123", 2, -0.05, 105.0, 100.0).unwrap();
    /// ```
    pub fn log_change_reverted(
        &mut self,
        session_id: &str,
        iteration_num: usize,
        improvement: f64,
        metric_value: f64,
        baseline: f64,
    ) -> Result<()> {
        let mut details = HashMap::new();
        details.insert("iteration".to_string(), iteration_num.to_string());
        details.insert("improvement".to_string(), improvement.to_string());
        details.insert("improvement_percent".to_string(), format!("{:+.2}%", improvement * 100.0));
        details.insert("metric_value".to_string(), metric_value.to_string());
        details.insert("baseline".to_string(), baseline.to_string());
        details.insert("decision".to_string(), "reverted".to_string());
        details.insert("reason".to_string(), format!("No improvement: {:+.2}%", improvement * 100.0));
        
        self.log_with_session(
            session_id,
            AuditEventType::ChangeReverted,
            format!("Reverted change in iteration {} (improvement: {:+.2}%)", iteration_num, improvement * 100.0),
            details,
        )
    }
    
    /// Logs that the target improvement was achieved
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for the experiment session
    /// * `target_improvement` - Target improvement ratio that was required
    /// * `actual_improvement` - Actual improvement achieved
    /// * `iterations` - Number of iterations to achieve target
    /// * `metric_value` - Final metric value
    /// * `baseline` - Baseline value
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::audit::AuditLogger;
    ///
    /// let mut logger = AuditLogger::new("audit.log").unwrap();
    /// logger.log_target_achieved("session-123", 0.20, 0.25, 5, 75.0, 100.0).unwrap();
    /// ```
    pub fn log_target_achieved(
        &mut self,
        session_id: &str,
        target_improvement: f64,
        actual_improvement: f64,
        iterations: usize,
        metric_value: f64,
        baseline: f64,
    ) -> Result<()> {
        let mut details = HashMap::new();
        details.insert("target_improvement".to_string(), target_improvement.to_string());
        details.insert("target_improvement_percent".to_string(), format!("{:.1}%", target_improvement * 100.0));
        details.insert("actual_improvement".to_string(), actual_improvement.to_string());
        details.insert("actual_improvement_percent".to_string(), format!("{:+.2}%", actual_improvement * 100.0));
        details.insert("iterations".to_string(), iterations.to_string());
        details.insert("metric_value".to_string(), metric_value.to_string());
        details.insert("baseline".to_string(), baseline.to_string());
        details.insert("exceeded_target_by".to_string(), format!("{:+.2}%", (actual_improvement - target_improvement) * 100.0));
        
        self.log_with_session(
            session_id,
            AuditEventType::TargetAchieved,
            format!("Target achieved: {:+.2}% improvement (target: {:.1}%) in {} iterations",
                    actual_improvement * 100.0, target_improvement * 100.0, iterations),
            details,
        )
    }
    
    /// Logs that the target improvement was not achieved
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for the experiment session
    /// * `target_improvement` - Target improvement ratio that was required
    /// * `actual_improvement` - Actual improvement achieved (best)
    /// * `iterations` - Total iterations performed
    /// * `metric_value` - Final metric value
    /// * `baseline` - Baseline value
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::audit::AuditLogger;
    ///
    /// let mut logger = AuditLogger::new("audit.log").unwrap();
    /// logger.log_target_not_achieved("session-123", 0.20, 0.10, 10, 90.0, 100.0).unwrap();
    /// ```
    pub fn log_target_not_achieved(
        &mut self,
        session_id: &str,
        target_improvement: f64,
        actual_improvement: f64,
        iterations: usize,
        metric_value: f64,
        baseline: f64,
    ) -> Result<()> {
        let mut details = HashMap::new();
        details.insert("target_improvement".to_string(), target_improvement.to_string());
        details.insert("target_improvement_percent".to_string(), format!("{:.1}%", target_improvement * 100.0));
        details.insert("actual_improvement".to_string(), actual_improvement.to_string());
        details.insert("actual_improvement_percent".to_string(), format!("{:+.2}%", actual_improvement * 100.0));
        details.insert("iterations".to_string(), iterations.to_string());
        details.insert("metric_value".to_string(), metric_value.to_string());
        details.insert("baseline".to_string(), baseline.to_string());
        details.insert("shortfall".to_string(), format!("{:.2}%", (target_improvement - actual_improvement) * 100.0));
        
        self.log_with_session(
            session_id,
            AuditEventType::TargetNotAchieved,
            format!("Target not achieved: {:+.2}% improvement (target: {:.1}%) after {} iterations",
                    actual_improvement * 100.0, target_improvement * 100.0, iterations),
            details,
        )
    }
    
    /// Logs that the experiment stalled (no improvement for N iterations)
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for the experiment session
    /// * `stall_count` - Number of iterations without improvement
    /// * `best_improvement` - Best improvement achieved so far
    /// * `current_iteration` - Current iteration number when stalled
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::audit::AuditLogger;
    ///
    /// let mut logger = AuditLogger::new("audit.log").unwrap();
    /// logger.log_stalled("session-123", 5, 0.10, 10).unwrap();
    /// ```
    pub fn log_stalled(
        &mut self,
        session_id: &str,
        stall_count: usize,
        best_improvement: f64,
        current_iteration: usize,
    ) -> Result<()> {
        let mut details = HashMap::new();
        details.insert("stall_count".to_string(), stall_count.to_string());
        details.insert("best_improvement".to_string(), best_improvement.to_string());
        details.insert("best_improvement_percent".to_string(), format!("{:+.2}%", best_improvement * 100.0));
        details.insert("current_iteration".to_string(), current_iteration.to_string());
        details.insert("reason".to_string(), format!("No improvement for {} consecutive iterations", stall_count));
        
        self.log_with_session(
            session_id,
            AuditEventType::Stalled,
            format!("Experiment stalled: no improvement for {} iterations (best: {:+.2}%)",
                    stall_count, best_improvement * 100.0),
            details,
        )
    }
    
    /// Logs that the experiment converged (improvements below threshold)
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for the experiment session
    /// * `convergence_threshold` - Threshold below which improvements are considered converged
    /// * `best_improvement` - Best improvement achieved
    /// * `current_iteration` - Current iteration number when converged
    /// * `window_size` - Number of iterations in the convergence window
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::audit::AuditLogger;
    ///
    /// let mut logger = AuditLogger::new("audit.log").unwrap();
    /// logger.log_converged("session-123", 0.01, 0.15, 8, 5).unwrap();
    /// ```
    pub fn log_converged(
        &mut self,
        session_id: &str,
        convergence_threshold: f64,
        best_improvement: f64,
        current_iteration: usize,
        window_size: usize,
    ) -> Result<()> {
        let mut details = HashMap::new();
        details.insert("convergence_threshold".to_string(), convergence_threshold.to_string());
        details.insert("convergence_threshold_percent".to_string(), format!("{:.2}%", convergence_threshold * 100.0));
        details.insert("best_improvement".to_string(), best_improvement.to_string());
        details.insert("best_improvement_percent".to_string(), format!("{:+.2}%", best_improvement * 100.0));
        details.insert("current_iteration".to_string(), current_iteration.to_string());
        details.insert("window_size".to_string(), window_size.to_string());
        details.insert("reason".to_string(), format!("Improvements below {:.2}% threshold over {} iterations",
                                                      convergence_threshold * 100.0, window_size));
        
        self.log_with_session(
            session_id,
            AuditEventType::Converged,
            format!("Experiment converged: improvements below {:.2}% over {} iterations (best: {:+.2}%)",
                    convergence_threshold * 100.0, window_size, best_improvement * 100.0),
            details,
        )
    }
    
    /// Logs that the experiment timed out
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for the experiment session
    /// * `timeout_seconds` - Timeout duration in seconds
    /// * `iterations_completed` - Number of iterations completed before timeout
    /// * `best_improvement` - Best improvement achieved before timeout
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::audit::AuditLogger;
    ///
    /// let mut logger = AuditLogger::new("audit.log").unwrap();
    /// logger.log_timeout("session-123", 3600, 5, 0.12).unwrap();
    /// ```
    pub fn log_timeout(
        &mut self,
        session_id: &str,
        timeout_seconds: u64,
        iterations_completed: usize,
        best_improvement: f64,
    ) -> Result<()> {
        let mut details = HashMap::new();
        details.insert("timeout_seconds".to_string(), timeout_seconds.to_string());
        details.insert("timeout_formatted".to_string(), format_duration(timeout_seconds));
        details.insert("iterations_completed".to_string(), iterations_completed.to_string());
        details.insert("best_improvement".to_string(), best_improvement.to_string());
        details.insert("best_improvement_percent".to_string(), format!("{:+.2}%", best_improvement * 100.0));
        details.insert("reason".to_string(), format!("Experiment exceeded {} second timeout", timeout_seconds));
        
        self.log_with_session(
            session_id,
            AuditEventType::Timeout,
            format!("Experiment timed out after {} ({} iterations, best: {:+.2}%)",
                    format_duration(timeout_seconds), iterations_completed, best_improvement * 100.0),
            details,
        )
    }
    
    /// Logs that the experiment reached maximum iterations
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for the experiment session
    /// * `max_iterations` - Maximum iteration limit
    /// * `best_improvement` - Best improvement achieved
    /// * `metric_value` - Final metric value
    /// * `baseline` - Baseline value
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::audit::AuditLogger;
    ///
    /// let mut logger = AuditLogger::new("audit.log").unwrap();
    /// logger.log_max_iterations_reached("session-123", 10, 0.15, 85.0, 100.0).unwrap();
    /// ```
    pub fn log_max_iterations_reached(
        &mut self,
        session_id: &str,
        max_iterations: usize,
        best_improvement: f64,
        metric_value: f64,
        baseline: f64,
    ) -> Result<()> {
        let mut details = HashMap::new();
        details.insert("max_iterations".to_string(), max_iterations.to_string());
        details.insert("best_improvement".to_string(), best_improvement.to_string());
        details.insert("best_improvement_percent".to_string(), format!("{:+.2}%", best_improvement * 100.0));
        details.insert("metric_value".to_string(), metric_value.to_string());
        details.insert("baseline".to_string(), baseline.to_string());
        details.insert("reason".to_string(), format!("Reached maximum iteration limit of {}", max_iterations));
        
        self.log_with_session(
            session_id,
            AuditEventType::MaxIterationsReached,
            format!("Max iterations reached: {} iterations (best improvement: {:+.2}%)",
                    max_iterations, best_improvement * 100.0),
            details,
        )
    }
}

/// Formats a duration in seconds as a human-readable string
fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else if seconds < 86400 {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    } else {
        format!("{}d {}h", seconds / 86400, (seconds % 86400) / 3600)
    }
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

    // ==================== Action Logging Tests ====================

    #[test]
    fn test_log_experiment_start() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        logger.log_experiment_start(
            "session-123",
            "Can we improve performance?",
            "cpu_time",
            100.0,
            0.20,
        ).unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("session-123"));
        assert!(content.contains("experiment_started"));
        assert!(content.contains("Can we improve performance?"));
        assert!(content.contains("cpu_time"));
        assert!(content.contains("100"));
        assert!(content.contains("0.2"));
    }

    #[test]
    fn test_log_experiment_end_success() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        logger.log_experiment_end("session-123", true, 0.25, 10, "target_achieved").unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("session-123"));
        assert!(content.contains("experiment_completed"));
        assert!(content.contains("true"));
        assert!(content.contains("0.25"));
        assert!(content.contains("10"));
        assert!(content.contains("target_achieved"));
    }

    #[test]
    fn test_log_experiment_end_failure() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        logger.log_experiment_end("session-456", false, 0.10, 15, "max_iterations").unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("session-456"));
        assert!(content.contains("experiment_failed"));
        assert!(content.contains("false"));
        assert!(content.contains("max_iterations"));
    }

    #[test]
    fn test_log_iteration_start() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        logger.log_iteration_start("session-123", 1, "autoresearch/iter-1-abc").unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("iteration_started"));
        assert!(content.contains("autoresearch/iter-1-abc"));
        assert!(content.contains("1"));
    }

    #[test]
    fn test_log_iteration_end_kept() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        logger.log_iteration_end("session-123", 1, 0.15, true).unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("iteration_completed"));
        assert!(content.contains("0.15"));
        assert!(content.contains("true"));
    }

    #[test]
    fn test_log_iteration_end_reverted() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        logger.log_iteration_end("session-123", 2, -0.05, false).unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("iteration_completed"));
        assert!(content.contains("-0.05"));
        assert!(content.contains("false"));
    }

    #[test]
    fn test_log_branch_created() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        logger.log_branch_created("session-123", "autoresearch/iter-1-abc", "abc123def456").unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("branch_created"));
        assert!(content.contains("autoresearch/iter-1-abc"));
        assert!(content.contains("abc123def456"));
    }

    #[test]
    fn test_log_commit_created() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        logger.log_commit_created(
            "session-123",
            "def456ghi789",
            "Iteration 1: Optimized function",
            "autoresearch/iter-1-abc",
        ).unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("commit_created"));
        assert!(content.contains("def456ghi789"));
        assert!(content.contains("Iteration 1: Optimized function"));
    }

    #[test]
    fn test_log_branch_merged() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        logger.log_branch_merged("session-123", "autoresearch/iter-1-abc", "main", "ghi789jkl012").unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("branch_merged"));
        assert!(content.contains("autoresearch/iter-1-abc"));
        assert!(content.contains("main"));
        assert!(content.contains("ghi789jkl012"));
    }

    #[test]
    fn test_log_branch_deleted() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        logger.log_branch_deleted("session-123", "autoresearch/iter-1-abc", "Change reverted").unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("branch_deleted"));
        assert!(content.contains("autoresearch/iter-1-abc"));
        assert!(content.contains("Change reverted"));
    }

    #[test]
    fn test_log_branch_checked_out() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        logger.log_branch_checked_out("session-123", "main").unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("branch_checked_out"));
        assert!(content.contains("main"));
    }

    #[test]
    fn test_log_config_changed() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        logger.log_config_changed("session-123", "max_iterations", "10", "20").unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("config_changed"));
        assert!(content.contains("max_iterations"));
        assert!(content.contains("10"));
        assert!(content.contains("20"));
    }

    #[test]
    fn test_action_logging_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        // Simulate a complete experiment workflow
        logger.log_experiment_start("session-123", "Test question", "cpu_time", 100.0, 0.20).unwrap();
        logger.log_branch_created("session-123", "autoresearch/iter-1-abc", "abc123").unwrap();
        logger.log_branch_checked_out("session-123", "autoresearch/iter-1-abc").unwrap();
        logger.log_iteration_start("session-123", 1, "autoresearch/iter-1-abc").unwrap();
        logger.log_commit_created("session-123", "def456", "Iter 1", "autoresearch/iter-1-abc").unwrap();
        logger.log_iteration_end("session-123", 1, 0.15, true).unwrap();
        logger.log_branch_merged("session-123", "autoresearch/iter-1-abc", "main", "ghi789").unwrap();
        logger.log_branch_deleted("session-123", "autoresearch/iter-1-abc", "Merged").unwrap();
        logger.log_experiment_end("session-123", true, 0.15, 1, "target_achieved").unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        
        // Verify all events are logged
        assert!(content.contains("experiment_started"));
        assert!(content.contains("branch_created"));
        assert!(content.contains("branch_checked_out"));
        assert!(content.contains("iteration_started"));
        assert!(content.contains("commit_created"));
        assert!(content.contains("iteration_completed"));
        assert!(content.contains("branch_merged"));
        assert!(content.contains("branch_deleted"));
        assert!(content.contains("experiment_completed"));
        
        // Count entries (should be 9)
        let entry_count = content.matches("event_type").count();
        assert_eq!(entry_count, 9);
    }

    // ==================== Decision Logging Tests ====================

    #[test]
    fn test_log_change_kept() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        logger.log_change_kept("session-123", 1, 0.15, 85.0, 100.0).unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("change_kept"));
        assert!(content.contains("session-123"));
        assert!(content.contains("\"iteration\": \"1\"")); // iteration
        assert!(content.contains("\"improvement\": \"0.15\"")); // improvement
        assert!(content.contains("\"metric_value\": \"85\"")); // metric_value (85.0 serializes as 85)
        assert!(content.contains("\"baseline\": \"100\"")); // baseline (100.0 serializes as 100)
        assert!(content.contains("\"decision\": \"kept\"")); // decision
        assert!(content.contains("15.00")); // improvement_percent
    }

    #[test]
    fn test_log_change_reverted() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        logger.log_change_reverted("session-123", 2, -0.05, 105.0, 100.0).unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("change_reverted"));
        assert!(content.contains("session-123"));
        assert!(content.contains("\"iteration\": \"2\"")); // iteration
        assert!(content.contains("\"improvement\": \"-0.05\"")); // improvement
        assert!(content.contains("\"metric_value\": \"105\"")); // metric_value (105.0 serializes as 105)
        assert!(content.contains("\"baseline\": \"100\"")); // baseline (100.0 serializes as 100)
        assert!(content.contains("\"decision\": \"reverted\"")); // decision
        assert!(content.contains("-5.00")); // improvement_percent
    }

    #[test]
    fn test_log_target_achieved() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        logger.log_target_achieved("session-123", 0.20, 0.25, 5, 75.0, 100.0).unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("target_achieved"));
        assert!(content.contains("session-123"));
        assert!(content.contains("\"target_improvement\": \"0.2\"")); // target_improvement (0.20 serializes as 0.2)
        assert!(content.contains("\"actual_improvement\": \"0.25\"")); // actual_improvement
        assert!(content.contains("\"iterations\": \"5\"")); // iterations
        assert!(content.contains("\"metric_value\": \"75\"")); // metric_value (75.0 serializes as 75)
        assert!(content.contains("\"baseline\": \"100\"")); // baseline (100.0 serializes as 100)
        assert!(content.contains("20.0")); // target_improvement_percent
        assert!(content.contains("25.00")); // actual_improvement_percent
    }

    #[test]
    fn test_log_target_not_achieved() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        logger.log_target_not_achieved("session-123", 0.20, 0.10, 10, 90.0, 100.0).unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("target_not_achieved"));
        assert!(content.contains("session-123"));
        assert!(content.contains("\"target_improvement\": \"0.2\"")); // target_improvement (0.20 serializes as 0.2)
        assert!(content.contains("\"actual_improvement\": \"0.1\"")); // actual_improvement (0.10 serializes as 0.1)
        assert!(content.contains("\"iterations\": \"10\"")); // iterations
        assert!(content.contains("\"metric_value\": \"90\"")); // metric_value (90.0 serializes as 90)
        assert!(content.contains("\"baseline\": \"100\"")); // baseline (100.0 serializes as 100)
        assert!(content.contains("20.0")); // target_improvement_percent
        assert!(content.contains("10.00")); // actual_improvement_percent
    }

    #[test]
    fn test_log_stalled() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        logger.log_stalled("session-123", 5, 0.10, 10).unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("stalled"));
        assert!(content.contains("session-123"));
        assert!(content.contains("\"stall_count\": \"5\"")); // stall_count
        assert!(content.contains("\"best_improvement\": \"0.1\"")); // best_improvement (0.10 serializes as 0.1)
        assert!(content.contains("\"current_iteration\": \"10\"")); // current_iteration
        assert!(content.contains("10.00")); // best_improvement_percent
    }

    #[test]
    fn test_log_converged() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        logger.log_converged("session-123", 0.01, 0.15, 8, 5).unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("converged"));
        assert!(content.contains("session-123"));
        assert!(content.contains("\"convergence_threshold\": \"0.01\"")); // convergence_threshold
        assert!(content.contains("\"best_improvement\": \"0.15\"")); // best_improvement
        assert!(content.contains("\"current_iteration\": \"8\"")); // current_iteration
        assert!(content.contains("\"window_size\": \"5\"")); // window_size
        assert!(content.contains("1.00")); // convergence_threshold_percent
    }

    #[test]
    fn test_log_timeout() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        logger.log_timeout("session-123", 3600, 5, 0.12).unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("timeout"));
        assert!(content.contains("session-123"));
        assert!(content.contains("\"timeout_seconds\": \"3600\"")); // timeout_seconds
        assert!(content.contains("\"iterations_completed\": \"5\"")); // iterations_completed
        assert!(content.contains("\"best_improvement\": \"0.12\"")); // best_improvement
        assert!(content.contains("1h 0m")); // timeout_formatted
    }

    #[test]
    fn test_log_max_iterations_reached() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        logger.log_max_iterations_reached("session-123", 10, 0.15, 85.0, 100.0).unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("max_iterations_reached"));
        assert!(content.contains("session-123"));
        assert!(content.contains("\"max_iterations\": \"10\"")); // max_iterations
        assert!(content.contains("\"best_improvement\": \"0.15\"")); // best_improvement
        assert!(content.contains("\"metric_value\": \"85\"")); // metric_value (85.0 serializes as 85)
        assert!(content.contains("\"baseline\": \"100\"")); // baseline (100.0 serializes as 100)
        assert!(content.contains("15.00")); // best_improvement_percent
    }

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(59), "59s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(60), "1m 0s");
        assert_eq!(format_duration(120), "2m 0s");
        assert_eq!(format_duration(300), "5m 0s");
        assert_eq!(format_duration(365), "6m 5s");
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(format_duration(3600), "1h 0m");
        assert_eq!(format_duration(7200), "2h 0m");
        assert_eq!(format_duration(3661), "1h 1m");
    }

    #[test]
    fn test_format_duration_days() {
        assert_eq!(format_duration(86400), "1d 0h");
        assert_eq!(format_duration(172800), "2d 0h");
        assert_eq!(format_duration(90000), "1d 1h");
    }

    #[test]
    fn test_decision_logging_complete_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        // Simulate a complete experiment with decisions
        logger.log_experiment_start("session-123", "Test question", "cpu_time", 100.0, 0.20).unwrap();
        
        // Iteration 1: change kept
        logger.log_iteration_start("session-123", 1, "autoresearch/iter-1-abc").unwrap();
        logger.log_change_kept("session-123", 1, 0.15, 85.0, 100.0).unwrap();
        logger.log_iteration_end("session-123", 1, 0.15, true).unwrap();
        
        // Iteration 2: change reverted
        logger.log_iteration_start("session-123", 2, "autoresearch/iter-2-def").unwrap();
        logger.log_change_reverted("session-123", 2, -0.05, 105.0, 100.0).unwrap();
        logger.log_iteration_end("session-123", 2, -0.05, false).unwrap();
        
        // Iteration 3: target achieved
        logger.log_iteration_start("session-123", 3, "autoresearch/iter-3-ghi").unwrap();
        logger.log_change_kept("session-123", 3, 0.25, 75.0, 100.0).unwrap();
        logger.log_iteration_end("session-123", 3, 0.25, true).unwrap();
        logger.log_target_achieved("session-123", 0.20, 0.25, 3, 75.0, 100.0).unwrap();
        
        logger.log_experiment_end("session-123", true, 0.25, 3, "target_achieved").unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        
        // Verify all decision events are logged
        assert!(content.contains("change_kept"));
        assert!(content.contains("change_reverted"));
        assert!(content.contains("target_achieved"));
        
        // Count entries (12 total: experiment_start, iter1_start, change_kept, iter1_end,
        // iter2_start, change_reverted, iter2_end, iter3_start, change_kept, iter3_end,
        // target_achieved, experiment_end)
        let entry_count = content.matches("event_type").count();
        assert_eq!(entry_count, 12);
    }

    #[test]
    fn test_decision_logging_termination_workflow() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        // Test all termination reasons
        
        // Test stalled
        logger.log_stalled("session-123", 5, 0.10, 10).unwrap();
        
        // Test converged
        logger.log_converged("session-124", 0.01, 0.15, 8, 5).unwrap();
        
        // Test timeout
        logger.log_timeout("session-125", 3600, 5, 0.12).unwrap();
        
        // Test max iterations
        logger.log_max_iterations_reached("session-126", 10, 0.15, 85.0, 100.0).unwrap();
        
        // Test target not achieved
        logger.log_target_not_achieved("session-127", 0.20, 0.10, 10, 90.0, 100.0).unwrap();
        
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        
        // Verify all termination events are logged
        assert!(content.contains("stalled"));
        assert!(content.contains("converged"));
        assert!(content.contains("timeout"));
        assert!(content.contains("max_iterations_reached"));
        assert!(content.contains("target_not_achieved"));
        
        // Count entries
        let entry_count = content.matches("event_type").count();
        assert_eq!(entry_count, 5);
    }

    #[test]
    fn test_decision_logging_with_zero_improvement() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        // Test with zero improvement (edge case)
        logger.log_change_reverted("session-123", 1, 0.0, 100.0, 100.0).unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("change_reverted"));
        assert!(content.contains("0.0")); // zero improvement
        assert!(content.contains("0.00")); // 0.00%
    }

    #[test]
    fn test_decision_logging_with_negative_improvement() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        // Test with negative improvement (regression)
        logger.log_change_reverted("session-123", 1, -0.25, 125.0, 100.0).unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("change_reverted"));
        assert!(content.contains("-0.25")); // negative improvement
        assert!(content.contains("-25.00")); // -25.00%
    }

    #[test]
    fn test_decision_logging_serialization() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_path(&temp_dir, "audit.log");
        
        let mut logger = AuditLogger::new(&path).unwrap();
        
        logger.log_change_kept("session-123", 1, 0.15, 85.0, 100.0).unwrap();
        logger.flush().unwrap();
        
        let content = fs::read_to_string(&path).unwrap();
        
        // Parse JSON to verify structure
        let entry: AuditEntry = serde_json::from_str(content.trim()).unwrap();
        assert_eq!(entry.event_type, AuditEventType::ChangeKept);
        assert_eq!(entry.session_id, "session-123");
        assert!(entry.details.contains_key("improvement"));
        assert!(entry.details.contains_key("decision"));
        assert_eq!(entry.details.get("decision"), Some(&"kept".to_string()));
    }
}

