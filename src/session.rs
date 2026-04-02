use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;

use crate::phase1_design::{BaselineRecord, ExperimentDesign};
use crate::phase2_iterate::IterationRecord;

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
    pub fn new(session_id: String, question: String, design: ExperimentDesign, baseline_record: BaselineRecord) -> Self {
        Self { session_id, question, design, baseline_record, iterations: Vec::new(), best_iteration: None, start_time: chrono::Utc::now().to_rfc3339(), end_time: None, status: "running".to_string() }
    }
    pub fn add_iteration(&mut self, iteration: IterationRecord) { self.iterations.push(iteration); }
    pub fn finalize(&mut self, best_iteration: Option<usize>, status: String) { self.best_iteration = best_iteration; self.end_time = Some(chrono::Utc::now().to_rfc3339()); self.status = status; }
    pub fn calculate_final_improvement(&self) -> f64 {
        let baseline = self.baseline_record.value;
        match self.iterations.iter().filter(|i| i.kept).map(|i| i.metric_value).min_by(|a, b| a.total_cmp(b)) { Some(best) => (baseline - best) / baseline, None => 0.0 }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum SessionRecord { Baseline(BaselineRecord), Iteration(IterationRecord), Experiment(Box<ExperimentSession>) }

pub struct SessionManager { session_file: String }

impl SessionManager {
    pub fn new(session_file: String) -> Self { Self { session_file } }
    pub fn save_baseline(&self, record: &BaselineRecord) -> Result<()> {
        let json_line = serde_json::to_string(record)?;
        self.append_line(&json_line)?;
        eprintln!("Baseline recorded to: {}", self.session_file);
        Ok(())
    }
    pub fn save_iteration(&self, record: &IterationRecord) -> Result<()> {
        let json_line = serde_json::to_string(record)?;
        self.append_line(&json_line)?;
        Ok(())
    }
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
    pub fn find_session(&self, session_id: &str) -> Result<Option<ExperimentSession>> {
        for record in &self.read_all()? {
            if let SessionRecord::Experiment(ref session) = record { if session.session_id == session_id { return Ok(Some(*session.clone())); } }
        }
        Ok(None)
    }
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
