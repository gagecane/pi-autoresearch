use anyhow::Result;
use std::process::Command;

use crate::phase1_design::ExperimentDesign;

#[derive(Debug)]
pub struct BeadsIntegration {
    enabled: bool,
    bead_id: Option<String>,
}

impl BeadsIntegration {
    pub fn new(enabled: bool) -> Self { Self { enabled, bead_id: None } }
    pub fn create_experiment_bead(&mut self, question: &str, design: &ExperimentDesign) -> Result<()> {
        if !self.enabled { return Ok(()); }
        let title = format!("[AutoResearch] {}", question);
        let description = format!("Autonomous research experiment\n\n**Hypothesis**: {}\n\n**Metric**: {}\n**Measurement**: {}\n\n**Baseline**: {:.2}\n**Target Improvement**: {:.0}%",
            design.hypothesis, design.metric, design.measurement, design.baseline, design.target_improvement * 100.0);
        let output = Command::new("bd").args(["create", "--title", &title, "--description", &description, "--type", "task", "--labels", "autoresearch,experiment"]).output()?;
        if !output.status.success() {
            eprintln!("Warning: Failed to create bead issue: {}", String::from_utf8_lossy(&output.stderr));
            self.enabled = false;
            return Ok(());
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Some(id) = stdout.lines().next().and_then(|line| line.trim().strip_prefix("Created ").or_else(|| line.trim().strip_prefix("Created: "))) {
            self.bead_id = Some(id.to_string());
            eprintln!("Created bead issue: {}", id);
        }
        Ok(())
    }
    pub fn update_bead_progress(&self, iteration: usize, metric_value: f64, improvement: f64, kept: bool) -> Result<()> {
        if !self.enabled || self.bead_id.is_none() { return Ok(()); }
        let bead_id = self.bead_id.as_ref().unwrap();
        let status = if kept { format!("✓ Kept - improvement: {:+.2}%", improvement * 100.0) } else { format!("✗ Reverted - degradation: {:-.2}%", improvement * 100.0) };
        let note = format!("Iteration {}: Metric: {:.2} - {}", iteration, metric_value, status);
        let output = Command::new("bd").args(["note", bead_id, &note]).output()?;
        if !output.status.success() { eprintln!("Warning: Failed to update bead {}: {}", bead_id, String::from_utf8_lossy(&output.stderr)); }
        Ok(())
    }
    pub fn close_bead(&self, success: bool, final_improvement: f64, iterations: usize) -> Result<()> {
        if !self.enabled || self.bead_id.is_none() { return Ok(()); }
        let bead_id = self.bead_id.as_ref().unwrap();
        let reason = if success {
            format!("Experiment successful - achieved {:+.2}% improvement in {} iterations", final_improvement * 100.0, iterations)
        } else {
            format!("Experiment did not meet target - achieved {:+.2}% improvement in {} iterations", final_improvement * 100.0, iterations)
        };
        let output = Command::new("bd").args(["close", bead_id, "--reason", &reason]).output()?;
        if !output.status.success() { eprintln!("Warning: Failed to close bead {}: {}", bead_id, String::from_utf8_lossy(&output.stderr)); }
        else { eprintln!("Closed bead issue: {}", bead_id); }
        Ok(())
    }
}

impl Default for BeadsIntegration { fn default() -> Self { Self::new(false) } }
