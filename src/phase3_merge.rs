use std::process::Command;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::session::ExperimentSession;
use crate::stuck_detector::StuckReason;

#[derive(Serialize, Deserialize, Debug)]
pub struct FinalizationResult {
    pub success: bool,
    pub final_improvement: f64,
    pub best_value: Option<f64>,
    pub branch_name: Option<String>,
    pub commit_message: Option<String>,
    pub key_changes: Vec<String>,
    pub error_message: Option<String>,
    pub failure_report: Option<FailureReport>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FailureReport {
    pub best_improvement: f64,
    pub best_value: Option<f64>,
    pub baseline: f64,
    pub target_improvement: f64,
    pub iterations_completed: usize,
    pub stuck_reason: Option<String>,
    pub recommendations: Vec<String>,
}

pub struct FinalizationExecutor;

impl FinalizationExecutor {
    pub fn new() -> Self { Self }
    pub fn finalize(&self, session: &ExperimentSession, target_improvement: f64, stuck_reason: Option<&StuckReason>) -> anyhow::Result<FinalizationResult> {
        let baseline = session.baseline_record.value;
        let best_kept_value: Option<f64> = session.iterations.iter().filter(|i| i.kept).map(|i| i.metric_value).fold(None, |a, b| Some(a.map(|min| min.min(b)).unwrap_or(b)));
        let final_improvement = match best_kept_value { Some(val) => (baseline - val) / baseline, None => 0.0 };
        let success = final_improvement >= target_improvement;
        if !success {
            let recommendations = self.generate_failure_recommendations(session, final_improvement, target_improvement, stuck_reason);
            return Ok(FinalizationResult {
                success: false, final_improvement, best_value: best_kept_value, branch_name: None,
                commit_message: None, key_changes: Vec::new(), error_message: None,
                failure_report: Some(FailureReport { best_improvement: final_improvement, best_value: best_kept_value, baseline, target_improvement, iterations_completed: session.iterations.len(), stuck_reason: stuck_reason.map(|r| r.to_string()), recommendations }),
            });
        }
        self.create_merge_branch(session, final_improvement, best_kept_value)
    }
    fn generate_failure_recommendations(&self, session: &ExperimentSession, final_improvement: f64, target_improvement: f64, stuck_reason: Option<&StuckReason>) -> Vec<String> {
        let mut recommendations = Vec::new();
        recommendations.push(format!("Gap Analysis: Achieved {:+.1}% improvement, target was {:+.1}%", final_improvement * 100.0, target_improvement * 100.0));
        if let Some(StuckReason::MaxIterationsReached) = stuck_reason {
            recommendations.push(format!("Increase --max-iterations beyond {} to allow more exploration", session.iterations.len()));
        }
        recommendations
    }
    fn create_merge_branch(&self, session: &ExperimentSession, final_improvement: f64, best_value: Option<f64>) -> anyhow::Result<FinalizationResult> {
        let baseline = session.baseline_record.value;
        let best_val = best_value.unwrap_or(baseline);
        let timestamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
        let branch_name = format!("autoresearch/{}-{}", timestamp, generate_uuid());
        let key_changes = self.extract_key_changes(session);
        let commit_message = format!("[autoresearch] Reduce {} by {:.1}% ({:.0} → {:.0})\n\nIterations: {} | Key changes:\n{}\n\nMetric: {} | Baseline: {:.0} | Best: {:.0}",
            session.design.metric, final_improvement * 100.0, baseline, best_val,
            session.iterations.len(), key_changes.join("\n"), session.design.metric, baseline, best_val);
        let current_branch = self.get_current_branch()?;
        let mut result = FinalizationResult {
            success: true, final_improvement, best_value: Some(best_val),
            branch_name: Some(branch_name.clone()), commit_message: Some(commit_message.clone()),
            key_changes, error_message: None, failure_report: None,
        };
        if !self.create_branch(&branch_name)? {
            result.error_message = Some("Failed to create git branch".to_string());
            result.success = false;
            result.branch_name = None;
            let _ = self.checkout_branch(&current_branch);
            return Ok(result);
        }
        if !self.stage_changes()? {
            result.error_message = Some("Failed to stage changes".to_string());
            result.success = false;
            let _ = self.checkout_branch(&current_branch);
            return Ok(result);
        }
        if !self.commit(&commit_message)? {
            result.error_message = Some("Failed to commit".to_string());
            result.success = false;
            let _ = self.checkout_branch(&current_branch);
            return Ok(result);
        }
        if self.has_remote()? && !self.push_branch(&branch_name)? {
            result.error_message = Some("Failed to push branch".to_string());
            result.success = false;
            let _ = self.checkout_branch(&current_branch);
            return Ok(result);
        }
        let _ = self.checkout_branch(&current_branch);
        Ok(result)
    }
    fn extract_key_changes(&self, session: &ExperimentSession) -> Vec<String> {
        let mut key_changes = Vec::new();
        for iter in &session.iterations {
            if iter.kept {
                key_changes.push(format!("Iteration {}: {}", iter.iteration, iter.agent_action.split('.').next().unwrap_or(&iter.agent_action).trim()));
            }
        }
        key_changes
    }
    fn get_current_branch(&self) -> anyhow::Result<String> {
        let output = Command::new("git").args(["rev-parse", "--abbrev-ref", "HEAD"]).output()?;
        if output.status.success() { Ok(String::from_utf8_lossy(&output.stdout).trim().to_string()) } else { Ok("main".to_string()) }
    }
    fn create_branch(&self, branch_name: &str) -> anyhow::Result<bool> {
        let output = Command::new("git").args(["checkout", "-b", branch_name]).output()?;
        Ok(output.status.success())
    }
    fn checkout_branch(&self, branch_name: &str) -> anyhow::Result<()> { let _ = Command::new("git").args(["checkout", branch_name]).output(); Ok(()) }
    fn stage_changes(&self) -> anyhow::Result<bool> { let output = Command::new("git").args(["add", "-A"]).output()?; Ok(output.status.success()) }
    fn commit(&self, message: &str) -> anyhow::Result<bool> { let output = Command::new("git").args(["commit", "--allow-empty", "-m", message]).output()?; Ok(output.status.success()) }
    fn has_remote(&self) -> anyhow::Result<bool> { let output = Command::new("git").args(["remote", "get-url", "origin"]).output()?; Ok(output.status.success()) }
    fn push_branch(&self, branch_name: &str) -> anyhow::Result<bool> { let output = Command::new("git").args(["push", "-u", "origin", branch_name]).output()?; Ok(output.status.success()) }
}

impl Default for FinalizationExecutor { fn default() -> Self { Self::new() } }

fn generate_uuid() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::Instant;
    let mut hasher = DefaultHasher::new();
    format!("{:?}{}", Instant::now(), std::process::id()).hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
