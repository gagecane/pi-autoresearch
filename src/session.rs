use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;

use crate::phase1_design::{BaselineRecord, ExperimentDesign};
use crate::phase2_iterate::IterationRecord;

/// Complete experiment session record
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
    pub fn new(
        session_id: String,
        question: String,
        design: ExperimentDesign,
        baseline_record: BaselineRecord,
    ) -> Self {
        Self {
            session_id,
            question,
            design,
            baseline_record,
            iterations: Vec::new(),
            best_iteration: None,
            start_time: chrono::Utc::now().to_rfc3339(),
            end_time: None,
            status: "running".to_string(),
        }
    }

    pub fn add_iteration(&mut self, iteration: IterationRecord) {
        self.iterations.push(iteration);
    }

    pub fn finalize(&mut self, best_iteration: Option<usize>, status: String) {
        self.best_iteration = best_iteration;
        self.end_time = Some(chrono::Utc::now().to_rfc3339());
        self.status = status;
    }

    pub fn get_best_metric(&self) -> Option<f64> {
        self.iterations
            .iter()
            .filter(|i| i.kept)
            .map(|i| i.metric_value)
            .min_by(|a, b| a.total_cmp(b))
    }

    pub fn calculate_final_improvement(&self) -> f64 {
        let baseline = self.baseline_record.value;
        match self.get_best_metric() {
            Some(best) => (baseline - best) / baseline,
            None => 0.0,
        }
    }
}

/// Union type for session file records
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum SessionRecord {
    Baseline(BaselineRecord),
    Iteration(IterationRecord),
    Experiment(ExperimentSession),
}

/// Session file manager for persistence
pub struct SessionManager {
    session_file: String,
}

impl SessionManager {
    pub fn new(session_file: String) -> Self {
        Self { session_file }
    }

    /// Save a baseline record to the session file
    pub fn save_baseline(&self, record: &BaselineRecord) -> Result<()> {
        let json_line = serde_json::to_string(record)?;
        self.append_line(&json_line)?;
        eprintln!("Baseline recorded to: {}", self.session_file);
        Ok(())
    }

    /// Save an iteration record to the session file
    pub fn save_iteration(&self, record: &IterationRecord) -> Result<()> {
        let json_line = serde_json::to_string(record)?;
        self.append_line(&json_line)?;
        Ok(())
    }

    /// Save a complete experiment session
    pub fn save_session(&self, session: &ExperimentSession) -> Result<()> {
        let json_line = serde_json::to_string_pretty(session)?;
        self.append_line(&json_line)?;
        Ok(())
    }

    /// Append a line to the session file
    fn append_line(&self, line: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.session_file)?;
        writeln!(file, "{}", line)?;
        Ok(())
    }

    /// Read all records from the session file
    pub fn read_all(&self) -> Result<Vec<SessionRecord>> {
        if !std::path::Path::new(&self.session_file).exists() {
            return Ok(Vec::new());
        }

        let contents = std::fs::read_to_string(&self.session_file)?;
        let mut records = Vec::new();

        for line in contents.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let trimmed = line.trim();

            // Try to parse as ExperimentSession first (has session_id field)
            if trimmed.contains("\"session_id\"") {
                if let Ok(session) = serde_json::from_str::<ExperimentSession>(trimmed) {
                    records.push(SessionRecord::Experiment(session));
                    continue;
                }
            }

            // Try to parse as IterationRecord (has iteration field)
            if trimmed.contains("\"iteration\"") && trimmed.contains("\"agent_action\"") {
                if let Ok(iteration) = serde_json::from_str::<IterationRecord>(trimmed) {
                    records.push(SessionRecord::Iteration(iteration));
                    continue;
                }
            }

            // Try to parse as BaselineRecord (has verification_runs field)
            if trimmed.contains("\"verification_runs\"") {
                if let Ok(baseline) = serde_json::from_str::<BaselineRecord>(trimmed) {
                    records.push(SessionRecord::Baseline(baseline));
                    continue;
                }
            }

            // Fallback: try each type
            if let Ok(session) = serde_json::from_str::<ExperimentSession>(trimmed) {
                records.push(SessionRecord::Experiment(session));
            } else if let Ok(iteration) = serde_json::from_str::<IterationRecord>(trimmed) {
                records.push(SessionRecord::Iteration(iteration));
            } else if let Ok(baseline) = serde_json::from_str::<BaselineRecord>(trimmed) {
                records.push(SessionRecord::Baseline(baseline));
            }
        }

        Ok(records)
    }

    /// Find a session by ID
    pub fn find_session(&self, session_id: &str) -> Result<Option<ExperimentSession>> {
        let records = self.read_all()?;

        for record in &records {
            if let SessionRecord::Experiment(ref session) = record {
                if session.session_id == session_id {
                    return Ok(Some(session.clone()));
                }
            }
        }

        Ok(None)
    }

    /// List all experiments from the session file
    pub fn list_history(&self) -> Result<String> {
        let records = self.read_all()?;

        let mut experiments: Vec<&ExperimentSession> = Vec::new();
        for record in &records {
            if let SessionRecord::Experiment(ref session) = record {
                experiments.push(session);
            }
        }

        if experiments.is_empty() {
            return Ok(format!("No experiments found in {}", self.session_file));
        }

        let mut output = String::from("\n=== Experiment History ===\n\n");
        output.push_str(&format!("Found {} experiment(s)\n\n", experiments.len()));

        for (i, session) in experiments.iter().enumerate() {
            let improvement: f64 = session
                .iterations
                .iter()
                .filter(|it| it.kept)
                .map(|it| it.improvement)
                .fold(0.0, |a, b| a.max(b));

            output.push_str(&format!(
                "[{}] Session: {}\n",
                i + 1, session.session_id
            ));
            output.push_str(&format!("    Question: {}\n", session.question));
            output.push_str(&format!("    Metric: {}\n", session.design.metric));
            output.push_str(&format!(
                "    Baseline: {:.2}\n",
                session.baseline_record.value
            ));
            output.push_str(&format!("    Iterations: {}\n", session.iterations.len()));
            output.push_str(&format!(
                "    Best improvement: {:+.2}%\n",
                improvement * 100.0
            ));
            output.push_str(&format!("    Status: {}\n", session.status));
            output.push_str(&format!("    Started: {}\n", session.start_time));
            if let Some(ref end) = session.end_time {
                output.push_str(&format!("    Ended: {}\n", end));
            }
            output.push('\n');
        }

        output.push_str("Use --resume <SESSION_ID> to continue a specific experiment");
        Ok(output)
    }
}

/// Generate a unique ID for sessions
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

    #[test]
    fn test_session_id_generation() {
        let id1 = generate_session_id();
        let id2 = generate_session_id();
        assert!(!id1.is_empty());
        assert!(!id2.is_empty());
    }

    #[test]
    fn test_session_creation() {
        let design = ExperimentDesign::new(
            "Test".to_string(),
            "metric".to_string(),
            "measure".to_string(),
            100.0,
            0.30,
        );
        let baseline = BaselineRecord::new(
            "2024-01-01".to_string(),
            "abc123".to_string(),
            "metric".to_string(),
            "measure".to_string(),
            100.0,
            vec![100.0, 100.0],
            0.0,
            true,
        );

        let session = ExperimentSession::new("test".to_string(), "Q".to_string(), design, baseline);
        assert_eq!(session.status, "running");
        assert!(session.iterations.is_empty());
    }
}
