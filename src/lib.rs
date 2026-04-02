//! Pi AutoResearch - Autonomous research experiment orchestrator
//!
//! This library provides a modular implementation of the autoresearch framework
//! inspired by karpathy's autoresearch, wrapping the PI coding agent to iteratively
//! explore solutions, measure progress, and converge on optimal results.

pub mod cli;
pub mod orchestrator;
pub mod phase1_design;
pub mod phase2_iterate;
pub mod phase3_merge;
pub mod stuck_detector;
pub mod metric_evaluator;
pub mod pi_agent;
pub mod session;
pub mod beads;
pub mod ralph_tui;

// Re-export main types for convenience
pub use cli::Cli;
pub use orchestrator::ExperimentOrchestrator;
pub use phase1_design::{ExperimentDesign, generate_design};
pub use phase2_iterate::IterationRecord;
pub use stuck_detector::{StuckReason, StuckDetector};
pub use session::ExperimentSession;
