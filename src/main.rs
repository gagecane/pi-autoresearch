use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::process::Command;
use chrono::Utc;
use std::time::{Duration, Instant};
use std::fs::OpenOptions;


#[derive(Parser, Debug)]
#[command(name = "pi-autoresearch")]
#[command(about = "Autonomous research experiment orchestrator")]
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

#[derive(Serialize, Deserialize, Debug)]
struct IterationRecord {
    iteration: usize,
    timestamp: String,
    agent_action: String,
    metric_value: f64,
    improvement: f64,
    kept: bool,
}

#[derive(Serialize, Deserialize, Debug)]
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

impl std::fmt::Display for BaselineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BaselineError {}

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

fn verify_baseline(
    metric: &str,
    measurement_command: &str,
    max_variance: f64,
) -> Result<BaselineVerificationResult> {
    eprintln!("Verifying baseline measurement...");
    eprintln!("  Metric: {}", metric);
    eprintln!("  Command: {}", measurement_command);
    eprintln!("  Max variance: {:.1}%", max_variance * 100.0);

    let git_commit = get_git_commit_hash()?;
    let timestamp = Utc::now().to_rfc3339();

    let mut runs: Vec<f64> = Vec::new();

    for i in 1..=2 {
        eprint!("  Run {}/2... ", i);
        io::stdout().flush()?;

        match execute_measurement(measurement_command) {
            Ok(value) => {
                eprintln!("{:.2}", value);
                runs.push(value);
            }
            Err(e) => {
                eprintln!("FAILED");
                return Ok(BaselineVerificationResult {
                    success: false,
                    baseline_record: None,
                    error_message: Some(format!("Run {} failed: {}", i, e)),
                });
            }
        }
    }

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
        eprintln!(
            "\nWARNING: Baseline variance {:.2}% exceeds threshold {:.2}%",
            variance * 100.0,
            max_variance * 100.0
        );
        eprintln!("  Run 1: {:.2}", runs[0]);
        eprintln!("  Run 2: {:.2}", runs[1]);

        return Ok(BaselineVerificationResult {
            success: false,
            baseline_record: Some(baseline_record),
            error_message: Some(format!(
                "Variance {:.2}% exceeds threshold {:.2}%",
                variance * 100.0, max_variance * 100.0
            )),
        });
    }

    eprintln!("\nBaseline verified successfully!");
    eprintln!("  Value: {:.2}", baseline_value);
    eprintln!("  Variance: {:.2}%", variance * 100.0);
    eprintln!("  Git commit: {}", baseline_record.git_commit);

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
    
    eprintln!("Baseline recorded to: {}", session_file);
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
) -> Result<(f64, String, bool)> {
    let current_state = format!("Iteration {}, best metric: {:.2}", iteration, best_metric);
    let metric_feedback = format!("Baseline: {:.2}, current best: {:.2}", baseline_value, best_metric);
    
    let agent_action = invoke_pi_agent(question, &current_state, &metric_feedback);
    
    let branch_name = apply_changes_in_branch(&agent_action)?;
    
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
    
    log_iteration(session_file, &record)?;
    
    if kept {
        keep_changes(&branch_name)?;
    } else {
        revert_changes(&branch_name)?;
    }
    
    Ok((metric_value, agent_action, kept))
}

fn run_iterative_loop(
    question: &str,
    design: &ExperimentDesign,
    baseline_record: &BaselineRecord,
    cli: &Cli,
) -> Result<(ExperimentSession, Option<StuckReason>)> {
    let max_iterations = cli.max_iterations.unwrap_or(20);
    let baseline_value = baseline_record.value;
    let iteration_timeout = Duration::from_secs(cli.iteration_timeout_minutes.unwrap_or(10) as u64 * 60);
    let total_timeout = Duration::from_secs(cli.total_timeout_minutes.unwrap_or(120) as u64 * 60);
    let stall_limit = cli.stall_limit.unwrap_or(5);
    let convergence_threshold = cli.convergence_threshold.unwrap_or(0.01);
    let convergence_window = cli.convergence_window.unwrap_or(3);
    
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
        eprintln!("\nStarting iterative exploration loop...");
        eprintln!("Max iterations: {}", max_iterations);
        eprintln!("Baseline: {:.2}", baseline_value);
        eprintln!();
    }
    
    while state.current_iteration < max_iterations {
        // Layer 3: Check total runtime limit
        if state.start_time.elapsed() >= total_timeout {
            if !cli.quiet {
                eprintln!("\nTotal timeout reached ({:.0}s). Saving best result.", total_timeout.as_secs());
            }
            stuck_reason = Some(StuckReason::TotalTimeout);
            break;
        }
        
        state.current_iteration += 1;
        
        let display_status = |iter: usize, best: f64, stall: usize, elapsed: Duration| {
            let improvement_pct = (baseline_value - best) / baseline_value * 100.0;
            eprintln!(
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
        
        let iteration_result = run_iteration(
            state.current_iteration,
            question,
            baseline_value,
            state.best_metric,
            &design.measurement,
            &cli.session_file,
        );
        
        if iteration_start.elapsed() > iteration_timeout {
            if !cli.quiet {
                eprintln!("\nIteration {} exceeded timeout ({:.0}s). Marking as timeout.", 
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
                        eprintln!("  ✓ Kept - improvement: {:+.2}%", improvement * 100.0);
                    }
                    if cli.verbose {
                        eprintln!("    Metric value: {:.2}", metric_value);
                        eprintln!("    Agent action: {}", agent_action.split('.').next().unwrap_or(&agent_action));
                    }
                } else {
                    state.consecutive_no_improvement += 1;
                    if !cli.quiet {
                        eprintln!("  ✗ Reverted - degradation: {:-.2}%", improvement * 100.0);
                    }
                    if cli.verbose {
                        eprintln!("    Metric value: {:.2}", metric_value);
                    }
                }
                
                iterations.push(IterationRecord {
                    iteration: state.current_iteration,
                    timestamp: Utc::now().to_rfc3339(),
                    agent_action,
                    metric_value,
                    improvement,
                    kept,
                });
            }
            Err(e) => {
                if !cli.quiet {
                    eprintln!("  ✗ Iteration failed: {}", e);
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
                    eprintln!("\nConvergence achieved! Variance {:.4} < threshold {:.4}", 
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
                    eprintln!("\nStall limit reached after {} backoffs. Aborting.", state.backoff_count);
                }
                stuck_reason = Some(StuckReason::StallLimitReached);
                break;
            }
            if !cli.quiet {
                eprintln!("\nStall limit reached. Backing off (attempt {}/2)...", state.backoff_count);
            }
            state.consecutive_no_improvement = 0;
        }
    }
    
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

fn finalize_experiment(
    session: &ExperimentSession,
    target_improvement: f64,
) -> Result<FinalizationResult> {
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
    
    let success = final_improvement >= target_improvement;
    
    if !success {
        return Ok(FinalizationResult {
            success: false,
            final_improvement,
            best_value: best_kept_value,
            branch_name: None,
            commit_message: None,
            key_changes: Vec::new(),
            error_message: None,
        });
    }
    
    let timestamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let branch_name = format!("autoresearch/{}", timestamp);
    
    let runtime_secs = session.start_time
        .parse::<chrono::DateTime<Utc>>()
        .ok()
        .and_then(|start| {
            Some(session.end_time.clone()
                .and_then(|end_str| end_str.parse::<chrono::DateTime<Utc>>().ok())
                .map(|end| end.signed_duration_since(start).num_seconds())
                .unwrap_or(0))
        })
        .unwrap_or(0);
    
    let iterations_count = session.iterations.len();
    let best_value = best_kept_value.unwrap_or(baseline);
    
    let mut key_changes = Vec::new();
    if let Some(best_iter_num) = session.best_iteration {
        if let Some(best_iter) = session.iterations.iter().find(|i| i.iteration == best_iter_num) {
            let change_summary = best_iter.agent_action
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
            let change_summary = iter.agent_action
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
    
    let convergence_status = match &session.status {
        s if s.contains("completed") => "achieved",
        _ => "not achieved",
    };
    
    let commit_message = format!(
        "[autoresearch] Reduce {} by {:.1}% ({:.0} → {:.0})\n\n",
        session.design.metric,
        final_improvement * 100.0,
        baseline,
        best_value
    );
    
    let commit_message = format!(
        "{}Iterations: {}/{} | Runtime: {}s | Convergence: {}\n\n",
        commit_message,
        iterations_count,
        session.iterations.len().max(20),
        runtime_secs,
        convergence_status
    );
    
    let commit_message = format!(
        "{}Key changes:\n{}\n\n",
        commit_message,
        key_changes.join("\n")
    );
    
    let commit_message = format!(
        "{}Metric: {} | Baseline: {:.0} | Best: {:.0}",
        commit_message,
        session.design.metric,
        baseline,
        best_value
    );
    
    let mut result = FinalizationResult {
        success: true,
        final_improvement,
        best_value: Some(best_value),
        branch_name: Some(branch_name.clone()),
        commit_message: Some(commit_message.clone()),
        key_changes,
        error_message: None,
    };
    
    let create_branch_output = Command::new("git")
        .args(["checkout", "-b", &branch_name])
        .output();
    
    if create_branch_output.is_err() {
        result.error_message = Some("Failed to create git branch".to_string());
        result.success = false;
        result.branch_name = None;
        return Ok(result);
    }
    
    if let Ok(output) = create_branch_output {
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            result.error_message = Some(format!("Failed to create branch: {}", stderr));
            result.success = false;
            result.branch_name = None;
            return Ok(result);
        }
    }
    
    let has_remote = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false);
    
    let add_output = Command::new("git")
        .args(["add", "-A"])
        .output();
    
    if let Ok(output) = add_output {
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            result.error_message = Some(format!("Failed to stage changes: {}", stderr));
            result.success = false;
            return Ok(result);
        }
    }
    
    let commit_msg = commit_message.as_str();
    let commit_output = Command::new("git")
        .args(["commit", "-m", commit_msg])
        .output();
    
    if let Ok(output) = commit_output {
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("no changes") {
                result.error_message = Some(format!("Failed to commit: {}", stderr));
                result.success = false;
                return Ok(result);
            }
        }
    }
    
    if has_remote {
        let push_output = Command::new("git")
            .args(["push", "-u", "origin", &branch_name])
            .output();
        
        if let Ok(output) = push_output {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                result.error_message = Some(format!("Failed to push branch: {}", stderr));
                result.success = false;
                return Ok(result);
            }
        }
    }
    
    Ok(result)
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
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    if cli.verify_baseline {
        let metric = cli.metric.clone().ok_or_else(|| {
            anyhow::anyhow!("--metric is required for baseline verification")
        })?;
        
        let measurement = cli.measure.clone().ok_or_else(|| {
            anyhow::anyhow!("--measure is required for baseline verification")
        })?;

        let max_variance = cli.max_variance;

        match verify_baseline(&metric, &measurement, max_variance) {
            Ok(result) => {
                if result.success {
                    if let Some(ref record) = result.baseline_record {
                        save_to_session_file(&cli.session_file, record)?;
                        
                        let json_output = serde_json::to_string_pretty(&result)?;
                        println!("{}", json_output);
                        
                        return Ok(());
                    }
                }
                
                if let Some(err) = result.error_message {
                    eprintln!("Baseline verification failed: {}", err);
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Baseline verification error: {}", e);
                std::process::exit(1);
            }
        }
    }

    let question = match &cli.question {
        Some(q) => q.clone(),
        None => read_question_from_stdin()?,
    };

    let design = generate_design(&question);
    
    let json_output = serde_json::to_string_pretty(&design)?;
    println!("{}", json_output);

    if !cli.auto_approve {
        print!("\nApprove this design? [y/N]: ");
        io::stdout().flush()?;
        
        let mut response = String::new();
        io::stdin().read_line(&mut response)?;
        
        if response.trim().to_lowercase() != "y" && response.trim().to_lowercase() != "yes" {
            println!("Design not approved. Exiting.");
            std::process::exit(1);
        }
    }
    
    if cli.max_iterations.is_some() {
        let metric_to_use = cli.metric.clone().unwrap_or(design.metric.clone());
        let measure_to_use = cli.measure.clone().unwrap_or(design.measurement.clone());
        let baseline_to_use = cli.baseline.unwrap_or(design.baseline);
        
        let baseline_record = if cli.baseline.is_some() {
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
                cli.max_variance,
            )?;
            
            if !baseline_result.success {
                if let Some(err) = baseline_result.error_message {
                    eprintln!("Baseline verification failed: {}", err);
                    std::process::exit(1);
                }
            }
            
            baseline_result.baseline_record.ok_or_else(|| {
                anyhow::anyhow!("No baseline record available")
            })?
        };
        
        save_to_session_file(&cli.session_file, &baseline_record)?;
        
        let mut design_to_use = design.clone();
        design_to_use.metric = metric_to_use;
        design_to_use.measurement = measure_to_use;
        design_to_use.baseline = baseline_to_use;
        
        let (session, stuck_reason) = run_iterative_loop(&question, &design_to_use, &baseline_record, &cli)?;
        
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
            eprintln!("\n=== Experiment Complete ===");
            eprintln!("Iterations: {}", session.iterations.len());
            eprintln!("Best improvement: {:+.2}%", final_improvement * 100.0);
            eprintln!("Session ID: {}", session.session_id);
            if let Some(reason) = &stuck_reason {
                eprintln!("Termination: {:?}", reason);
            }
            
            if session.best_iteration.is_some() {
                let best_iter = session.iterations
                    .iter()
                    .find(|i| i.iteration == session.best_iteration.unwrap());
                if let Some(best) = best_iter {
                    eprintln!("Best iteration: {} (metric: {:.2})", best.iteration, best.metric_value);
                }
            }
        } else {
            eprintln!("Final improvement: {:+.2}%", final_improvement * 100.0);
        }
        
        let session_json = serde_json::to_string_pretty(&session)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&cli.session_file)?;
        writeln!(file, "{}", session_json)?;
        
        let target_improvement = cli.target_improvement.unwrap_or(design.target_improvement);
        
        let finalization_result = finalize_experiment(&session, target_improvement)?;
        
        if !cli.quiet {
            eprintln!("\n=== Finalization ===");
            if finalization_result.success {
                eprintln!("✓ Target improvement achieved: {:+.2}%", finalization_result.final_improvement * 100.0);
                if let Some(ref branch) = finalization_result.branch_name {
                    eprintln!("✓ Created branch: {}", branch);
                }
                if let Some(ref msg) = finalization_result.commit_message {
                    eprintln!("\nCommit message:\n{}", msg);
                }
            } else {
                eprintln!("✗ Target improvement not met");
                eprintln!("  Target: {:.2}%, Achieved: {:+.2}%", target_improvement * 100.0, finalization_result.final_improvement * 100.0);
                if let Some(best) = finalization_result.best_value {
                    eprintln!("  Best value: {:.2} (baseline: {:.2})", best, session.baseline_record.value);
                }
                if let Some(ref err) = finalization_result.error_message {
                    eprintln!("  Error: {}", err);
                }
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

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}
