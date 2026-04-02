//! Session management for pi-autoresearch experiments.
//!
//! This module provides types and functions for managing experiment sessions,
//! including session creation, iteration tracking, and file persistence.
//!
//! # Examples
//!
//! ```
//! use pi_autoresearch::session::{ExperimentSession, SessionManager, generate_session_id};
//! use pi_autoresearch::phase1_design::{ExperimentDesign, BaselineRecord};
//!
//! // Create a new session
//! let session_id = generate_session_id();
//! let design = ExperimentDesign::new(
//!     "Optimize memory usage".to_string(),
//!     "memory".to_string(),
//!     "./measure-memory".to_string(),
//!     100.0,
//!     10.0,
//! );
//! let baseline = BaselineRecord::new(
//!     "2024-01-01T00:00:00Z".to_string(),
//!     "abc123".to_string(),
//!     "memory".to_string(),
//!     "./measure-memory".to_string(),
//!     100.0,
//!     vec![100.0, 101.0],
//!     0.01,
//!     true,
//! );
//! let session = ExperimentSession::new(session_id, "How can I reduce memory?".to_string(), design, baseline);
//! ```

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;

use crate::phase1_design::{BaselineRecord, ExperimentDesign};
use crate::phase2_iterate::IterationRecord;

/// Represents an experiment session that tracks the entire optimization workflow.
///
/// A session contains the experiment design, baseline measurement, all iterations,
/// and metadata about the experiment's progress and results.
///
/// # Fields
///
/// * `session_id` - Unique identifier for this session
/// * `question` - The optimization question being addressed
/// * `design` - The experiment design (hypothesis, metric, target)
/// * `baseline_record` - The initial baseline measurement
/// * `iterations` - All iteration records from the experiment
/// * `best_iteration` - Index of the best iteration (if any)
/// * `start_time` - RFC3339 timestamp when session started
/// * `end_time` - RFC3339 timestamp when session ended (if completed)
/// * `status` - Current status: "running", "completed", or "failed"
///
/// # Examples
///
/// ```
/// use pi_autoresearch::session::ExperimentSession;
/// use pi_autoresearch::phase1_design::{ExperimentDesign, BaselineRecord};
///
/// let design = ExperimentDesign::new(
///     "Reduce allocations".to_string(),
///     "memory".to_string(),
///     "./measure".to_string(),
///     100.0,
///     10.0,
/// );
/// let baseline = BaselineRecord::new(
///     "2024-01-01T00:00:00Z".to_string(),
///     "abc123".to_string(),
///     "memory".to_string(),
///     "./measure".to_string(),
///     100.0,
///     vec![100.0],
///     0.0,
///     true,
/// );
/// let session = ExperimentSession::new(
///     "session-123".to_string(),
///     "How to reduce memory?".to_string(),
///     design,
///     baseline,
/// );
/// assert_eq!(session.status, "running");
/// assert!(session.iterations.is_empty());
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExperimentSession {
    pub session_id: String,
    pub question: String,
    pub design: ExperimentDesign,
    pub baseline_record: BaselineRecord,
    pub iterations: Vec<IterationRecord>,
    pub best_iteration: Option<usize>,
    pub start_time: String,
    pub end_time: Option<String>,
    pub status: String,
}

impl ExperimentSession {
    /// Creates a new experiment session with the given parameters.
    ///
    /// The session is initialized with status "running", empty iterations,
    /// and the current time as start_time.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Unique identifier for this session
    /// * `question` - The optimization question
    /// * `design` - Experiment design with hypothesis and targets
    /// * `baseline_record` - Verified baseline measurement
    ///
    /// # Returns
    ///
    /// A new `ExperimentSession` ready to track iterations
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::session::ExperimentSession;
    /// use pi_autoresearch::phase1_design::{ExperimentDesign, BaselineRecord};
    ///
    /// let design = ExperimentDesign::new(
    ///     "Test hypothesis".to_string(),
    ///     "metric".to_string(),
    ///     "./measure".to_string(),
    ///     100.0,
    ///     10.0,
    /// );
    /// let baseline = BaselineRecord::new(
    ///     "2024-01-01T00:00:00Z".to_string(),
    ///     "abc".to_string(),
    ///     "metric".to_string(),
    ///     "./measure".to_string(),
    ///     100.0,
    ///     vec![100.0],
    ///     0.0,
    ///     true,
    /// );
    /// let session = ExperimentSession::new(
    ///     "session-1".to_string(),
    ///     "Optimize performance?".to_string(),
    ///     design,
    ///     baseline,
    /// );
    ///
    /// assert_eq!(session.session_id, "session-1");
    /// assert_eq!(session.status, "running");
    /// assert!(session.iterations.is_empty());
    /// assert!(session.end_time.is_none());
    /// ```
    pub fn new(session_id: String, question: String, design: ExperimentDesign, baseline_record: BaselineRecord) -> Self {
        Self { session_id, question, design, baseline_record, iterations: Vec::new(), best_iteration: None, start_time: chrono::Utc::now().to_rfc3339(), end_time: None, status: "running".to_string() }
    }

    /// Adds an iteration record to the session.
    ///
    /// # Arguments
    ///
    /// * `iteration` - The iteration record to add
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::session::ExperimentSession;
    /// use pi_autoresearch::phase1_design::{ExperimentDesign, BaselineRecord};
    /// use pi_autoresearch::phase2_iterate::IterationRecord;
    ///
    /// let design = ExperimentDesign::new(
    ///     "hypothesis".to_string(),
    ///     "metric".to_string(),
    ///     "./measure".to_string(),
    ///     100.0,
    ///     10.0,
    /// );
    /// let baseline = BaselineRecord::new(
    ///     "2024-01-01T00:00:00Z".to_string(),
    ///     "abc".to_string(),
    ///     "metric".to_string(),
    ///     "./measure".to_string(),
    ///     100.0,
    ///     vec![100.0],
    ///     0.0,
    ///     true,
    /// );
    /// let mut session = ExperimentSession::new(
    ///     "session-1".to_string(),
    ///     "question".to_string(),
    ///     design,
    ///     baseline,
    /// );
    ///
    /// let iteration = IterationRecord::new(1, "action".to_string(), 90.0, 10.0, true);
    /// session.add_iteration(iteration);
    ///
    /// assert_eq!(session.iterations.len(), 1);
    /// ```
    pub fn add_iteration(&mut self, iteration: IterationRecord) { self.iterations.push(iteration); }

    /// Finalizes the experiment session with the best iteration and final status.
    ///
    /// Sets the end_time to the current time and updates the status.
    ///
    /// # Arguments
    ///
    /// * `best_iteration` - Index of the best iteration (if any)
    /// * `status` - Final status: "completed" or "failed"
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::session::ExperimentSession;
    /// use pi_autoresearch::phase1_design::{ExperimentDesign, BaselineRecord};
    ///
    /// let design = ExperimentDesign::new(
    ///     "hypothesis".to_string(),
    ///     "metric".to_string(),
    ///     "./measure".to_string(),
    ///     100.0,
    ///     10.0,
    /// );
    /// let baseline = BaselineRecord::new(
    ///     "2024-01-01T00:00:00Z".to_string(),
    ///     "abc".to_string(),
    ///     "metric".to_string(),
    ///     "./measure".to_string(),
    ///     100.0,
    ///     vec![100.0],
    ///     0.0,
    ///     true,
    /// );
    /// let mut session = ExperimentSession::new(
    ///     "session-1".to_string(),
    ///     "question".to_string(),
    ///     design,
    ///     baseline,
    /// );
    ///
    /// session.finalize(Some(1), "completed".to_string());
    ///
    /// assert_eq!(session.best_iteration, Some(1));
    /// assert!(session.end_time.is_some());
    /// assert_eq!(session.status, "completed");
    /// ```
    pub fn finalize(&mut self, best_iteration: Option<usize>, status: String) { self.best_iteration = best_iteration; self.end_time = Some(chrono::Utc::now().to_rfc3339()); self.status = status; }

    /// Calculates the final improvement percentage from baseline to best iteration.
    ///
    /// Returns the improvement as a fraction (e.g., 0.2 for 20% improvement).
    /// For metrics where lower is better, positive values indicate improvement.
    ///
    /// # Returns
    ///
    /// The improvement fraction (baseline - best) / baseline, or 0.0 if no kept iterations
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::session::ExperimentSession;
    /// use pi_autoresearch::phase1_design::{ExperimentDesign, BaselineRecord};
    /// use pi_autoresearch::phase2_iterate::IterationRecord;
    ///
    /// let design = ExperimentDesign::new(
    ///     "hypothesis".to_string(),
    ///     "metric".to_string(),
    ///     "./measure".to_string(),
    ///     100.0,
    ///     10.0,
    /// );
    /// let baseline = BaselineRecord::new(
    ///     "2024-01-01T00:00:00Z".to_string(),
    ///     "abc".to_string(),
    ///     "metric".to_string(),
    ///     "./measure".to_string(),
    ///     100.0,
    ///     vec![100.0],
    ///     0.0,
    ///     true,
    /// );
    /// let mut session = ExperimentSession::new(
    ///     "session-1".to_string(),
    ///     "question".to_string(),
    ///     design,
    ///     baseline,
    /// );
    ///
    /// // Add iteration with 20% improvement (80 vs 100 baseline)
    /// let iteration = IterationRecord::new(1, "action".to_string(), 80.0, 20.0, true);
    /// session.add_iteration(iteration);
    ///
    /// let improvement = session.calculate_final_improvement();
    /// assert!((improvement - 0.2).abs() < 0.001); // 20% improvement
    /// ```
    pub fn calculate_final_improvement(&self) -> f64 {
        let baseline = self.baseline_record.value;
        match self.iterations.iter().filter(|i| i.kept).map(|i| i.metric_value).min_by(|a, b| a.total_cmp(b)) { Some(best) => (baseline - best) / baseline, None => 0.0 }
    }
}

/// Represents a record in the session file.
///
/// Session files can contain three types of records:
/// * `Baseline` - Initial baseline measurement
/// * `Iteration` - Individual iteration results
/// * `Experiment` - Complete experiment session summary
///
/// # Examples
///
/// ```
/// use pi_autoresearch::session::{SessionRecord, ExperimentSession};
/// use pi_autoresearch::phase1_design::{ExperimentDesign, BaselineRecord};
///
/// // Create a baseline record
/// let baseline = BaselineRecord::new(
///     "2024-01-01T00:00:00Z".to_string(),
///     "abc".to_string(),
///     "metric".to_string(),
///     "./measure".to_string(),
///     100.0,
///     vec![100.0],
///     0.0,
///     true,
/// );
/// let record = SessionRecord::Baseline(baseline);
///
/// match record {
///     SessionRecord::Baseline(b) => assert_eq!(b.metric, "metric"),
///     _ => panic!("Expected Baseline"),
/// }
/// ```
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum SessionRecord { Baseline(BaselineRecord), Iteration(IterationRecord), Experiment(Box<ExperimentSession>) }

/// Manages session file persistence for experiments.
///
/// The SessionManager handles reading and writing session data to disk,
/// supporting baseline records, iteration records, and complete experiment sessions.
///
/// # Examples
///
/// ```no_run
/// use pi_autoresearch::session::{SessionManager, ExperimentSession};
/// use pi_autoresearch::phase1_design::{ExperimentDesign, BaselineRecord};
///
/// let manager = SessionManager::new("experiments.jsonl".to_string());
///
/// let design = ExperimentDesign::new(
///     "hypothesis".to_string(),
///     "metric".to_string(),
///     "./measure".to_string(),
///     100.0,
///     10.0,
/// );
/// let baseline = BaselineRecord::new(
///     "2024-01-01T00:00:00Z".to_string(),
///     "abc".to_string(),
///     "metric".to_string(),
///     "./measure".to_string(),
///     100.0,
///     vec![100.0],
///     0.0,
///     true,
/// );
///
/// // Save baseline
/// manager.save_baseline(&baseline).unwrap();
///
/// // List history
/// let history = manager.list_history().unwrap();
/// println!("{}", history);
/// ```
pub struct SessionManager { session_file: String }

impl SessionManager {
    /// Creates a new SessionManager for the given session file.
    ///
    /// # Arguments
    ///
    /// * `session_file` - Path to the session file (JSONL format)
    ///
    /// # Returns
    ///
    /// A new `SessionManager` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::session::SessionManager;
    ///
    /// let manager = SessionManager::new("experiments.jsonl".to_string());
    /// ```
    pub fn new(session_file: String) -> Self { Self { session_file } }

    /// Saves a baseline record to the session file.
    ///
    /// Appends the baseline record as a JSON line to the session file.
    ///
    /// # Arguments
    ///
    /// * `record` - The baseline record to save
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the record was saved successfully
    /// * `Err` if there was a file I/O or serialization error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pi_autoresearch::session::SessionManager;
    /// use pi_autoresearch::phase1_design::BaselineRecord;
    ///
    /// let manager = SessionManager::new("experiments.jsonl".to_string());
    /// let baseline = BaselineRecord::new(
    ///     "2024-01-01T00:00:00Z".to_string(),
    ///     "abc".to_string(),
    ///     "metric".to_string(),
    ///     "./measure".to_string(),
    ///     100.0,
    ///     vec![100.0],
    ///     0.0,
    ///     true,
    /// );
    ///
    /// manager.save_baseline(&baseline).unwrap();
    /// ```
    pub fn save_baseline(&self, record: &BaselineRecord) -> Result<()> {
        let json_line = serde_json::to_string(record)?;
        self.append_line(&json_line)?;
        eprintln!("Baseline recorded to: {}", self.session_file);
        Ok(())
    }

    /// Saves an iteration record to the session file.
    ///
    /// Appends the iteration record as a JSON line to the session file.
    ///
    /// # Arguments
    ///
    /// * `record` - The iteration record to save
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the record was saved successfully
    /// * `Err` if there was a file I/O or serialization error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pi_autoresearch::session::SessionManager;
    /// use pi_autoresearch::phase2_iterate::IterationRecord;
    ///
    /// let manager = SessionManager::new("experiments.jsonl".to_string());
    /// let iteration = IterationRecord::new(1, "action".to_string(), 90.0, 10.0, true);
    ///
    /// manager.save_iteration(&iteration).unwrap();
    /// ```
    pub fn save_iteration(&self, record: &IterationRecord) -> Result<()> {
        let json_line = serde_json::to_string(record)?;
        self.append_line(&json_line)?;
        Ok(())
    }

    /// Saves a complete experiment session to the session file.
    ///
    /// Appends the session as pretty-printed JSON to the session file.
    ///
    /// # Arguments
    ///
    /// * `session` - The experiment session to save
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the session was saved successfully
    /// * `Err` if there was a file I/O or serialization error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pi_autoresearch::session::{SessionManager, ExperimentSession};
    /// use pi_autoresearch::phase1_design::{ExperimentDesign, BaselineRecord};
    ///
    /// let manager = SessionManager::new("experiments.jsonl".to_string());
    ///
    /// let design = ExperimentDesign::new(
    ///     "hypothesis".to_string(),
    ///     "metric".to_string(),
    ///     "./measure".to_string(),
    ///     100.0,
    ///     10.0,
    /// );
    /// let baseline = BaselineRecord::new(
    ///     "2024-01-01T00:00:00Z".to_string(),
    ///     "abc".to_string(),
    ///     "metric".to_string(),
    ///     "./measure".to_string(),
    ///     100.0,
    ///     vec![100.0],
    ///     0.0,
    ///     true,
    /// );
    /// let session = ExperimentSession::new(
    ///     "session-1".to_string(),
    ///     "question".to_string(),
    ///     design,
    ///     baseline,
    /// );
    ///
    /// manager.save_session(&session).unwrap();
    /// ```
    pub fn save_session(&self, session: &ExperimentSession) -> Result<()> {
        let json_line = serde_json::to_string_pretty(session)?;
        self.append_line(&json_line)?;
        Ok(())
    }

    fn append_line(&self, line: &str) -> Result<()> {
        let mut file = OpenOptions::new().create(true).append(true).open(&self.session_file)?;
        writeln!(file, "{}", line)?;
        Ok(())
    }

    /// Reads all records from the session file.
    ///
    /// Parses the session file and returns all records (baseline, iteration, and experiment).
    /// Returns an empty vector if the file doesn't exist.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<SessionRecord>)` - All records found in the file
    /// * `Err` if there was a file I/O or parsing error
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::session::SessionManager;
    /// use tempfile::NamedTempFile;
    /// use std::fs;
    ///
    /// let temp_file = NamedTempFile::new().unwrap();
    /// // Write a simple baseline record
    /// fs::write(&temp_file, r#"{"timestamp":"2024-01-01T00:00:00Z","git_commit":"abc","metric":"m","measurement_command":"c","value":100.0,"verification_runs":[],"variance":0.0,"within_threshold":true}"#).unwrap();
    ///
    /// let manager = SessionManager::new(temp_file.path().to_string_lossy().to_string());
    /// let records = manager.read_all().unwrap();
    ///
    /// assert_eq!(records.len(), 1);
    /// ```
    pub fn read_all(&self) -> Result<Vec<SessionRecord>> {
        if !std::path::Path::new(&self.session_file).exists() { return Ok(Vec::new()); }
        let contents = std::fs::read_to_string(&self.session_file)?;
        let mut records = Vec::new();
        for line in contents.lines() {
            if line.trim().is_empty() { continue; }
            let trimmed = line.trim();
            if let Ok(session) = serde_json::from_str::<ExperimentSession>(trimmed) { records.push(SessionRecord::Experiment(Box::new(session))); }
            else if let Ok(iteration) = serde_json::from_str::<IterationRecord>(trimmed) { records.push(SessionRecord::Iteration(iteration)); }
            else if let Ok(baseline) = serde_json::from_str::<BaselineRecord>(trimmed) { records.push(SessionRecord::Baseline(baseline)); }
        }
        Ok(records)
    }

    /// Finds a session by its ID.
    ///
    /// Searches all records in the session file for a matching session ID.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID to search for
    ///
    /// # Returns
    ///
    /// * `Ok(Some(ExperimentSession))` if a session with the ID was found
    /// * `Ok(None)` if no session with the ID exists
    /// * `Err` if there was a file I/O or parsing error
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::session::SessionManager;
    ///
    /// let manager = SessionManager::new("experiments.jsonl".to_string());
    ///
    /// // Search for a session (returns None if not found)
    /// let session = manager.find_session("nonexistent-id").unwrap();
    /// assert!(session.is_none());
    /// ```
    pub fn find_session(&self, session_id: &str) -> Result<Option<ExperimentSession>> {
        for record in &self.read_all()? {
            if let SessionRecord::Experiment(ref session) = record { if session.session_id == session_id { return Ok(Some(*session.clone())); } }
        }
        Ok(None)
    }

    /// Lists all experiment sessions in the session file with summary information.
    ///
    /// Returns a formatted string showing all experiments with their key metrics.
    ///
    /// # Returns
    ///
    /// A formatted string containing experiment history, or a message if no experiments found
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::session::SessionManager;
    /// use tempfile::NamedTempFile;
    ///
    /// let temp_file = NamedTempFile::new().unwrap();
    /// let manager = SessionManager::new(temp_file.path().to_string_lossy().to_string());
    ///
    /// let history = manager.list_history().unwrap();
    /// assert!(history.contains("No experiments found") || history.contains("Experiment History"));
    /// ```
    pub fn list_history(&self) -> Result<String> {
        let records = self.read_all()?;
        let experiments: Vec<&ExperimentSession> = records.iter().filter_map(|r| if let SessionRecord::Experiment(ref s) = r { Some(&**s) } else { None }).collect();
        if experiments.is_empty() { return Ok(format!("No experiments found in {}", self.session_file)); }
        let mut output = String::from("\n=== Experiment History ===\n\n");
        output.push_str(&format!("Found {} experiment(s)\n\n", experiments.len()));
        for (i, session) in experiments.iter().enumerate() {
            let improvement: f64 = session.iterations.iter().filter(|it| it.kept).map(|it| it.improvement).fold(0.0, |a, b| a.max(b));
            output.push_str(&format!("[{}] Session: {}\n    Question: {}\n    Metric: {}\n    Baseline: {:.2}\n    Iterations: {}\n    Best improvement: {:+.2}%\n    Status: {}\n    Started: {}\n\n", i + 1, session.session_id, session.question, session.design.metric, session.baseline_record.value, session.iterations.len(), improvement * 100.0, session.status, session.start_time));
        }
        Ok(output)
    }
}

/// Generates a unique session ID based on the current time and process ID.
///
/// The session ID is a hexadecimal string derived from hashing the current
/// instant and process ID together.
///
/// # Returns
///
/// A unique session ID as a hexadecimal string (up to 16 characters)
///
/// # Examples
///
/// ```
/// use pi_autoresearch::session::generate_session_id;
///
/// let id = generate_session_id();
///
/// // Session ID is a hex string
/// assert!(!id.is_empty());
/// assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
///
/// // Each call generates a unique ID
/// let id2 = generate_session_id();
/// assert_ne!(id, id2);
/// ```
pub fn generate_session_id() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::Instant;
    let mut hasher = DefaultHasher::new();
    format!("{:?}{}", Instant::now(), std::process::id()).hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_baseline() -> BaselineRecord {
        BaselineRecord::new(
            "2024-01-01T00:00:00Z".to_string(),
            "abc123".to_string(),
            "test-metric".to_string(),
            "test-command".to_string(),
            100.0,
            vec![100.0, 101.0],
            0.01,
            true,
        )
    }

    fn create_test_design() -> ExperimentDesign {
        ExperimentDesign::new(
            "test hypothesis".to_string(),
            "test-metric".to_string(),
            "test-command".to_string(),
            100.0,
            10.0,
        )
    }

    fn create_test_iteration() -> IterationRecord {
        IterationRecord::new(
            1,
            "test-action".to_string(),
            90.0,
            10.0,
            true,
        )
    }

    // ExperimentSession tests
    #[test]
    fn test_experiment_session_new() {
        let session_id = "test-session".to_string();
        let question = "test question".to_string();
        let design = create_test_design();
        let baseline = create_test_baseline();

        let session = ExperimentSession::new(session_id.clone(), question.clone(), design, baseline);

        assert_eq!(session.session_id, session_id);
        assert_eq!(session.question, question);
        assert_eq!(session.status, "running");
        assert!(session.iterations.is_empty());
        assert!(session.end_time.is_none());
    }

    #[test]
    fn test_experiment_session_add_iteration() {
        let session = ExperimentSession::new(
            "test".to_string(),
            "question".to_string(),
            create_test_design(),
            create_test_baseline(),
        );

        let mut session = session;
        let iteration = create_test_iteration();

        session.add_iteration(iteration);

        assert_eq!(session.iterations.len(), 1);
    }

    #[test]
    fn test_experiment_session_finalize() {
        let mut session = ExperimentSession::new(
            "test".to_string(),
            "question".to_string(),
            create_test_design(),
            create_test_baseline(),
        );

        session.finalize(Some(1), "completed".to_string());

        assert_eq!(session.best_iteration, Some(1));
        assert!(session.end_time.is_some());
        assert_eq!(session.status, "completed");
    }

    #[test]
    fn test_experiment_session_calculate_final_improvement_with_improvement() {
        let mut session = ExperimentSession::new(
            "test".to_string(),
            "question".to_string(),
            create_test_design(),
            create_test_baseline(),
        );

        let iteration = IterationRecord::new(
            1,
            "cmd".to_string(),
            80.0, // 20% improvement from baseline of 100.0
            20.0,
            true,
        );
        session.add_iteration(iteration);

        let improvement = session.calculate_final_improvement();
        assert!((improvement - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_experiment_session_calculate_final_improvement_no_iterations() {
        let session = ExperimentSession::new(
            "test".to_string(),
            "question".to_string(),
            create_test_design(),
            create_test_baseline(),
        );

        let improvement = session.calculate_final_improvement();
        assert_eq!(improvement, 0.0);
    }

    #[test]
    fn test_experiment_session_clone() {
        let session = ExperimentSession::new(
            "test".to_string(),
            "question".to_string(),
            create_test_design(),
            create_test_baseline(),
        );

        let session_clone = session.clone();
        assert_eq!(session.session_id, session_clone.session_id);
    }

    #[test]
    fn test_experiment_session_debug() {
        let session = ExperimentSession::new(
            "test".to_string(),
            "question".to_string(),
            create_test_design(),
            create_test_baseline(),
        );

        let debug_str = format!("{:?}", session);
        assert!(debug_str.contains("test"));
    }

    // SessionRecord tests
    #[test]
    fn test_session_record_baseline() {
        let baseline = create_test_baseline();
        let record = SessionRecord::Baseline(baseline);

        match record {
            SessionRecord::Baseline(b) => assert_eq!(b.measurement_command, "test-command"),
            _ => panic!("Expected Baseline variant"),
        }
    }

    #[test]
    fn test_session_record_iteration() {
        let iteration = create_test_iteration();
        let record = SessionRecord::Iteration(iteration);

        match record {
            SessionRecord::Iteration(i) => assert_eq!(i.iteration, 1),
            _ => panic!("Expected Iteration variant"),
        }
    }

    #[test]
    fn test_session_record_experiment() {
        let session = ExperimentSession::new(
            "test".to_string(),
            "question".to_string(),
            create_test_design(),
            create_test_baseline(),
        );
        let record = SessionRecord::Experiment(Box::new(session));

        match record {
            SessionRecord::Experiment(s) => assert_eq!(s.session_id, "test"),
            _ => panic!("Expected Experiment variant"),
        }
    }

    // SessionManager tests
    #[test]
    fn test_session_manager_new() {
        let manager = SessionManager::new("test-session.jsonl".to_string());
        // Just verify it creates without error
        assert!(!manager.session_file.is_empty());
    }

    #[test]
    fn test_session_manager_save_baseline() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let manager = SessionManager::new(temp_file.path().to_string_lossy().to_string());
        let baseline = create_test_baseline();

        let result = manager.save_baseline(&baseline);

        assert!(result.is_ok());
        assert!(temp_file.path().exists());
    }

    #[test]
    fn test_session_manager_save_iteration() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let manager = SessionManager::new(temp_file.path().to_string_lossy().to_string());
        let iteration = create_test_iteration();

        let result = manager.save_iteration(&iteration);

        assert!(result.is_ok());
    }

    #[test]
    fn test_session_manager_save_session() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let manager = SessionManager::new(temp_file.path().to_string_lossy().to_string());
        let session = ExperimentSession::new(
            "test".to_string(),
            "question".to_string(),
            create_test_design(),
            create_test_baseline(),
        );

        let result = manager.save_session(&session);

        assert!(result.is_ok());
    }

    #[test]
    fn test_session_manager_read_all_empty_file() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let manager = SessionManager::new(temp_file.path().to_string_lossy().to_string());

        let records = manager.read_all().unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn test_session_manager_read_all_nonexistent_file() {
        let manager = SessionManager::new("/nonexistent/path/file.jsonl".to_string());

        let records = manager.read_all().unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn test_session_manager_read_all_with_data() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let baseline = create_test_baseline();
        let json_line = serde_json::to_string(&baseline).unwrap();
        std::fs::write(&temp_file, json_line).unwrap();

        let manager = SessionManager::new(temp_file.path().to_string_lossy().to_string());
        let records = manager.read_all().unwrap();

        assert_eq!(records.len(), 1);
        match &records[0] {
            SessionRecord::Baseline(b) => assert_eq!(b.measurement_command, "test-command"),
            _ => panic!("Expected Baseline record"),
        }
    }

    #[test]
    fn test_session_manager_find_session_not_found() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let manager = SessionManager::new(temp_file.path().to_string_lossy().to_string());

        let session = manager.find_session("nonexistent-id").unwrap();
        assert!(session.is_none());
    }

    #[test]
    fn test_session_manager_find_session_found() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let session = ExperimentSession::new(
            "test-id".to_string(),
            "question".to_string(),
            create_test_design(),
            create_test_baseline(),
        );
        let json_line = serde_json::to_string(&session).unwrap();
        std::fs::write(&temp_file, json_line).unwrap();

        let manager = SessionManager::new(temp_file.path().to_string_lossy().to_string());
        let found = manager.find_session("test-id").unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().session_id, "test-id");
    }

    #[test]
    fn test_session_manager_list_history_empty() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let manager = SessionManager::new(temp_file.path().to_string_lossy().to_string());

        let history = manager.list_history().unwrap();
        assert!(history.contains("No experiments found"));
    }

    #[test]
    fn test_session_manager_list_history_with_experiments() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let session = ExperimentSession::new(
            "test-id".to_string(),
            "test question".to_string(),
            create_test_design(),
            create_test_baseline(),
        );
        let json_line = serde_json::to_string(&session).unwrap();
        std::fs::write(&temp_file, json_line).unwrap();

        let manager = SessionManager::new(temp_file.path().to_string_lossy().to_string());
        let history = manager.list_history().unwrap();

        assert!(history.contains("Experiment History"));
        assert!(history.contains("test-id"));
    }

    // generate_session_id tests
    #[test]
    fn test_generate_session_id_format() {
        let id = generate_session_id();
        // Should be a hex string
        assert!(!id.is_empty());
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_session_id_uniqueness() {
        let id1 = generate_session_id();
        let id2 = generate_session_id();

        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_session_id_length() {
        let id = generate_session_id();
        // DefaultHasher produces u64, so hex string should be up to 16 chars
        assert!(id.len() <= 16);
    }
}
