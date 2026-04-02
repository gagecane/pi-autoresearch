use clap::{Parser, ValueEnum};

/// Export format for experiment results
#[derive(ValueEnum, Debug, Clone, Default, PartialEq, Eq)]
pub enum ExportFormat {
    /// CSV format for spreadsheet analysis
    #[value(name = "csv")]
    Csv,
    /// JSON format for programmatic access
    #[value(name = "json")]
    #[default]
    Json,
    /// PDF format for professional reports
    #[value(name = "pdf")]
    Pdf,
    /// Markdown format for human-readable reports
    #[value(name = "markdown")]
    Markdown,
}

impl ExportFormat {
    /// Returns the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Csv => "csv",
            ExportFormat::Json => "json",
            ExportFormat::Pdf => "pdf",
            ExportFormat::Markdown => "md",
        }
    }
}

#[derive(Parser, Debug, Clone, Default)]
#[command(name = "pi-autoresearch")]
#[command(about = "Autonomous research experiment orchestrator")]
pub struct Cli {
    #[arg(long)] pub question: Option<String>,
    #[arg(long)] pub auto_approve: bool,
    #[arg(long)] pub metric: Option<String>,
    #[arg(long)] pub measure: Option<String>,
    #[arg(long)] pub baseline: Option<f64>,
    #[arg(long)] pub target_improvement: Option<f64>,
    #[arg(long)] pub max_iterations: Option<usize>,
    #[arg(long, alias = "iteration-timeout-minutes")] pub iteration_timeout_minutes: Option<usize>,
    #[arg(long, alias = "total-timeout-minutes")] pub total_timeout_minutes: Option<usize>,
    #[arg(long)] pub stall_limit: Option<usize>,
    #[arg(long)] pub convergence_threshold: Option<f64>,
    #[arg(long)] pub convergence_window: Option<usize>,
    #[arg(long)] pub verify_baseline: bool,
    #[arg(long, default_value = "autoresearch.jsonl")] pub session_file: String,
    #[arg(long, default_value = "0.05")] pub max_variance: f64,
    #[arg(long)] pub verbose: bool,
    #[arg(long)] pub quiet: bool,
    #[arg(long)] pub resume: Option<String>,
    #[arg(long)] pub history: bool,
    #[arg(long)] pub beads_enabled: bool,
    #[arg(long)] pub ralph_tui_enabled: bool,
    #[arg(long)] pub ralph_task_file: Option<String>,
    #[arg(long, value_enum)] pub export: Option<ExportFormat>,
    #[arg(long, alias = "export-path")] pub export_path: Option<String>,
}

impl Cli {
    /// Returns the effective max iterations value, using CLI value if provided, otherwise default of 20.
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::cli::Cli;
    /// let cli = Cli { max_iterations: None, ..Default::default() };
    /// assert_eq!(cli.effective_max_iterations(), 20);
    ///
    /// let cli = Cli { max_iterations: Some(50), ..Default::default() };
    /// assert_eq!(cli.effective_max_iterations(), 50);
    /// ```
    pub fn effective_max_iterations(&self) -> usize { self.max_iterations.unwrap_or(20) }

    /// Returns the effective iteration timeout in seconds, using CLI value if provided, otherwise default of 10 minutes.
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::cli::Cli;
    /// let cli = Cli { iteration_timeout_minutes: None, ..Default::default() };
    /// assert_eq!(cli.effective_iteration_timeout_secs(), 600); // 10 minutes
    ///
    /// let cli = Cli { iteration_timeout_minutes: Some(30), ..Default::default() };
    /// assert_eq!(cli.effective_iteration_timeout_secs(), 1800); // 30 minutes
    /// ```
    pub fn effective_iteration_timeout_secs(&self) -> u64 { self.iteration_timeout_minutes.unwrap_or(10) as u64 * 60 }

    /// Returns the effective total timeout in seconds, using CLI value if provided, otherwise default of 120 minutes.
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::cli::Cli;
    /// let cli = Cli { total_timeout_minutes: None, ..Default::default() };
    /// assert_eq!(cli.effective_total_timeout_secs(), 7200); // 120 minutes
    ///
    /// let cli = Cli { total_timeout_minutes: Some(60), ..Default::default() };
    /// assert_eq!(cli.effective_total_timeout_secs(), 3600); // 60 minutes
    /// ```
    pub fn effective_total_timeout_secs(&self) -> u64 { self.total_timeout_minutes.unwrap_or(120) as u64 * 60 }

    /// Returns the effective stall limit value, using CLI value if provided, otherwise default of 5.
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::cli::Cli;
    /// let cli = Cli { stall_limit: None, ..Default::default() };
    /// assert_eq!(cli.effective_stall_limit(), 5);
    ///
    /// let cli = Cli { stall_limit: Some(10), ..Default::default() };
    /// assert_eq!(cli.effective_stall_limit(), 10);
    /// ```
    pub fn effective_stall_limit(&self) -> usize { self.stall_limit.unwrap_or(5) }

    /// Returns the effective convergence threshold value, using CLI value if provided, otherwise default of 0.01.
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::cli::Cli;
    /// let cli = Cli { convergence_threshold: None, ..Default::default() };
    /// assert_eq!(cli.effective_convergence_threshold(), 0.01);
    ///
    /// let cli = Cli { convergence_threshold: Some(0.05), ..Default::default() };
    /// assert_eq!(cli.effective_convergence_threshold(), 0.05);
    /// ```
    pub fn effective_convergence_threshold(&self) -> f64 { self.convergence_threshold.unwrap_or(0.01) }

    /// Returns the effective convergence window value, using CLI value if provided, otherwise default of 3.
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::cli::Cli;
    /// let cli = Cli { convergence_window: None, ..Default::default() };
    /// assert_eq!(cli.effective_convergence_window(), 3);
    ///
    /// let cli = Cli { convergence_window: Some(10), ..Default::default() };
    /// assert_eq!(cli.effective_convergence_window(), 10);
    /// ```
    pub fn effective_convergence_window(&self) -> usize { self.convergence_window.unwrap_or(3) }

    /// Returns the export format if specified, otherwise None.
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::cli::{Cli, ExportFormat};
    /// let cli = Cli { export: None, ..Default::default() };
    /// assert!(cli.get_export_format().is_none());
    ///
    /// let cli = Cli { export: Some(ExportFormat::Csv), ..Default::default() };
    /// assert_eq!(cli.get_export_format(), Some(&ExportFormat::Csv));
    /// ```
    pub fn get_export_format(&self) -> Option<&ExportFormat> {
        self.export.as_ref()
    }

    /// Returns the export path, generating a default if not specified.
    ///
    /// The default format is: `export_{session_id}_{format}.{ext}`
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID to use in the default filename
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::cli::{Cli, ExportFormat};
    /// let cli = Cli { export_path: None, export: Some(ExportFormat::Json), ..Default::default() };
    /// let path = cli.get_export_path("session_123");
    /// assert!(path.contains("export_session_123_json.json"));
    ///
    /// let cli = Cli { export_path: Some("/custom/path.json".to_string()), ..Default::default() };
    /// assert_eq!(cli.get_export_path("session_123"), "/custom/path.json");
    /// ```
    pub fn get_export_path(&self, session_id: &str) -> String {
        if let Some(ref path) = self.export_path {
            path.clone()
        } else {
            let format_ext = self.export.as_ref().map_or("json", |f| f.extension());
            format!("export_{}_{}.{}", session_id, format_ext, format_ext)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effective_max_iterations_default() {
        let cli = Cli {
            max_iterations: None,
            ..Default::default()
        };
        assert_eq!(cli.effective_max_iterations(), 20);
    }

    #[test]
    fn test_effective_max_iterations_custom() {
        let cli = Cli {
            max_iterations: Some(50),
            ..Default::default()
        };
        assert_eq!(cli.effective_max_iterations(), 50);
    }

    #[test]
    fn test_effective_iteration_timeout_secs_default() {
        let cli = Cli {
            iteration_timeout_minutes: None,
            ..Default::default()
        };
        assert_eq!(cli.effective_iteration_timeout_secs(), 600); // 10 minutes * 60
    }

    #[test]
    fn test_effective_iteration_timeout_secs_custom() {
        let cli = Cli {
            iteration_timeout_minutes: Some(30),
            ..Default::default()
        };
        assert_eq!(cli.effective_iteration_timeout_secs(), 1800); // 30 minutes * 60
    }

    #[test]
    fn test_effective_total_timeout_secs_default() {
        let cli = Cli {
            total_timeout_minutes: None,
            ..Default::default()
        };
        assert_eq!(cli.effective_total_timeout_secs(), 7200); // 120 minutes * 60
    }

    #[test]
    fn test_effective_total_timeout_secs_custom() {
        let cli = Cli {
            total_timeout_minutes: Some(60),
            ..Default::default()
        };
        assert_eq!(cli.effective_total_timeout_secs(), 3600); // 60 minutes * 60
    }

    #[test]
    fn test_effective_stall_limit_default() {
        let cli = Cli {
            stall_limit: None,
            ..Default::default()
        };
        assert_eq!(cli.effective_stall_limit(), 5);
    }

    #[test]
    fn test_effective_stall_limit_custom() {
        let cli = Cli {
            stall_limit: Some(10),
            ..Default::default()
        };
        assert_eq!(cli.effective_stall_limit(), 10);
    }

    #[test]
    fn test_effective_convergence_threshold_default() {
        let cli = Cli {
            convergence_threshold: None,
            ..Default::default()
        };
        assert_eq!(cli.effective_convergence_threshold(), 0.01);
    }

    #[test]
    fn test_effective_convergence_threshold_custom() {
        let cli = Cli {
            convergence_threshold: Some(0.05),
            ..Default::default()
        };
        assert_eq!(cli.effective_convergence_threshold(), 0.05);
    }

    #[test]
    fn test_effective_convergence_window_default() {
        let cli = Cli {
            convergence_window: None,
            ..Default::default()
        };
        assert_eq!(cli.effective_convergence_window(), 3);
    }

    #[test]
    fn test_effective_convergence_window_custom() {
        let cli = Cli {
            convergence_window: Some(10),
            ..Default::default()
        };
        assert_eq!(cli.effective_convergence_window(), 10);
    }

    #[test]
    fn test_cli_parse_question() {
        let cli = Cli::parse_from(["test", "--question", "How can I optimize?"]);
        assert_eq!(cli.question, Some("How can I optimize?".to_string()));
    }

    #[test]
    fn test_cli_parse_all_options() {
        let cli = Cli::parse_from([
            "test",
            "--question", "Test question",
            "--auto-approve",
            "--metric", "execution_time_ms",
            "--measure", "./measure.sh",
            "--baseline", "100.0",
            "--target-improvement", "0.20",
            "--max-iterations", "10",
            "--iteration-timeout-minutes", "30",
            "--total-timeout-minutes", "120",
            "--stall-limit", "3",
            "--convergence-threshold", "0.05",
            "--convergence-window", "5",
            "--verify-baseline",
            "--session-file", "test.jsonl",
            "--max-variance", "0.10",
            "--verbose",
            "--beads-enabled",
        ]);

        assert_eq!(cli.question, Some("Test question".to_string()));
        assert!(cli.auto_approve);
        assert_eq!(cli.metric, Some("execution_time_ms".to_string()));
        assert_eq!(cli.measure, Some("./measure.sh".to_string()));
        assert_eq!(cli.baseline, Some(100.0));
        assert_eq!(cli.target_improvement, Some(0.20));
        assert_eq!(cli.max_iterations, Some(10));
        assert_eq!(cli.iteration_timeout_minutes, Some(30));
        assert_eq!(cli.total_timeout_minutes, Some(120));
        assert_eq!(cli.stall_limit, Some(3));
        assert_eq!(cli.convergence_threshold, Some(0.05));
        assert_eq!(cli.convergence_window, Some(5));
        assert!(cli.verify_baseline);
        assert_eq!(cli.session_file, "test.jsonl");
        assert_eq!(cli.max_variance, 0.10);
        assert!(cli.verbose);
        assert!(cli.beads_enabled);
    }

    #[test]
    fn test_cli_default_values() {
        let cli = Cli::parse_from(["test"]);
        assert_eq!(cli.session_file, "autoresearch.jsonl");
        assert_eq!(cli.max_variance, 0.05);
        assert!(!cli.auto_approve);
        assert!(!cli.verbose);
        assert!(!cli.quiet);
        assert!(!cli.beads_enabled);
    }

    #[test]
    fn test_export_format_extension() {
        assert_eq!(ExportFormat::Csv.extension(), "csv");
        assert_eq!(ExportFormat::Json.extension(), "json");
        assert_eq!(ExportFormat::Pdf.extension(), "pdf");
        assert_eq!(ExportFormat::Markdown.extension(), "md");
    }

    #[test]
    fn test_export_format_default() {
        assert_eq!(ExportFormat::default(), ExportFormat::Json);
    }

    #[test]
    fn test_cli_parse_export_format() {
        let cli = Cli::parse_from(["test", "--export", "csv"]);
        assert_eq!(cli.export, Some(ExportFormat::Csv));

        let cli = Cli::parse_from(["test", "--export", "json"]);
        assert_eq!(cli.export, Some(ExportFormat::Json));

        let cli = Cli::parse_from(["test", "--export", "pdf"]);
        assert_eq!(cli.export, Some(ExportFormat::Pdf));

        let cli = Cli::parse_from(["test", "--export", "markdown"]);
        assert_eq!(cli.export, Some(ExportFormat::Markdown));
    }

    #[test]
    fn test_cli_parse_export_path() {
        let cli = Cli::parse_from(["test", "--export-path", "/custom/path.json"]);
        assert_eq!(cli.export_path, Some("/custom/path.json".to_string()));
    }

    #[test]
    fn test_get_export_format_none() {
        let cli = Cli { export: None, ..Default::default() };
        assert!(cli.get_export_format().is_none());
    }

    #[test]
    fn test_get_export_format_some() {
        let cli = Cli { export: Some(ExportFormat::Csv), ..Default::default() };
        assert_eq!(cli.get_export_format(), Some(&ExportFormat::Csv));
    }

    #[test]
    fn test_get_export_path_custom() {
        let cli = Cli {
            export_path: Some("/custom/path.json".to_string()),
            ..Default::default()
        };
        assert_eq!(cli.get_export_path("session_123"), "/custom/path.json");
    }

    #[test]
    fn test_get_export_path_default_json() {
        let cli = Cli {
            export_path: None,
            export: Some(ExportFormat::Json),
            ..Default::default()
        };
        let path = cli.get_export_path("session_123");
        assert_eq!(path, "export_session_123_json.json");
    }

    #[test]
    fn test_get_export_path_default_csv() {
        let cli = Cli {
            export_path: None,
            export: Some(ExportFormat::Csv),
            ..Default::default()
        };
        let path = cli.get_export_path("session_456");
        assert_eq!(path, "export_session_456_csv.csv");
    }

    #[test]
    fn test_get_export_path_default_markdown() {
        let cli = Cli {
            export_path: None,
            export: Some(ExportFormat::Markdown),
            ..Default::default()
        };
        let path = cli.get_export_path("session_789");
        assert_eq!(path, "export_session_789_md.md");
    }

    #[test]
    fn test_get_export_path_default_when_no_format() {
        let cli = Cli {
            export_path: None,
            export: None,
            ..Default::default()
        };
        let path = cli.get_export_path("session_abc");
        assert_eq!(path, "export_session_abc_json.json");
    }

    #[test]
    fn test_cli_parse_export_and_path() {
        let cli = Cli::parse_from([
            "test",
            "--export", "csv",
            "--export-path", "/tmp/report.csv"
        ]);
        assert_eq!(cli.export, Some(ExportFormat::Csv));
        assert_eq!(cli.export_path, Some("/tmp/report.csv".to_string()));
    }
}
