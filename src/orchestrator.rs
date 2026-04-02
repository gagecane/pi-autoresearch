use anyhow::Result;
use std::io::{self, Write};

use crate::cli::Cli;
use crate::phase1_design::{ExperimentDesign, generate_design};
use crate::phase2_iterate::{IterationConfig, IterationExecutor};
use crate::phase3_merge::FinalizationExecutor;
use crate::metric_evaluator::MetricEvaluator;
use crate::session::{ExperimentSession, SessionManager, generate_session_id};
use crate::beads::BeadsIntegration;
use crate::ralph_tui::RalphTuiIntegration;

/// Main orchestrator for the autoresearch experiment
pub struct ExperimentOrchestrator {
    cli: Cli,
    session_manager: SessionManager,
    metric_evaluator: MetricEvaluator,
    beads: Option<BeadsIntegration>,
    ralph: Option<RalphTuiIntegration>,
}

impl ExperimentOrchestrator {
    pub fn new(cli: Cli) -> Self {
        let session_manager = SessionManager::new(cli.session_file.clone());
        let metric_evaluator = MetricEvaluator::new(cli.max_variance);

        let beads = if cli.beads_enabled {
            Some(BeadsIntegration::new(true))
        } else {
            None
        };

        let ralph = if cli.ralph_tui_enabled {
            Some(RalphTuiIntegration::new(true, cli.ralph_task_file.clone()))
        } else {
            None
        };

        Self {
            cli,
            session_manager,
            metric_evaluator,
            beads,
            ralph,
        }
    }

    /// Run the complete experiment orchestration
    pub async fn run(&mut self) -> Result<()> {
        // Handle --history flag
        if self.cli.history {
            let history = self.session_manager.list_history()?;
            println!("{}", history);
            return Ok(());
        }

        // Handle --resume flag
        if let Some(ref session_id) = self.cli.resume {
            return self.resume_experiment(session_id).await;
        }

        // Handle --verify-baseline flag
        if self.cli.verify_baseline {
            return self.verify_baseline_only().await;
        }

        // Get research question
        let question = self.get_question()?;

        // Phase 1: Generate experiment design
        let design = self.generate_experiment_design(&question);

        // Output design
        let json_output = serde_json::to_string_pretty(&design)?;
        println!("{}", json_output);

        // Create bead issue if enabled
        if let Some(ref mut beads_integration) = self.beads {
            beads_integration.create_experiment_bead(&question, &design)?;
        }

        // Get user approval
        if !self.cli.auto_approve {
            if !self.get_user_approval() {
                println!("Design not approved. Exiting.");
                std::process::exit(1);
            }
        }

        // Only run iterations if max_iterations is specified
        if self.cli.max_iterations.is_some() {
            return self.run_full_experiment(&question, design).await;
        }

        Ok(())
    }

    fn get_question(&self) -> Result<String> {
        // Try Ralph-TUI task file first
        if let Some(ref ralph) = self.ralph {
            if let Ok(Some(question)) = ralph.read_task_from_file() {
                return Ok(question);
            }
        }

        // Try CLI argument
        if let Some(ref q) = self.cli.question {
            return Ok(q.clone());
        }

        // Read from stdin
        read_question_from_stdin()
    }

    fn generate_experiment_design(&self, question: &str) -> ExperimentDesign {
        let mut design = generate_design(question);

        // Override with CLI values if provided
        if let Some(ref metric) = self.cli.metric {
            design.metric = metric.clone();
        }
        if let Some(ref measure) = self.cli.measure {
            design.measurement = measure.clone();
        }
        if let Some(baseline) = self.cli.baseline {
            design.baseline = baseline;
        }
        if let Some(target) = self.cli.target_improvement {
            design.target_improvement = target;
        }

        design
    }

    fn get_user_approval(&self) -> bool {
        print!("\nApprove this design? [y/N]: ");
        io::stdout().flush().unwrap();

        let mut response = String::new();
        io::stdin().read_line(&mut response).unwrap();

        let response = response.trim().to_lowercase();
        response == "y" || response == "yes"
    }

    async fn verify_baseline_only(&self) -> Result<()> {
        let metric = self.cli.metric.clone().ok_or_else(|| {
            anyhow::anyhow!("--metric is required for baseline verification")
        })?;

        let measurement = self.cli.measure.clone().ok_or_else(|| {
            anyhow::anyhow!("--measure is required for baseline verification")
        })?;

        let result = self.metric_evaluator.verify_baseline(&metric, &measurement)?;

        if result.success {
            if let Some(ref record) = result.baseline_record {
                self.session_manager.save_baseline(record)?;

                let json_output = serde_json::to_string_pretty(&result)?;
                println!("{}", json_output);

                return Ok(());
            }
        }

        if let Some(err) = result.error_message {
            eprintln!("Baseline verification failed: {}", err);
            std::process::exit(1);
        }

        Ok(())
    }

    async fn resume_experiment(&self, session_id: &str) -> Result<()> {
        let session = self
            .session_manager
            .find_session(session_id)?
            .ok_or_else(|| anyhow::anyhow!("Session '{}' not found", session_id))?;

        println!("Resuming session: {}", session_id);
        println!("Question: {}", session.question);
        println!("Current best iteration: {:?}", session.best_iteration);
        println!("Iterations completed: {}", session.iterations.len());
        println!();

        // Calculate resume baseline from best kept iteration
        let resume_baseline = session
            .iterations
            .iter()
            .filter(|it| it.kept)
            .map(|it| it.metric_value)
            .fold(session.baseline_record.value, |min, val| min.min(val));

        println!("Continuing from baseline: {:.2}", resume_baseline);
        println!();

        // Create updated design
        let mut design = session.design.clone();
        design.baseline = resume_baseline;

        // Override with CLI values if provided
        if let Some(ref metric) = self.cli.metric {
            design.metric = metric.clone();
        }
        if let Some(ref measure) = self.cli.measure {
            design.measurement = measure.clone();
        }

        // Save baseline record
        self.session_manager.save_baseline(&session.baseline_record)?;

        // Create iteration config
        let config = self.create_iteration_config();
        let executor = IterationExecutor::new(config, self.cli.max_variance);

        // Run iterations
        let result = executor.run_loop(&session.question, resume_baseline, &design.measurement)?;

        // Merge iterations with existing session
        let mut merged_session = session.clone();
        let max_old_iter = session.iterations.iter().map(|it| it.iteration).max().unwrap_or(0);

        for mut iter in result.iterations {
            iter.iteration = max_old_iter + iter.iteration;
            merged_session.iterations.push(iter);
        }

        merged_session.end_time = Some(chrono::Utc::now().to_rfc3339());

        // Calculate final improvement
        let final_improvement = merged_session.calculate_final_improvement();

        if !self.cli.quiet {
            eprintln!("\n=== Resumed Experiment Complete ===");
            eprintln!("Total iterations: {}", merged_session.iterations.len());
            eprintln!("Best improvement: {:+.2}%", final_improvement * 100.0);
            eprintln!("Session ID: {}", session_id);
            if let Some(reason) = &result.stuck_reason {
                eprintln!("Termination: {}", reason);
            }
        }

        // Save session
        self.session_manager.save_session(&merged_session)?;

        // Finalize
        let target_improvement = self.cli.target_improvement.unwrap_or(design.target_improvement);
        let finalization_executor = FinalizationExecutor::new();
        let finalization_result = finalization_executor.finalize(
            &merged_session,
            target_improvement,
            result.stuck_reason.as_ref(),
        )?;

        self.print_finalization_result(&finalization_result, target_improvement);

        if finalization_result.success {
            Ok(())
        } else {
            std::process::exit(1);
        }
    }

    async fn run_full_experiment(&mut self, question: &str, design: ExperimentDesign) -> Result<()> {
        // Get or verify baseline
        let baseline_record = if let Some(baseline) = self.cli.baseline {
            let git_commit = self.metric_evaluator.get_git_commit_hash()?;
            crate::phase1_design::BaselineRecord::new(
                chrono::Utc::now().to_rfc3339(),
                git_commit,
                design.metric.clone(),
                design.measurement.clone(),
                baseline,
                vec![baseline, baseline],
                0.0,
                true,
            )
        } else {
            let baseline_result = self
                .metric_evaluator
                .verify_baseline(&design.metric, &design.measurement)?;

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

        // Save baseline
        self.session_manager.save_baseline(&baseline_record)?;

        // Create iteration config and executor
        let config = self.create_iteration_config();
        let executor = IterationExecutor::new(config, self.cli.max_variance);

        // Run iterations
        let result = executor.run_loop(question, baseline_record.value, &design.measurement)?;

        if !self.cli.quiet {
            eprintln!("\n=== Experiment Complete ===");
            eprintln!("Iterations: {}", result.iterations.len());

            let final_improvement = (baseline_record.value - result.best_metric) / baseline_record.value;
            eprintln!("Best improvement: {:+.2}%", final_improvement * 100.0);
            eprintln!("Session ID: {}", generate_session_id());

            if let Some(reason) = &result.stuck_reason {
                eprintln!("Termination: {}", reason);
            }

            if let Some(best_iter_num) = result.best_iteration {
                if let Some(best) = result.iterations.iter().find(|i| i.iteration == best_iter_num)
                {
                    eprintln!("Best iteration: {} (metric: {:.2})", best.iteration, best.metric_value);
                }
            }
        } else {
            let final_improvement = (baseline_record.value - result.best_metric) / baseline_record.value;
            eprintln!("Final improvement: {:+.2}%", final_improvement * 100.0);
        }

        // Create and save session
        let session_id = generate_session_id();
        let mut session = ExperimentSession::new(
            session_id,
            question.to_string(),
            design.clone(),
            baseline_record.clone(),
        );

        for iter in &result.iterations {
            session.add_iteration(iter.clone());
        }

        session.finalize(
            result.best_iteration,
            if result.best_metric < baseline_record.value {
                "completed".to_string()
            } else {
                "no_improvement".to_string()
            },
        );

        self.session_manager.save_session(&session)?;

        // Finalize experiment
        let target_improvement = self.cli.target_improvement.unwrap_or(design.target_improvement);
        let finalization_executor = FinalizationExecutor::new();
        let finalization_result = finalization_executor.finalize(
            &session,
            target_improvement,
            result.stuck_reason.as_ref(),
        )?;

        // Close bead issue
        if let Some(ref mut beads_integration) = self.beads {
            let _ = beads_integration.close_bead(
                finalization_result.success,
                finalization_result.final_improvement,
                session.iterations.len(),
            );
        }

        // Mark Ralph task complete
        if let Some(ref ralph_integration) = self.ralph {
            let _ = ralph_integration.mark_task_complete(
                finalization_result.success,
                finalization_result.final_improvement,
                session.iterations.len(),
            );
        }

        self.print_finalization_result(&finalization_result, target_improvement);

        if finalization_result.success {
            Ok(())
        } else {
            std::process::exit(1);
        }
    }

    fn create_iteration_config(&self) -> IterationConfig {
        IterationConfig {
            max_iterations: self.cli.effective_max_iterations(),
            iteration_timeout_secs: self.cli.effective_iteration_timeout_secs(),
            total_timeout_secs: self.cli.effective_total_timeout_secs(),
            stall_limit: self.cli.effective_stall_limit(),
            convergence_threshold: self.cli.effective_convergence_threshold(),
            convergence_window: self.cli.effective_convergence_window(),
            verbose: self.cli.verbose,
            quiet: self.cli.quiet,
        }
    }

    fn print_finalization_result(&self, result: &crate::phase3_merge::FinalizationResult, target_improvement: f64) {
        if self.cli.quiet {
            return;
        }

        eprintln!("\n=== Finalization ===");
        if result.success {
            eprintln!("✓ Target improvement achieved: {:+.2}%", result.final_improvement * 100.0);
            if let Some(ref branch) = result.branch_name {
                eprintln!("✓ Created branch: {}", branch);
            }
            if let Some(ref msg) = result.commit_message {
                eprintln!("\nCommit message:\n{}", msg);
            }
        } else {
            self.print_failure_report(result, target_improvement);
        }
    }

    fn print_failure_report(&self, result: &crate::phase3_merge::FinalizationResult, target_improvement: f64) {
        eprintln!("✗ Target improvement not met");
        eprintln!();

        if let Some(ref report) = result.failure_report {
            eprintln!("=== Failed Experiment Report ===");
            eprintln!();

            eprintln!("Best Improvement Achieved:");
            eprintln!("  Baseline: {:.2}", report.baseline);
            if let Some(best) = report.best_value {
                eprintln!("  Best value: {:.2}", best);
                eprintln!("  Improvement: {:+.2}%", report.best_improvement * 100.0);
            } else {
                eprintln!("  No improvement achieved");
            }
            eprintln!("  Target: {:+.2}%", report.target_improvement * 100.0);
            eprintln!(
                "  Gap: {:.2}%",
                (report.target_improvement - report.best_improvement) * 100.0
            );
            eprintln!();

            eprintln!("Exploration Summary:");
            eprintln!("  Iterations completed: {}", report.iterations_completed);
            if let Some(ref reason) = report.stuck_reason {
                eprintln!("  Stuck reason: {}", reason);
            }
            eprintln!();

            eprintln!("Recommendations for Retry:");
            for (i, rec) in report.recommendations.iter().enumerate() {
                eprintln!("  {}. {}", i + 1, rec);
            }
            eprintln!();

            eprintln!("Note: No changes were applied to the codebase.");
        } else {
            eprintln!(
                "  Target: {:.2}%, Achieved: {:+.2}%",
                target_improvement * 100.0,
                result.final_improvement * 100.0
            );
            if let Some(best) = result.best_value {
                eprintln!("  Best value: {:.2}", best);
            }
            if let Some(ref err) = result.error_message {
                eprintln!("  Error: {}", err);
            }
        }
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
