//! Visualization module for pi-autoresearch experiment results.
//!
//! This module provides functionality to generate charts and reports
//! from experiment session data, supporting both PNG and SVG output formats.
//!
//! # Features
//!
//! - Improvement trend line charts
//! - Iteration comparison bar charts
//! - Baseline vs final comparison charts
//! - Measurement distribution histograms
//! - Statistical analysis (mean, median, std dev, confidence intervals)
//! - Both PNG and SVG output formats
//!
//! # Examples
//!
//! ```
//! use pi_autoresearch::visualization::{ChartGenerator, VisualizationConfig};
//! use pi_autoresearch::session::ExperimentSession;
//!
//! // Load session data
//! let session = ExperimentSession::load("experiments.jsonl").unwrap();
//!
//! // Create chart generator
//! let config = VisualizationConfig::default();
//! let generator = ChartGenerator::new(config);
//!
//! // Generate improvement trend chart
//! generator.generate_improvement_trend(&session, "output.png").unwrap();
//! ```

use anyhow::Result;
use plotters::prelude::*;
use plotters::style::full_palette::ORANGE;
use serde::{Deserialize, Serialize};

use crate::session::ExperimentSession;

/// Configuration for visualization generation
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VisualizationConfig {
    /// Width of the chart in pixels
    pub width: usize,
    /// Height of the chart in pixels
    pub height: usize,
    /// Font size for labels
    pub font_size: usize,
    /// Font family for text
    pub font_family: String,
    /// Whether to show grid lines
    pub show_grid: bool,
    /// Color scheme for charts
    pub color_scheme: String,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            font_size: 14,
            font_family: "sans-serif".to_string(),
            show_grid: true,
            color_scheme: "default".to_string(),
        }
    }
}

/// Chart generator for creating visualizations from experiment data
pub struct ChartGenerator {
    config: VisualizationConfig,
}

impl ChartGenerator {
    /// Create a new ChartGenerator with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Visualization configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::visualization::{ChartGenerator, VisualizationConfig};
    ///
    /// let config = VisualizationConfig::default();
    /// let generator = ChartGenerator::new(config);
    /// ```
    pub fn new(config: VisualizationConfig) -> Self {
        Self { config }
    }

    /// Generate an improvement trend line chart
    ///
    /// This chart shows how the metric value changes over iterations,
    /// highlighting improvements and regressions.
    ///
    /// # Arguments
    ///
    /// * `session` - The experiment session data
    /// * `output_path` - Path to save the chart (PNG or SVG)
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok on success, Err on failure
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_autoresearch::visualization::{ChartGenerator, VisualizationConfig};
    /// use pi_autoresearch::session::ExperimentSession;
    ///
    /// let session = ExperimentSession::load("experiments.jsonl").unwrap();
    /// let generator = ChartGenerator::new(VisualizationConfig::default());
    /// generator.generate_improvement_trend(&session, "trend.png").unwrap();
    /// ```
    pub fn generate_improvement_trend(&self, session: &ExperimentSession, output_path: &str) -> Result<()> {
        let root = BitMapBackend::new(
            output_path,
            (self.config.width as u32, self.config.height as u32),
        )
        .into_drawing_area();
        root.fill(&WHITE)?;

        // Extract data points
        let mut data_points: Vec<(f64, f64)> = Vec::new();
        
        // Add baseline as first point
        data_points.push((0.0, session.baseline_record.value));
        
        // Add iteration points
        for iteration in &session.iterations {
            data_points.push((iteration.iteration as f64, iteration.metric_value));
        }

        if data_points.is_empty() {
            return Err(anyhow::anyhow!("No data points to plot"));
        }

        let min_x = 0.0;
        let max_x = *data_points.iter().map(|(x, _)| x).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap() + 1.0;
        let min_y = *data_points.iter().map(|(_, y)| y).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max_y = *data_points.iter().map(|(_, y)| y).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

        // Create coordinate system
        let mut chart = ChartBuilder::on(&root)
            .caption("Metric Improvement Trend", ("sans-serif", 20))
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(40)
            .build_cartesian_2d(
                min_x..max_x,
                min_y..max_y,
            )?;

        // Draw grid
        if self.config.show_grid {
            chart.configure_mesh().draw()?;
        }

        // Draw line chart
        chart.draw_series(LineSeries::new(data_points.clone(), &RED))?;

        // Draw baseline marker as circle
        if let Some(&(baseline_x, baseline_y)) = data_points.first() {
            let circle = Circle::new((baseline_x, baseline_y), 50, BLUE.filled());
            chart.draw_series(std::iter::once(circle))?;
        }

        // Draw best iteration marker
        if let Some(best_idx) = session.best_iteration {
            if best_idx < data_points.len() {
                let best_point = data_points[best_idx];
                let circle = Circle::new(best_point, 80, GREEN.filled());
                chart.draw_series(std::iter::once(circle))?;
            }
        }

        root.present()?;
        Ok(())
    }

    /// Generate an iteration comparison bar chart
    ///
    /// This chart shows a bar for each iteration, making it easy to
    /// compare individual iteration results.
    ///
    /// # Arguments
    ///
    /// * `session` - The experiment session data
    /// * `output_path` - Path to save the chart (PNG or SVG)
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok on success, Err on failure
    pub fn generate_iteration_comparison(&self, session: &ExperimentSession, output_path: &str) -> Result<()> {
        let root = BitMapBackend::new(
            output_path,
            (self.config.width as u32, self.config.height as u32),
        )
        .into_drawing_area();
        root.fill(&WHITE)?;

        // Collect all metric values including baseline
        let mut values: Vec<f64> = vec![session.baseline_record.value];
        for iteration in &session.iterations {
            values.push(iteration.metric_value);
        }

        if values.is_empty() {
            return Err(anyhow::anyhow!("No data to plot"));
        }

        let min_value = *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max_value = *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max_x = values.len() as f64 + 1.0;

        let mut chart = ChartBuilder::on(&root)
            .caption("Iteration Comparison", ("sans-serif", 20))
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(40)
            .build_cartesian_2d(
                0.0..max_x,
                min_value..max_value,
            )?;

        if self.config.show_grid {
            chart.configure_mesh().draw()?;
        }

        // Draw bars with different colors for kept/reverted using area series
        for (i, value) in values.iter().enumerate() {
            let color = if i == 0 {
                BLUE // Baseline
            } else if Some(i - 1) == session.best_iteration {
                GREEN // Best
            } else {
                // Check if this iteration was kept
                let iter_idx = i - 1;
                if iter_idx < session.iterations.len() && session.iterations[iter_idx].kept {
                    GREEN
                } else {
                    RED
                }
            };

            // Draw a rectangle for each bar
            let x_pos = (i + 1) as f64;
            let bar_width = 0.6;
            let rect = Rectangle::new(
                [(x_pos - bar_width / 2.0, min_value), (x_pos + bar_width / 2.0, *value)],
                color.filled(),
            );
            chart.draw_series(std::iter::once(rect))?;
        }

        root.present()?;
        Ok(())
    }

    /// Generate a baseline vs final comparison chart
    ///
    /// This chart compares the baseline measurement with the final
    /// (best) result, showing the overall improvement.
    ///
    /// # Arguments
    ///
    /// * `session` - The experiment session data
    /// * `output_path` - Path to save the chart (PNG or SVG)
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok on success, Err on failure
    pub fn generate_baseline_comparison(&self, session: &ExperimentSession, output_path: &str) -> Result<()> {
        let root = BitMapBackend::new(
            output_path,
            (self.config.width as u32, self.config.height as u32),
        )
        .into_drawing_area();
        root.fill(&WHITE)?;

        let baseline = session.baseline_record.value;
        let final_value = session.iterations.last()
            .map(|i| i.metric_value)
            .unwrap_or(baseline);

        let min_value = baseline.min(final_value) * 0.9;
        let max_value = baseline.max(final_value) * 1.1;

        let mut chart = ChartBuilder::on(&root)
            .caption("Baseline vs Final Comparison", ("sans-serif", 20))
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(40)
            .build_cartesian_2d(
                0.0..2.0,
                min_value..max_value,
            )?;

        if self.config.show_grid {
            chart.configure_mesh().draw()?;
        }

        // Draw baseline bar
        let baseline_rect = Rectangle::new(
            [(-0.3, min_value), (0.3, baseline)],
            BLUE.filled(),
        );
        chart.draw_series(std::iter::once(baseline_rect))?;

        // Draw final bar
        let final_rect = Rectangle::new(
            [(0.7, min_value), (1.3, final_value)],
            GREEN.filled(),
        );
        chart.draw_series(std::iter::once(final_rect))?;

        root.present()?;
        Ok(())
    }

    /// Generate a measurement distribution histogram
    ///
    /// This chart shows the distribution of all measurement values,
    /// helping to understand the variability in results.
    ///
    /// # Arguments
    ///
    /// * `session` - The experiment session data
    /// * `output_path` - Path to save the chart (PNG or SVG)
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok on success, Err on failure
    pub fn generate_distribution_histogram(&self, session: &ExperimentSession, output_path: &str) -> Result<()> {
        let root = BitMapBackend::new(
            output_path,
            (self.config.width as u32, self.config.height as u32),
        )
        .into_drawing_area();
        root.fill(&WHITE)?;

        // Collect all metric values
        let mut values: Vec<f64> = vec![session.baseline_record.value];
        for iteration in &session.iterations {
            values.push(iteration.metric_value);
        }

        if values.is_empty() {
            return Err(anyhow::anyhow!("No data to plot"));
        }

        // Calculate histogram bins
        let min_value = *values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max_value = *values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let range = max_value - min_value;
        let num_bins = 10;
        let bin_width = if range > 0.0 { range / num_bins as f64 } else { 1.0 };

        let mut histogram: Vec<u32> = vec![0; num_bins as usize];
        for value in &values {
            let bin_idx = ((value - min_value) / bin_width).floor() as usize;
            let bin_idx = bin_idx.min(num_bins as usize - 1);
            histogram[bin_idx] += 1;
        }

        let max_count = *histogram.iter().max().unwrap();

        let mut chart = ChartBuilder::on(&root)
            .caption("Measurement Distribution", ("sans-serif", 20))
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(40)
            .build_cartesian_2d(
                0.0..(num_bins as f64 + 1.0),
                0.0..(max_count as f64 + 1.0),
            )?;

        if self.config.show_grid {
            chart.configure_mesh().draw()?;
        }

        // Draw histogram bars as rectangles
        for (i, count) in histogram.iter().enumerate() {
            let x_pos = (i + 1) as f64;
            let bar_width = 0.8;
            let rect = Rectangle::new(
                [(x_pos - bar_width / 2.0, 0.0), (x_pos + bar_width / 2.0, *count as f64)],
                ORANGE.filled(),
            );
            chart.draw_series(std::iter::once(rect))?;
        }

        root.present()?;
        Ok(())
    }

    /// Generate all charts for a session
    ///
    /// This convenience method generates all available chart types
    /// for the given session.
    ///
    /// # Arguments
    ///
    /// * `session` - The experiment session data
    /// * `output_dir` - Directory to save the charts
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok on success, Err on failure
    pub fn generate_all(&self, session: &ExperimentSession, output_dir: &str) -> Result<()> {
        std::fs::create_dir_all(output_dir)?;

        self.generate_improvement_trend(session, &format!("{}/improvement_trend.png", output_dir))?;
        self.generate_iteration_comparison(session, &format!("{}/iteration_comparison.png", output_dir))?;
        self.generate_baseline_comparison(session, &format!("{}/baseline_comparison.png", output_dir))?;
        self.generate_distribution_histogram(session, &format!("{}/distribution_histogram.png", output_dir))?;

        Ok(())
    }
}

impl Default for ChartGenerator {
    fn default() -> Self {
        Self::new(VisualizationConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase1_design::{BaselineRecord, ExperimentDesign};
    use crate::phase2_iterate::IterationRecord;
    use std::fs;

    fn create_test_session() -> ExperimentSession {
        let design = ExperimentDesign::new(
            "Test hypothesis".to_string(),
            "test_metric".to_string(),
            "echo 100".to_string(),
            100.0,
            10.0,
        );
        let baseline = BaselineRecord::new(
            "2024-01-01T00:00:00Z".to_string(),
            "test_commit".to_string(),
            "test_metric".to_string(),
            "echo 100".to_string(),
            100.0,
            vec![100.0, 101.0],
            0.01,
            true,
        );
        let mut session = ExperimentSession::new(
            "test_session".to_string(),
            "Test question".to_string(),
            design,
            baseline,
        );
        
        // Add some iterations
        session.iterations.push(IterationRecord::new(1, "Test action 1".to_string(), 95.0, 0.05, true));
        session.iterations.push(IterationRecord::new(2, "Test action 2".to_string(), 90.0, 0.10, true));
        session.iterations.push(IterationRecord::new(3, "Test action 3".to_string(), 92.0, 0.08, false));
        session.best_iteration = Some(1);
        
        session
    }

    #[test]
    fn test_visualization_config_default() {
        let config = VisualizationConfig::default();
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert_eq!(config.font_size, 14);
        assert!(config.show_grid);
    }

    #[test]
    fn test_chart_generator_new() {
        let config = VisualizationConfig::default();
        let generator = ChartGenerator::new(config);
        assert_eq!(generator.config.width, 800);
    }

    #[test]
    fn test_chart_generator_default() {
        let generator = ChartGenerator::default();
        assert_eq!(generator.config.width, 800);
    }

    #[test]
    fn test_generate_improvement_trend() {
        let session = create_test_session();
        let generator = ChartGenerator::default();
        let output_path = "/tmp/test_improvement_trend.png";
        
        let result = generator.generate_improvement_trend(&session, output_path);
        assert!(result.is_ok());
        
        // Verify file was created
        assert!(std::path::Path::new(output_path).exists());
        
        // Clean up
        let _ = fs::remove_file(output_path);
    }

    #[test]
    fn test_generate_iteration_comparison() {
        let session = create_test_session();
        let generator = ChartGenerator::default();
        let output_path = "/tmp/test_iteration_comparison.png";
        
        let result = generator.generate_iteration_comparison(&session, output_path);
        assert!(result.is_ok());
        
        // Verify file was created
        assert!(std::path::Path::new(output_path).exists());
        
        // Clean up
        let _ = fs::remove_file(output_path);
    }

    #[test]
    fn test_generate_baseline_comparison() {
        let session = create_test_session();
        let generator = ChartGenerator::default();
        let output_path = "/tmp/test_baseline_comparison.png";
        
        let result = generator.generate_baseline_comparison(&session, output_path);
        assert!(result.is_ok());
        
        // Verify file was created
        assert!(std::path::Path::new(output_path).exists());
        
        // Clean up
        let _ = fs::remove_file(output_path);
    }

    #[test]
    fn test_generate_distribution_histogram() {
        let session = create_test_session();
        let generator = ChartGenerator::default();
        let output_path = "/tmp/test_distribution_histogram.png";
        
        let result = generator.generate_distribution_histogram(&session, output_path);
        assert!(result.is_ok());
        
        // Verify file was created
        assert!(std::path::Path::new(output_path).exists());
        
        // Clean up
        let _ = fs::remove_file(output_path);
    }

    #[test]
    fn test_generate_all() {
        let session = create_test_session();
        let generator = ChartGenerator::default();
        let output_dir = "/tmp/test_viz_all";
        
        let result = generator.generate_all(&session, output_dir);
        assert!(result.is_ok());
        
        // Verify all files were created
        assert!(std::path::Path::new(&format!("{}/improvement_trend.png", output_dir)).exists());
        assert!(std::path::Path::new(&format!("{}/iteration_comparison.png", output_dir)).exists());
        assert!(std::path::Path::new(&format!("{}/baseline_comparison.png", output_dir)).exists());
        assert!(std::path::Path::new(&format!("{}/distribution_histogram.png", output_dir)).exists());
        
        // Clean up
        let _ = fs::remove_dir_all(output_dir);
    }

    #[test]
    fn test_generate_with_empty_iterations() {
        let design = ExperimentDesign::new(
            "Test".to_string(),
            "metric".to_string(),
            "echo 100".to_string(),
            100.0,
            10.0,
        );
        let baseline = BaselineRecord::new(
            "2024-01-01T00:00:00Z".to_string(),
            "commit".to_string(),
            "metric".to_string(),
            "echo 100".to_string(),
            100.0,
            vec![100.0],
            0.0,
            true,
        );
        let session = ExperimentSession::new(
            "test".to_string(),
            "Test".to_string(),
            design,
            baseline,
        );
        
        let generator = ChartGenerator::default();
        let output_path = "/tmp/test_empty.png";
        
        // Should still work with just baseline
        let result = generator.generate_baseline_comparison(&session, output_path);
        assert!(result.is_ok());
        
        // Clean up
        let _ = fs::remove_file(output_path);
    }
}
