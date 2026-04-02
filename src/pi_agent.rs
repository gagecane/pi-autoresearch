#[derive(Debug, Clone)]
pub struct PiAgent { _simulated: bool }

impl PiAgent {
    pub fn new(_simulated: bool) -> Self { Self { _simulated: _simulated } }
    pub fn propose_change(&self, question: &str, current_state: &str, metric_feedback: &str) -> String {
        format!("Proposed change for '{}': Based on current state '{}' and metric feedback '{}', I propose implementing incremental optimization.", question, current_state, metric_feedback)
    }
}

impl Default for PiAgent { fn default() -> Self { Self::new(true) } }

pub struct BranchManager;

impl BranchManager {
    pub fn apply_changes_in_branch(&self, _action: &str) -> Result<String, anyhow::Error> { Ok(format!("autoresearch/iter-{}", generate_uuid())) }
    pub fn revert_changes(&self, _branch_name: &str) -> Result<(), anyhow::Error> { Ok(()) }
    pub fn keep_changes(&self, _branch_name: &str) -> Result<(), anyhow::Error> { Ok(()) }
}

impl Default for BranchManager { fn default() -> Self { Self } }

fn generate_uuid() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::Instant;
    let mut hasher = DefaultHasher::new();
    format!("{:?}{}", Instant::now(), std::process::id()).hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
