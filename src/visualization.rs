//! Visualization module for pi-autoresearch experiment results.
//!
//! This module provides functionality to generate charts and reports
//! from experiment session data, supporting PNG, SVG, and HTML output formats.
//!
//! # Features
//!
//! - Improvement trend line charts
//! - Iteration comparison bar charts
//! - Baseline vs final comparison charts
//! - Measurement distribution histograms
//! - HTML report generation with embedded charts
//! - Statistical analysis (mean, median, std dev, confidence intervals)
//! - PNG, SVG, and HTML output formats
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

/// Statistical analysis results
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Statistics {
    /// Number of data points
    pub count: usize,
    /// Mean value
    pub mean: f64,
    /// Median value
    pub median: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Lower bound of confidence interval
    pub ci_lower: f64,
    /// Upper bound of confidence interval
    pub ci_upper: f64,
    /// Confidence level (e.g., 95.0 for 95%)
    pub confidence_level: f64,
    /// Trend line slope
    pub trend_slope: f64,
    /// Trend line intercept
    pub trend_intercept: f64,
    /// R-squared value for trend line
    pub r_squared: f64,
}

impl Default for Statistics {
    fn default() -> Self {
        Self {
            count: 0,
            mean: 0.0,
            median: 0.0,
            std_dev: 0.0,
            min: 0.0,
            max: 0.0,
            ci_lower: 0.0,
            ci_upper: 0.0,
            confidence_level: 95.0,
            trend_slope: 0.0,
            trend_intercept: 0.0,
            r_squared: 0.0,
        }
    }
}

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

    /// Generate an HTML report with embedded charts and statistics
    ///
    /// This method creates a comprehensive HTML report that includes:
    /// - Experiment summary with question and hypothesis
    /// - Key metrics (baseline, best improvement, iterations, runtime)
    /// - All generated charts as embedded images
    /// - Iteration timeline table
    /// - Statistical analysis
    /// - Responsive design for different screen sizes
    ///
    /// # Arguments
    ///
    /// * `session` - The experiment session data
    /// * `output_path` - Path to save the HTML report
    /// * `chart_dir` - Directory where charts are stored (will be linked from HTML)
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
    ///
    /// // First generate charts
    /// generator.generate_all(&session, "./charts").unwrap();
    ///
    /// // Then generate HTML report
    /// generator.generate_html_report(&session, "report.html", "./charts").unwrap();
    /// ```
    pub fn generate_html_report(&self, session: &ExperimentSession, output_path: &str, chart_dir: &str) -> Result<()> {
        // Generate charts first if they don't exist
        std::fs::create_dir_all(chart_dir)?;
        self.generate_all(session, chart_dir)?;

        // Calculate statistics
        let stats = self.calculate_statistics(session);
        let runtime_seconds = self.calculate_runtime_seconds(session);
        let target_achieved = session.status == "completed";

        // Build HTML content
        let html_content = self.build_html_report(
            session,
            &stats,
            runtime_seconds,
            target_achieved,
            chart_dir,
        );

        // Write HTML file
        std::fs::write(output_path, html_content)?;

        Ok(())
    }

    /// Calculate statistical analysis for the session
    ///
    /// # Arguments
    ///
    /// * `session` - The experiment session data
    ///
    /// # Returns
    ///
    /// * `Statistics` - Calculated statistical measures
    fn calculate_statistics(&self, session: &ExperimentSession) -> Statistics {
        let mut values: Vec<f64> = vec![session.baseline_record.value];
        for iteration in &session.iterations {
            values.push(iteration.metric_value);
        }

        if values.is_empty() {
            return Statistics::default();
        }

        let n = values.len() as f64;
        let sum: f64 = values.iter().sum();
        let mean = sum / n;

        // Calculate variance and std dev
        let squared_diffs: Vec<f64> = values.iter().map(|v| (v - mean).powi(2)).collect();
        let variance = squared_diffs.iter().sum::<f64>() / n;
        let std_dev = variance.sqrt();

        // Calculate median
        let mut sorted_values = values.clone();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = if n % 2.0 == 0.0 {
            (sorted_values[(n as usize) / 2 - 1] + sorted_values[(n as usize) / 2]) / 2.0
        } else {
            sorted_values[(n as usize) / 2]
        };

        // Calculate confidence interval (95%)
        let t_value = if n > 30.0 {
            1.96
        } else if n > 20.0 {
            2.09
        } else if n > 10.0 {
            2.26
        } else {
            2.58
        };
        let margin_of_error = t_value * std_dev / (n.sqrt());
        let ci_lower = mean - margin_of_error;
        let ci_upper = mean + margin_of_error;

        // Calculate trend line (linear regression)
        let trend = self.calculate_trend_line(&values);

        Statistics {
            count: values.len(),
            mean,
            median,
            std_dev,
            min: *sorted_values.first().unwrap(),
            max: *sorted_values.last().unwrap(),
            ci_lower,
            ci_upper,
            confidence_level: 95.0,
            trend_slope: trend.0,
            trend_intercept: trend.1,
            r_squared: trend.2,
        }
    }

    /// Calculate linear regression trend line
    ///
    /// # Returns
    ///
    /// * `(slope, intercept, r_squared)` - Linear regression parameters
    fn calculate_trend_line(&self, values: &[f64]) -> (f64, f64, f64) {
        let n = values.len() as f64;
        if n < 2.0 {
            return (0.0, if values.is_empty() { 0.0 } else { values[0] }, 0.0);
        }

        let sum_x: f64 = (0..values.len() as i64).map(|x| x as f64).sum();
        let sum_y: f64 = values.iter().sum();
        let sum_xy: f64 = values.iter().enumerate().map(|(i, v)| i as f64 * v).sum();
        let sum_x2: f64 = (0..values.len() as i64).map(|x| (x as f64).powi(2)).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;

        // Calculate R-squared
        let mean_y = sum_y / n;
        let ss_tot: f64 = values.iter().map(|v| (v - mean_y).powi(2)).sum();
        let ss_res: f64 = values.iter().enumerate().map(|(i, v)| {
            let predicted = slope * (i as f64) + intercept;
            (v - predicted).powi(2)
        }).sum();

        let r_squared = if ss_tot > 0.0 {
            1.0 - (ss_res / ss_tot)
        } else {
            0.0
        };

        (slope, intercept, r_squared)
    }

    /// Calculate runtime in seconds from session timestamps
    fn calculate_runtime_seconds(&self, session: &ExperimentSession) -> f64 {
        let start = &session.start_time;
        let end = session.end_time.as_ref();
        
        if let Some(end_str) = end {
            if let (Ok(start_dt), Ok(end_dt)) = (
                chrono::DateTime::parse_from_rfc3339(start),
                chrono::DateTime::parse_from_rfc3339(end_str)
            ) {
                return end_dt.signed_duration_since(start_dt).num_seconds() as f64;
            }
        }
        0.0
    }

    /// Calculate the best improvement from the session
    fn calculate_best_improvement(&self, session: &ExperimentSession) -> f64 {
        if let Some(best_idx) = session.best_iteration {
            if best_idx < session.iterations.len() {
                return session.iterations[best_idx].improvement;
            }
        }
        // Fallback: find the best improvement manually
        session.iterations.iter()
            .map(|i| i.improvement)
            .fold(0.0, f64::max)
    }

    /// Build the HTML report content
    fn build_html_report(&self, session: &ExperimentSession, stats: &Statistics, runtime_seconds: f64, target_achieved: bool, chart_dir: &str) -> String {
        let status_text = if target_achieved { "Target Achieved ✅" } else { "Target Not Achieved ❌" };
        let status_color = if target_achieved { "#2ecc71" } else { "#e74c3c" };
        let best_improvement = self.calculate_best_improvement(session);
        let version = env!("CARGO_PKG_VERSION");

        let iterations_html: String = session.iterations.iter().map(|iter| {
            let status_icon = if iter.kept { "✅" } else { "❌" };
            let status_class = if iter.kept { "kept" } else { "reverted" };
            format!(
                r#"<tr class="{}">
                    <td>{}</td>
                    <td>{}</td>
                    <td>{:.4}</td>
                    <td>{:+.2}%</td>
                    <td>{}</td>
                </tr>"#,
                status_class,
                iter.iteration,
                iter.timestamp,
                iter.metric_value,
                iter.improvement * 100.0,
                status_icon
            )
        }).collect();

        let chart_relative_path = if chart_dir.starts_with('/') {
            chart_dir.to_string()
        } else {
            format!("./{}", chart_dir)
        };

        format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Experiment Report - {}</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            background: #f5f5f5;
            padding: 20px;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            overflow: hidden;
        }}
        header {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 40px;
            text-align: center;
        }}
        h1 {{
            font-size: 2.5em;
            margin-bottom: 10px;
        }}
        .status {{
            display: inline-block;
            padding: 8px 16px;
            border-radius: 20px;
            background: {};
            color: white;
            font-weight: bold;
            margin-top: 10px;
        }}
        .section {{
            padding: 30px;
            border-bottom: 1px solid #eee;
        }}
        .section:last-child {{
            border-bottom: none;
        }}
        h2 {{
            color: #667eea;
            margin-bottom: 20px;
            font-size: 1.5em;
        }}
        .metrics-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }}
        .metric-card {{
            background: #f8f9fa;
            padding: 20px;
            border-radius: 8px;
            text-align: center;
            border-left: 4px solid #667eea;
        }}
        .metric-value {{
            font-size: 2em;
            font-weight: bold;
            color: #667eea;
            margin-bottom: 5px;
        }}
        .metric-label {{
            color: #666;
            font-size: 0.9em;
        }}
        .charts-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
            gap: 30px;
            margin: 20px 0;
        }}
        .chart-container {{
            background: #f8f9fa;
            padding: 20px;
            border-radius: 8px;
            text-align: center;
        }}
        .chart-container img {{
            max-width: 100%;
            height: auto;
            border-radius: 4px;
        }}
        .chart-title {{
            font-weight: bold;
            margin-bottom: 10px;
            color: #333;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin: 20px 0;
        }}
        th, td {{
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid #eee;
        }}
        th {{
            background: #f8f9fa;
            font-weight: bold;
            color: #333;
        }}
        tr:hover {{
            background: #f8f9fa;
        }}
        .kept {{
            background: #d4edda !important;
        }}
        .reverted {{
            background: #f8d7da !important;
        }}
        .stats-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
            gap: 15px;
        }}
        .stat-item {{
            background: #f8f9fa;
            padding: 15px;
            border-radius: 6px;
        }}
        .stat-label {{
            font-size: 0.85em;
            color: #666;
            margin-bottom: 5px;
        }}
        .stat-value {{
            font-size: 1.2em;
            font-weight: bold;
            color: #333;
        }}
        footer {{
            background: #f8f9fa;
            padding: 20px;
            text-align: center;
            color: #666;
            font-size: 0.9em;
        }}
        @media (max-width: 768px) {{
            .charts-grid {{
                grid-template-columns: 1fr;
            }}
            .metrics-grid {{
                grid-template-columns: repeat(2, 1fr);
            }}
            h1 {{
                font-size: 1.8em;
            }}
        }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>🧪 Experiment Report</h1>
            <p style="font-size: 1.2em; margin-top: 10px;">{}</p>
            <span class="status">{}</span>
        </header>

        <div class="section">
            <h2>📊 Key Metrics</h2>
            <div class="metrics-grid">
                <div class="metric-card">
                    <div class="metric-value">{}</div>
                    <div class="metric-label">Baseline</div>
                </div>
                <div class="metric-card">
                    <div class="metric-value">{:+.2}%</div>
                    <div class="metric-label">Best Improvement</div>
                </div>
                <div class="metric-card">
                    <div class="metric-value">{}</div>
                    <div class="metric-label">Iterations</div>
                </div>
                <div class="metric-card">
                    <div class="metric-value">{}</div>
                    <div class="metric-label">Runtime</div>
                </div>
            </div>
        </div>

        <div class="section">
            <h2>📈 Charts</h2>
            <div class="charts-grid">
                <div class="chart-container">
                    <div class="chart-title">Improvement Trend</div>
                    <img src="{}/improvement_trend.png" alt="Improvement Trend Chart">
                </div>
                <div class="chart-container">
                    <div class="chart-title">Iteration Comparison</div>
                    <img src="{}/iteration_comparison.png" alt="Iteration Comparison Chart">
                </div>
                <div class="chart-container">
                    <div class="chart-title">Baseline vs Final</div>
                    <img src="{}/baseline_comparison.png" alt="Baseline Comparison Chart">
                </div>
                <div class="chart-container">
                    <div class="chart-title">Distribution Histogram</div>
                    <img src="{}/distribution_histogram.png" alt="Distribution Histogram">
                </div>
            </div>
        </div>

        <div class="section">
            <h2>📊 Statistical Analysis</h2>
            <div class="stats-grid">
                <div class="stat-item">
                    <div class="stat-label">Count</div>
                    <div class="stat-value">{}</div>
                </div>
                <div class="stat-item">
                    <div class="stat-label">Mean</div>
                    <div class="stat-value">{:.4}</div>
                </div>
                <div class="stat-item">
                    <div class="stat-label">Median</div>
                    <div class="stat-value">{:.4}</div>
                </div>
                <div class="stat-item">
                    <div class="stat-label">Std Dev</div>
                    <div class="stat-value">{:.4}</div>
                </div>
                <div class="stat-item">
                    <div class="stat-label">Min</div>
                    <div class="stat-value">{:.4}</div>
                </div>
                <div class="stat-item">
                    <div class="stat-label">Max</div>
                    <div class="stat-value">{:.4}</div>
                </div>
                <div class="stat-item">
                    <div class="stat-label">95% CI</div>
                    <div class="stat-value">[{:.4}, {:.4}]</div>
                </div>
                <div class="stat-item">
                    <div class="stat-label">R² (Trend)</div>
                    <div class="stat-value">{:.4}</div>
                </div>
            </div>
        </div>

        <div class="section">
            <h2>📝 Iteration Timeline</h2>
            <table>
                <thead>
                    <tr>
                        <th>Iteration</th>
                        <th>Timestamp</th>
                        <th>Metric Value</th>
                        <th>Improvement</th>
                        <th>Status</th>
                    </tr>
                </thead>
                <tbody>
                    {}
                </tbody>
            </table>
        </div>

        <div class="section">
            <h2>ℹ️ Metadata</h2>
            <table>
                <tr>
                    <td><strong>Session ID</strong></td>
                    <td>{}</td>
                </tr>
                <tr>
                    <td><strong>Metric</strong></td>
                    <td>{}</td>
                </tr>
                <tr>
                    <td><strong>Target Improvement</strong></td>
                    <td>{:.2}%</td>
                </tr>
                <tr>
                    <td><strong>Start Time</strong></td>
                    <td>{}</td>
                </tr>
                <tr>
                    <td><strong>End Time</strong></td>
                    <td>{}</td>
                </tr>
                <tr>
                    <td><strong>Version</strong></td>
                    <td>{}</td>
                </tr>
            </table>
        </div>

        <footer>
            <p>Generated by pi-autoresearch v{} at {}</p>
        </footer>
    </div>
</body>
</html>"##,
            session.session_id,
            session.question,
            status_text,
            status_color,
            session.baseline_record.value,
            best_improvement * 100.0,
            session.iterations.len(),
            self.format_duration(runtime_seconds),
            chart_relative_path,
            chart_relative_path,
            chart_relative_path,
            chart_relative_path,
            stats.count,
            stats.mean,
            stats.median,
            stats.std_dev,
            stats.min,
            stats.max,
            stats.ci_lower,
            stats.ci_upper,
            stats.r_squared,
            iterations_html,
            session.session_id,
            session.design.metric,
            session.design.target_improvement,
            &session.start_time,
            session.end_time.as_deref().unwrap_or("N/A"),
            version,
            version,
            chrono::Utc::now().to_rfc3339()
        )
    }

    /// Format duration in human-readable format
    fn format_duration(&self, seconds: f64) -> String {
        if seconds < 60.0 {
            format!("{:.1}s", seconds)
        } else if seconds < 3600.0 {
            let mins = seconds / 60.0;
            format!("{:.1}m", mins)
        } else {
            let hours = seconds / 3600.0;
            format!("{:.2}h", hours)
        }
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

    #[test]
    fn test_statistics_default() {
        let stats = Statistics::default();
        assert_eq!(stats.count, 0);
        assert_eq!(stats.mean, 0.0);
        assert_eq!(stats.confidence_level, 95.0);
    }

    #[test]
    fn test_statistics_clone() {
        let stats = Statistics {
            count: 10,
            mean: 100.0,
            median: 99.0,
            std_dev: 5.0,
            min: 90.0,
            max: 110.0,
            ci_lower: 98.0,
            ci_upper: 102.0,
            confidence_level: 95.0,
            trend_slope: -2.5,
            trend_intercept: 100.0,
            r_squared: 0.85,
        };
        let cloned = stats.clone();
        assert_eq!(stats, cloned);
    }

    #[test]
    fn test_calculate_statistics() {
        let session = create_test_session();
        let generator = ChartGenerator::default();
        let stats = generator.calculate_statistics(&session);
        
        assert_eq!(stats.count, 4); // baseline + 3 iterations
        assert!(stats.mean > 0.0);
        assert!(stats.median > 0.0);
        assert!(stats.std_dev >= 0.0);
        assert!(stats.min > 0.0);
        assert!(stats.max > 0.0);
        assert!(stats.ci_lower <= stats.ci_upper);
    }

    #[test]
    fn test_calculate_trend_line() {
        let generator = ChartGenerator::default();
        let values = vec![100.0, 95.0, 90.0, 92.0];
        let (slope, intercept, r_squared) = generator.calculate_trend_line(&values);
        
        assert!(slope < 0.0); // Should be negative (decreasing)
        assert!(intercept > 0.0);
        assert!(r_squared >= 0.0 && r_squared <= 1.0);
    }

    #[test]
    fn test_calculate_trend_line_single_point() {
        let generator = ChartGenerator::default();
        let values = vec![100.0];
        let (slope, intercept, r_squared) = generator.calculate_trend_line(&values);
        
        assert_eq!(slope, 0.0);
        assert_eq!(intercept, 100.0);
        assert_eq!(r_squared, 0.0);
    }

    #[test]
    fn test_calculate_trend_line_empty() {
        let generator = ChartGenerator::default();
        let values: Vec<f64> = vec![];
        let (slope, intercept, r_squared) = generator.calculate_trend_line(&values);
        
        assert_eq!(slope, 0.0);
        assert_eq!(intercept, 0.0);
        assert_eq!(r_squared, 0.0);
    }

    #[test]
    fn test_calculate_runtime_seconds() {
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
        let mut session = ExperimentSession::new(
            "test".to_string(),
            "Test".to_string(),
            design,
            baseline,
        );
        
        // Set timestamps
        session.start_time = "2024-01-01T00:00:00Z".to_string();
        session.end_time = Some("2024-01-01T00:01:30Z".to_string());
        
        let generator = ChartGenerator::default();
        let runtime = generator.calculate_runtime_seconds(&session);
        
        assert_eq!(runtime, 90.0); // 1 minute 30 seconds
    }

    #[test]
    fn test_calculate_runtime_seconds_no_end_time() {
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
        let runtime = generator.calculate_runtime_seconds(&session);
        
        assert_eq!(runtime, 0.0); // No timestamps, returns 0
    }

    #[test]
    fn test_format_duration_seconds() {
        let generator = ChartGenerator::default();
        assert_eq!(generator.format_duration(30.0), "30.0s");
        assert_eq!(generator.format_duration(45.5), "45.5s");
    }

    #[test]
    fn test_format_duration_minutes() {
        let generator = ChartGenerator::default();
        assert_eq!(generator.format_duration(90.0), "1.5m");
        assert_eq!(generator.format_duration(300.0), "5.0m");
    }

    #[test]
    fn test_format_duration_hours() {
        let generator = ChartGenerator::default();
        assert_eq!(generator.format_duration(3600.0), "1.00h");
        assert_eq!(generator.format_duration(7200.0), "2.00h");
    }

    #[test]
    fn test_generate_html_report() {
        let session = create_test_session();
        let generator = ChartGenerator::default();
        let output_path = "/tmp/test_report.html";
        let chart_dir = "/tmp/test_charts";
        
        let result = generator.generate_html_report(&session, output_path, chart_dir);
        assert!(result.is_ok());
        
        // Verify file was created
        assert!(std::path::Path::new(output_path).exists());
        
        // Verify HTML content
        let content = fs::read_to_string(output_path).unwrap();
        assert!(content.contains("<!DOCTYPE html"));
        assert!(content.contains("Experiment Report"));
        assert!(content.contains("Test question"));
        assert!(content.contains("Key Metrics"));
        assert!(content.contains("Charts"));
        assert!(content.contains("Statistical Analysis"));
        assert!(content.contains("Iteration Timeline"));
        
        // Clean up
        let _ = fs::remove_file(output_path);
        let _ = fs::remove_dir_all(chart_dir);
    }

    #[test]
    fn test_generate_html_report_with_target_achieved() {
        let mut session = create_test_session();
        session.status = "completed".to_string();
        
        let generator = ChartGenerator::default();
        let output_path = "/tmp/test_report_success.html";
        let chart_dir = "/tmp/test_charts_success";
        
        let result = generator.generate_html_report(&session, output_path, chart_dir);
        assert!(result.is_ok());
        
        let content = fs::read_to_string(output_path).unwrap();
        assert!(content.contains("Target Achieved"));
        assert!(content.contains("✅"));
        
        // Clean up
        let _ = fs::remove_file(output_path);
        let _ = fs::remove_dir_all(chart_dir);
    }

    #[test]
    fn test_generate_html_report_without_target_achieved() {
        let mut session = create_test_session();
        session.status = "failed".to_string();
        
        let generator = ChartGenerator::default();
        let output_path = "/tmp/test_report_failure.html";
        let chart_dir = "/tmp/test_charts_failure";
        
        let result = generator.generate_html_report(&session, output_path, chart_dir);
        assert!(result.is_ok());
        
        let content = fs::read_to_string(output_path).unwrap();
        assert!(content.contains("Target Not Achieved"));
        assert!(content.contains("❌"));
        
        // Clean up
        let _ = fs::remove_file(output_path);
        let _ = fs::remove_dir_all(chart_dir);
    }

    #[test]
    fn test_html_report_contains_charts() {
        let session = create_test_session();
        let generator = ChartGenerator::default();
        let output_path = "/tmp/test_report_charts.html";
        let chart_dir = "/tmp/test_charts_html";
        
        let result = generator.generate_html_report(&session, output_path, chart_dir);
        assert!(result.is_ok());
        
        let content = fs::read_to_string(output_path).unwrap();
        assert!(content.contains("improvement_trend.png"));
        assert!(content.contains("iteration_comparison.png"));
        assert!(content.contains("baseline_comparison.png"));
        assert!(content.contains("distribution_histogram.png"));
        
        // Clean up
        let _ = fs::remove_file(output_path);
        let _ = fs::remove_dir_all(chart_dir);
    }

    #[test]
    fn test_html_report_contains_statistics() {
        let session = create_test_session();
        let generator = ChartGenerator::default();
        let output_path = "/tmp/test_report_stats.html";
        let chart_dir = "/tmp/test_charts_stats";
        
        let result = generator.generate_html_report(&session, output_path, chart_dir);
        assert!(result.is_ok());
        
        let content = fs::read_to_string(output_path).unwrap();
        assert!(content.contains("Mean"));
        assert!(content.contains("Median"));
        assert!(content.contains("Std Dev"));
        assert!(content.contains("95% CI"));
        
        // Clean up
        let _ = fs::remove_file(output_path);
        let _ = fs::remove_dir_all(chart_dir);
    }

    #[test]
    fn test_html_report_contains_iteration_timeline() {
        let session = create_test_session();
        let generator = ChartGenerator::default();
        let output_path = "/tmp/test_report_timeline.html";
        let chart_dir = "/tmp/test_charts_timeline";
        
        let result = generator.generate_html_report(&session, output_path, chart_dir);
        assert!(result.is_ok());
        
        let content = fs::read_to_string(output_path).unwrap();
        assert!(content.contains("Iteration Timeline"));
        assert!(content.contains("<table"));
        assert!(content.contains("<thead"));
        assert!(content.contains("<tbody"));
        assert!(content.contains("kept")); // CSS class for kept iterations
        assert!(content.contains("reverted")); // CSS class for reverted iterations
        
        // Clean up
        let _ = fs::remove_file(output_path);
        let _ = fs::remove_dir_all(chart_dir);
    }

    #[test]
    fn test_html_report_contains_metadata() {
        let session = create_test_session();
        let generator = ChartGenerator::default();
        let output_path = "/tmp/test_report_metadata.html";
        let chart_dir = "/tmp/test_charts_metadata";
        
        let result = generator.generate_html_report(&session, output_path, chart_dir);
        assert!(result.is_ok());
        
        let content = fs::read_to_string(output_path).unwrap();
        assert!(content.contains("Session ID"));
        assert!(content.contains("test_session"));
        assert!(content.contains("Metric"));
        assert!(content.contains("Target Improvement"));
        assert!(content.contains("Version"));
        
        // Clean up
        let _ = fs::remove_file(output_path);
        let _ = fs::remove_dir_all(chart_dir);
    }

    #[test]
    fn test_html_report_responsive_design() {
        let session = create_test_session();
        let generator = ChartGenerator::default();
        let output_path = "/tmp/test_report_responsive.html";
        let chart_dir = "/tmp/test_charts_responsive";
        
        let result = generator.generate_html_report(&session, output_path, chart_dir);
        assert!(result.is_ok());
        
        let content = fs::read_to_string(output_path).unwrap();
        assert!(content.contains("@media"));
        assert!(content.contains("max-width"));
        assert!(content.contains("viewport"));
        
        // Clean up
        let _ = fs::remove_file(output_path);
        let _ = fs::remove_dir_all(chart_dir);
    }

    #[test]
    fn test_statistics_serialization() {
        let stats = Statistics {
            count: 10,
            mean: 100.0,
            median: 99.0,
            std_dev: 5.0,
            min: 90.0,
            max: 110.0,
            ci_lower: 98.0,
            ci_upper: 102.0,
            confidence_level: 95.0,
            trend_slope: -2.5,
            trend_intercept: 100.0,
            r_squared: 0.85,
        };
        
        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"count\":10"));
        assert!(json.contains("\"mean\":100.0"));
        
        let deserialized: Statistics = serde_json::from_str(&json).unwrap();
        assert_eq!(stats, deserialized);
    }

    #[test]
    fn test_html_report_creates_chart_directory() {
        let session = create_test_session();
        let generator = ChartGenerator::default();
        let output_path = "/tmp/test_report_create_dir.html";
        let chart_dir = "/tmp/test_charts_create_dir";
        
        // Ensure directory doesn't exist
        let _ = fs::remove_dir_all(chart_dir);
        assert!(!std::path::Path::new(chart_dir).exists());
        
        let result = generator.generate_html_report(&session, output_path, chart_dir);
        assert!(result.is_ok());
        
        // Verify directory was created
        assert!(std::path::Path::new(chart_dir).exists());
        
        // Clean up
        let _ = fs::remove_file(output_path);
        let _ = fs::remove_dir_all(chart_dir);
    }

    #[test]
    fn test_statistics_with_single_value() {
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
        let stats = generator.calculate_statistics(&session);
        
        assert_eq!(stats.count, 1);
        assert_eq!(stats.mean, 100.0);
        assert_eq!(stats.median, 100.0);
        assert_eq!(stats.std_dev, 0.0);
        assert_eq!(stats.min, 100.0);
        assert_eq!(stats.max, 100.0);
    }

    #[test]
    fn test_statistics_with_improving_values() {
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
        let mut session = ExperimentSession::new(
            "test".to_string(),
            "Test".to_string(),
            design,
            baseline,
        );
        
        // Add improving iterations (lower is better)
        session.iterations.push(IterationRecord::new(1, "Action 1".to_string(), 90.0, 0.10, true));
        session.iterations.push(IterationRecord::new(2, "Action 2".to_string(), 80.0, 0.20, true));
        session.iterations.push(IterationRecord::new(3, "Action 3".to_string(), 70.0, 0.30, true));
        
        let generator = ChartGenerator::default();
        let stats = generator.calculate_statistics(&session);
        
        assert_eq!(stats.count, 4);
        assert_eq!(stats.mean, 85.0); // (100 + 90 + 80 + 70) / 4
        assert_eq!(stats.min, 70.0);
        assert_eq!(stats.max, 100.0);
    }

    #[test]
    fn test_trend_line_r_squared_perfect_fit() {
        let generator = ChartGenerator::default();
        // Perfect linear relationship: y = 2x + 10
        let values = vec![10.0, 12.0, 14.0, 16.0, 18.0];
        let (slope, _intercept, r_squared) = generator.calculate_trend_line(&values);
        
        assert!((slope - 2.0).abs() < 0.01);
        assert!((r_squared - 1.0).abs() < 0.01); // Should be very close to 1.0
    }

    #[test]
    fn test_trend_line_r_squared_no_correlation() {
        let generator = ChartGenerator::default();
        // Random values with no correlation
        let values = vec![10.0, 50.0, 20.0, 80.0, 30.0];
        let (_slope, _intercept, r_squared) = generator.calculate_trend_line(&values);
        
        assert!(r_squared >= -1.0 && r_squared <= 1.0); // R² can be negative for bad fits
    }
}
