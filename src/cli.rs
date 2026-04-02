use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "pi-autoresearch")]
#[command(about = "Autonomous research experiment orchestrator")]
pub struct Cli {
    #[arg(long)]
    pub question: Option<String>,
    #[arg(long)]
    pub auto_approve: bool,
    #[arg(long)]
    pub metric: Option<String>,
    #[arg(long)]
    pub measure: Option<String>,
    #[arg(long)]
    pub baseline: Option<f64>,
    #[arg(long)]
    pub target_improvement: Option<f64>,
    #[arg(long)]
    pub max_iterations: Option<usize>,
    #[arg(long, alias = "iteration-timeout-minutes")]
    pub iteration_timeout_minutes: Option<usize>,
    #[arg(long, alias = "total-timeout-minutes")]
    pub total_timeout_minutes: Option<usize>,
    #[arg(long)]
    pub stall_limit: Option<usize>,
    #[arg(long)]
    pub convergence_threshold: Option<f64>,
    #[arg(long)]
    pub convergence_window: Option<usize>,
    #[arg(long)]
    pub verify_baseline: bool,
    #[arg(long, default_value = "autoresearch.jsonl")]
    pub session_file: String,
    #[arg(long, default_value = "0.05")]
    pub max_variance: f64,
    #[arg(long)]
    pub verbose: bool,
    #[arg(long)]
    pub quiet: bool,
    #[arg(long)]
    pub resume: Option<String>,
    #[arg(long)]
    pub history: bool,
    #[arg(long)]
    pub beads_enabled: bool,
    #[arg(long)]
    pub ralph_tui_enabled: bool,
    #[arg(long)]
    pub ralph_task_file: Option<String>,
}

impl Cli {
    pub fn effective_max_iterations(&self) -> usize { self.max_iterations.unwrap_or(20) }
    pub fn effective_iteration_timeout_secs(&self) -> u64 { self.iteration_timeout_minutes.unwrap_or(10) as u64 * 60 }
    pub fn effective_total_timeout_secs(&self) -> u64 { self.total_timeout_minutes.unwrap_or(120) as u64 * 60 }
    pub fn effective_stall_limit(&self) -> usize { self.stall_limit.unwrap_or(5) }
    pub fn effective_convergence_threshold(&self) -> f64 { self.convergence_threshold.unwrap_or(0.01) }
    pub fn effective_convergence_window(&self) -> usize { self.convergence_window.unwrap_or(3) }
}
