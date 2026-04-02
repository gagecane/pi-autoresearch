use std::process::Command;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::session::ExperimentSession;
use crate::stuck_detector::StuckReason;

/// Result of experiment finalization
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

/// Report for failed experiments
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

/// Finalization executor
pub struct FinalizationExecutor;

impl FinalizationExecutor {
    pub fn new() -> Self {
        Self
    }

    /// Finalize an experiment and create merge PR if successful
    pub fn finalize(
        &self,
        session: &ExperimentSession,
        target_improvement: f64,
        stuck_reason: Option<&StuckReason>,
    ) -> anyhow::Result<FinalizationResult> {
        let baseline = session.baseline_record.value;
        let best_kept_value: Option<f64> = session
            .iterations
            .iter()
            .filter(|i| i.kept)
            .map(|i| i.metric_value)
            .fold(None, |a, b| Some(a.map(|min| min.min(b)).unwrap_or(b)));

        let final_improvement = match best_kept_value {
            Some(val) => (baseline - val) / baseline,
            None => 0.0,
        };

        let success = final_improvement >= target_improvement;

        if !success {
            let recommendations = self.generate_failure_recommendations(
                session,
                final_improvement,
                target_improvement,
                stuck_reason,
            );

            let failure_report = FailureReport {
                best_improvement: final_improvement,
                best_value: best_kept_value,
                baseline,
                target_improvement,
                iterations_completed: session.iterations.len(),
                stuck_reason: stuck_reason.map(|r| r.to_string()),
                recommendations,
            };

            return Ok(FinalizationResult {
                success: false,
                final_improvement,
                best_value: best_kept_value,
                branch_name: None,
                commit_message: None,
                key_changes: Vec::new(),
                error_message: None,
                failure_report: Some(failure_report),
            });
        }

        self.create_merge_branch(session, final_improvement, best_kept_value)
    }

    fn generate_failure_recommendations(
        &self,
        session: &ExperimentSession,
        final_improvement: f64,
        target_improvement: f64,
        stuck_reason: Option<&StuckReason>,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        let improvement_achieved = final_improvement * 100.0;
        let target_achieved = target_improvement * 100.0;
        let gap = target_achieved - improvement_achieved;

        recommendations.push(format!(
            "Gap Analysis: Achieved {:+.1}% improvement, target was {:+.1}% (gap: {:.1}%)",
            improvement_achieved, target_achieved, gap
        ));

        match stuck_reason {
            Some(StuckReason::StallLimitReached) => {
                recommendations.push(
                    "Consider trying a different optimization strategy or approach".to_string(),
                );
                recommendations.push(
                    "Try relaxing constraints or exploring a broader solution space".to_string(),
                );
            }
            Some(StuckReason::MaxIterationsReached) => {
                recommendations.push(format!(
                    "Increase --max-iterations beyond {} to allow more exploration",
                    session.iterations.len()
                ));
                recommendations.push(
                    "The experiment showed progress but needs more iterations".to_string(),
                );
            }
            Some(StuckReason::TotalTimeout) => {
                recommendations.push(
                    "Increase --total-timeout-minutes to allow longer runtime".to_string(),
                );
                recommendations.push(
                    "Consider optimizing the measurement command for faster feedback".to_string(),
                );
            }
            Some(StuckReason::IterationTimeout) => {
                recommendations.push(
                    "Increase --iteration-timeout-minutes for slower iterations".to_string(),
                );
                recommendations.push(
                    "Simplify the agent task or break into smaller steps".to_string(),
                );
            }
            Some(StuckReason::ConvergenceAchieved) => {
                recommendations.push(
                    "The metric has converged - consider using a different metric".to_string(),
                );
                recommendations.push(
                    "Try a more aggressive target or different optimization direction".to_string(),
                );
            }
            None => {
                recommendations.push(
                    "Review iteration logs for patterns in successful vs failed changes".to_string(),
                );
            }
        }

        if final_improvement > 0.0 && final_improvement < target_improvement {
            recommendations.push(format!(
                "Partial progress achieved ({:+.1}%) - consider lowering target or combining with manual optimizations",
                improvement_achieved
            ));
        }

        if final_improvement <= 0.0 {
            recommendations.push(
                "No improvement achieved - reconsider the metric or measurement methodology".to_string(),
            );
            recommendations.push(
                "Verify the baseline measurement is accurate and reproducible".to_string(),
            );
        }

        recommendations.push(format!(
            "Retry with adjusted parameters: current metric '{}' may need refinement",
            session.design.metric
        ));

        recommendations
    }

    fn create_merge_branch(
        &self,
        session: &ExperimentSession,
        final_improvement: f64,
        best_value: Option<f64>,
    ) -> anyhow::Result<FinalizationResult> {
        let baseline = session.baseline_record.value;
        let best_val = best_value.unwrap_or(baseline);

        let timestamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
        let branch_name = format!("autoresearch/{}-{}", timestamp, generate_uuid());

        let runtime_secs = self.calculate_runtime(session)?;
        let iterations_count = session.iterations.len();

        let key_changes = self.extract_key_changes(session, baseline);
        let convergence_status = match &session.status {
            s if s.contains("completed") => "achieved",
            _ => "not achieved",
        };

        let commit_message = self.generate_commit_message(
            session,
            final_improvement,
            baseline,
            best_val,
            iterations_count,
            runtime_secs,
            convergence_status,
            &key_changes,
        );

        // Store current branch
        let current_branch = self.get_current_branch()?;

        let mut result = FinalizationResult {
            success: true,
            final_improvement,
            best_value: Some(best_val),
            branch_name: Some(branch_name.clone()),
            commit_message: Some(commit_message.clone()),
            key_changes,
            error_message: None,
            failure_report: None,
        };

        // Create and checkout branch
        if !self.create_branch(&branch_name)? {
            result.error_message = Some("Failed to create git branch".to_string());
            result.success = false;
            result.branch_name = None;
            let _ = self.checkout_branch(&current_branch);
            return Ok(result);
        }

        // Stage changes
        if !self.stage_changes()? {
            result.error_message = Some("Failed to stage changes".to_string());
            result.success = false;
            let _ = self.checkout_branch(&current_branch);
            return Ok(result);
        }

        // Commit
        if !self.commit(&commit_message)? {
            result.error_message = Some("Failed to commit".to_string());
            result.success = false;
            let _ = self.checkout_branch(&current_branch);
            return Ok(result);
        }

        // Push if remote exists
        if self.has_remote()? {
            if !self.push_branch(&branch_name)? {
                result.error_message = Some("Failed to push branch".to_string());
                result.success = false;
                let _ = self.checkout_branch(&current_branch);
                return Ok(result);
            }
        }

        // Return to original branch
        let _ = self.checkout_branch(&current_branch);

        Ok(result)
    }

    fn calculate_runtime(&self, session: &ExperimentSession) -> anyhow::Result<i64> {
        if let Ok(start) = session.start_time.parse::<chrono::DateTime<chrono::Utc>>() {
            if let Some(ref end_str) = session.end_time {
                if let Ok(end) = end_str.parse::<chrono::DateTime<chrono::Utc>>() {
                    return Ok(end.signed_duration_since(start).num_seconds());
                }
            }
        }
        Ok(0)
    }

    fn extract_key_changes(&self, session: &ExperimentSession, baseline: f64) -> Vec<String> {
        let mut key_changes = Vec::new();

        if let Some(best_iter_num) = session.best_iteration {
            if let Some(best_iter) = session.iterations.iter().find(|i| i.iteration == best_iter_num)
            {
                let change_summary = best_iter
                    .agent_action
                    .split(':')
                    .last()
                    .unwrap_or(&best_iter.agent_action)
                    .trim()
                    .split('.')
                    .next()
                    .unwrap_or(&best_iter.agent_action)
                    .trim()
                    .to_string();
                let iter_improvement = (baseline - best_iter.metric_value) / baseline * 100.0;
                key_changes.push(format!(
                    "Iteration {}: {} (-{:.1}%)",
                    best_iter_num, change_summary, iter_improvement
                ));
            }
        }

        for iter in &session.iterations {
            if iter.kept && iter.iteration != session.best_iteration.unwrap_or(0) {
                let change_summary = iter
                    .agent_action
                    .split(':')
                    .last()
                    .unwrap_or(&iter.agent_action)
                    .trim()
                    .split('.')
                    .next()
                    .unwrap_or(&iter.agent_action)
                    .trim()
                    .to_string();
                let iter_improvement = (baseline - iter.metric_value) / baseline * 100.0;
                key_changes.push(format!(
                    "Iteration {}: {} (-{:.1}%)",
                    iter.iteration, change_summary, iter_improvement
                ));
            }
        }

        key_changes
    }

    fn generate_commit_message(
        &self,
        session: &ExperimentSession,
        final_improvement: f64,
        baseline: f64,
        best_value: f64,
        iterations_count: usize,
        runtime_secs: i64,
        convergence_status: &str,
        key_changes: &[String],
    ) -> String {
        let mut msg = format!(
            "[autoresearch] Reduce {} by {:.1}% ({:.0} → {:.0})\n\n",
            session.design.metric,
            final_improvement * 100.0,
            baseline,
            best_value
        );

        msg.push_str(&format!(
            "Iterations: {}/{} | Runtime: {}s | Convergence: {}\n\n",
            iterations_count,
            session.iterations.len().max(20),
            runtime_secs,
            convergence_status
        ));

        msg.push_str(&format!("Key changes:\n{}\n\n", key_changes.join("\n")));
        msg.push_str(&format!(
            "Metric: {} | Baseline: {:.0} | Best: {:.0}",
            session.design.metric, baseline, best_value
        ));

        msg
    }

    fn get_current_branch(&self) -> anyhow::Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()?;

        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(branch)
        } else {
            Ok("main".to_string())
        }
    }

    fn create_branch(&self, branch_name: &str) -> anyhow::Result<bool> {
        let branch_exists_output = Command::new("git")
            .args(["show-ref", "--verify", "refs/heads/", branch_name])
            .output();

        let branch_exists = branch_exists_output
            .as_ref()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let create_branch_output = if branch_exists {
            Command::new("git").args(["checkout", branch_name]).output()
        } else {
            Command::new("git").args(["checkout", "-b", branch_name]).output()
        };

        match create_branch_output {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }

    fn checkout_branch(&self, branch_name: &str) -> anyhow::Result<()> {
        let _ = Command::new("git").args(["checkout", branch_name]).output();
        Ok(())
    }

    fn stage_changes(&self) -> anyhow::Result<bool> {
        let output = Command::new("git").args(["add", "-A"]).output()?;
        Ok(output.status.success())
    }

    fn commit(&self, message: &str) -> anyhow::Result<bool> {
        let output = Command::new("git")
            .args(["commit", "--allow-empty", "-m", message])
            .output()?;
        Ok(output.status.success())
    }

    fn has_remote(&self) -> anyhow::Result<bool> {
        let output = Command::new("git")
            .args(["remote", "get-url", "origin"])
            .output()?;
        Ok(output.status.success())
    }

    fn push_branch(&self, branch_name: &str) -> anyhow::Result<bool> {
        let output = Command::new("git")
            .args(["push", "-u", "origin", branch_name])
            .output()?;
        Ok(output.status.success())
    }
}

impl Default for FinalizationExecutor {
    fn default() -> Self {
        Self::new()
    }
}

fn generate_uuid() -> String {
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
    fn test_finalization_executor_creation() {
        let executor = FinalizationExecutor::new();
        drop(executor);
    }
}
