use clap::Parser;

/// Pi AutoResearch CLI - Autonomous research experiment orchestrator
#[derive(Parser, Debug, Clone)]
#[command(name = "pi-autoresearch")]
#[command(about = "Autonomous research experiment orchestrator")]
#[command(long_about = "\nPi AutoResearch is a CLI tool that orchestrates autonomous research experiments using a 3-phase framework inspired by karpathy's autoresearch. It wraps the PI coding agent to iteratively explore solutions, measure progress, and converge on optimal results.")]
pub struct Cli {
    /// Research question to explore
    #[arg(long)]
    pub question: Option<String>,

    /// Auto-approve design without confirmation
    #[arg(long)]
    pub auto_approve: bool,

    /// Metric name for measurement
    #[arg(long)]
    pub metric: Option<String>,

    /// Measurement command
    #[arg(long)]
    pub measure: Option<String>,

    /// Baseline value
    #[arg(long)]
    pub baseline: Option<f64>,

    /// Target improvement ratio (e.g., 0.30 for 30%)
    #[arg(long)]
    pub target_improvement: Option<f64>,

    /// Maximum iterations
    #[arg(long)]
    pub max_iterations: Option<usize>,

    /// Iteration timeout in minutes
    #[arg(long, alias = "iteration-timeout-minutes")]
    pub iteration_timeout_minutes: Option<usize>,

    /// Total timeout in minutes
    #[arg(long, alias = "total-timeout-minutes")]
    pub total_timeout_minutes: Option<usize>,

    /// Stall limit before backing off
    #[arg(long)]
    pub stall_limit: Option<usize>,

    /// Convergence threshold
    #[arg(long)]
    pub convergence_threshold: Option<f64>,

    /// Convergence window size
    #[arg(long)]
    pub convergence_window: Option<usize>,

    /// Verify baseline measurement
    #[arg(long)]
    pub verify_baseline: bool,

    /// Session file path
    #[arg(long, default_value = "autoresearch.jsonl")]
    pub session_file: String,

    /// Maximum variance between baseline measurements (default: 0.05 for 5%)
    #[arg(long, default_value = "0.05")]
    pub max_variance: f64,

    /// Verbose output with per-iteration details
    #[arg(long)]
    pub verbose: bool,

    /// Quiet mode - only show final result
    #[arg(long)]
    pub quiet: bool,

    /// Resume a previous experiment by session ID
    #[arg(long)]
    pub resume: Option<String>,

    /// List prior experiments from session file
    #[arg(long)]
    pub history: bool,

    /// Enable beads (bd) integration for issue tracking
    #[arg(long)]
    pub beads_enabled: bool,

    /// Enable Ralph-TUI compatibility mode
    #[arg(long)]
    pub ralph_tui_enabled: bool,

    /// Path to Ralph-TUI task file (e.g., tasks/example-task.md)
    #[arg(long)]
    pub ralph_task_file: Option<String>,
}

impl Cli {
    /// Get the effective max iterations (from CLI or default)
    pub fn effective_max_iterations(&self) -> usize {
        self.max_iterations.unwrap_or(20)
    }

    /// Get the effective iteration timeout in seconds
    pub fn effective_iteration_timeout_secs(&self) -> u64 {
        self.iteration_timeout_minutes.unwrap_or(10) as u64 * 60
    }

    /// Get the effective total timeout in seconds
    pub fn effective_total_timeout_secs(&self) -> u64 {
        self.total_timeout_minutes.unwrap_or(120) as u64 * 60
    }

    /// Get the effective stall limit
    pub fn effective_stall_limit(&self) -> usize {
        self.stall_limit.unwrap_or(5)
    }

    /// Get the effective convergence threshold
    pub fn effective_convergence_threshold(&self) -> f64 {
        self.convergence_threshold.unwrap_or(0.01)
    }

    /// Get the effective convergence window
    pub fn effective_convergence_window(&self) -> usize {
        self.convergence_window.unwrap_or(3)
    }
}
