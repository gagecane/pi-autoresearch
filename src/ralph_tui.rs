use anyhow::Result;
use serde_json::json;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub struct RalphTuiIntegration {
    enabled: bool,
    task_file: Option<String>,
    task_id: Option<String>,
}

impl RalphTuiIntegration {
    pub fn new(enabled: bool, task_file: Option<String>) -> Self {
        let task_id = task_file.as_ref().and_then(|f| Path::new(f).file_stem().and_then(|s| s.to_str()).map(|s| s.to_string()));
        Self { enabled, task_file, task_id }
    }
    pub fn read_task_from_file(&self) -> Result<Option<String>> {
        if !self.enabled || self.task_file.is_none() { return Ok(None); }
        let contents = fs::read_to_string(self.task_file.as_ref().unwrap())?;
        let mut question = None;
        let mut in_description = false;
        for line in contents.lines() {
            if line.starts_with("## Description") { in_description = true; continue; }
            if in_description {
                if line.starts_with("##") { break; }
                let trimmed = line.trim();
                if !trimmed.is_empty() { question = Some(trimmed.to_string()); break; }
            }
        }
        Ok(question)
    }
    pub fn output_ralph_status(&self, iteration: usize, metric_value: f64, improvement: f64, kept: bool) {
        if !self.enabled { return; }
        let status = if kept { "kept" } else { "reverted" };
        println!("RALPH_STATUS|iteration={}|metric={:.2}|improvement={:+.2}|status={}", iteration, metric_value, improvement * 100.0, status);
    }
    pub fn update_task_status(&self, iteration: usize, metric_value: f64, improvement: f64, kept: bool) -> Result<()> {
        if !self.enabled { return Ok(()); }
        let status_dir = ".ralph-tui";
        if !Path::new(status_dir).exists() { fs::create_dir_all(status_dir)?; }
        let status_data = json!({"taskId": self.task_id, "iteration": iteration, "metricValue": metric_value, "improvement": improvement, "kept": kept, "timestamp": chrono::Utc::now().to_rfc3339()});
        fs::write(format!("{}/status.json", status_dir), serde_json::to_string_pretty(&status_data)?)?;
        Ok(())
    }
    pub fn mark_task_complete(&self, success: bool, final_improvement: f64, iterations: usize) -> Result<()> {
        if !self.enabled { return Ok(()); }
        let status_data = json!({"taskId": self.task_id, "status": if success { "completed" } else { "failed" }, "finalImprovement": final_improvement, "iterations": iterations, "timestamp": chrono::Utc::now().to_rfc3339()});
        fs::write(".ralph-tui/status.json", serde_json::to_string_pretty(&status_data)?)?;
        Ok(())
    }
}

impl Default for RalphTuiIntegration { fn default() -> Self { Self::new(false, None) } }
