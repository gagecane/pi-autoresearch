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

impl Clone for BranchManager {
    fn clone(&self) -> Self {
        BranchManager
    }
}

impl std::fmt::Debug for BranchManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BranchManager").finish()
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

    // PiAgent tests
    #[test]
    fn test_pi_agent_new() {
        let agent = PiAgent::new(true);
        assert!(agent._simulated);
        
        let agent2 = PiAgent::new(false);
        assert!(!agent2._simulated);
    }

    #[test]
    fn test_pi_agent_default() {
        let agent = PiAgent::default();
        assert!(agent._simulated);
    }

    #[test]
    fn test_pi_agent_clone() {
        let agent = PiAgent::new(true);
        let agent_clone = agent.clone();
        assert!(agent_clone._simulated);
    }

    #[test]
    fn test_pi_agent_debug() {
        let agent = PiAgent::new(true);
        let debug_str = format!("{:?}", agent);
        assert!(debug_str.contains("PiAgent"));
        assert!(debug_str.contains("_simulated"));
    }

    #[test]
    fn test_pi_agent_propose_change_basic() {
        let agent = PiAgent::new(true);
        let proposal = agent.propose_change("test question", "current state", "metric feedback");
        assert!(proposal.contains("test question"));
        assert!(proposal.contains("current state"));
        assert!(proposal.contains("metric feedback"));
        assert!(proposal.contains("Proposed change for"));
        assert!(proposal.contains("incremental optimization"));
    }

    #[test]
    fn test_pi_agent_propose_change_empty_strings() {
        let agent = PiAgent::new(true);
        let proposal = agent.propose_change("", "", "");
        assert!(proposal.contains("Proposed change for"));
        assert!(proposal.contains("Based on current state"));
        assert!(proposal.contains("metric feedback"));
    }

    #[test]
    fn test_pi_agent_propose_change_long_input() {
        let agent = PiAgent::new(true);
        let long_question = "How can I optimize the performance of this very complex system that has many different components and dependencies?";
        let long_state = "The current state is very detailed with lots of information about the system architecture and its various components.";
        let long_feedback = "The metric feedback indicates that we need to focus on reducing latency and improving throughput across all components.";
        let proposal = agent.propose_change(long_question, long_state, long_feedback);
        assert!(proposal.contains(long_question));
        assert!(proposal.contains(long_state));
        assert!(proposal.contains(long_feedback));
    }

    #[test]
    fn test_pi_agent_propose_change_special_characters() {
        let agent = PiAgent::new(true);
        let proposal = agent.propose_change("Test with 'quotes' and \"double quotes\"", "State with \n newline", "Feedback with \t tab");
        assert!(proposal.contains("quotes"));
        assert!(proposal.contains("State with"));
        assert!(proposal.contains("Feedback with"));
    }

    // BranchManager tests
    #[test]
    fn test_branch_manager_default() {
        let manager = BranchManager::default();
        let _ = manager; // Use the variable to avoid unused warning
    }

    #[test]
    fn test_branch_manager_apply_changes_in_branch() {
        let manager = BranchManager::default();
        let result = manager.apply_changes_in_branch("test action");
        assert!(result.is_ok());
        let branch_name = result.unwrap();
        assert!(branch_name.starts_with("autoresearch/iter-"));
        assert!(branch_name.len() > 18); // Has UUID after prefix
    }

    #[test]
    fn test_branch_manager_apply_changes_in_branch_unique() {
        let manager = BranchManager::default();
        let branch1 = manager.apply_changes_in_branch("action 1").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1)); // Small delay to ensure different timestamp
        let branch2 = manager.apply_changes_in_branch("action 2").unwrap();
        assert_ne!(branch1, branch2);
    }

    #[test]
    fn test_branch_manager_revert_changes() {
        let manager = BranchManager::default();
        let result = manager.revert_changes("test-branch");
        assert!(result.is_ok());
    }

    #[test]
    fn test_branch_manager_keep_changes() {
        let manager = BranchManager::default();
        let result = manager.keep_changes("test-branch");
        assert!(result.is_ok());
    }

    #[test]
    fn test_branch_manager_empty_branch_name() {
        let manager = BranchManager::default();
        let result = manager.revert_changes("");
        assert!(result.is_ok());
        
        let result = manager.keep_changes("");
        assert!(result.is_ok());
    }

    // generate_uuid tests
    #[test]
    fn test_generate_uuid_format() {
        let uuid = generate_uuid();
        assert!(!uuid.is_empty());
        assert!(uuid.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_uuid_uniqueness() {
        let uuid1 = generate_uuid();
        std::thread::sleep(std::time::Duration::from_millis(1)); // Small delay
        let uuid2 = generate_uuid();
        assert_ne!(uuid1, uuid2);
    }

    #[test]
    fn test_generate_uuid_length() {
        let uuid = generate_uuid();
        // UUID should be a hex string, typically 16 characters for u64
        assert!(uuid.len() >= 8 && uuid.len() <= 16);
    }

    #[test]
    fn test_generate_uuid_many_unique() {
        let mut uuids = std::collections::HashSet::new();
        for _ in 0..100 {
            uuids.insert(generate_uuid());
        }
        assert_eq!(uuids.len(), 100);
    }

    // Integration tests
    #[test]
    fn test_pi_agent_and_branch_manager_workflow() {
        let agent = PiAgent::new(true);
        let manager = BranchManager::default();
        
        let proposal = agent.propose_change("Optimize performance", "Current state", "Need 20% improvement");
        assert!(!proposal.is_empty());
        
        let branch = manager.apply_changes_in_branch(&proposal);
        assert!(branch.is_ok());
        
        let branch_name = branch.unwrap();
        assert!(branch_name.starts_with("autoresearch/iter-"));
        
        let keep_result = manager.keep_changes(&branch_name);
        assert!(keep_result.is_ok());
    }

    #[test]
    fn test_pi_agent_simulated_mode() {
        let simulated_agent = PiAgent::new(true);
        let real_agent = PiAgent::new(false);
        
        let simulated_proposal = simulated_agent.propose_change("test", "state", "feedback");
        let real_proposal = real_agent.propose_change("test", "state", "feedback");
        
        // Both should produce similar proposals (behavior is the same)
        assert_eq!(simulated_proposal, real_proposal);
    }

    #[test]
    fn test_branch_manager_clone_and_debug() {
        let manager1 = BranchManager::default();
        let manager2 = manager1.clone();
        let _ = manager2; // Use the variable
        
        // Check debug formatting
        let debug_str = format!("{:?}", manager1);
        assert!(debug_str.contains("BranchManager"));
    }
}
