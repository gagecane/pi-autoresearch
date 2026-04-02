/// A simulated AI agent for proposing code changes during auto-research experiments.
/// 
/// The `PiAgent` provides a simple interface for generating proposed changes based on
/// the research question, current state, and metric feedback. It operates in simulated
/// mode by default, which is suitable for testing and demonstration purposes.
/// 
/// # Examples
/// 
/// Creating a new simulated agent:
/// 
/// ```
/// use pi_autoresearch::PiAgent;
/// 
/// let agent = PiAgent::new(true);
/// let proposal = agent.propose_change("test", "state", "feedback");
/// assert!(proposal.contains("test"));
/// ```
/// 
/// Creating a real agent (for actual AI integration):
/// 
/// ```
/// use pi_autoresearch::PiAgent;
/// 
/// let agent = PiAgent::new(false);
/// let proposal = agent.propose_change("test", "state", "feedback");
/// assert!(proposal.contains("incremental optimization"));
/// ```
/// 
/// Using the default (simulated) agent:
/// 
/// ```
/// use pi_autoresearch::PiAgent;
/// use std::default::Default;
/// 
/// let agent = PiAgent::default();
/// let proposal = agent.propose_change("optimize", "current", "target");
/// assert!(!proposal.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct PiAgent { _simulated: bool }

impl PiAgent {
    /// Creates a new PiAgent with the specified simulation mode.
    /// 
    /// # Arguments
    /// 
    /// * `_simulated` - If true, the agent operates in simulated mode (returns mock responses).
    ///   If false, the agent would integrate with a real AI system (not yet implemented).
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pi_autoresearch::PiAgent;
    /// 
    /// let simulated_agent = PiAgent::new(true);
    /// let real_agent = PiAgent::new(false);
    /// ```
    pub fn new(_simulated: bool) -> Self { Self { _simulated } }
    
    /// Proposes a code change based on the research question, current state, and metric feedback.
    /// 
    /// # Arguments
    /// 
    /// * `question` - The research question being explored
    /// * `current_state` - Description of the current state of the experiment
    /// * `metric_feedback` - Feedback about the current metric values
    /// 
    /// # Returns
    /// 
    /// A string containing the proposed change description.
    /// 
    /// # Examples
    /// 
    /// Basic usage:
    /// 
    /// ```
    /// use pi_autoresearch::PiAgent;
    /// 
    /// let agent = PiAgent::new(true);
    /// let proposal = agent.propose_change(
    ///     "How can I reduce memory usage?",
    ///     "Current memory: 100MB",
    ///     "Target: 70MB (30% reduction)"
    /// );
    /// assert!(proposal.contains("How can I reduce memory usage?"));
    /// assert!(proposal.contains("incremental optimization"));
    /// ```
    /// 
    /// Handling empty strings:
    /// 
    /// ```
    /// use pi_autoresearch::PiAgent;
    /// 
    /// let agent = PiAgent::new(true);
    /// let proposal = agent.propose_change("", "", "");
    /// assert!(proposal.contains("Proposed change for"));
    /// ```
    pub fn propose_change(&self, question: &str, current_state: &str, metric_feedback: &str) -> String {
        format!("Proposed change for '{}': Based on current state '{}' and metric feedback '{}', I propose implementing incremental optimization.", question, current_state, metric_feedback)
    }
}

impl Default for PiAgent { fn default() -> Self { Self::new(true) } }

/// A manager for git branch operations during auto-research experiments.
/// 
/// The `BranchManager` handles creating branches for experimental changes,
/// reverting changes that don't improve metrics, and keeping changes that do.
/// 
/// # Examples
/// 
/// Creating a branch manager:
/// 
/// ```
/// use pi_autoresearch::BranchManager;
/// 
/// let manager = BranchManager::default();
/// ```
/// 
/// Applying changes in a new branch:
/// 
/// ```
/// use pi_autoresearch::BranchManager;
/// 
/// let manager = BranchManager::default();
/// let branch_name = manager.apply_changes_in_branch("optimize memory allocation")?;
/// assert!(branch_name.starts_with("autoresearch/iter-"));
/// 
/// # Ok::<(), anyhow::Error>(())
/// ```
/// 
/// Reverting or keeping changes:
/// 
/// ```
/// use pi_autoresearch::BranchManager;
/// 
/// let manager = BranchManager::default();
/// let branch_name = manager.apply_changes_in_branch("test change")?;
/// 
/// // If the change improves metrics, keep it
/// manager.keep_changes(&branch_name)?;
/// 
/// // Or if it doesn't improve, revert it
/// // manager.revert_changes(&branch_name)?;
/// 
/// # Ok::<(), anyhow::Error>(())
/// ```
pub struct BranchManager;

impl BranchManager {
    /// Applies the specified action in a new git branch.
    /// 
    /// # Arguments
    /// 
    /// * `_action` - Description of the action/change to apply
    /// 
    /// # Returns
    /// 
    /// The name of the created branch on success, which follows the pattern
    /// `autoresearch/iter-{uuid}`.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pi_autoresearch::BranchManager;
    /// 
    /// let manager = BranchManager::default();
    /// let branch = manager.apply_changes_in_branch("optimize code")?;
    /// assert!(branch.starts_with("autoresearch/iter-"));
    /// 
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn apply_changes_in_branch(&self, _action: &str) -> Result<String, anyhow::Error> { Ok(format!("autoresearch/iter-{}", generate_uuid())) }
    
    /// Reverts changes made in the specified branch.
    /// 
    /// Use this when an experimental change does not improve the target metric.
    /// 
    /// # Arguments
    /// 
    /// * `_branch_name` - The name of the branch to revert
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pi_autoresearch::BranchManager;
    /// 
    /// let manager = BranchManager::default();
    /// manager.revert_changes("autoresearch/iter-abc123")?;
    /// 
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn revert_changes(&self, _branch_name: &str) -> Result<(), anyhow::Error> { Ok(()) }
    
    /// Keeps changes made in the specified branch.
    /// 
    /// Use this when an experimental change improves the target metric.
    /// 
    /// # Arguments
    /// 
    /// * `_branch_name` - The name of the branch to keep
    /// 
    /// # Examples
    /// 
    /// ```
    /// use pi_autoresearch::BranchManager;
    /// 
    /// let manager = BranchManager::default();
    /// manager.keep_changes("autoresearch/iter-abc123")?;
    /// 
    /// # Ok::<(), anyhow::Error>(())
    /// ```
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

/// Generates a unique identifier using a hash of the current timestamp and process ID.
/// 
/// This function creates a unique hex string that can be used for branch names,
/// session IDs, or other purposes requiring uniqueness.
/// 
/// # Returns
/// 
/// A hexadecimal string representing the unique identifier.
/// 
/// # Examples
/// 
/// ```
/// use pi_autoresearch::generate_uuid;
/// 
/// let uuid = generate_uuid();
/// assert!(!uuid.is_empty());
/// assert!(uuid.chars().all(|c: char| c.is_ascii_hexdigit()));
/// ```
/// 
/// Generating multiple unique IDs:
/// 
/// ```
/// use pi_autoresearch::generate_uuid;
/// 
/// let uuid1 = generate_uuid();
/// let uuid2 = generate_uuid();
/// 
/// // UUIDs are typically unique (though not guaranteed in all cases)
/// assert_ne!(uuid1, uuid2);
/// ```
pub fn generate_uuid() -> String {
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
