//! Export module for experiment results.
//!
//! This module provides functionality to export experiment sessions to various formats
//! including JSON, CSV, Markdown, and PDF.
//!
//! # Examples
//!
//! ```
//! use pi_autoresearch::export::export_json;
//! use pi_autoresearch::session::{ExperimentSession, SessionManager};
//! use pi_autoresearch::phase1_design::{ExperimentDesign, BaselineRecord};
//!
//! // After running an experiment, export results to JSON
//! let session = ExperimentSession::new(
//!     "session-123".to_string(),
//!     "How to optimize?".to_string(),
//!     ExperimentDesign::new(
//!         "hypothesis".to_string(),
//!         "metric".to_string(),
//!         "./measure".to_string(),
//!         100.0,
//!         10.0,
//!     ),
//!     BaselineRecord::new(
//!         "2024-01-01T00:00:00Z".to_string(),
//!         "abc".to_string(),
//!         "metric".to_string(),
//!         "./measure".to_string(),
//!         100.0,
//!         vec![100.0],
//!         0.0,
//!         true,
//!     ),
//! );
//!
//! // Export to JSON file
//! export_json(&session, "export.json").unwrap();
//! ```

use anyhow::Result;
use std::fs::OpenOptions;
use std::io::Write;

use crate::session::ExperimentSession;
use crate::cli::ExportFormat;

/// Export experiment session to JSON format.
///
/// Exports the complete ExperimentSession structure to a JSON file with pretty printing.
/// The exported JSON includes all session metadata, baseline, iterations, and results.
///
/// # Arguments
///
/// * `session` - The experiment session to export
/// * `path` - Path to the output JSON file
///
/// # Returns
///
/// * `Ok(())` if the export was successful
/// * `Err` if there was a file I/O or serialization error
///
/// # Examples
///
/// ```
/// use pi_autoresearch::export::export_json;
/// use pi_autoresearch::session::ExperimentSession;
/// use pi_autoresearch::phase1_design::{ExperimentDesign, BaselineRecord};
/// use tempfile::NamedTempFile;
///
/// let temp_file = NamedTempFile::new().unwrap();
/// let session = ExperimentSession::new(
///     "session-1".to_string(),
///     "question".to_string(),
///     ExperimentDesign::new(
///         "hypothesis".to_string(),
///         "metric".to_string(),
///         "./measure".to_string(),
///         100.0,
///         10.0,
///     ),
///     BaselineRecord::new(
///         "2024-01-01T00:00:00Z".to_string(),
///         "abc".to_string(),
///         "metric".to_string(),
///         "./measure".to_string(),
///         100.0,
///         vec![100.0],
///         0.0,
///         true,
///     ),
/// );
///
/// export_json(&session, temp_file.path()).unwrap();
/// ```
pub fn export_json(session: &ExperimentSession, path: &str) -> Result<()> {
    let json_output = serde_json::to_string_pretty(session)?;
    
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    
    file.write_all(json_output.as_bytes())?;
    eprintln!("Exported JSON to: {}", path);
    
    Ok(())
}

/// Export experiment session to CSV format.
///
/// Exports iteration data as CSV with columns for iteration number, timestamp,
/// metric value, improvement ratio, status, and runtime.
///
/// # Arguments
///
/// * `session` - The experiment session to export
/// * `path` - Path to the output CSV file
///
/// # Returns
///
/// * `Ok(())` if the export was successful
/// * `Err` if there was a file I/O error
///
/// # Examples
///
/// ```
/// use pi_autoresearch::export::export_csv;
/// use pi_autoresearch::session::ExperimentSession;
/// use pi_autoresearch::phase1_design::{ExperimentDesign, BaselineRecord};
/// use tempfile::NamedTempFile;
///
/// let temp_file = NamedTempFile::new().unwrap();
/// let session = ExperimentSession::new(
///     "session-1".to_string(),
///     "question".to_string(),
///     ExperimentDesign::new(
///         "hypothesis".to_string(),
///         "metric".to_string(),
///         "./measure".to_string(),
///         100.0,
///         10.0,
///     ),
///     BaselineRecord::new(
///         "2024-01-01T00:00:00Z".to_string(),
///         "abc".to_string(),
///         "metric".to_string(),
///         "./measure".to_string(),
///         100.0,
///         vec![100.0],
///         0.0,
///         true,
///     ),
/// );
///
/// export_csv(&session, temp_file.path()).unwrap();
/// ```
pub fn export_csv(session: &ExperimentSession, path: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    
    // Write metadata as comments
    writeln!(file, "# Experiment Session: {}", session.session_id)?;
    writeln!(file, "# Question: {}", session.question)?;
    writeln!(file, "# Metric: {}", session.design.metric)?;
    writeln!(file, "# Baseline: {}", session.baseline_record.value)?;
    writeln!(file, "# Status: {}", session.status)?;
    writeln!(file, "# Started: {}", session.start_time)?;
    if let Some(ref end_time) = session.end_time {
        writeln!(file, "# Ended: {}", end_time)?;
    }
    if let Some(best_idx) = session.best_iteration {
        writeln!(file, "# Best Iteration: {}", best_idx)?;
    }
    
    // Write CSV header
    writeln!(file, "iteration,timestamp,metric_value,improvement,kept")?;
    
    // Write baseline as first row (iteration 0)
    writeln!(
        file,
        "0,{},{},0.0,baseline",
        session.baseline_record.timestamp,
        session.baseline_record.value
    )?;
    
    // Write iteration rows
    for iteration in &session.iterations {
        let status = if iteration.kept { "kept" } else { "reverted" };
        writeln!(
            file,
            "{},{},{},{},{}",
            iteration.iteration,
            iteration.timestamp,
            iteration.metric_value,
            iteration.improvement,
            status
        )?;
    }
    
    eprintln!("Exported CSV to: {}", path);
    
    Ok(())
}

/// Export experiment session to Markdown format.
///
/// Exports a human-readable Markdown report with sections for summary,
/// timeline, and detailed results.
///
/// # Arguments
///
/// * `session` - The experiment session to export
/// * `path` - Path to the output Markdown file
///
/// # Returns
///
/// * `Ok(())` if the export was successful
/// * `Err` if there was a file I/O error
///
/// # Examples
///
/// ```
/// use pi_autoresearch::export::export_markdown;
/// use pi_autoresearch::session::ExperimentSession;
/// use pi_autoresearch::phase1_design::{ExperimentDesign, BaselineRecord};
/// use tempfile::NamedTempFile;
///
/// let temp_file = NamedTempFile::new().unwrap();
/// let session = ExperimentSession::new(
///     "session-1".to_string(),
///     "question".to_string(),
///     ExperimentDesign::new(
///         "hypothesis".to_string(),
///         "metric".to_string(),
///         "./measure".to_string(),
///         100.0,
///         10.0,
///     ),
///     BaselineRecord::new(
///         "2024-01-01T00:00:00Z".to_string(),
///         "abc".to_string(),
///         "metric".to_string(),
///         "./measure".to_string(),
///         100.0,
///         vec![100.0],
///         0.0,
///         true,
///     ),
/// );
///
/// export_markdown(&session, temp_file.path()).unwrap();
/// ```
pub fn export_markdown(session: &ExperimentSession, path: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    
    let improvement = session.calculate_final_improvement();
    let best_iteration = session.best_iteration;
    
    // Title
    writeln!(file, "# Experiment Results: {}", session.question)?;
    writeln!(file)?;
    writeln!(file, "**Session ID:** {}", session.session_id)?;
    writeln!(file)?;
    
    // Summary section
    writeln!(file, "## Summary")?;
    writeln!(file)?;
    writeln!(file, "| Property | Value |")?;
    writeln!(file, "|----------|-------|")?;
    writeln!(file, "| Metric | {} |", session.design.metric)?;
    writeln!(file, "| Baseline | {:.2} |", session.baseline_record.value)?;
    writeln!(file, "| Target Improvement | {:.1}% |", session.design.target_improvement * 100.0)?;
    writeln!(file, "| Total Iterations | {} |", session.iterations.len())?;
    writeln!(file, "| Best Improvement | {:+.1}% |", improvement * 100.0)?;
    if let Some(best_idx) = best_iteration {
        writeln!(file, "| Best Iteration | {} |", best_idx)?;
    }
    writeln!(file, "| Status | {} |", session.status)?;
    writeln!(file)?;
    
    // Timeline section
    writeln!(file, "## Iteration Timeline")?;
    writeln!(file)?;
    writeln!(file, "| Iteration | Timestamp | Value | Improvement | Status |")?;
    writeln!(file, "|-----------|-----------|-------|-------------|--------|")?;
    
    // Baseline row
    writeln!(
        file,
        "| 0 | {} | {:.2} | - | Baseline |",
        session.baseline_record.timestamp,
        session.baseline_record.value
    )?;
    
    // Iteration rows
    for iteration in &session.iterations {
        let status = if iteration.kept { "✓ Kept" } else { "✗ Reverted" };
        writeln!(
            file,
            "| {} | {} | {:.2} | {:+.1}% | {} |",
            iteration.iteration,
            iteration.timestamp,
            iteration.metric_value,
            iteration.improvement * 100.0,
            status
        )?;
    }
    writeln!(file)?;
    
    // Details section
    writeln!(file, "## Details")?;
    writeln!(file)?;
    writeln!(file, "### Hypothesis")?;
    writeln!(file)?;
    writeln!(file, "{}", session.design.hypothesis)?;
    writeln!(file)?;
    
    writeln!(file, "### Measurement")?;
    writeln!(file)?;
    writeln!(file, "- **Command:** {}", session.design.measurement)?;
    writeln!(file, "- **Variance:** {:.2}%", session.baseline_record.variance * 100.0)?;
    writeln!(file)?;
    
    // Metadata section
    writeln!(file, "### Metadata")?;
    writeln!(file)?;
    writeln!(file, "- **Started:** {}", session.start_time)?;
    if let Some(ref end_time) = session.end_time {
        writeln!(file, "- **Ended:** {}", end_time)?;
    }
    writeln!(file, "- **Exported:** {}", chrono::Utc::now().to_rfc3339())?;
    writeln!(file, "- **Tool:** pi-autoresearch")?;
    
    eprintln!("Exported Markdown to: {}", path);
    
    Ok(())
}

/// Export experiment session to PDF format.
///
/// Currently generates a Markdown file and suggests conversion to PDF.
/// This is a placeholder for future PDF generation implementation.
///
/// # Arguments
///
/// * `session` - The experiment session to export
/// * `path` - Path to the output PDF file
///
/// # Returns
///
/// * `Ok(())` if the export was successful
/// * `Err` if there was a file I/O error
///
/// # Examples
///
/// ```
/// use pi_autoresearch::export::export_pdf;
/// use pi_autoresearch::session::ExperimentSession;
/// use pi_autoresearch::phase1_design::{ExperimentDesign, BaselineRecord};
/// use tempfile::NamedTempFile;
///
/// let temp_file = NamedTempFile::new().unwrap();
/// let session = ExperimentSession::new(
///     "session-1".to_string(),
///     "question".to_string(),
///     ExperimentDesign::new(
///         "hypothesis".to_string(),
///         "metric".to_string(),
///         "./measure".to_string(),
///         100.0,
///         10.0,
///     ),
///     BaselineRecord::new(
///         "2024-01-01T00:00:00Z".to_string(),
///         "abc".to_string(),
///         "metric".to_string(),
///         "./measure".to_string(),
///         100.0,
///         vec![100.0],
///         0.0,
///         true,
///     ),
/// );
///
/// export_pdf(&session, temp_file.path()).unwrap();
/// ```
pub fn export_pdf(session: &ExperimentSession, path: &str) -> Result<()> {
    // For now, generate a simple text file with PDF-like content
    // In the future, this could use a PDF library or call an external tool
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    
    let improvement = session.calculate_final_improvement();
    
    writeln!(file, "EXPERIMENT RESULTS")?;
    writeln!(file, "==================")?;
    writeln!(file)?;
    writeln!(file, "Question: {}", session.question)?;
    writeln!(file, "Session ID: {}", session.session_id)?;
    writeln!(file)?;
    writeln!(file, "SUMMARY")?;
    writeln!(file, "-------")?;
    writeln!(file, "Metric: {}", session.design.metric)?;
    writeln!(file, "Baseline: {:.2}", session.baseline_record.value)?;
    writeln!(file, "Target Improvement: {:.1}%", session.design.target_improvement * 100.0)?;
    writeln!(file, "Total Iterations: {}", session.iterations.len())?;
    writeln!(file, "Best Improvement: {:+.1}%", improvement * 100.0)?;
    writeln!(file, "Status: {}", session.status)?;
    writeln!(file)?;
    writeln!(file, "ITERATIONS")?;
    writeln!(file, "----------")?;
    
    for iteration in &session.iterations {
        let status = if iteration.kept { "Kept" } else { "Reverted" };
        writeln!(
            file,
            "Iteration {}: {:.2} ({:+.1}%) - {}",
            iteration.iteration,
            iteration.metric_value,
            iteration.improvement * 100.0,
            status
        )?;
    }
    
    writeln!(file)?;
    writeln!(file, "Generated by pi-autoresearch at {}", chrono::Utc::now().to_rfc3339())?;
    writeln!(file)?;
    writeln!(file, "Note: This is a text representation. For a proper PDF, consider:")?;
    writeln!(file, "  1. Using --export markdown and converting with pandoc")?;
    writeln!(file, "  2. Using a PDF generation library in a future version")?;
    
    eprintln!("Exported PDF (text format) to: {}", path);
    eprintln!("Note: PDF export currently generates text format. Consider using --export markdown for better formatting.");
    
    Ok(())
}

/// Export experiment session to the specified format.
///
/// Dispatches to the appropriate export function based on the format.
///
/// # Arguments
///
/// * `session` - The experiment session to export
/// * `format` - The export format (JSON, CSV, Markdown, or PDF)
/// * `path` - Path to the output file
///
/// # Returns
///
/// * `Ok(())` if the export was successful
/// * `Err` if there was a file I/O or serialization error
///
/// # Examples
///
/// ```
/// use pi_autoresearch::export::export;
/// use pi_autoresearch::cli::ExportFormat;
/// use pi_autoresearch::session::ExperimentSession;
/// use pi_autoresearch::phase1_design::{ExperimentDesign, BaselineRecord};
/// use tempfile::NamedTempFile;
///
/// let temp_file = NamedTempFile::new().unwrap();
/// let session = ExperimentSession::new(
///     "session-1".to_string(),
///     "question".to_string(),
///     ExperimentDesign::new(
///         "hypothesis".to_string(),
///         "metric".to_string(),
///         "./measure".to_string(),
///         100.0,
///         10.0,
///     ),
///     BaselineRecord::new(
///         "2024-01-01T00:00:00Z".to_string(),
///         "abc".to_string(),
///         "metric".to_string(),
///         "./measure".to_string(),
///         100.0,
///         vec![100.0],
///         0.0,
///         true,
///     ),
/// );
///
/// export(&session, ExportFormat::Json, temp_file.path()).unwrap();
/// ```
pub fn export(session: &ExperimentSession, format: ExportFormat, path: &str) -> Result<()> {
    match format {
        ExportFormat::Json => export_json(session, path),
        ExportFormat::Csv => export_csv(session, path),
        ExportFormat::Markdown => export_markdown(session, path),
        ExportFormat::Pdf => export_pdf(session, path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase1_design::{BaselineRecord, ExperimentDesign};
    use crate::phase2_iterate::IterationRecord;

    fn create_test_session() -> ExperimentSession {
        ExperimentSession::new(
            "test-session".to_string(),
            "How to optimize performance?".to_string(),
            ExperimentDesign::new(
                "Reduce memory allocations".to_string(),
                "memory_mb".to_string(),
                "./measure-memory".to_string(),
                100.0,
                10.0,
            ),
            BaselineRecord::new(
                "2024-01-01T00:00:00Z".to_string(),
                "abc123".to_string(),
                "memory_mb".to_string(),
                "./measure-memory".to_string(),
                100.0,
                vec![100.0, 101.0],
                0.01,
                true,
            ),
        )
    }

    #[test]
    fn test_export_json() {
        let mut session = create_test_session();
        let iteration = IterationRecord::new(
            1,
            "Optimized data structures".to_string(),
            90.0,
            10.0,
            true,
        );
        session.add_iteration(iteration);
        session.finalize(Some(1), "completed".to_string());

        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let result = export_json(&session, temp_file.path().to_str().unwrap());

        assert!(result.is_ok());
        
        // Verify the file contains valid JSON
        let contents = std::fs::read_to_string(temp_file.path()).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&contents).unwrap();
        assert_eq!(parsed["session_id"].as_str(), Some("test-session"));
        assert_eq!(parsed["status"].as_str(), Some("completed"));
    }

    #[test]
    fn test_export_csv() {
        let mut session = create_test_session();
        let iteration1 = IterationRecord::new(
            1,
            "First optimization".to_string(),
            90.0,
            10.0,
            true,
        );
        let iteration2 = IterationRecord::new(
            2,
            "Second optimization".to_string(),
            95.0,
            5.0,
            false,
        );
        session.add_iteration(iteration1);
        session.add_iteration(iteration2);

        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let result = export_csv(&session, temp_file.path().to_str().unwrap());

        assert!(result.is_ok());
        
        let contents = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(contents.contains("iteration,timestamp,metric_value,improvement,kept"));
        assert!(contents.contains("0,")); // Baseline row
        assert!(contents.contains("1,")); // Iteration 1
        assert!(contents.contains("2,")); // Iteration 2
        assert!(contents.contains("# Experiment Session: test-session"));
    }

    #[test]
    fn test_export_markdown() {
        let mut session = create_test_session();
        let iteration = IterationRecord::new(
            1,
            "Optimization action".to_string(),
            80.0,
            20.0,
            true,
        );
        session.add_iteration(iteration);
        session.finalize(Some(1), "completed".to_string());

        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let result = export_markdown(&session, temp_file.path().to_str().unwrap());

        assert!(result.is_ok());
        
        let contents = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(contents.contains("# Experiment Results:"));
        assert!(contents.contains("## Summary"));
        assert!(contents.contains("## Iteration Timeline"));
        assert!(contents.contains("## Details"));
        assert!(contents.contains("| Metric | memory_mb |"));
    }

    #[test]
    fn test_export_pdf() {
        let session = create_test_session();

        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let result = export_pdf(&session, temp_file.path().to_str().unwrap());

        assert!(result.is_ok());
        
        let contents = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(contents.contains("EXPERIMENT RESULTS"));
        assert!(contents.contains("SUMMARY"));
        assert!(contents.contains("ITERATIONS"));
    }

    #[test]
    fn test_export_dispatch_json() {
        let session = create_test_session();
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        
        let result = export(&session, ExportFormat::Json, temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_dispatch_csv() {
        let session = create_test_session();
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        
        let result = export(&session, ExportFormat::Csv, temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_dispatch_markdown() {
        let session = create_test_session();
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        
        let result = export(&session, ExportFormat::Markdown, temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_dispatch_pdf() {
        let session = create_test_session();
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        
        let result = export(&session, ExportFormat::Pdf, temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_json_empty_iterations() {
        let session = create_test_session();
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        
        let result = export_json(&session, temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let contents = std::fs::read_to_string(temp_file.path()).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&contents).unwrap();
        assert!(parsed["iterations"].is_array());
        assert_eq!(parsed["iterations"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_export_csv_empty_iterations() {
        let session = create_test_session();
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        
        let result = export_csv(&session, temp_file.path().to_str().unwrap());
        assert!(result.is_ok());
        
        let contents = std::fs::read_to_string(temp_file.path()).unwrap();
        // Should have header and baseline row only
        let lines: Vec<&str> = contents.lines().filter(|l| !l.starts_with('#')).collect();
        assert_eq!(lines.len(), 2); // Header + baseline
    }
}
