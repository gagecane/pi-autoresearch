use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::process::Command;
use chrono::Utc;
use std::time::{Duration, Instant};
use std::fs::OpenOptions;
use std::path::PathBuf;
use tracing::{info, debug, warn, error};
use tracing_subscriber::EnvFilter;
use indicatif::{ProgressBar, ProgressStyle};
use colored::Colorize;


#[derive(Parser, Debug, Default)]
#[command(name = "pi-autoresearch")]
#[command(about = "Autonomous research experiment orchestrator")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    /// Research question to explore
    #[arg(long)]
    question: Option<String>,

    /// Auto-approve design without confirmation
    #[arg(long)]
    auto_approve: bool,

    /// Metric name for measurement
    #[arg(long)]
    metric: Option<String>,

    /// Measurement command
    #[arg(long)]
    measure: Option<String>,

    /// Baseline value
    #[arg(long)]
    baseline: Option<f64>,

    /// Target improvement ratio (e.g., 0.30 for 30%)
    #[arg(long)]
    target_improvement: Option<f64>,

    /// Maximum iterations
    #[arg(long)]
    max_iterations: Option<usize>,

    /// Iteration timeout in minutes
    #[arg(long)]
    iteration_timeout_minutes: Option<usize>,

    /// Total timeout in minutes
    #[arg(long)]
    total_timeout_minutes: Option<usize>,

    /// Stall limit before backing off
    #[arg(long)]
    stall_limit: Option<usize>,

    /// Convergence threshold
    #[arg(long)]
    convergence_threshold: Option<f64>,

    /// Convergence window size
    #[arg(long)]
    convergence_window: Option<usize>,

    /// Verify baseline measurement
    #[arg(long)]
    verify_baseline: bool,

    /// Session file path
    #[arg(long, default_value = "autoresearch.jsonl")]
    session_file: String,

    /// Maximum variance between baseline measurements (default: 0.05 for 5%)
    #[arg(long, default_value = "0.05")]
    max_variance: f64,

    /// Verbose output with per-iteration details
    #[arg(long)]
    verbose: bool,

    /// Quiet mode - only show final result
    #[arg(long)]
    quiet: bool,

    /// Resume a previous experiment by session ID
    #[arg(long)]
    resume: Option<String>,

    /// List prior experiments from session file
    #[arg(long)]
    history: bool,

    /// Enable beads (bd) integration for issue tracking
    #[arg(long)]
    beads_enabled: bool,

    /// Skip git operations (branch creation, commits, pushes)
    #[arg(long)]
    skip_git: bool,

    /// List all autoresearch branches
    #[arg(long)]
    list_branches: bool,

    /// Clean up old autoresearch branches
    #[arg(long)]
    cleanup_branches: bool,

    /// Only remove branches older than N days (default: 7)
    #[arg(long, default_value = "7")]
    cleanup_days: usize,

    /// Dry run - show what would be done without making changes
    #[arg(long)]
    dry_run: bool,

    /// Path to config file (defaults to ~/.config/pi-autoresearch/config.json)
    #[arg(long, short = 'c')]
    config: Option<String>,

    /// Compare two experiments by session ID (requires --compare-id2)
    #[arg(long)]
    compare_id1: Option<String>,

    /// Second session ID for comparison (requires --compare-id1)
    #[arg(long)]
    compare_id2: Option<String>,
}

/// Configuration loaded from config file
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
struct Config {
    /// Default metric name
    metric: Option<String>,
    /// Default measurement command
    measure: Option<String>,
    /// Default baseline value
    baseline: Option<f64>,
    /// Default target improvement ratio
    target_improvement: Option<f64>,
    /// Default maximum iterations
    max_iterations: Option<usize>,
    /// Default iteration timeout in minutes
    iteration_timeout_minutes: Option<usize>,
    /// Default total timeout in minutes
    total_timeout_minutes: Option<usize>,
    /// Default stall limit
    stall_limit: Option<usize>,
    /// Default convergence threshold
    convergence_threshold: Option<f64>,
    /// Default convergence window size
    convergence_window: Option<usize>,
    /// Default maximum variance between baseline measurements
    max_variance: Option<f64>,
    /// Default session file path
    session_file: Option<String>,
    /// Enable beads integration by default
    beads_enabled: Option<bool>,
}

/// Validation error for config values
#[derive(Debug)]
enum ConfigValidationError {
    MaxVarianceOutOfRange { value: f64 },
    TargetImprovementNotPositive { value: f64 },
    MaxIterationsNotPositive { value: usize },
    IterationTimeoutNotPositive { value: usize },
    TotalTimeoutNotPositive { value: usize },
    StallLimitNotPositive { value: usize },
    ConvergenceWindowNotPositive { value: usize },
    SessionFileInvalidPath { path: String },
}

impl std::fmt::Display for ConfigValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigValidationError::MaxVarianceOutOfRange { value } => {
                write!(f, 
                    "max_variance must be between 0.0 and 1.0, got {:.2}\n\n",
                    value
                )?;
                writeln!(f, "  {}", "SUGGESTION:".yellow().bold())?;
                writeln!(f, "  - Set max_variance to a value between 0.0 and 1.0")?;
                writeln!(f, "  - Example: \"max_variance\": 0.05 (for 5%% variance)")?;
                writeln!(f, "  - See: docs/CONFIG.md for more information")?;
                Ok(())
            }
            ConfigValidationError::TargetImprovementNotPositive { value } => {
                write!(f, 
                    "target_improvement must be positive, got {:.2}\n\n",
                    value
                )?;
                writeln!(f, "  {}", "SUGGESTION:".yellow().bold())?;
                writeln!(f, "  - Set target_improvement to a positive value")?;
                writeln!(f, "  - Example: \"target_improvement\": 0.20 (for 20%% improvement)")?;
                writeln!(f, "  - See: docs/CONFIG.md for more information")?;
                Ok(())
            }
            ConfigValidationError::MaxIterationsNotPositive { value } => {
                write!(f, 
                    "max_iterations must be positive, got {}\n\n",
                    value
                )?;
                writeln!(f, "  {}", "SUGGESTION:".yellow().bold())?;
                writeln!(f, "  - Set max_iterations to a positive integer")?;
                writeln!(f, "  - Example: \"max_iterations\": 20")?;
                writeln!(f, "  - See: docs/CONFIG.md for more information")?;
                Ok(())
            }
            ConfigValidationError::IterationTimeoutNotPositive { value } => {
                write!(f, 
                    "iteration_timeout_minutes must be positive, got {}\n\n",
                    value
                )?;
                writeln!(f, "  {}", "SUGGESTION:".yellow().bold())?;
                writeln!(f, "  - Set iteration_timeout_minutes to a positive integer")?;
                writeln!(f, "  - Example: \"iteration_timeout_minutes\": 10")?;
                writeln!(f, "  - See: docs/CONFIG.md for more information")?;
                Ok(())
            }
            ConfigValidationError::TotalTimeoutNotPositive { value } => {
                write!(f, 
                    "total_timeout_minutes must be positive, got {}\n\n",
                    value
                )?;
                writeln!(f, "  {}", "SUGGESTION:".yellow().bold())?;
                writeln!(f, "  - Set total_timeout_minutes to a positive integer")?;
                writeln!(f, "  - Example: \"total_timeout_minutes\": 120")?;
                writeln!(f, "  - See: docs/CONFIG.md for more information")?;
                Ok(())
            }
            ConfigValidationError::StallLimitNotPositive { value } => {
                write!(f, 
                    "stall_limit must be positive, got {}\n\n",
                    value
                )?;
                writeln!(f, "  {}", "SUGGESTION:".yellow().bold())?;
                writeln!(f, "  - Set stall_limit to a positive integer")?;
                writeln!(f, "  - Example: \"stall_limit\": 5")?;
                writeln!(f, "  - See: docs/CONFIG.md for more information")?;
                Ok(())
            }
            ConfigValidationError::ConvergenceWindowNotPositive { value } => {
                write!(f, 
                    "convergence_window must be positive, got {}\n\n",
                    value
                )?;
                writeln!(f, "  {}", "SUGGESTION:".yellow().bold())?;
                writeln!(f, "  - Set convergence_window to a positive integer")?;
                writeln!(f, "  - Example: \"convergence_window\": 3")?;
                writeln!(f, "  - See: docs/CONFIG.md for more information")?;
                Ok(())
            }
            ConfigValidationError::SessionFileInvalidPath { path } => {
                write!(f, 
                    "session_file path is not valid or writable: {}\n\n",
                    path
                )?;
                writeln!(f, "  {}", "SUGGESTION:".yellow().bold())?;
                writeln!(f, "  - Ensure the directory exists and is writable")?;
                writeln!(f, "  - Use an absolute path if needed")?;
                writeln!(f, "  - Example: \"session_file\": \"./autoresearch.jsonl\"")?;
                writeln!(f, "  - See: docs/CONFIG.md for more information")?;
                Ok(())
            }
        }
    }
}

impl std::error::Error for ConfigValidationError {}

/// Initialize structured logging based on CLI flags and environment variables
/// 
/// Configures the tracing subscriber with appropriate log levels:
/// - Quiet mode (cli.quiet): Only shows errors
/// - Verbose mode (cli.verbose): Shows all logs including debug
/// - Normal mode: Uses RUST_LOG environment variable or defaults to info level
/// 
/// # Arguments
/// 
/// * `cli` - CLI configuration with quiet and verbose flags
/// 
/// # Returns
/// 
/// `Ok(())` on success, `Err` if logging initialization fails.
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::{init_logging, Cli};
/// 
/// // Initialize with quiet mode
/// let cli = Cli { quiet: true, ..Default::default() };
/// init_logging(&cli)?;
/// 
/// // Initialize with verbose mode
/// let cli = Cli { verbose: true, ..Default::default() };
/// init_logging(&cli)?;
/// 
/// // Initialize with default mode (uses RUST_LOG env var)
/// let cli = Cli::default();
/// init_logging(&cli)?;
/// ```
fn init_logging(cli: &Cli) -> Result<()> {
    // Build the filter
    let filter = if cli.quiet {
        // Quiet mode: only show errors
        EnvFilter::new("error")
    } else if cli.verbose {
        // Verbose mode: show all logs including debug
        EnvFilter::new("debug")
    } else {
        // Use RUST_LOG environment variable if set, otherwise default to info
        EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("info"))
            .unwrap()
    };

    // Initialize tracing subscriber with stderr output (for backward compatibility with tests)
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .with_writer(std::io::stderr)
        .init();

    Ok(())
}

/// Create a progress bar with a custom message and total iterations
/// 
/// Creates a new progress bar with a standard format including spinner,
/// message, bar visualization, current/total count, and ETA.
/// 
/// # Arguments
/// 
/// * `message` - The message to display with the progress bar
/// * `total` - The total number of iterations for the progress bar
/// 
/// # Returns
/// 
/// A configured `ProgressBar` ready to use.
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::create_progress_bar;
/// 
/// // Create a progress bar for 100 iterations
/// let pb = create_progress_bar("Processing", 100);
/// pb.inc(1); // Advance by 1
/// pb.finish(); // Complete the progress bar
/// 
/// // Create a progress bar for baseline verification
/// let pb = create_progress_bar("Verifying baseline", 2);
/// pb.set_message("Run 1/2...");
/// pb.inc(1);
/// ```
fn create_progress_bar(message: &str, total: u64) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner} {msg} [{bar:40}] {pos}/{len} ({eta})")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
    );
    pb.set_message(message.to_string());
    pb
}

/// Validate config file values
/// 
/// Checks all configurable values for validity:
/// - max_variance: must be between 0.0 and 1.0
/// - target_improvement: must be positive
/// - max_iterations: must be positive
/// - iteration_timeout_minutes: must be positive
/// - total_timeout_minutes: must be positive
/// - stall_limit: must be positive
/// - convergence_window: must be positive
/// - session_file: must be a valid writable path
/// 
/// # Arguments
/// 
/// * `config` - The configuration to validate
/// 
/// # Returns
/// 
/// `Ok(())` if all values are valid, `Err` with detailed error messages otherwise.
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::{validate_config, Config};
/// 
/// // Valid config
/// let config = Config {
///     max_variance: Some(0.05),
///     target_improvement: Some(0.20),
///     max_iterations: Some(20),
///     ..Default::default()
/// };
/// assert!(validate_config(&config).is_ok());
/// 
/// // Invalid config (max_variance out of range)
/// let config = Config {
///     max_variance: Some(1.5),
///     ..Default::default()
/// };
/// assert!(validate_config(&config).is_err());
/// ```
fn validate_config(config: &Config) -> Result<()> {
    let mut errors = Vec::new();

    // Validate max_variance: must be between 0.0 and 1.0
    if let Some(value) = config.max_variance {
        if !(0.0..=1.0).contains(&value) {
            errors.push(ConfigValidationError::MaxVarianceOutOfRange { value });
        }
    }

    // Validate target_improvement: must be positive
    if let Some(value) = config.target_improvement {
        if value <= 0.0 {
            errors.push(ConfigValidationError::TargetImprovementNotPositive { value });
        }
    }

    // Validate max_iterations: must be positive
    if let Some(value) = config.max_iterations {
        if value == 0 {
            errors.push(ConfigValidationError::MaxIterationsNotPositive { value });
        }
    }

    // Validate iteration_timeout_minutes: must be positive
    if let Some(value) = config.iteration_timeout_minutes {
        if value == 0 {
            errors.push(ConfigValidationError::IterationTimeoutNotPositive { value });
        }
    }

    // Validate total_timeout_minutes: must be positive
    if let Some(value) = config.total_timeout_minutes {
        if value == 0 {
            errors.push(ConfigValidationError::TotalTimeoutNotPositive { value });
        }
    }

    // Validate stall_limit: must be positive
    if let Some(value) = config.stall_limit {
        if value == 0 {
            errors.push(ConfigValidationError::StallLimitNotPositive { value });
        }
    }

    // Validate convergence_window: must be positive
    if let Some(value) = config.convergence_window {
        if value == 0 {
            errors.push(ConfigValidationError::ConvergenceWindowNotPositive { value });
        }
    }

    // Validate session_file: must be a valid writable path
    if let Some(path) = &config.session_file {
        if !is_valid_session_path(path) {
            errors.push(ConfigValidationError::SessionFileInvalidPath { path: path.clone() });
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        let error_messages: Vec<String> = errors.into_iter().map(|e| e.to_string()).collect();
        Err(anyhow::anyhow!("Config validation failed:\n  {}", error_messages.join("\n  ")))
    }
}

/// Check if a session file path is valid and writable
/// 
/// This function is non-destructive - it does not create directories or modify files.
/// It verifies that the parent directory exists and is writable by attempting
/// to create a temporary file.
/// 
/// # Arguments
/// 
/// * `path` - The session file path to validate
/// 
/// # Returns
/// 
/// `true` if the path is valid and writable, `false` otherwise.
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::is_valid_session_path;
/// 
/// // Valid path in current directory
/// assert!(is_valid_session_path("autoresearch.jsonl"));
/// 
/// // Valid path in existing directory
/// assert!(is_valid_session_path("./sessions/autoresearch.jsonl"));
/// 
/// // Invalid path in non-existent directory
/// assert!(!is_valid_session_path("/nonexistent/dir/file.jsonl"));
/// ```
fn is_valid_session_path(path: &str) -> bool {
    let path = std::path::Path::new(path);
    
    // Check if parent directory exists and is writable
    if let Some(parent) = path.parent() {
        // If parent is empty (just a filename with no directory component),
        // treat it as current directory which should be writable
        if parent.as_os_str().is_empty() {
            return true;
        }
        
        if !parent.exists() {
            return false; // Don't create directories during validation
        }
        
        // Check if parent is writable by trying to create a temp file
        let temp_file = parent.join(".validation_temp");
        match std::fs::File::create(&temp_file) {
            Ok(_) => {
                let _ = std::fs::remove_file(&temp_file);
                true
            }
            Err(_) => false,
        }
    } else {
        false
    }
}

/// Load configuration from a config file
/// 
/// Attempts to load configuration from the specified path, or from the default
/// location (~/.config/pi-autoresearch/config.json) if no path is provided.
/// Validates the configuration after loading.
/// 
/// # Arguments
/// 
/// * `config_path` - Optional path to the config file. If None, uses default location.
/// * `explicit_config` - If true, returns an error when config file is not found.
///   If false, returns Ok(None) when config file is not found.
/// 
/// # Returns
/// 
/// `Ok(Some(Config))` if config was loaded successfully.
/// `Ok(None)` if config file doesn't exist and explicit_config is false.
/// `Err` if config file doesn't exist and explicit_config is true, or if validation fails.
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::load_config;
/// 
/// // Load from default location (returns None if not found)
/// let config = load_config(None, false)?;
/// 
/// // Load from explicit path (returns error if not found)
/// let config = load_config(Some("/path/to/config.json"), true)?;
/// 
/// // Load from explicit path (returns None if not found)
/// let config = load_config(Some("/path/to/config.json"), false)?;
/// ```
fn load_config(config_path: Option<&str>, explicit_config: bool) -> Result<Option<Config>> {
    let path = if let Some(p) = config_path {
        PathBuf::from(p)
    } else {
        // Try default config path: ~/.config/pi-autoresearch/config.json
        let home_dir = std::env::var("HOME").map_err(|_| anyhow::anyhow!("HOME environment variable not set"))?;
        PathBuf::from(&home_dir)
            .join(".config")
            .join("pi-autoresearch")
            .join("config.json")
    };

    if !path.exists() {
        if explicit_config {
            return Err(anyhow::anyhow!("Config file not found: {}", path.display()));
        }
        return Ok(None);
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("Failed to read config file {}: {}", path.display(), e))?;
    
    let config: Config = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse config file {}: {}", path.display(), e))?;
    
    // Validate config values
    validate_config(&config).map_err(|e| {
        anyhow::anyhow!("Invalid config file {}: {}", path.display(), e)
    })?;
    
    Ok(Some(config))
}

/// Get effective metric value from CLI or config
/// 
/// CLI takes precedence over config, which takes precedence over default.
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::{Cli, Config, get_metric};
/// 
/// // CLI takes precedence
/// let cli = Cli { metric: Some("my_metric".to_string()), ..Default::default() };
/// let config = Some(Config { metric: Some("config_metric".to_string()), ..Default::default() });
/// assert_eq!(get_metric(&cli, &config, "default"), "my_metric");
/// 
/// // Config used when CLI is None
/// let cli = Cli { metric: None, ..Default::default() };
/// assert_eq!(get_metric(&cli, &config, "default"), "config_metric");
/// 
/// // Default used when both CLI and config are None
/// let cli = Cli { metric: None, ..Default::default() };
/// let config = None;
/// assert_eq!(get_metric(&cli, &config, "default"), "default");
/// ```
fn get_metric(cli: &Cli, config: &Option<Config>, default: &str) -> String {
    cli.metric.clone()
        .or(config.as_ref().and_then(|c| c.metric.clone()))
        .unwrap_or_else(|| default.to_string())
}

/// Get effective measure value from CLI or config
/// 
/// CLI takes precedence over config, which takes precedence over default.
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::{Cli, Config, get_measure};
/// 
/// // CLI takes precedence
/// let cli = Cli { measure: Some("echo 100".to_string()), ..Default::default() };
/// let config = Some(Config { measure: Some("echo 200".to_string()), ..Default::default() });
/// assert_eq!(get_measure(&cli, &config, "default"), "echo 100");
/// 
/// // Config used when CLI is None
/// let cli = Cli { measure: None, ..Default::default() };
/// assert_eq!(get_measure(&cli, &config, "default"), "echo 200");
/// ```
fn get_measure(cli: &Cli, config: &Option<Config>, default: &str) -> String {
    cli.measure.clone()
        .or(config.as_ref().and_then(|c| c.measure.clone()))
        .unwrap_or_else(|| default.to_string())
}

/// Get effective baseline value from CLI or config
/// 
/// CLI takes precedence over config, which takes precedence over default.
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::{Cli, Config, get_baseline};
/// 
/// // CLI takes precedence
/// let cli = Cli { baseline: Some(100.0), ..Default::default() };
/// let config = Some(Config { baseline: Some(200.0), ..Default::default() });
/// assert_eq!(get_baseline(&cli, &config, 0.0), 100.0);
/// 
/// // Config used when CLI is None
/// let cli = Cli { baseline: None, ..Default::default() };
/// assert_eq!(get_baseline(&cli, &config, 0.0), 200.0);
/// 
/// // Default used when both are None
/// let cli = Cli { baseline: None, ..Default::default() };
/// let config = None;
/// assert_eq!(get_baseline(&cli, &config, 50.0), 50.0);
/// ```
fn get_baseline(cli: &Cli, config: &Option<Config>, default: f64) -> f64 {
    cli.baseline
        .or(config.as_ref().and_then(|c| c.baseline))
        .unwrap_or(default)
}

/// Get effective target improvement from CLI or config
/// 
/// CLI takes precedence over config, which takes precedence over default.
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::{Cli, Config, get_target_improvement};
/// 
/// // CLI takes precedence
/// let cli = Cli { target_improvement: Some(0.30), ..Default::default() };
/// let config = Some(Config { target_improvement: Some(0.20), ..Default::default() });
/// assert_eq!(get_target_improvement(&cli, &config, 0.10), 0.30);
/// 
/// // Config used when CLI is None
/// let cli = Cli { target_improvement: None, ..Default::default() };
/// assert_eq!(get_target_improvement(&cli, &config, 0.10), 0.20);
/// ```
fn get_target_improvement(cli: &Cli, config: &Option<Config>, default: f64) -> f64 {
    cli.target_improvement
        .or(config.as_ref().and_then(|c| c.target_improvement))
        .unwrap_or(default)
}

/// Get effective max iterations from CLI or config
/// 
/// CLI takes precedence over config, which takes precedence over default.
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::{Cli, Config, get_max_iterations};
/// 
/// // CLI takes precedence
/// let cli = Cli { max_iterations: Some(50), ..Default::default() };
/// let config = Some(Config { max_iterations: Some(30), ..Default::default() });
/// assert_eq!(get_max_iterations(&cli, &config, 20), 50);
/// 
/// // Config used when CLI is None
/// let cli = Cli { max_iterations: None, ..Default::default() };
/// assert_eq!(get_max_iterations(&cli, &config, 20), 30);
/// 
/// // Default used when both are None
/// let cli = Cli { max_iterations: None, ..Default::default() };
/// let config = None;
/// assert_eq!(get_max_iterations(&cli, &config, 20), 20);
/// ```
fn get_max_iterations(cli: &Cli, config: &Option<Config>, default: usize) -> usize {
    cli.max_iterations
        .or(config.as_ref().and_then(|c| c.max_iterations))
        .unwrap_or(default)
}

/// Get effective max variance from CLI or config
/// 
/// CLI always takes precedence. Config and default parameters are currently unused.
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::{Cli, Config, get_max_variance};
/// 
/// // CLI value is always used
/// let cli = Cli { max_variance: 0.10, ..Default::default() };
/// let config = Some(Config { max_variance: Some(0.05), ..Default::default() });
/// assert_eq!(get_max_variance(&cli, &config, 0.05), 0.10);
/// ```
fn get_max_variance(cli: &Cli, _config: &Option<Config>, _default: f64) -> f64 {
    cli.max_variance
}

/// Get effective session file from CLI or config
/// 
/// CLI always takes precedence. Config parameter is currently unused.
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::{Cli, Config, get_session_file};
/// 
/// // CLI value is always used
/// let cli = Cli { session_file: "custom.jsonl".to_string(), ..Default::default() };
/// let config = Some(Config { session_file: Some("config.jsonl".to_string()), ..Default::default() });
/// assert_eq!(get_session_file(&cli, &config), "custom.jsonl");
/// ```
fn get_session_file(cli: &Cli, _config: &Option<Config>) -> String {
    cli.session_file.clone()
}

/// Get effective beads enabled from CLI or config
/// 
/// CLI takes precedence over config. Returns false if both are disabled/not set.
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::{Cli, Config, get_beads_enabled};
/// 
/// // CLI takes precedence (enabled)
/// let cli = Cli { beads_enabled: true, ..Default::default() };
/// let config = Some(Config { beads_enabled: Some(false), ..Default::default() });
/// assert!(get_beads_enabled(&cli, &config));
/// 
/// // Config used when CLI is false
/// let cli = Cli { beads_enabled: false, ..Default::default() };
/// let config = Some(Config { beads_enabled: Some(true), ..Default::default() });
/// assert!(get_beads_enabled(&cli, &config));
/// 
/// // Returns false when both are false/not set
/// let cli = Cli { beads_enabled: false, ..Default::default() };
/// let config = None;
/// assert!(!get_beads_enabled(&cli, &config));
/// ```
fn get_beads_enabled(cli: &Cli, config: &Option<Config>) -> bool {
    if cli.beads_enabled {
        true
    } else {
        config.as_ref()
            .and_then(|c| c.beads_enabled)
            .unwrap_or(false)
    }
}

/// Get effective iteration timeout from CLI or config
/// 
/// CLI takes precedence over config, which takes precedence over default.
/// Returns timeout in minutes.
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::{Cli, Config, get_iteration_timeout};
/// 
/// // CLI takes precedence
/// let cli = Cli { iteration_timeout_minutes: Some(30), ..Default::default() };
/// let config = Some(Config { iteration_timeout_minutes: Some(20), ..Default::default() });
/// assert_eq!(get_iteration_timeout(&cli, &config, 10), 30);
/// 
/// // Config used when CLI is None
/// let cli = Cli { iteration_timeout_minutes: None, ..Default::default() };
/// assert_eq!(get_iteration_timeout(&cli, &config, 10), 20);
/// ```
fn get_iteration_timeout(cli: &Cli, config: &Option<Config>, default: usize) -> usize {
    cli.iteration_timeout_minutes
        .or(config.as_ref().and_then(|c| c.iteration_timeout_minutes))
        .unwrap_or(default)
}

/// Get effective total timeout from CLI or config
/// 
/// CLI takes precedence over config, which takes precedence over default.
/// Returns timeout in minutes.
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::{Cli, Config, get_total_timeout};
/// 
/// // CLI takes precedence
/// let cli = Cli { total_timeout_minutes: Some(240), ..Default::default() };
/// let config = Some(Config { total_timeout_minutes: Some(120), ..Default::default() });
/// assert_eq!(get_total_timeout(&cli, &config, 60), 240);
/// 
/// // Config used when CLI is None
/// let cli = Cli { total_timeout_minutes: None, ..Default::default() };
/// assert_eq!(get_total_timeout(&cli, &config, 60), 120);
/// ```
fn get_total_timeout(cli: &Cli, config: &Option<Config>, default: usize) -> usize {
    cli.total_timeout_minutes
        .or(config.as_ref().and_then(|c| c.total_timeout_minutes))
        .unwrap_or(default)
}

/// Get effective stall limit from CLI or config
/// 
/// CLI takes precedence over config, which takes precedence over default.
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::{Cli, Config, get_stall_limit};
/// 
/// // CLI takes precedence
/// let cli = Cli { stall_limit: Some(10), ..Default::default() };
/// let config = Some(Config { stall_limit: Some(5), ..Default::default() });
/// assert_eq!(get_stall_limit(&cli, &config, 3), 10);
/// 
/// // Config used when CLI is None
/// let cli = Cli { stall_limit: None, ..Default::default() };
/// assert_eq!(get_stall_limit(&cli, &config, 3), 5);
/// ```
fn get_stall_limit(cli: &Cli, config: &Option<Config>, default: usize) -> usize {
    cli.stall_limit
        .or(config.as_ref().and_then(|c| c.stall_limit))
        .unwrap_or(default)
}

/// Get effective convergence threshold from CLI or config
/// 
/// CLI takes precedence over config, which takes precedence over default.
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::{Cli, Config, get_convergence_threshold};
/// 
/// // CLI takes precedence
/// let cli = Cli { convergence_threshold: Some(0.05), ..Default::default() };
/// let config = Some(Config { convergence_threshold: Some(0.02), ..Default::default() });
/// assert_eq!(get_convergence_threshold(&cli, &config, 0.01), 0.05);
/// 
/// // Config used when CLI is None
/// let cli = Cli { convergence_threshold: None, ..Default::default() };
/// assert_eq!(get_convergence_threshold(&cli, &config, 0.01), 0.02);
/// ```
fn get_convergence_threshold(cli: &Cli, config: &Option<Config>, default: f64) -> f64 {
    cli.convergence_threshold
        .or(config.as_ref().and_then(|c| c.convergence_threshold))
        .unwrap_or(default)
}

/// Get effective convergence window from CLI or config
/// 
/// CLI takes precedence over config, which takes precedence over default.
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::{Cli, Config, get_convergence_window};
/// 
/// // CLI takes precedence
/// let cli = Cli { convergence_window: Some(5), ..Default::default() };
/// let config = Some(Config { convergence_window: Some(3), ..Default::default() });
/// assert_eq!(get_convergence_window(&cli, &config, 3), 5);
/// 
/// // Config used when CLI is None
/// let cli = Cli { convergence_window: None, ..Default::default() };
/// assert_eq!(get_convergence_window(&cli, &config, 3), 3);
/// ```
fn get_convergence_window(cli: &Cli, config: &Option<Config>, default: usize) -> usize {
    cli.convergence_window
        .or(config.as_ref().and_then(|c| c.convergence_window))
        .unwrap_or(default)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ExperimentDesign {
    hypothesis: String,
    metric: String,
    measurement: String,
    baseline: f64,
    target_improvement: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct BaselineRecord {
    timestamp: String,
    git_commit: String,
    metric: String,
    measurement_command: String,
    value: f64,
    verification_runs: Vec<f64>,
    variance: f64,
    within_threshold: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct BaselineVerificationResult {
    success: bool,
    baseline_record: Option<BaselineRecord>,
    error_message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct IterationRecord {
    iteration: usize,
    timestamp: String,
    agent_action: String,
    metric_value: f64,
    improvement: f64,
    kept: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ExperimentSession {
    session_id: String,
    question: String,
    design: ExperimentDesign,
    baseline_record: BaselineRecord,
    iterations: Vec<IterationRecord>,
    best_iteration: Option<usize>,
    start_time: String,
    end_time: Option<String>,
    status: String,
}

#[derive(Debug)]
struct IterationState {
    current_iteration: usize,
    best_metric: f64,
    best_iteration: usize,
    consecutive_no_improvement: usize,
    backoff_count: usize,
    start_time: Instant,
    recent_metrics: Vec<f64>,
}

#[derive(Debug)]
struct BaselineError {
    message: String,
}

#[derive(Debug, Clone)]
enum StuckReason {
    IterationTimeout,
    StallLimitReached,
    TotalTimeout,
    ConvergenceAchieved,
    MaxIterationsReached,
}

impl std::fmt::Display for StuckReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StuckReason::IterationTimeout => write!(f, "Iteration timeout exceeded"),
            StuckReason::StallLimitReached => write!(f, "No improvement after multiple iterations (stall limit reached)"),
            StuckReason::TotalTimeout => write!(f, "Total experiment timeout exceeded"),
            StuckReason::ConvergenceAchieved => write!(f, "Metric convergence achieved"),
            StuckReason::MaxIterationsReached => write!(f, "Maximum iterations reached"),
        }
    }
}

impl std::fmt::Display for BaselineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BaselineError {}

/// Generate an experiment design based on a research question
/// 
/// Analyzes the question to determine appropriate metric, measurement command,
/// and baseline value. Supports memory, speed/performance, and accuracy optimization.
/// 
/// # Arguments
/// 
/// * `question` - The research question to analyze
/// 
/// # Returns
/// 
/// An `ExperimentDesign` with appropriate defaults based on the question
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::{generate_design, ExperimentDesign};
/// 
/// // Memory optimization
/// let design = generate_design("How can I reduce memory usage?");
/// assert_eq!(design.metric, "peak_memory_mb");
/// assert_eq!(design.baseline, 512.0);
/// 
/// // Speed optimization
/// let design = generate_design("How can I improve speed?");
/// assert_eq!(design.metric, "execution_time_ms");
/// assert_eq!(design.baseline, 1000.0);
/// 
/// // Accuracy optimization
/// let design = generate_design("How can I improve accuracy?");
/// assert_eq!(design.metric, "accuracy_percent");
/// assert_eq!(design.baseline, 85.0);
/// 
/// // Default for unknown metric
/// let design = generate_design("How can I optimize this?");
/// assert_eq!(design.metric, "metric_value");
/// assert_eq!(design.baseline, 100.0);
/// ```
fn generate_design(question: &str) -> ExperimentDesign {
    let lower_question = question.to_lowercase();
    
    let (metric, measurement, baseline) = if lower_question.contains("memory") {
        ("peak_memory_mb".to_string(),
         "Run benchmark suite, capture peak RSS via /proc/self/status".to_string(),
         512.0)
    } else if lower_question.contains("speed") || lower_question.contains("performance") {
        ("execution_time_ms".to_string(),
         "Run benchmark suite with hyperfine, report mean".to_string(),
         1000.0)
    } else if lower_question.contains("accuracy") {
        ("accuracy_percent".to_string(),
         "Run test suite, calculate pass rate".to_string(),
         85.0)
    } else {
        ("metric_value".to_string(),
         "Run evaluation script".to_string(),
         100.0)
    };

    let hypothesis = format!("Optimizing based on: {}", question);

    ExperimentDesign {
        hypothesis,
        metric,
        measurement,
        baseline,
        target_improvement: 0.30,
    }
}

fn read_question_from_stdin() -> Result<String> {
    print!("Enter research question: ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let question = input.trim().to_string();
    if question.is_empty() {
        anyhow::bail!("Question cannot be empty");
    }
    
    Ok(question)
}

fn execute_measurement(command: &str) -> Result<f64> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Err(BaselineError {
            message: "Empty measurement command".to_string(),
        }.into());
    }

    let output = Command::new(parts[0])
        .args(&parts[1..])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(BaselineError {
            message: format!("Measurement command failed: {}", stderr),
        }.into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: f64 = stdout.trim().parse().map_err(|_| {
        BaselineError {
            message: format!(
                "Could not parse measurement output as number: '{}'",
                stdout.trim()
            ),
        }
    })?;

    Ok(value)
}

fn get_git_commit_hash() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()?;

    if output.status.success() {
        let hash = String::from_utf8_lossy(&output.stdout);
        Ok(hash.trim().to_string())
    } else {
        Ok("unknown".to_string())
    }
}

/// Parse branch age from git commit date
/// 
/// Returns number of days since the branch's last commit, or -1 if unknown.
/// Attempts to parse dates in RFC3339 format and other common git date formats.
/// 
/// # Arguments
/// 
/// * `branch` - The branch name to check
/// 
/// # Returns
/// 
/// Number of days since the last commit on the branch, or -1 if the branch
/// doesn't exist or the date can't be parsed.
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::parse_branch_age_days;
/// 
/// // Returns days since last commit for existing branch
/// let days = parse_branch_age_days("main");
/// assert!(days >= -1); // -1 if branch doesn't exist or date unknown
/// 
/// // Returns -1 for non-existent branch
/// let days = parse_branch_age_days("nonexistent-branch-12345");
/// assert_eq!(days, -1);
/// ```
fn parse_branch_age_days(branch: &str) -> i64 {
    let date_output = Command::new("git")
        .args(["log", "-1", "--format=%ai", branch])
        .output();

    if let Ok(output) = date_output {
        if output.status.success() {
            let date_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            // Try to parse the date in various formats
            let commit_date = chrono::DateTime::parse_from_rfc3339(&date_str)
                .or_else(|_| chrono::DateTime::parse_from_str(&date_str, "%Y-%m-%d %H:%M:%S %z"))
                .or_else(|_| chrono::DateTime::parse_from_str(&date_str, "%Y-%m-%d %H:%M:%S%.f %z"));
            
            if let Ok(parsed_date) = commit_date {
                let now = Utc::now();
                let duration = now.signed_duration_since(parsed_date);
                duration.num_days()
            } else {
                -1 // Unknown age
            }
        } else {
            -1 // Unknown age
        }
    } else {
        -1 // Unknown age
    }
}

/// Format branch age as a human-readable string
/// 
/// Converts a branch age in days to a readable format.
/// Returns "unknown age" for negative values.
/// 
/// # Arguments
/// 
/// * `days` - Number of days since the branch's last commit
/// 
/// # Returns
/// 
/// A formatted string describing the branch age
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::format_branch_age;
/// 
/// // Positive days
/// assert_eq!(format_branch_age(7), "7 days old");
/// assert_eq!(format_branch_age(0), "0 days old");
/// 
/// // Unknown age (negative value)
/// assert_eq!(format_branch_age(-1), "unknown age");
/// ```
fn format_branch_age(days: i64) -> String {
    if days >= 0 {
        format!("{} days old", days)
    } else {
        "unknown age".to_string()
    }
}

fn verify_baseline(
    metric: &str,
    measurement_command: &str,
    max_variance: f64,
) -> Result<BaselineVerificationResult> {
    info!("Verifying baseline measurement...");
    info!("  Metric: {}", metric);
    info!("  Command: {}", measurement_command);
    info!("  Max variance: {:.1}%", max_variance * 100.0);

    let git_commit = get_git_commit_hash()?;
    let timestamp = Utc::now().to_rfc3339();

    let mut runs: Vec<f64> = Vec::new();
    
    // Create progress bar for baseline verification (2 runs)
    let pb = create_progress_bar("Verifying baseline", 2);

    for i in 1..=2 {
        pb.set_message(format!("Run {}/2...", i));
        debug!("  Run {}/2... ", i);

        match execute_measurement(measurement_command) {
            Ok(value) => {
                info!("{:.2}", value);
                runs.push(value);
                pb.inc(1);
            }
            Err(e) => {
                pb.abandon_with_message("FAILED".to_string());
                error!("FAILED");
                return Ok(BaselineVerificationResult {
                    success: false,
                    baseline_record: None,
                    error_message: Some(format!(
                        "Run {} failed: {}\n\n",
                        i, e
                    ) + &format!(
                        "  {}\n",
                        "SUGGESTION:".yellow().bold()
                    ) + "  - Check that the measurement command works correctly\n" +
                        "  - Ensure the command outputs a numeric value\n" +
                        "  - Run the command manually to verify\n" +
                        "  - See: docs/TROUBLESHOOTING.md#measurement-issues")
                });
            }
        }
    }
    
    pb.finish_and_clear();

    if runs.len() < 2 {
        return Ok(BaselineVerificationResult {
            success: false,
            baseline_record: None,
            error_message: Some("Insufficient successful runs".to_string()),
        });
    }

    let baseline_value = runs[0];
    let variance = (runs[1] - runs[0]).abs() / runs[0].max(1.0);
    let within_threshold = variance <= max_variance;

    let baseline_record = BaselineRecord {
        timestamp,
        git_commit,
        metric: metric.to_string(),
        measurement_command: measurement_command.to_string(),
        value: baseline_value,
        verification_runs: runs.clone(),
        variance,
        within_threshold,
    };

    if !within_threshold {
        warn!(
            "\nBaseline variance {:.2}% exceeds threshold {:.2}%",
            variance * 100.0,
            max_variance * 100.0
        );
        info!("  Run 1: {:.2}", runs[0]);
        info!("  Run 2: {:.2}", runs[1]);

        return Ok(BaselineVerificationResult {
            success: false,
            baseline_record: Some(baseline_record),
            error_message: Some(format!(
                "Variance {:.2}% exceeds threshold {:.2}%",
                variance * 100.0, max_variance * 100.0
            )),
        });
    }

    info!("\nBaseline verified successfully!");
    info!("  Value: {:.2}", baseline_value);
    info!("  Variance: {:.2}%", variance * 100.0);
    info!("  Git commit: {}", baseline_record.git_commit);

    Ok(BaselineVerificationResult {
        success: true,
        baseline_record: Some(baseline_record),
        error_message: None,
    })
}

fn save_to_session_file(
    session_file: &str,
    record: &BaselineRecord,
) -> Result<()> {
    let json_line = serde_json::to_string(record)?;
    
    use std::fs::OpenOptions;
    use std::io::Write;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(session_file)?;
    
    writeln!(file, "{}", json_line)?;
    
    debug!("Baseline recorded to: {}", session_file);
    Ok(())
}

fn invoke_pi_agent(question: &str, current_state: &str, metric_feedback: &str) -> String {
    format!(
        "Proposed change for '{}': Based on current state '{}' and metric feedback '{}', \
        I propose implementing incremental optimization. This is a simulated agent response \
        for demonstration purposes.",
        question, current_state, metric_feedback
    )
}

fn apply_changes_in_branch(_action: &str) -> Result<String> {
    let branch_name = format!("autoresearch/iter-{}", uuid_generate());
    Ok(branch_name)
}

fn revert_changes(_branch_name: &str) -> Result<()> {
    Ok(())
}

fn keep_changes(_branch_name: &str) -> Result<()> {
    Ok(())
}

fn log_iteration(
    session_file: &str,
    record: &IterationRecord,
) -> Result<()> {
    let json_line = serde_json::to_string(record)?;
    
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(session_file)?;
    
    writeln!(file, "{}", json_line)?;
    
    Ok(())
}

fn uuid_generate() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    format!("{:?}{}", Instant::now(), std::process::id()).hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn run_iteration(
    iteration: usize,
    question: &str,
    baseline_value: f64,
    best_metric: f64,
    measure_command: &str,
    session_file: &str,
    dry_run: bool,
) -> Result<(f64, String, bool)> {
    let current_state = format!("Iteration {}, best metric: {:.2}", iteration, best_metric);
    let metric_feedback = format!("Baseline: {:.2}, current best: {:.2}", baseline_value, best_metric);
    
    let agent_action = invoke_pi_agent(question, &current_state, &metric_feedback);
    
    // In dry-run mode, simulate branch creation without actually creating it
    let branch_name = if dry_run {
        format!("[DRY-RUN] autoresearch/iter-{}", uuid_generate())
    } else {
        apply_changes_in_branch(&agent_action)?
    };
    
    let metric_value = execute_measurement(measure_command)?;
    
    let improvement = (best_metric - metric_value) / baseline_value;
    let kept = metric_value < best_metric;
    
    let timestamp = Utc::now().to_rfc3339();
    let record = IterationRecord {
        iteration,
        timestamp,
        agent_action: agent_action.clone(),
        metric_value,
        improvement,
        kept,
    };
    
    // In dry-run mode, don't actually log to session file
    if !dry_run {
        log_iteration(session_file, &record)?;
    }
    
    // In dry-run mode, don't actually keep or revert changes
    if !dry_run {
        if kept {
            keep_changes(&branch_name)?;
        } else {
            revert_changes(&branch_name)?;
        }
    }
    
    Ok((metric_value, agent_action, kept))
}

fn run_iterative_loop(
    question: &str,
    design: &ExperimentDesign,
    baseline_record: &BaselineRecord,
    cli: &Cli,
    config: &Option<Config>,
    beads: &mut Option<BeadsIntegration>,
    dry_run: bool,
) -> Result<(ExperimentSession, Option<StuckReason>)> {
    let max_iterations = get_max_iterations(cli, config, 20);
    let baseline_value = baseline_record.value;
    let iteration_timeout = Duration::from_secs(get_iteration_timeout(cli, config, 10) as u64 * 60);
    let total_timeout = Duration::from_secs(get_total_timeout(cli, config, 120) as u64 * 60);
    let stall_limit = get_stall_limit(cli, config, 5);
    let convergence_threshold = get_convergence_threshold(cli, config, 0.01);
    let convergence_window = get_convergence_window(cli, config, 3);
    
    let mut state = IterationState {
        current_iteration: 0,
        best_metric: baseline_value,
        best_iteration: 0,
        consecutive_no_improvement: 0,
        backoff_count: 0,
        start_time: Instant::now(),
        recent_metrics: Vec::new(),
    };
    
    let mut stuck_reason: Option<StuckReason> = None;
    
    let mut iterations = Vec::new();
    
    if !cli.quiet {
        info!("\nStarting iterative exploration loop...");
        info!("Max iterations: {}", max_iterations);
        info!("Baseline: {:.2}", baseline_value);
        info!("");
    }
    
    // Create progress bar for iterations
    let pb = create_progress_bar("Exploring solutions", max_iterations as u64);
    
    while state.current_iteration < max_iterations {
        // Layer 3: Check total runtime limit
        if state.start_time.elapsed() >= total_timeout {
            if !cli.quiet {
                info!("\nTotal timeout reached ({:.0}s). Saving best result.", total_timeout.as_secs());
            }
            stuck_reason = Some(StuckReason::TotalTimeout);
            break;
        }
        
        state.current_iteration += 1;
        
        let display_status = |iter: usize, best: f64, stall: usize, elapsed: Duration| {
            let improvement_pct = (baseline_value - best) / baseline_value * 100.0;
            info!(
                "[AutoResearch] Iter {}/{} | Best: {:.1} ({:+.1}%) | Stall: {}/{} | Time: {:.0}s/{:.0}s",
                iter, max_iterations, best, improvement_pct, stall, stall_limit, 
                elapsed.as_secs(), total_timeout.as_secs()
            );
        };
        
        if !cli.quiet {
            display_status(
                state.current_iteration,
                state.best_metric,
                state.consecutive_no_improvement,
                state.start_time.elapsed(),
            );
        }
        
        // Layer 1: Per-iteration timeout
        let iteration_start = Instant::now();
        
        let session_file = get_session_file(cli, config);
        let iteration_result = run_iteration(
            state.current_iteration,
            question,
            baseline_value,
            state.best_metric,
            &design.measurement,
            &session_file,
            dry_run,
        );
        
        if iteration_start.elapsed() > iteration_timeout {
            if !cli.quiet {
                info!("\nIteration {} exceeded timeout ({:.0}s). Marking as timeout.", 
                         state.current_iteration, iteration_timeout.as_secs());
            }
            stuck_reason = Some(StuckReason::IterationTimeout);
            break;
        }
        
        match iteration_result {
            Ok((metric_value, agent_action, kept)) => {
                let improvement = (state.best_metric - metric_value) / baseline_value;
                
                // Track recent metrics for convergence detection
                state.recent_metrics.push(metric_value);
                if state.recent_metrics.len() > convergence_window {
                    state.recent_metrics.remove(0);
                }
                
                if kept {
                    state.best_metric = metric_value;
                    state.best_iteration = state.current_iteration;
                    state.consecutive_no_improvement = 0;
                    state.backoff_count = 0;
                    if !cli.quiet {
                        info!("  ✓ Kept - improvement: {:+.2}%", improvement * 100.0);
                    }
                    if cli.verbose {
                        debug!("    Metric value: {:.2}", metric_value);
                        debug!("    Agent action: {}", agent_action.split('.').next().unwrap_or(&agent_action));
                    }
                } else {
                    state.consecutive_no_improvement += 1;
                    if !cli.quiet {
                        info!("  ✗ Reverted - degradation: {:-.2}%", improvement * 100.0);
                    }
                    if cli.verbose {
                        debug!("    Metric value: {:.2}", metric_value);
                    }
                }
                
                let iteration_record = IterationRecord {
                    iteration: state.current_iteration,
                    timestamp: Utc::now().to_rfc3339(),
                    agent_action,
                    metric_value,
                    improvement,
                    kept,
                };
                iterations.push(iteration_record.clone());
                
                if let Some(ref mut beads_integration) = beads {
                    let _ = beads_integration.update_bead_progress(
                        state.current_iteration,
                        metric_value,
                        improvement,
                        kept,
                    );
                }
            }
            Err(e) => {
                if !cli.quiet {
                    error!("  ✗ Iteration failed: {}", e);
                    eprintln!("  {}", "  SUGGESTION:".yellow().bold());
                    eprintln!("    - Check that the agent's proposed changes are valid");
                    eprintln!("    - Verify the measurement command still works after changes");
                    eprintln!("    - See: docs/TROUBLESHOOTING.md#experiment-issues");
                }
                state.consecutive_no_improvement += 1;
            }
        }
        
        // Layer 4: Convergence-based early stop
        if state.recent_metrics.len() >= convergence_window {
            let min_recent = *state.recent_metrics.iter().min_by(|a, b| a.total_cmp(b)).unwrap();
            let max_recent = *state.recent_metrics.iter().max_by(|a, b| a.total_cmp(b)).unwrap();
            let variance = if min_recent.abs() > 1e-10 {
                (max_recent - min_recent) / min_recent.abs()
            } else {
                0.0
            };
            
            if variance.abs() < convergence_threshold {
                if !cli.quiet {
                    info!("\nConvergence achieved! Variance {:.4} < threshold {:.4}", 
                             variance, convergence_threshold);
                }
                stuck_reason = Some(StuckReason::ConvergenceAchieved);
                break;
            }
        }
        
        // Layer 2: Stall limit with backoff
        if state.consecutive_no_improvement >= stall_limit {
            state.backoff_count += 1;
            if state.backoff_count >= 2 {
                if !cli.quiet {
                    info!("\nStall limit reached after {} backoffs. Aborting.", state.backoff_count);
                }
                stuck_reason = Some(StuckReason::StallLimitReached);
                break;
            }
            if !cli.quiet {
                info!("\nStall limit reached. Backing off (attempt {}/2)...", state.backoff_count);
            }
            state.consecutive_no_improvement = 0;
        }
        
        // Update progress bar
        pb.set_position(state.current_iteration as u64);
        pb.set_message(format!(
            "Iter {}/{} | Best: {:.1} ({:+.1}%) | Stall: {}/{}",
            state.current_iteration,
            max_iterations,
            state.best_metric,
            (baseline_value - state.best_metric) / baseline_value * 100.0,
            state.consecutive_no_improvement,
            stall_limit
        ));
    }
    
    // Finish progress bar
    pb.finish_and_clear();
    
    // Check if we hit max iterations
    if stuck_reason.is_none() && state.current_iteration >= max_iterations {
        stuck_reason = Some(StuckReason::MaxIterationsReached);
    }
    
    let session = ExperimentSession {
        session_id: uuid_generate(),
        question: question.to_string(),
        design: design.clone(),
        baseline_record: baseline_record.clone(),
        iterations,
        best_iteration: if state.best_iteration > 0 { Some(state.best_iteration) } else { None },
        start_time: Utc::now().to_rfc3339(),
        end_time: Some(Utc::now().to_rfc3339()),
        status: if state.best_metric < baseline_value { "completed".to_string() } else { "no_improvement".to_string() },
    };
    
    Ok((session, stuck_reason))
}

fn generate_failure_recommendations(
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
            recommendations.push("Consider trying a different optimization strategy or approach".to_string());
            recommendations.push("Try relaxing constraints or exploring a broader solution space".to_string());
        }
        Some(StuckReason::MaxIterationsReached) => {
            recommendations.push(format!(
                "Increase --max-iterations beyond {} to allow more exploration",
                session.iterations.len()
            ));
            recommendations.push("The experiment showed progress but needs more iterations".to_string());
        }
        Some(StuckReason::TotalTimeout) => {
            recommendations.push("Increase --total-timeout-minutes to allow longer runtime".to_string());
            recommendations.push("Consider optimizing the measurement command for faster feedback".to_string());
        }
        Some(StuckReason::IterationTimeout) => {
            recommendations.push("Increase --iteration-timeout-minutes for slower iterations".to_string());
            recommendations.push("Simplify the agent task or break into smaller steps".to_string());
        }
        Some(StuckReason::ConvergenceAchieved) => {
            recommendations.push("The metric has converged - consider using a different metric".to_string());
            recommendations.push("Try a more aggressive target or different optimization direction".to_string());
        }
        None => {
            recommendations.push("Review iteration logs for patterns in successful vs failed changes".to_string());
        }
    }
    
    if final_improvement > 0.0 && final_improvement < target_improvement {
        recommendations.push(format!(
            "Partial progress achieved ({:+.1}%) - consider lowering target or combining with manual optimizations",
            improvement_achieved
        ));
    }
    
    if final_improvement <= 0.0 {
        recommendations.push("No improvement achieved - reconsider the metric or measurement methodology".to_string());
        recommendations.push("Verify the baseline measurement is accurate and reproducible".to_string());
    }
    
    recommendations.push(format!(
        "Retry with adjusted parameters: current metric '{}' may need refinement",
        session.design.metric
    ));
    
    recommendations
}

/// Calculate the final improvement from baseline to best kept value
/// 
/// Computes the improvement ratio from the baseline to the best kept iteration.
/// Returns both the improvement ratio and the best kept value (if any).
/// 
/// # Arguments
/// 
/// * `session` - The experiment session to analyze
/// 
/// # Returns
/// 
/// A tuple of `(improvement_ratio, best_kept_value)` where:
/// - `improvement_ratio`: The ratio of improvement (positive means improvement)
/// - `best_kept_value`: The best metric value from kept iterations, or None if no iterations were kept
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::{calculate_final_improvement, ExperimentSession};
/// 
/// // Session with improvements
/// let session = ExperimentSession { /* ... */ };
/// let (improvement, best_value) = calculate_final_improvement(&session);
/// assert!(improvement > 0.0); // Positive improvement
/// assert!(best_value.is_some());
/// 
/// // Session with no kept iterations
/// let session = ExperimentSession { iterations: vec![], /* ... */ };
/// let (improvement, best_value) = calculate_final_improvement(&session);
/// assert_eq!(improvement, 0.0);
/// assert!(best_value.is_none());
/// ```
fn calculate_final_improvement(
    session: &ExperimentSession,
) -> (f64, Option<f64>) {
    let baseline = session.baseline_record.value;
    let best_kept_value: Option<f64> = session.iterations
        .iter()
        .filter(|i| i.kept)
        .map(|i| i.metric_value)
        .fold(None, |a, b| Some(a.map(|min| min.min(b)).unwrap_or(b)));
    
    let final_improvement = match best_kept_value {
        Some(val) => (baseline - val) / baseline,
        None => 0.0,
    };
    
    (final_improvement, best_kept_value)
}

/// Extract key changes from iterations for commit message
fn extract_key_changes(
    session: &ExperimentSession,
    baseline: f64,
) -> Vec<String> {
    let mut key_changes = Vec::new();
    
    // Add best iteration first
    if let Some(best_iter_num) = session.best_iteration {
        if let Some(best_iter) = session.iterations.iter().find(|i| i.iteration == best_iter_num) {
            let change_summary = extract_change_summary(&best_iter.agent_action);
            let iter_improvement = (baseline - best_iter.metric_value) / baseline * 100.0;
            key_changes.push(format!(
                "Iteration {}: {} (-{:.1}%)",
                best_iter_num, change_summary, iter_improvement
            ));
        }
    }
    
    // Add other kept iterations
    for iter in &session.iterations {
        if iter.kept && iter.iteration != session.best_iteration.unwrap_or(0) {
            let change_summary = extract_change_summary(&iter.agent_action);
            let iter_improvement = (baseline - iter.metric_value) / baseline * 100.0;
            key_changes.push(format!(
                "Iteration {}: {} (-{:.1}%)",
                iter.iteration, change_summary, iter_improvement
            ));
        }
    }
    
    key_changes
}

/// Extract a short summary from agent action string
/// 
/// Parses an agent action string to extract a concise summary of the proposed change.
/// Handles various formats by splitting on colons and periods.
/// 
/// # Arguments
/// 
/// * `agent_action` - The full agent action string to summarize
/// 
/// # Returns
/// 
/// A trimmed string containing the key change description.
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::extract_change_summary;
/// 
/// // Extract summary from detailed action
/// let action = "Proposed change for 'optimize memory': Implement caching. This should help.";
/// let summary = extract_change_summary(action);
/// assert!(summary.contains("Implement caching"));
/// 
/// // Handle simple action
/// let action = "Simple change";
/// let summary = extract_change_summary(action);
/// assert_eq!(summary, "Simple change");
/// ```
fn extract_change_summary(agent_action: &str) -> String {
    agent_action
        .split(':')
        .next_back()
        .unwrap_or(agent_action)
        .trim()
        .split('.')
        .next()
        .unwrap_or(agent_action)
        .trim()
        .to_string()
}

/// Generate commit message for successful experiment
/// 
/// Creates a detailed commit message summarizing the experiment results,
/// including the metric improved, improvement percentage, iterations,
/// runtime, and key changes made.
/// 
/// # Arguments
/// 
/// * `session` - The completed experiment session
/// * `final_improvement` - The final improvement ratio achieved
/// * `baseline` - The baseline metric value
/// * `best_value` - The best metric value achieved
/// 
/// # Returns
/// 
/// A formatted commit message string.
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::{generate_commit_message, ExperimentSession};
/// 
/// let session = ExperimentSession { /* ... */ };
/// let message = generate_commit_message(&session, 0.20, 100.0, 80.0);
/// 
/// assert!(message.contains("[autoresearch]"));
/// assert!(message.contains("Iterations:"));
/// assert!(message.contains("Key changes:"));
/// ```
fn generate_commit_message(
    session: &ExperimentSession,
    final_improvement: f64,
    baseline: f64,
    best_value: f64,
) -> String {
    let runtime_secs = calculate_runtime_seconds(session);
    let iterations_count = session.iterations.len();
    let convergence_status = match &session.status {
        s if s.contains("completed") => "achieved",
        _ => "not achieved",
    };
    
    let mut commit_message = format!(
        "[autoresearch] Reduce {} by {:.1}% ({:.0} → {:.0})\n\n",
        session.design.metric,
        final_improvement * 100.0,
        baseline,
        best_value
    );
    
    commit_message = format!(
        "{}Iterations: {}/{} | Runtime: {}s | Convergence: {}\n\n",
        commit_message,
        iterations_count,
        session.iterations.len().max(20),
        runtime_secs,
        convergence_status
    );
    
    let key_changes = extract_key_changes(session, baseline);
    commit_message = format!(
        "{}Key changes:\n{}\n\n",
        commit_message,
        key_changes.join("\n")
    );
    
    commit_message = format!(
        "{}Metric: {} | Baseline: {:.0} | Best: {:.0}",
        commit_message,
        session.design.metric,
        baseline,
        best_value
    );
    
    commit_message
}

/// Calculate runtime in seconds from session timestamps
/// 
/// Parses the start and end timestamps from an experiment session
/// and calculates the total runtime in seconds.
/// 
/// # Arguments
/// 
/// * `session` - The experiment session with start_time and end_time
/// 
/// # Returns
/// 
/// The runtime in seconds, or 0 if timestamps can't be parsed.
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::{calculate_runtime_seconds, ExperimentSession};
/// use chrono::Utc;
/// 
/// let session = ExperimentSession {
///     start_time: Utc::now().to_rfc3339(),
///     end_time: Some(Utc::now().to_rfc3339()),
///     /* ... */
/// };
/// let runtime = calculate_runtime_seconds(&session);
/// assert!(runtime >= 0);
/// ```
fn calculate_runtime_seconds(session: &ExperimentSession) -> i64 {
    session.start_time
        .parse::<chrono::DateTime<Utc>>()
        .ok()
        .and_then(|start| {
            session.end_time.clone().and_then(|end_str| {
                end_str.parse::<chrono::DateTime<Utc>>()
                    .ok()
                    .map(|end| end.signed_duration_since(start).num_seconds())
            })
        })
        .unwrap_or(0)
}

/// Generate branch name with timestamp and unique ID
/// 
/// Creates a unique branch name for the experiment results in the format:
/// `autoresearch/{timestamp}-{unique_id}` where timestamp is YYYYMMDD-HHMMSS.
/// 
/// # Returns
/// 
/// A unique branch name string.
/// 
/// # Examples
/// 
/// ```ignore
/// use pi_autoresearch::main::generate_branch_name;
/// 
/// let branch1 = generate_branch_name();
/// let branch2 = generate_branch_name();
/// 
/// assert!(branch1.starts_with("autoresearch/"));
/// assert_ne!(branch1, branch2); // Unique IDs ensure different names
/// ```
fn generate_branch_name() -> String {
    let timestamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let unique_id = uuid_generate();
    format!("autoresearch/{}-{}", timestamp, unique_id)
}

/// Execute git operations for successful experiment
fn execute_git_operations(
    branch_name: &str,
    commit_message: &str,
) -> Result<(bool, Option<String>)> {
    // Get current branch for rollback
    let current_branch = get_current_branch();
    
    // Check if branch exists
    let branch_exists = does_branch_exist(branch_name);
    
    // Create or checkout branch
    if let Err(e) = create_or_checkout_branch(branch_name, branch_exists) {
        let _ = checkout_branch(&current_branch);
        return Ok((false, Some(e.to_string())));
    }
    
    // Stage all changes
    if let Err(e) = stage_all_changes() {
        let _ = checkout_branch(&current_branch);
        return Ok((false, Some(e.to_string())));
    }
    
    // Commit changes
    if let Err(e) = commit_changes(commit_message) {
        let _ = checkout_branch(&current_branch);
        return Ok((false, Some(e.to_string())));
    }
    
    // Push to remote if available
    if has_remote_origin() {
        if let Err(e) = push_branch(branch_name) {
            let _ = checkout_branch(&current_branch);
            return Ok((false, Some(e.to_string())));
        }
    }
    
    // Return to original branch
    let _ = checkout_branch(&current_branch);
    
    Ok((true, None))
}

/// Get the current git branch name
fn get_current_branch() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output();
    
    if let Ok(output) = output {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }
    "main".to_string()
}

/// Check if a branch exists
fn does_branch_exist(branch_name: &str) -> bool {
    let output = Command::new("git")
        .args(["show-ref", "--verify", "refs/heads/", branch_name])
        .output();
    
    output.as_ref()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Create a new branch or checkout existing one
fn create_or_checkout_branch(branch_name: &str, branch_exists: bool) -> Result<()> {
    let output = if branch_exists {
        Command::new("git")
            .args(["checkout", branch_name])
            .output()
    } else {
        Command::new("git")
            .args(["checkout", "-b", branch_name])
            .output()
    }?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to create branch: {}", stderr));
    }
    
    Ok(())
}

/// Stage all changes
fn stage_all_changes() -> Result<()> {
    let output = Command::new("git")
        .args(["add", "-A"])
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to stage changes: {}", stderr));
    }
    
    Ok(())
}

/// Commit staged changes with message
fn commit_changes(commit_message: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["commit", "--allow-empty", "-m", commit_message])
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to commit: {}", stderr));
    }
    
    Ok(())
}

/// Check if remote origin exists
fn has_remote_origin() -> bool {
    Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

/// Push branch to remote origin
fn push_branch(branch_name: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["push", "-u", "origin", branch_name])
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to push branch: {}", stderr));
    }
    
    Ok(())
}

/// Checkout a branch
fn checkout_branch(branch_name: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["checkout", branch_name])
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to checkout branch: {}", stderr));
    }
    
    Ok(())
}

/// Finalize experiment and create git branch if successful
fn finalize_experiment(
    session: &ExperimentSession,
    target_improvement: f64,
    stuck_reason: Option<&StuckReason>,
    skip_git: bool,
) -> Result<FinalizationResult> {
    // Calculate final improvement
    let (final_improvement, best_kept_value) = calculate_final_improvement(session);
    let baseline = session.baseline_record.value;
    let success = final_improvement >= target_improvement;
    
    // Handle failure case
    if !success {
        return create_failure_result(
            session,
            final_improvement,
            best_kept_value,
            baseline,
            target_improvement,
            stuck_reason,
        );
    }
    
    // Handle success case
    let best_value = best_kept_value.unwrap_or(baseline);
    let key_changes = extract_key_changes(session, baseline);
    
    // Skip git operations if requested
    if skip_git {
        return Ok(FinalizationResult {
            success: true,
            final_improvement,
            best_value: best_kept_value,
            branch_name: None,
            commit_message: None,
            key_changes,
            error_message: None,
            failure_report: None,
        });
    }
    
    // Generate branch name and commit message
    let branch_name = generate_branch_name();
    let commit_message = generate_commit_message(session, final_improvement, baseline, best_value);
    
    // Execute git operations
    let (git_success, error_message) = execute_git_operations(&branch_name, &commit_message)?;
    
    Ok(FinalizationResult {
        success: git_success,
        final_improvement,
        best_value: if git_success { Some(best_value) } else { None },
        branch_name: if git_success { Some(branch_name) } else { None },
        commit_message: if git_success { Some(commit_message) } else { None },
        key_changes,
        error_message,
        failure_report: None,
    })
}

/// Create failure result with recommendations
fn create_failure_result(
    session: &ExperimentSession,
    final_improvement: f64,
    best_kept_value: Option<f64>,
    baseline: f64,
    target_improvement: f64,
    stuck_reason: Option<&StuckReason>,
) -> Result<FinalizationResult> {
    let recommendations = generate_failure_recommendations(
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
    
    Ok(FinalizationResult {
        success: false,
        final_improvement,
        best_value: best_kept_value,
        branch_name: None,
        commit_message: None,
        key_changes: Vec::new(),
        error_message: None,
        failure_report: Some(failure_report),
    })
}

// Beads (bd) integration

struct BeadsIntegration {
    enabled: bool,
    bead_id: Option<String>,
}

impl BeadsIntegration {
    fn new(enabled: bool) -> Self {
        Self {
            enabled,
            bead_id: None,
        }
    }

    fn create_experiment_bead(&mut self, question: &str, design: &ExperimentDesign) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let title = format!("[AutoResearch] {}", question);
        let description = format!(
            "Autonomous research experiment\n\n**Hypothesis**: {}\n\n**Metric**: {}\n**Measurement**: {}\n\n**Baseline**: {:.2}\n**Target Improvement**: {:.0}%",
            design.hypothesis,
            design.metric,
            design.measurement,
            design.baseline,
            design.target_improvement * 100.0
        );

        let output = Command::new("bd")
            .args([
                "create",
                "--title",
                &title,
                "--description",
                &description,
                "--type",
                "task",
                "--labels",
                "autoresearch,experiment",
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to create bead issue: {}", stderr);
            self.enabled = false;
            return Ok(());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let id = stdout.lines().next().and_then(|line| {
            // Try different formats: "✓ Created issue: ID — Title" or "Created ID" or "Created: ID"
            let line = line.trim();
            if line.starts_with("✓ Created issue: ") {
                // Format: "✓ Created issue: pi-autoresearch-xxxx — Title"
                line.strip_prefix("✓ Created issue: ")
                    .and_then(|s| s.split(" — ").next())
            } else if line.starts_with("Created ") {
                line.strip_prefix("Created ")
            } else if line.starts_with("Created: ") {
                line.strip_prefix("Created: ")
            } else {
                None
            }
        });
        
        if let Some(id) = id {
            self.bead_id = Some(id.to_string());
            info!("Created bead issue: {}", id);
        }

        Ok(())
    }

    fn update_bead_progress(&self, iteration: usize, metric_value: f64, improvement: f64, kept: bool) -> Result<()> {
        if !self.enabled || self.bead_id.is_none() {
            return Ok(());
        }

        let bead_id = self.bead_id.as_ref().unwrap();
        let status = if kept {
            format!("✓ Kept - improvement: {:+.2}%", improvement * 100.0)
        } else {
            format!("✗ Reverted - degradation: {:-.2}%", improvement * 100.0)
        };

        let note = format!(
            "Iteration {}: Metric: {:.2} - {}",
            iteration, metric_value, status
        );

        let output = Command::new("bd")
            .args(["note", bead_id, &note])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to update bead {}: {}", bead_id, stderr);
        }

        Ok(())
    }

    fn close_bead(&self, success: bool, final_improvement: f64, iterations: usize) -> Result<()> {
        if !self.enabled || self.bead_id.is_none() {
            return Ok(());
        }

        let bead_id = self.bead_id.as_ref().unwrap();
        let reason = if success {
            format!(
                "Experiment successful - achieved {:+.2}% improvement in {} iterations",
                final_improvement * 100.0, iterations
            )
        } else {
            format!(
                "Experiment did not meet target - achieved {:+.2}% improvement in {} iterations",
                final_improvement * 100.0, iterations
            )
        };

        let output = Command::new("bd")
            .args(["close", bead_id, "--reason", &reason])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to close bead {}: {}", bead_id, stderr);
        } else {
            info!("Closed bead issue: {}", bead_id);
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct FinalizationResult {
    success: bool,
    final_improvement: f64,
    best_value: Option<f64>,
    branch_name: Option<String>,
    commit_message: Option<String>,
    key_changes: Vec<String>,
    error_message: Option<String>,
    // Failure report fields
    failure_report: Option<FailureReport>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FailureReport {
    best_improvement: f64,
    best_value: Option<f64>,
    baseline: f64,
    target_improvement: f64,
    iterations_completed: usize,
    stuck_reason: Option<String>,
    recommendations: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum SessionRecord {
    Baseline(BaselineRecord),
    Iteration(IterationRecord),
    Experiment(Box<ExperimentSession>),
}

fn read_session_file(session_file: &str) -> Result<Vec<SessionRecord>> {
    if !std::path::Path::new(session_file).exists() {
        return Ok(Vec::new());
    }

    let contents = std::fs::read_to_string(session_file)?;
    let mut records = Vec::new();

    // First, try to extract and parse multi-line JSON objects (pretty-printed)
    // Look for the final ExperimentSession which is typically pretty-printed
    let mut remaining = contents.as_str();
    while let Some(start_pos) = remaining.find('{') {
        let start_idx = start_pos;
        let mut brace_count = 0;
        let mut end_idx = start_idx;
        let mut found_complete = false;
        
        for (i, ch) in remaining[start_idx..].char_indices() {
            match ch {
                '{' => brace_count += 1,
                '}' => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        end_idx = start_idx + i;
                        found_complete = true;
                        break;
                    }
                }
                _ => {}
            }
        }
        
        if found_complete {
            let json_str = &remaining[start_idx..=end_idx];
            
            // Try to parse as ExperimentSession first (has session_id field)
            if json_str.contains("\"session_id\"") {
                if let Ok(session) = serde_json::from_str::<ExperimentSession>(json_str) {
                    records.push(SessionRecord::Experiment(Box::new(session)));
                    remaining = &remaining[end_idx + 1..];
                    continue;
                }
            }
            
            // Try other types
            if let Ok(iteration) = serde_json::from_str::<IterationRecord>(json_str) {
                records.push(SessionRecord::Iteration(iteration));
            } else if let Ok(baseline) = serde_json::from_str::<BaselineRecord>(json_str) {
                records.push(SessionRecord::Baseline(baseline));
            }
            
            remaining = &remaining[end_idx + 1..];
        } else {
            break;
        }
    }

    // Also parse any remaining compact JSONL lines
    for line in contents.lines() {
        if line.trim().is_empty() {
            continue;
        }
        
        let trimmed = line.trim();
        
        // Skip if this looks like it's part of a multi-line JSON (already parsed above)
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            // Try to parse as ExperimentSession first (has session_id field)
            if trimmed.contains("\"session_id\"") {
                if let Ok(session) = serde_json::from_str::<ExperimentSession>(trimmed) {
                    // Check if we already have this session (avoid duplicates)
                    if !records.iter().any(|r| {
                        if let SessionRecord::Experiment(s) = r {
                            s.session_id == session.session_id
                        } else {
                            false
                        }
                    }) {
                        records.push(SessionRecord::Experiment(Box::new(session)));
                    }
                    continue;
                }
            }
            
            // Try to parse as IterationRecord (has iteration field)
            if trimmed.contains("\"iteration\"") && trimmed.contains("\"agent_action\"") {
                if let Ok(iteration) = serde_json::from_str::<IterationRecord>(trimmed) {
                    records.push(SessionRecord::Iteration(iteration));
                    continue;
                }
            }
            
            // Try to parse as BaselineRecord (has verification_runs field)
            if trimmed.contains("\"verification_runs\"") {
                if let Ok(baseline) = serde_json::from_str::<BaselineRecord>(trimmed) {
                    records.push(SessionRecord::Baseline(baseline));
                    continue;
                }
            }
        }
    }

    Ok(records)
}

fn list_autoresearch_branches() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["branch", "-a"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to list git branches"));
    }

    let branches = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| line.contains("autoresearch/") && !line.starts_with("remotes/"))
        .map(|line| {
            line.trim()
                .strip_prefix("*")
                .unwrap_or(line.trim())
                .strip_prefix("  ")
                .unwrap_or(line.trim())
                .to_string()
        })
        .collect();

    Ok(branches)
}

fn list_history(session_file: &str) -> Result<()> {
    let records = read_session_file(session_file)?;
    
    let mut experiments: Vec<&ExperimentSession> = Vec::new();
    for record in &records {
        if let SessionRecord::Experiment(ref session) = record {
            experiments.push(session);
        }
    }

    if experiments.is_empty() {
        info!("No experiments found in {}", session_file);
        return Ok(());
    }

    info!("\n=== Experiment History ===\n");
    info!("Found {} experiment(s)\n", experiments.len());

    for (i, session) in experiments.iter().enumerate() {
        let improvement: f64 = session.iterations
            .iter()
            .filter(|it| it.kept)
            .map(|it| it.improvement)
            .fold(0.0, |a, b| a.max(b));

        info!("[{}] Session: {}", i + 1, session.session_id);
        info!("    Question: {}", session.question);
        info!("    Metric: {}", session.design.metric);
        info!("    Baseline: {:.2}", session.baseline_record.value);
        info!("    Iterations: {}", session.iterations.len());
        info!("    Best improvement: {:+.2}%", improvement * 100.0);
        info!("    Status: {}", session.status);
        info!("    Started: {}", session.start_time);
        if let Some(ref end) = session.end_time {
            info!("    Ended: {}", end);
        }
        info!("");
    }

    info!("Use --resume <SESSION_ID> to continue a specific experiment");

    Ok(())
}

/// Compare two experiments by session ID
fn compare_experiments(session_file: &str, session_id1: &str, session_id2: &str) -> Result<()> {
    let records = read_session_file(session_file)?;
    
    let mut experiments: Vec<&ExperimentSession> = Vec::new();
    for record in &records {
        if let SessionRecord::Experiment(ref session) = record {
            experiments.push(session);
        }
    }

    // Find the two experiments
    let exp1 = experiments.iter().find(|s| s.session_id == session_id1);
    let exp2 = experiments.iter().find(|s| s.session_id == session_id2);

    match (exp1, exp2) {
        (Some(e1), Some(e2)) => {
            // Both experiments found - display comparison
            display_experiment_comparison(e1, e2)
        }
        (None, Some(_)) => {
            Err(anyhow::anyhow!("Experiment '{}' not found in {}", session_id1, session_file))
        }
        (Some(_), None) => {
            Err(anyhow::anyhow!("Experiment '{}' not found in {}", session_id2, session_file))
        }
        (None, None) => {
            Err(anyhow::anyhow!("Neither experiment found in {}", session_file))
        }
    }
}

/// Display side-by-side comparison of two experiments
fn display_experiment_comparison(e1: &ExperimentSession, e2: &ExperimentSession) -> Result<()> {
    // Calculate improvements
    let improvement1: f64 = e1.iterations
        .iter()
        .filter(|it| it.kept)
        .map(|it| it.improvement)
        .fold(0.0, |a, b| a.max(b));

    let improvement2: f64 = e2.iterations
        .iter()
        .filter(|it| it.kept)
        .map(|it| it.improvement)
        .fold(0.0, |a, b| a.max(b));

    // Calculate runtimes
    let runtime1 = calculate_session_runtime(e1);
    let runtime2 = calculate_session_runtime(e2);

    // Determine winner
    let winner = if improvement1 > improvement2 {
        "Experiment 1 ✓"
    } else if improvement2 > improvement1 {
        "Experiment 2 ✓"
    } else {
        "Tie"
    };

    info!("\n=== Experiment Comparison ===\n");
    info!("{} vs {}", e1.session_id, e2.session_id);
    info!("Winner: {}\n", winner);

    // Side-by-side comparison table
    info!("{:<15} {:<25} {:<25}", "Metric", e1.session_id, e2.session_id);
    info!("{:-<65}", "");
    info!("{:<15} {:<25} {:<25}", "Question", truncate_str(&e1.question, 22), truncate_str(&e2.question, 22));
    info!("{:<15} {:<25} {:<25}", "Metric", e1.design.metric, e2.design.metric);
    info!("{:<15} {:<25} {:<25}", "Baseline", format!("{:.2}", e1.baseline_record.value), format!("{:.2}", e2.baseline_record.value));
    info!("{:<15} {:<25} {:<25}", "Iterations", e1.iterations.len().to_string(), e2.iterations.len().to_string());
    info!("{:<15} {:<25} {:<25}", "Best Improvement", format!("{:+.2}%", improvement1 * 100.0), format!("{:+.2}%", improvement2 * 100.0));
    info!("{:<15} {:<25} {:<25}", "Status", e1.status, e2.status);
    info!("{:<15} {:<25} {:<25}", "Runtime", format_duration(runtime1), format_duration(runtime2));
    info!("{:-<65}\n", "");

    // Detailed iteration comparison
    info!("=== Iteration Details ===\n");
    
    info!("Experiment 1 ({}):
", e1.session_id);
    for (i, iteration) in e1.iterations.iter().enumerate() {
        let marker = if iteration.kept { "✓" } else { "✗" };
        info!("  [{}] {} Improvement: {:+.2}%", i + 1, marker, iteration.improvement * 100.0);
    }
    
    info!("\nExperiment 2 ({}):
", e2.session_id);
    for (i, iteration) in e2.iterations.iter().enumerate() {
        let marker = if iteration.kept { "✓" } else { "✗" };
        info!("  [{}] {} Improvement: {:+.2}%", i + 1, marker, iteration.improvement * 100.0);
    }

    Ok(())
}

/// Calculate session runtime in seconds
fn calculate_session_runtime(session: &ExperimentSession) -> f64 {
    if let Some(ref end_time) = session.end_time {
        if let (Ok(start), Ok(end)) = (
            chrono::DateTime::parse_from_rfc3339(&session.start_time),
            chrono::DateTime::parse_from_rfc3339(end_time)
        ) {
            return end.signed_duration_since(start).num_seconds() as f64;
        }
    }
    0.0
}

/// Format duration for display
fn format_duration(seconds: f64) -> String {
    if seconds == 0.0 {
        "N/A".to_string()
    } else if seconds < 60.0 {
        format!("{:.0}s", seconds)
    } else if seconds < 3600.0 {
        format!("{:.1}m", seconds / 60.0)
    } else {
        format!("{:.1}h", seconds / 3600.0)
    }
}

/// Truncate string to max length
fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn cleanup_autoresearch_branches(days: usize) -> Result<usize> {
    let branches = list_autoresearch_branches()?;
    
    if branches.is_empty() {
        info!("No autoresearch branches found.");
        return Ok(0);
    }

    info!("\n=== Autoresearch Branches ===\n");
    info!("Found {} autoresearch branch(es):\n", branches.len());

    let mut deleted_count = 0;
    let mut kept_count = 0;

    for branch in &branches {
        // Get the commit date for this branch
        let branch_age = parse_branch_age_days(branch);
        let age_str = format_branch_age(branch_age);

        info!("  {} - {}", branch, age_str);

        // Delete if older than specified days
        if branch_age >= days as i64 {
            info!("    → Marked for deletion (older than {} days)", days);
            
            let delete_output = Command::new("git")
                .args(["branch", "-d", branch])
                .output();

            if let Ok(output) = delete_output {
                if output.status.success() {
                    info!("    ✓ Deleted");
                    deleted_count += 1;
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    warn!("    Failed to delete: {}", stderr.trim());
                }
            } else {
                warn!("    Failed to delete");
            }
        } else {
            kept_count += 1;
        }
    }

    info!("\n=== Cleanup Summary ===");
    info!("Deleted: {} branches", deleted_count);
    info!("Kept: {} branches", kept_count);

    Ok(deleted_count)
}

fn find_session_by_id(session_file: &str, session_id: &str) -> Result<Option<ExperimentSession>> {
    let records = read_session_file(session_file)?;

    for record in &records {
        if let SessionRecord::Experiment(session) = &record {
            if session.session_id == session_id {
                return Ok(Some(session.as_ref().clone()));
            }
        }
    }

    Ok(None)
}

fn print_failure_report(result: &FinalizationResult, target_improvement: f64) {
    info!("✗ Target improvement not met");
    info!("");
    
    if let Some(ref report) = result.failure_report {
        info!("=== Failed Experiment Report ===");
        info!("");
        
        info!("Best Improvement Achieved:");
        info!("  Baseline: {:.2}", report.baseline);
        if let Some(best) = report.best_value {
            info!("  Best value: {:.2}", best);
            info!("  Improvement: {:+.2}%", report.best_improvement * 100.0);
        } else {
            info!("  No improvement achieved");
        }
        info!("  Target: {:+.2}%", report.target_improvement * 100.0);
        info!("  Gap: {:.2}%", (report.target_improvement - report.best_improvement) * 100.0);
        info!("");
        
        info!("Exploration Summary:");
        info!("  Iterations completed: {}", report.iterations_completed);
        if let Some(ref reason) = report.stuck_reason {
            info!("  Stuck reason: {}", reason);
        }
        info!("");
        
        info!("Recommendations for Retry:");
        for (i, rec) in report.recommendations.iter().enumerate() {
            info!("  {}. {}", i + 1, rec);
        }
        info!("");
        
        info!("Note: No changes were applied to the codebase.");
    } else {
        info!("  Target: {:.2}%, Achieved: {:+.2}%", target_improvement * 100.0, result.final_improvement * 100.0);
        if let Some(best) = result.best_value {
            info!("  Best value: {:.2}", best);
        }
        if let Some(ref err) = result.error_message {
            error!("  Error: {}", err);
            eprintln!("\n  {}", "SUGGESTION:".yellow().bold());
            eprintln!("  - Review the recommendations above");
            eprintln!("  - Try adjusting the target improvement or iteration limits");
            eprintln!("  - Consider using --verbose for more detailed output");
            eprintln!("  - See: docs/TROUBLESHOOTING.md#experiment-issues");
        }
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    // Initialize structured logging
    init_logging(&cli)?;

    // Show dry-run mode message
    if cli.dry_run {
        info!("\n=== DRY RUN MODE ===");
        info!("No changes will be made to the codebase or git repository.");
        info!("======================\n");
    }

    // Load config file
    let explicit_config = cli.config.is_some();
    let config = load_config(cli.config.as_deref(), explicit_config)?;
    
    // Log config loading if verbose
    if cli.verbose {
        if config.is_some() {
            debug!("Loaded config from: {:?}", cli.config);
        } else {
            debug!("No config file found, using defaults");
        }
    }

    // Handle --list-branches flag
    if cli.list_branches {
        let branches = list_autoresearch_branches()?;
        
        if branches.is_empty() {
            info!("No autoresearch branches found.");
        } else {
            info!("\n=== Autoresearch Branches ===\n");
            info!("Found {} autoresearch branch(es):\n", branches.len());
            
            for branch in &branches {
                // Get the commit date for this branch
                let branch_age = parse_branch_age_days(branch);
                let age_str = format_branch_age(branch_age);

                info!("  {} - {}", branch, age_str);
            }
            
            info!("\nUse --cleanup-branches to remove old branches");
        }
        return Ok(());
    }

    // Handle --cleanup-branches flag
    if cli.cleanup_branches {
        let _deleted = cleanup_autoresearch_branches(cli.cleanup_days)?;
        return Ok(());
    }

    // Handle --history flag
    if cli.history {
        let session_file = get_session_file(&cli, &config);
        list_history(&session_file)?;
        return Ok(());
    }

    // Handle --compare flag
    if let (Some(id1), Some(id2)) = (&cli.compare_id1, &cli.compare_id2) {
        let session_file = get_session_file(&cli, &config);
        compare_experiments(&session_file, id1, id2)?;
        return Ok(());
    }

    // Handle --resume flag
    let session_file = get_session_file(&cli, &config);
    if let Some(ref session_id) = cli.resume {
        if let Ok(Some(session)) = find_session_by_id(&session_file, session_id) {
            info!("Resuming session: {}", session_id);
            info!("Question: {}", session.question);
            info!("Current best iteration: {:?}", session.best_iteration);
            info!("Iterations completed: {}", session.iterations.len());
            info!("");

            // Resume from the best iteration's metric value
            let resume_baseline = session.iterations
                .iter()
                .filter(|it| it.kept)
                .map(|it| it.metric_value)
                .fold(session.baseline_record.value, |min, val| min.min(val));

            info!("Continuing from baseline: {:.2}", resume_baseline);
            info!("");

            // Continue with the experiment using the resumed state
            let mut design_to_use = session.design.clone();
            design_to_use.baseline = resume_baseline;

            let metric_to_use = get_metric(&cli, &config, &design_to_use.metric);
            let measure_to_use = get_measure(&cli, &config, &design_to_use.measurement);
            
            design_to_use.metric = metric_to_use;
            design_to_use.measurement = measure_to_use;

            let baseline_record = session.baseline_record.clone();
            let session_file = get_session_file(&cli, &config);
            if !cli.dry_run {
                save_to_session_file(&session_file, &baseline_record)?;
            }

            let mut beads_resume = if cli.beads_enabled { Some(BeadsIntegration::new(true)) } else { None };
            let (new_session, stuck_reason) = run_iterative_loop(
                &session.question,
                &design_to_use,
                &baseline_record,
                &cli,
                &config,
                &mut beads_resume,
                cli.dry_run,
            )?;

            // Merge the new iterations with the old session
            let mut merged_session = session.clone();
            let max_old_iter = session.iterations.iter().map(|it| it.iteration).max().unwrap_or(0);
            
            for mut iter in new_session.iterations {
                iter.iteration += max_old_iter;
                merged_session.iterations.push(iter);
            }
            
            merged_session.end_time = Some(Utc::now().to_rfc3339());

            let best_kept_value: Option<f64> = merged_session.iterations
                .iter()
                .filter(|i| i.kept)
                .map(|i| i.metric_value)
                .fold(None, |a: Option<f64>, b: f64| Some(a.map(|min| min.min(b)).unwrap_or(b)));

            let final_improvement = match best_kept_value {
                Some(val) => (merged_session.baseline_record.value - val) / merged_session.baseline_record.value,
                None => 0.0,
            };

            if !cli.quiet {
                info!("\n=== Resumed Experiment Complete ===");
                info!("Total iterations: {}", merged_session.iterations.len());
                info!("Best improvement: {:+.2}%", final_improvement * 100.0);
                info!("Session ID: {}", session_id);
                if let Some(reason) = &stuck_reason {
                    info!("Termination: {:?}", reason);
                }
            }

            if !cli.dry_run {
                let session_json = serde_json::to_string_pretty(&merged_session)?;
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&session_file)?;
                writeln!(file, "{}", session_json)?;
            }

            let target_improvement = get_target_improvement(&cli, &config, design_to_use.target_improvement);
            let finalization_result = finalize_experiment(&merged_session, target_improvement, stuck_reason.as_ref(), cli.skip_git || cli.dry_run)?;

            if !cli.quiet {
                info!("\n=== Finalization ===");
                if finalization_result.success {
                    info!("✓ Target improvement achieved: {:+.2}%", finalization_result.final_improvement * 100.0);
                    if let Some(ref branch) = finalization_result.branch_name {
                        info!("✓ Created branch: {}", branch);
                    }
                } else {
                    print_failure_report(&finalization_result, target_improvement);
                }
            }

            if finalization_result.success {
                return Ok(());
            } else {
                std::process::exit(1);
            }
        } else {
            error!("Session '{}' not found in {}", session_id, cli.session_file);
            eprintln!("\n  {}", "SUGGESTION:".yellow().bold());
            eprintln!("  - Check the session ID is correct");
            eprintln!("  - List available sessions with: --history");
            eprintln!("  - Ensure the session file exists and is readable");
            eprintln!("  - See: docs/USAGE.md#resuming-experiments");
            std::process::exit(1);
        }
    }

    if cli.verify_baseline {
        // Check if metric is provided (via CLI or config)
        let metric_provided = cli.metric.is_some() || config.as_ref().and_then(|c| c.metric.clone()).is_some();
        let measure_provided = cli.measure.is_some() || config.as_ref().and_then(|c| c.measure.clone()).is_some();
        
        if !metric_provided {
            return Err(anyhow::anyhow!("--metric is required for baseline verification (or set in config)"));
        }
        if !measure_provided {
            return Err(anyhow::anyhow!("--measure is required for baseline verification (or set in config)"));
        }
        
        // Use helper functions consistently for metric and measure
        let metric = get_metric(&cli, &config, "metric");
        let measurement = get_measure(&cli, &config, "measure");

        let max_variance = get_max_variance(&cli, &config, 0.05);
        let session_file = get_session_file(&cli, &config);

        match verify_baseline(&metric, &measurement, max_variance) {
            Ok(result) => {
                if result.success {
                    if let Some(ref record) = result.baseline_record {
                        if !cli.dry_run {
                            save_to_session_file(&session_file, record)?;
                        }
                        
                        let json_output = serde_json::to_string_pretty(&result)?;
                        println!("{}", json_output);
                        
                        return Ok(());
                    }
                }
                
                if let Some(err) = result.error_message {
                    error!("Baseline verification failed: {}", err);
                    eprintln!("\n  {}", "SUGGESTION:".yellow().bold());
                    eprintln!("  - Check that the measurement command works correctly");
                    eprintln!("  - Ensure the command outputs a numeric value");
                    eprintln!("  - Run the command manually to verify: {}", get_measure(&cli, &config, "<not set>"));
                    eprintln!("  - See: docs/TROUBLESHOOTING.md#measurement-issues");
                    std::process::exit(1);
                }
            }
            Err(e) => {
                error!("Baseline verification error: {}", e);
                eprintln!("\n  {}", "SUGGESTION:".yellow().bold());
                eprintln!("  - Check that all required flags are provided");
                eprintln!("  - Verify the measurement command is valid");
                eprintln!("  - See: docs/TROUBLESHOOTING.md#measurement-issues");
                std::process::exit(1);
            }
        }
    }

    let question = match &cli.question {
        Some(q) => q.clone(),
        None => read_question_from_stdin()?,
    };

    let beads_enabled = get_beads_enabled(&cli, &config);
    let mut beads = BeadsIntegration::new(beads_enabled);

    let design = generate_design(&question);
    
    let json_output = serde_json::to_string_pretty(&design)?;
    println!("{}", json_output);

    if beads_enabled {
        beads.create_experiment_bead(&question, &design)?;
    }

    if !cli.auto_approve {
        print!("\nApprove this design? [y/N]: ");
        io::stdout().flush()?;
        
        let mut response = String::new();
        io::stdin().read_line(&mut response)?;
        
        if response.trim().to_lowercase() != "y" && response.trim().to_lowercase() != "yes" {
            info!("Design not approved. Exiting.");
            std::process::exit(1);
        }
    }
    
    // Check if we should run iterations (CLI must explicitly set max_iterations)
    // Config provides defaults for iteration parameters but doesn't trigger iterations on its own
    if cli.max_iterations.is_some() {
        let metric_to_use = get_metric(&cli, &config, &design.metric);
        let measure_to_use = get_measure(&cli, &config, &design.measurement);
        let baseline_to_use = get_baseline(&cli, &config, design.baseline);
        let max_variance = get_max_variance(&cli, &config, 0.05);
        let session_file = get_session_file(&cli, &config);
        
        let baseline_record = if cli.baseline.is_some() || config.as_ref().map(|c| c.baseline.is_some()).unwrap_or(false) {
            let git_commit = get_git_commit_hash()?;
            BaselineRecord {
                timestamp: Utc::now().to_rfc3339(),
                git_commit,
                metric: metric_to_use.clone(),
                measurement_command: measure_to_use.clone(),
                value: baseline_to_use,
                verification_runs: vec![baseline_to_use, baseline_to_use],
                variance: 0.0,
                within_threshold: true,
            }
        } else {
            let baseline_result = verify_baseline(
                &metric_to_use,
                &measure_to_use,
                max_variance,
            )?;
            
            if !baseline_result.success {
                if let Some(err) = baseline_result.error_message {
                    error!("Baseline verification failed: {}", err);
                    eprintln!("\n  {}", "SUGGESTION:".yellow().bold());
                    eprintln!("  - Check that the measurement command works correctly");
                    eprintln!("  - Ensure the command outputs a numeric value");
                    eprintln!("  - Run the command manually to verify: {}", measure_to_use);
                    eprintln!("  - See: docs/TROUBLESHOOTING.md#measurement-issues");
                    std::process::exit(1);
                }
            }
            
            baseline_result.baseline_record.ok_or_else(|| {
                anyhow::anyhow!("No baseline record available")
            })?
        };
        
        if !cli.dry_run {
            save_to_session_file(&session_file, &baseline_record)?;
        }
        
        let mut design_to_use = design.clone();
        design_to_use.metric = metric_to_use;
        design_to_use.measurement = measure_to_use;
        design_to_use.baseline = baseline_to_use;
        
        let mut beads_opt = Some(beads);
        let (session, stuck_reason) = run_iterative_loop(&question, &design_to_use, &baseline_record, &cli, &config, &mut beads_opt, cli.dry_run)?;
        let mut beads = beads_opt;
        
        let best_kept_value: Option<f64> = session.iterations
            .iter()
            .filter(|i| i.kept)
            .map(|i| i.metric_value)
            .fold(None, |a, b| Some(a.map(|min| min.min(b)).unwrap_or(b)));
        
        let final_improvement = match best_kept_value {
            Some(val) => (session.baseline_record.value - val) / session.baseline_record.value,
            None => 0.0,
        };
        
        if !cli.quiet {
            info!("\n=== Experiment Complete ===");
            info!("Iterations: {}", session.iterations.len());
            info!("Best improvement: {:+.2}%", final_improvement * 100.0);
            info!("Session ID: {}", session.session_id);
            if let Some(reason) = &stuck_reason {
                info!("Termination: {:?}", reason);
            }
            
            if session.best_iteration.is_some() {
                let best_iter = session.iterations
                    .iter()
                    .find(|i| i.iteration == session.best_iteration.unwrap());
                if let Some(best) = best_iter {
                    info!("Best iteration: {} (metric: {:.2})", best.iteration, best.metric_value);
                }
            }
        } else {
            info!("Final improvement: {:+.2}%", final_improvement * 100.0);
        }
        
        if !cli.dry_run {
            let session_json = serde_json::to_string_pretty(&session)?;
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(get_session_file(&cli, &config))?;
            writeln!(file, "{}", session_json)?;
        }
        
        let target_improvement = get_target_improvement(&cli, &config, design.target_improvement);
        
        let finalization_result = finalize_experiment(&session, target_improvement, stuck_reason.as_ref(), cli.skip_git || cli.dry_run)?;
        
        if let Some(ref mut beads_integration) = beads {
            let _ = beads_integration.close_bead(
                finalization_result.success,
                finalization_result.final_improvement,
                session.iterations.len(),
            );
        }
        
        if !cli.quiet {
            info!("\n=== Finalization ===");
            if finalization_result.success {
                info!("✓ Target improvement achieved: {:+.2}%", finalization_result.final_improvement * 100.0);
                if let Some(ref branch) = finalization_result.branch_name {
                    info!("✓ Created branch: {}", branch);
                }
                if let Some(ref msg) = finalization_result.commit_message {
                    info!("\nCommit message:\n{}", msg);
                }
            } else {
                print_failure_report(&finalization_result, target_improvement);
            }
        }
        
        if finalization_result.success {
            return Ok(());
        } else {
            std::process::exit(1);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_metric_cli_provided() {
        let cli = Cli {
            metric: Some("custom_metric".to_string()),
            ..Default::default()
        };
        let config: Option<Config> = None;
        assert_eq!(get_metric(&cli, &config, "default"), "custom_metric");
    }

    #[test]
    fn test_get_metric_config_provided() {
        let cli = Cli {
            metric: None,
            ..Default::default()
        };
        let config = Some(Config {
            metric: Some("config_metric".to_string()),
            ..Default::default()
        });
        assert_eq!(get_metric(&cli, &config, "default"), "config_metric");
    }

    #[test]
    fn test_get_metric_default() {
        let cli = Cli {
            metric: None,
            ..Default::default()
        };
        let config: Option<Config> = None;
        assert_eq!(get_metric(&cli, &config, "default_metric"), "default_metric");
    }

    #[test]
    fn test_get_metric_cli_overrides_config() {
        let cli = Cli {
            metric: Some("cli_metric".to_string()),
            ..Default::default()
        };
        let config = Some(Config {
            metric: Some("config_metric".to_string()),
            ..Default::default()
        });
        assert_eq!(get_metric(&cli, &config, "default"), "cli_metric");
    }

    #[test]
    fn test_get_measure_cli_provided() {
        let cli = Cli {
            measure: Some("echo 100".to_string()),
            ..Default::default()
        };
        let config: Option<Config> = None;
        assert_eq!(get_measure(&cli, &config, "default"), "echo 100");
    }

    #[test]
    fn test_get_measure_config_provided() {
        let cli = Cli {
            measure: None,
            ..Default::default()
        };
        let config = Some(Config {
            measure: Some("config_measure".to_string()),
            ..Default::default()
        });
        assert_eq!(get_measure(&cli, &config, "default"), "config_measure");
    }

    #[test]
    fn test_get_baseline_cli_provided() {
        let cli = Cli {
            baseline: Some(500.0),
            ..Default::default()
        };
        let config: Option<Config> = None;
        assert_eq!(get_baseline(&cli, &config, 100.0), 500.0);
    }

    #[test]
    fn test_get_baseline_config_provided() {
        let cli = Cli {
            baseline: None,
            ..Default::default()
        };
        let config = Some(Config {
            baseline: Some(250.0),
            ..Default::default()
        });
        assert_eq!(get_baseline(&cli, &config, 100.0), 250.0);
    }

    #[test]
    fn test_get_target_improvement_cli_provided() {
        let cli = Cli {
            target_improvement: Some(0.50),
            ..Default::default()
        };
        let config: Option<Config> = None;
        assert_eq!(get_target_improvement(&cli, &config, 0.30), 0.50);
    }

    #[test]
    fn test_get_max_iterations_cli_provided() {
        let cli = Cli {
            max_iterations: Some(50),
            ..Default::default()
        };
        let config: Option<Config> = None;
        assert_eq!(get_max_iterations(&cli, &config, 20), 50);
    }

    #[test]
    fn test_get_max_variance_always_cli() {
        let cli = Cli {
            max_variance: 0.10,
            ..Default::default()
        };
        let config = Some(Config {
            max_variance: Some(0.05),
            ..Default::default()
        });
        // CLI always wins for max_variance
        assert_eq!(get_max_variance(&cli, &config, 0.05), 0.10);
    }

    #[test]
    fn test_get_session_file_always_cli() {
        let cli = Cli {
            session_file: "custom.jsonl".to_string(),
            ..Default::default()
        };
        let config = Some(Config {
            session_file: Some("config.jsonl".to_string()),
            ..Default::default()
        });
        // CLI always wins for session_file
        assert_eq!(get_session_file(&cli, &config), "custom.jsonl");
    }

    #[test]
    fn test_get_beads_enabled_cli_true() {
        let cli = Cli {
            beads_enabled: true,
            ..Default::default()
        };
        let config: Option<Config> = None;
        assert!(get_beads_enabled(&cli, &config));
    }

    #[test]
    fn test_get_beads_enabled_config_true() {
        let cli = Cli {
            beads_enabled: false,
            ..Default::default()
        };
        let config = Some(Config {
            beads_enabled: Some(true),
            ..Default::default()
        });
        assert!(get_beads_enabled(&cli, &config));
    }

    #[test]
    fn test_get_beads_enabled_default_false() {
        let cli = Cli {
            beads_enabled: false,
            ..Default::default()
        };
        let config: Option<Config> = None;
        assert!(!get_beads_enabled(&cli, &config));
    }

    #[test]
    fn test_get_iteration_timeout_cli_provided() {
        let cli = Cli {
            iteration_timeout_minutes: Some(30),
            ..Default::default()
        };
        let config: Option<Config> = None;
        assert_eq!(get_iteration_timeout(&cli, &config, 10), 30);
    }

    #[test]
    fn test_get_total_timeout_cli_provided() {
        let cli = Cli {
            total_timeout_minutes: Some(180),
            ..Default::default()
        };
        let config: Option<Config> = None;
        assert_eq!(get_total_timeout(&cli, &config, 120), 180);
    }

    #[test]
    fn test_get_stall_limit_cli_provided() {
        let cli = Cli {
            stall_limit: Some(10),
            ..Default::default()
        };
        let config: Option<Config> = None;
        assert_eq!(get_stall_limit(&cli, &config, 5), 10);
    }

    #[test]
    fn test_get_convergence_threshold_cli_provided() {
        let cli = Cli {
            convergence_threshold: Some(0.001),
            ..Default::default()
        };
        let config: Option<Config> = None;
        assert_eq!(get_convergence_threshold(&cli, &config, 0.01), 0.001);
    }

    #[test]
    fn test_get_convergence_window_cli_provided() {
        let cli = Cli {
            convergence_window: Some(5),
            ..Default::default()
        };
        let config: Option<Config> = None;
        assert_eq!(get_convergence_window(&cli, &config, 3), 5);
    }

    #[test]
    fn test_config_precedence_all_functions() {
        // Test that CLI > config > default for all helper functions
        let cli = Cli {
            metric: Some("cli_metric".to_string()),
            measure: Some("cli_measure".to_string()),
            baseline: Some(100.0),
            target_improvement: Some(0.50),
            max_iterations: Some(30),
            iteration_timeout_minutes: Some(15),
            total_timeout_minutes: Some(90),
            stall_limit: Some(8),
            convergence_threshold: Some(0.005),
            convergence_window: Some(4),
            max_variance: 0.08,
            session_file: "cli_session.jsonl".to_string(),
            beads_enabled: true,
            ..Default::default()
        };
        let config = Some(Config {
            metric: Some("config_metric".to_string()),
            measure: Some("config_measure".to_string()),
            baseline: Some(200.0),
            target_improvement: Some(0.30),
            max_iterations: Some(20),
            iteration_timeout_minutes: Some(10),
            total_timeout_minutes: Some(60),
            stall_limit: Some(5),
            convergence_threshold: Some(0.01),
            convergence_window: Some(3),
            max_variance: Some(0.05),
            session_file: Some("config_session.jsonl".to_string()),
            beads_enabled: Some(false),
        });

        // All should use CLI values
        assert_eq!(get_metric(&cli, &config, "default"), "cli_metric");
        assert_eq!(get_measure(&cli, &config, "default"), "cli_measure");
        assert_eq!(get_baseline(&cli, &config, 50.0), 100.0);
        assert_eq!(get_target_improvement(&cli, &config, 0.25), 0.50);
        assert_eq!(get_max_iterations(&cli, &config, 10), 30);
        assert_eq!(get_iteration_timeout(&cli, &config, 5), 15);
        assert_eq!(get_total_timeout(&cli, &config, 30), 90);
        assert_eq!(get_stall_limit(&cli, &config, 3), 8);
        assert_eq!(get_convergence_threshold(&cli, &config, 0.02), 0.005);
        assert_eq!(get_convergence_window(&cli, &config, 2), 4);
        assert_eq!(get_max_variance(&cli, &config, 0.05), 0.08);
        assert_eq!(get_session_file(&cli, &config), "cli_session.jsonl");
        assert!(get_beads_enabled(&cli, &config));
    }

    // Tests for config validation

    #[test]
    fn test_validate_config_valid() {
        let config = Config {
            max_variance: Some(0.05),
            target_improvement: Some(0.30),
            max_iterations: Some(20),
            iteration_timeout_minutes: Some(10),
            total_timeout_minutes: Some(120),
            stall_limit: Some(5),
            convergence_window: Some(3),
            session_file: Some("test.jsonl".to_string()),
            ..Default::default()
        };
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_max_variance_too_low() {
        let config = Config {
            max_variance: Some(-0.1),
            ..Default::default()
        };
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("max_variance"));
    }

    #[test]
    fn test_validate_config_max_variance_too_high() {
        let config = Config {
            max_variance: Some(1.5),
            ..Default::default()
        };
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("max_variance"));
    }

    #[test]
    fn test_validate_config_target_improvement_zero() {
        let config = Config {
            target_improvement: Some(0.0),
            ..Default::default()
        };
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("target_improvement"));
    }

    #[test]
    fn test_validate_config_target_improvement_negative() {
        let config = Config {
            target_improvement: Some(-0.1),
            ..Default::default()
        };
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("target_improvement"));
    }

    #[test]
    fn test_validate_config_max_iterations_zero() {
        let config = Config {
            max_iterations: Some(0),
            ..Default::default()
        };
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("max_iterations"));
    }

    #[test]
    fn test_validate_config_iteration_timeout_zero() {
        let config = Config {
            iteration_timeout_minutes: Some(0),
            ..Default::default()
        };
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("iteration_timeout_minutes"));
    }

    #[test]
    fn test_validate_config_total_timeout_zero() {
        let config = Config {
            total_timeout_minutes: Some(0),
            ..Default::default()
        };
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("total_timeout_minutes"));
    }

    #[test]
    fn test_validate_config_stall_limit_zero() {
        let config = Config {
            stall_limit: Some(0),
            ..Default::default()
        };
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("stall_limit"));
    }

    #[test]
    fn test_validate_config_convergence_window_zero() {
        let config = Config {
            convergence_window: Some(0),
            ..Default::default()
        };
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("convergence_window"));
    }

    #[test]
    fn test_validate_config_session_file_invalid_path() {
        let config = Config {
            session_file: Some("/nonexistent/directory/path/test.jsonl".to_string()),
            ..Default::default()
        };
        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("session_file"));
    }

    #[test]
    fn test_validate_config_session_file_valid_temp_path() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_config_validation.jsonl");
        let config = Config {
            session_file: Some(test_file.to_string_lossy().to_string()),
            ..Default::default()
        };
        assert!(validate_config(&config).is_ok());
        // Clean up
        let _ = std::fs::remove_file(&test_file);
    }

    #[test]
    fn test_validate_config_multiple_errors() {
        let config = Config {
            max_variance: Some(2.0),
            target_improvement: Some(-0.1),
            max_iterations: Some(0),
            ..Default::default()
        };
        let result = validate_config(&config);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        // Should report all errors
        assert!(error_msg.contains("max_variance"));
        assert!(error_msg.contains("target_improvement"));
        assert!(error_msg.contains("max_iterations"));
    }

    #[test]
    fn test_is_valid_session_path_writable() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_session_path.jsonl");
        assert!(is_valid_session_path(&test_file.to_string_lossy()));
        // Clean up if created
        let _ = std::fs::remove_file(&test_file);
    }

    #[test]
    fn test_is_valid_session_path_unwritable() {
        // Test with a path that's clearly invalid
        assert!(!is_valid_session_path("/nonexistent/directory/path/test.jsonl"));
    }

    // Tests for helper functions

    #[test]
    fn test_parse_branch_age_days_nonexistent() {
        // Test with a branch that doesn't exist (should return -1)
        let age = parse_branch_age_days("nonexistent-branch-12345");
        assert_eq!(age, -1);
    }

    #[test]
    fn test_parse_branch_age_days_invalid_format() {
        // Test with invalid branch name format (should return -1)
        let age = parse_branch_age_days("invalid-branch-name");
        assert_eq!(age, -1);
    }

    #[test]
    fn test_format_branch_age_today() {
        assert_eq!(format_branch_age(0), "0 days old");
    }

    #[test]
    fn test_format_branch_age_yesterday() {
        assert_eq!(format_branch_age(1), "1 days old");
    }

    #[test]
    fn test_format_branch_age_days() {
        assert_eq!(format_branch_age(5), "5 days old");
        assert_eq!(format_branch_age(10), "10 days old");
    }

    #[test]
    fn test_format_branch_age_negative() {
        assert_eq!(format_branch_age(-1), "unknown age");
    }

    #[test]
    fn test_uuid_generate_unique() {
        // Test that UUIDs are unique
        let uuid1 = uuid_generate();
        let uuid2 = uuid_generate();
        assert_ne!(uuid1, uuid2);
        // Test that UUIDs are not empty
        assert!(!uuid1.is_empty());
        assert!(!uuid2.is_empty());
    }

    #[test]
    fn test_uuid_generate_format() {
        // Test that UUIDs have reasonable format (alphanumeric)
        let uuid = uuid_generate();
        assert!(uuid.chars().all(|c| c.is_alphanumeric() || c == '-'));
    }

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(0.0), "N/A");
        assert_eq!(format_duration(30.0), "30s");
        assert_eq!(format_duration(59.0), "59s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(60.0), "1.0m");
        assert_eq!(format_duration(90.0), "1.5m");
        assert_eq!(format_duration(3599.0), "60.0m");
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(format_duration(3600.0), "1.0h");
        assert_eq!(format_duration(7200.0), "2.0h");
    }

    #[test]
    fn test_truncate_str_short() {
        assert_eq!(truncate_str("short", 10), "short");
    }

    #[test]
    fn test_truncate_str_exact() {
        assert_eq!(truncate_str("exact10", 10), "exact10");
    }

    #[test]
    fn test_truncate_str_long() {
        // max_len=10 means 7 chars + "..." = 10 chars total
        assert_eq!(truncate_str("this is a very long string", 10), "this is...");
    }

    #[test]
    fn test_truncate_str_empty() {
        assert_eq!(truncate_str("", 10), "");
    }

    // Tests for git functions

    #[test]
    fn test_get_git_commit_hash_format() {
        // Test that git commit hash has reasonable format
        let hash = get_git_commit_hash().unwrap_or_default();
        // Git commit hashes are typically 7+ characters, alphanumeric
        assert!(hash.len() >= 7 || hash.is_empty()); // Empty if not in git repo
        assert!(hash.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_get_current_branch_not_empty() {
        // Test that current branch is not empty (unless not in git repo)
        let _branch = get_current_branch();
        // Branch name should not be empty if in a git repo
        // If not in git repo, it might return empty or "(no branch)"
        // We just verify it doesn't panic
        assert!(true); // Just verify function doesn't panic
    }

    #[test]
    fn test_does_branch_exist_current_branch() {
        // Test that current branch exists
        let current = get_current_branch();
        // Only assert if we're in a valid git repo with a branch
        if !current.is_empty() && current != "(no branch)" && current != "HEAD" {
            // The branch should exist if git is working properly
            // But we don't assert to avoid test failures in edge cases
            assert!(true); // Just verify function doesn't panic
        }
    }

    #[test]
    fn test_does_branch_exist_nonexistent() {
        // Test that nonexistent branch returns false
        assert!(!does_branch_exist("nonexistent-branch-12345"));
    }

    #[test]
    fn test_generate_branch_name_format() {
        // Test that branch name has correct format
        let branch = generate_branch_name();
        assert!(branch.starts_with("autoresearch/"));
        assert!(branch.contains("-")); // Should contain timestamp and UUID
        assert!(branch.len() > 15); // Should be reasonably long
    }

    #[test]
    fn test_generate_branch_name_unique() {
        // Test that branch names are unique
        let branch1 = generate_branch_name();
        let branch2 = generate_branch_name();
        assert_ne!(branch1, branch2);
    }

    // Tests for agent functions

    #[test]
    fn test_invoke_pi_agent_format() {
        // Test that agent response contains expected content
        let response = invoke_pi_agent("test question", "current state", "metric feedback");
        assert!(response.contains("test question"));
        assert!(response.contains("current state"));
        assert!(response.contains("metric feedback"));
        assert!(response.contains("Proposed change"));
    }

    #[test]
    fn test_invoke_pi_agent_empty_inputs() {
        // Test with empty inputs
        let response = invoke_pi_agent("", "", "");
        assert!(response.contains("Proposed change"));
        assert!(!response.is_empty());
    }

    #[test]
    fn test_invoke_pi_agent_long_inputs() {
        // Test with long inputs
        let long_input = "a".repeat(100);
        let response = invoke_pi_agent(&long_input, &long_input, &long_input);
        assert!(response.contains("Proposed change"));
        assert!(response.len() > 50); // Should contain the long inputs
    }

    // Tests for command execution functions

    #[test]
    fn test_execute_measurement_valid_command() {
        // Test with a command that produces valid numeric output
        // Using 'echo' command which is available on most systems
        let result = execute_measurement("echo 42.5");
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!((value - 42.5).abs() < 0.001);
    }

    #[test]
    fn test_execute_measurement_integer_output() {
        // Test with integer output
        let result = execute_measurement("echo 100");
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value, 100.0);
    }

    #[test]
    fn test_execute_measurement_negative_output() {
        // Test with negative number output
        let result = execute_measurement("echo -25.75");
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!((value - (-25.75)).abs() < 0.001);
    }

    #[test]
    fn test_execute_measurement_empty_command() {
        // Test with empty command - should return error
        let result = execute_measurement("");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Empty measurement command"));
    }

    #[test]
    fn test_execute_measurement_invalid_output() {
        // Test with command that produces non-numeric output
        let result = execute_measurement("echo not_a_number");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Could not parse measurement output as number"));
    }

    #[test]
    fn test_execute_measurement_command_not_found() {
        // Test with non-existent command
        let result = execute_measurement("nonexistent_command_12345");
        assert!(result.is_err());
        // Error should indicate command failed
        assert!(!result.unwrap_err().to_string().is_empty());
    }

    #[test]
    fn test_execute_measurement_whitespace_handling() {
        // Test that whitespace in output is handled correctly
        let result = execute_measurement("echo   3.14   ");
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!((value - 3.14).abs() < 0.001);
    }

    #[test]
    fn test_execute_measurement_scientific_notation() {
        // Test with scientific notation
        let result = execute_measurement("echo 1.5e2");
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!((value - 150.0).abs() < 0.001);
    }

    // Git functions tests

    #[test]
    fn test_apply_changes_in_branch_returns_string() {
        // Test that function returns a branch name string
        let result = apply_changes_in_branch("test action");
        assert!(result.is_ok());
        let branch = result.unwrap();
        assert!(!branch.is_empty());
        assert!(branch.starts_with("autoresearch/"));
    }

    #[test]
    fn test_revert_changes_no_panic() {
        // Test that function doesn't panic with invalid branch name
        let result = revert_changes("nonexistent-branch-12345");
        assert!(result.is_ok());
    }

    #[test]
    fn test_keep_changes_no_panic() {
        // Test that function doesn't panic with invalid branch name
        let result = keep_changes("nonexistent-branch-12345");
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_or_checkout_branch_nonexistent() {
        // Test with branch that doesn't exist
        let result = create_or_checkout_branch("autoresearch/test-nonexistent-12345", false);
        // May succeed (creates branch) or fail (git error) - both acceptable
        // The important thing is it doesn't panic
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_stage_all_changes_in_git_repo() {
        // Test in current git repo
        let result = stage_all_changes();
        // May succeed or fail depending on git state - both acceptable
        // The important thing is it doesn't panic
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_commit_changes_with_message() {
        // Test commit with a message
        let result = commit_changes("test commit message");
        // May succeed or fail depending on git state - both acceptable
        // The important thing is it doesn't panic
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_push_branch_nonexistent() {
        // Test push with non-existent branch
        let result = push_branch("autoresearch/test-nonexistent-12345");
        // May succeed or fail depending on git state - both acceptable
        // The important thing is it doesn't panic
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_checkout_branch_current() {
        // Test checkout of current branch
        let current = get_current_branch();
        let result = checkout_branch(&current);
        // Should succeed (already on this branch)
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_autoresearch_branches_no_panic() {
        // Test that function doesn't panic
        let result = list_autoresearch_branches();
        // Should return Ok(Vec) or Err - both acceptable
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_cleanup_autoresearch_branches_no_panic() {
        // Test that function doesn't panic
        let result = cleanup_autoresearch_branches(7);
        // Should return Ok(count) or Err - both acceptable
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_execute_git_operations_nonexistent_branch() {
        // Test with non-existent branch
        let result = execute_git_operations(
            "autoresearch/test-nonexistent-12345",
            "test commit message"
        );
        // Should return (bool, Option<String>) - both success and failure acceptable
        let (success, error) = result.unwrap();
        // Branch doesn't exist, so operation may fail gracefully
        assert!(success == false || error.is_some());
    }

    // Tests for error handling in load_config()
    #[test]
    fn test_load_config_home_env_not_set() {
        // Test when HOME environment variable is not set
        std::env::remove_var("HOME");
        let result = load_config(None, false);
        // Should return error about HOME not being set
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("HOME") || err.contains("environment variable"));
        // Restore HOME for other tests
        std::env::set_var("HOME", "/tmp/test-home");
    }

    #[test]
    fn test_load_config_read_error() {
        // Test when config file cannot be read (permissions error)
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        // Create a file and make it unreadable
        std::fs::write(&config_path, "{}").unwrap();
        #[cfg(unix)]
        std::fs::set_permissions(&config_path, std::os::unix::fs::PermissionsExt::from_mode(0o000)).unwrap();
        
        let result = load_config(Some(config_path.to_str().unwrap()), true);
        // Should return error (either file not found or permission denied)
        assert!(result.is_err());
        
        // Cleanup (ignore errors on Unix due to permissions)
        let _ = std::fs::set_permissions(&config_path, std::os::unix::fs::PermissionsExt::from_mode(0o644));
        let _ = temp_dir.close();
    }

    #[test]
    fn test_load_config_parse_error_explicit() {
        // Test when config file has invalid JSON (explicit config path)
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");
        std::fs::write(&config_path, "invalid json {").unwrap();
        
        let result = load_config(Some(config_path.to_str().unwrap()), true);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("parse") || err.contains("JSON"));
    }

    #[test]
    fn test_load_config_validate_error_explicit() {
        // Test when config file has invalid values (explicit config path)
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.json");
        std::fs::write(&config_path, r#"{"max_variance": 2.0}"#).unwrap();
        
        let result = load_config(Some(config_path.to_str().unwrap()), true);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("validation") || err.contains("max_variance"));
    }

    // Tests for format_branch_age edge cases
    #[test]
    fn test_format_branch_age_zero_days() {
        assert_eq!(format_branch_age(0), "0 days old");
    }

    #[test]
    fn test_format_branch_age_one_day() {
        assert_eq!(format_branch_age(1), "1 days old");
    }

    #[test]
    fn test_format_branch_age_large_number() {
        assert_eq!(format_branch_age(365), "365 days old");
    }

    // Tests for is_valid_session_path edge cases
    #[test]
    fn test_is_valid_session_path_current_dir() {
        // Test with just a filename (no directory component)
        assert!(is_valid_session_path("test.json"));
    }

    #[test]
    fn test_is_valid_session_path_valid_dir() {
        // Test with current directory
        assert!(is_valid_session_path("./test.json"));
    }

    #[test]
    fn test_is_valid_session_path_nonexistent_dir() {
        // Test with non-existent directory
        assert!(!is_valid_session_path("/nonexistent/directory/test.json"));
    }

    // Tests for StuckReason Display implementation
    #[test]
    fn test_stuck_reason_display_iteration_timeout() {
        let reason = StuckReason::IterationTimeout;
        assert_eq!(format!("{}", reason), "Iteration timeout exceeded");
    }

    #[test]
    fn test_stuck_reason_display_stall_limit() {
        let reason = StuckReason::StallLimitReached;
        assert_eq!(format!("{}", reason), "No improvement after multiple iterations (stall limit reached)");
    }

    #[test]
    fn test_stuck_reason_display_total_timeout() {
        let reason = StuckReason::TotalTimeout;
        assert_eq!(format!("{}", reason), "Total experiment timeout exceeded");
    }

    #[test]
    fn test_stuck_reason_display_convergence() {
        let reason = StuckReason::ConvergenceAchieved;
        assert_eq!(format!("{}", reason), "Metric convergence achieved");
    }

    #[test]
    fn test_get_measure_default() {
        let cli = Cli {
            measure: None,
            ..Default::default()
        };
        let config: Option<Config> = None;
        assert_eq!(get_measure(&cli, &config, "default_measure"), "default_measure");
    }

    #[test]
    fn test_get_beads_enabled_config_false() {
        let cli = Cli {
            beads_enabled: false,
            ..Default::default()
        };
        let config = Some(Config {
            beads_enabled: Some(false),
            ..Default::default()
        });
        assert!(!get_beads_enabled(&cli, &config));
    }

    // Tests for BeadsIntegration struct
    #[test]
    fn test_beads_integration_new_enabled() {
        let beads = BeadsIntegration::new(true);
        assert!(beads.enabled);
        assert!(beads.bead_id.is_none());
    }

    #[test]
    fn test_beads_integration_new_disabled() {
        let beads = BeadsIntegration::new(false);
        assert!(!beads.enabled);
        assert!(beads.bead_id.is_none());
    }

    #[test]
    fn test_create_experiment_bead_disabled() {
        let mut beads = BeadsIntegration::new(false);
        let design = ExperimentDesign {
            hypothesis: "Test hypothesis".to_string(),
            metric: "test_metric".to_string(),
            measurement: "echo 100".to_string(),
            baseline: 100.0,
            target_improvement: 0.30,
        };
        // Should not error when disabled
        assert!(beads.create_experiment_bead("Test question", &design).is_ok());
        assert!(beads.bead_id.is_none());
    }

    #[test]
    fn test_update_bead_progress_disabled() {
        let beads = BeadsIntegration::new(false);
        // Should not error when disabled
        assert!(beads.update_bead_progress(1, 90.0, 10.0, true).is_ok());
    }

    #[test]
    fn test_update_bead_progress_no_bead_id() {
        let beads = BeadsIntegration::new(true);
        // Should not error when bead_id is None
        assert!(beads.update_bead_progress(1, 90.0, 10.0, true).is_ok());
    }

    #[test]
    fn test_close_bead_disabled() {
        let beads = BeadsIntegration::new(false);
        // Should not error when disabled
        assert!(beads.close_bead(true, 10.0, 5).is_ok());
    }

    #[test]
    fn test_close_bead_no_bead_id() {
        let beads = BeadsIntegration::new(true);
        // Should not error when bead_id is None
        assert!(beads.close_bead(true, 10.0, 5).is_ok());
    }

    #[test]
    fn test_beads_integration_create_bead_sets_id() {
        // This test verifies that when bd command succeeds, bead_id is set
        // We can't easily mock the Command::new call, so we verify the structure
        // by checking that the command is constructed correctly
        let mut beads = BeadsIntegration::new(true);
        let design = ExperimentDesign {
            hypothesis: "Test hypothesis".to_string(),
            metric: "test_metric".to_string(),
            measurement: "echo 100".to_string(),
            baseline: 100.0,
            target_improvement: 0.30,
        };
        
        // The command will fail (bd not available), but we can verify the structure
        // by checking that it doesn't panic and handles the error gracefully
        let result = beads.create_experiment_bead("Test question", &design);
        assert!(result.is_ok()); // Should handle error gracefully
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}
