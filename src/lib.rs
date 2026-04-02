pub mod cli;
pub mod phase1_design;
pub mod phase2_iterate;
pub mod stuck_detector;
pub mod metric_evaluator;
pub mod pi_agent;
pub mod session;

pub use cli::Cli;
pub use phase1_design::{ExperimentDesign, generate_design};
pub use phase2_iterate::IterationRecord;
pub use stuck_detector::{StuckReason, StuckDetector};
pub use session::ExperimentSession;
