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
pub enum SessionRecord { Baseline(BaselineRecord), Iteration(IterationRecord), Experiment(ExperimentSession) }

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
            if let Ok(session) = serde_json::from_str::<ExperimentSession>(trimmed) { records.push(SessionRecord::Experiment(session)); }
            else if let Ok(iteration) = serde_json::from_str::<IterationRecord>(trimmed) { records.push(SessionRecord::Iteration(iteration)); }
            else if let Ok(baseline) = serde_json::from_str::<BaselineRecord>(trimmed) { records.push(SessionRecord::Baseline(baseline)); }
        }
        Ok(records)
    }
    pub fn find_session(&self, session_id: &str) -> Result<Option<ExperimentSession>> {
        for record in &self.read_all()? {
            if let SessionRecord::Experiment(ref session) = record { if session.session_id == session_id { return Ok(Some(session.clone())); } }
        }
        Ok(None)
    }
    pub fn list_history(&self) -> Result<String> {
        let records = self.read_all()?;
        let experiments: Vec<&ExperimentSession> = records.iter().filter_map(|r| if let SessionRecord::Experiment(ref s) = r { Some(s) } else { None }).collect();
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
