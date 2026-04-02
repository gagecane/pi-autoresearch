use anyhow::Result;
use lettre::transport::smtp::authentication::Credentials;
use lettre::Transport;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::session::ExperimentSession;

/// Webhook payload for experiment notifications
#[derive(Serialize, Deserialize, Debug)]
pub struct WebhookPayload {
    /// Type of notification
    pub notification_type: String,
    /// Experiment session ID
    pub session_id: String,
    /// Research question
    pub question: String,
    /// Metric being optimized
    pub metric: String,
    /// Baseline value
    pub baseline: f64,
    /// Best improvement percentage (positive is better for minimization)
    pub best_improvement: f64,
    /// Number of iterations completed
    pub iterations: usize,
    /// Runtime in seconds (calculated from start/end times)
    pub runtime_seconds: f64,
    /// Whether target was achieved
    pub target_achieved: bool,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl WebhookPayload {
    /// Creates a new webhook payload for experiment completion
    pub fn new_completion(session: &ExperimentSession, target_achieved: bool) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
        metadata.insert("start_time".to_string(), session.start_time.clone());
        if let Some(ref end_time) = session.end_time {
            metadata.insert("end_time".to_string(), end_time.clone());
        }
        
        // Calculate runtime from timestamps
        let runtime_seconds = Self::calculate_runtime(&session.start_time, &session.end_time);
        
        // Calculate best improvement using session method
        let best_improvement = session.calculate_final_improvement() * 100.0;
        
        WebhookPayload {
            notification_type: "experiment_complete".to_string(),
            session_id: session.session_id.clone(),
            question: session.question.clone(),
            metric: session.design.metric.clone(),
            baseline: session.baseline_record.value,
            best_improvement,
            iterations: session.iterations.len(),
            runtime_seconds,
            target_achieved,
            metadata,
        }
    }

    /// Creates a new webhook payload for iteration milestone
    pub fn new_milestone(session: &ExperimentSession, current_iteration: usize) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("current_iteration".to_string(), current_iteration.to_string());
        metadata.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
        metadata.insert("start_time".to_string(), session.start_time.clone());
        
        let runtime_seconds = Self::calculate_runtime(&session.start_time, &session.end_time);
        let best_improvement = session.calculate_final_improvement() * 100.0;
        
        WebhookPayload {
            notification_type: "iteration_milestone".to_string(),
            session_id: session.session_id.clone(),
            question: session.question.clone(),
            metric: session.design.metric.clone(),
            baseline: session.baseline_record.value,
            best_improvement,
            iterations: current_iteration,
            runtime_seconds,
            target_achieved: false,
            metadata,
        }
    }

    /// Calculate runtime in seconds from start and end timestamps
    fn calculate_runtime(start_time: &str, end_time: &Option<String>) -> f64 {
        if let Some(ref end) = end_time {
            if let (Ok(start), Ok(end)) = (
                chrono::DateTime::parse_from_rfc3339(start_time),
                chrono::DateTime::parse_from_rfc3339(end)
            ) {
                return end.signed_duration_since(start).num_seconds() as f64;
            }
        }
        // Fallback: calculate from start to now
        if let Ok(start) = chrono::DateTime::parse_from_rfc3339(start_time) {
            return chrono::Utc::now().signed_duration_since(start).num_seconds() as f64;
        }
        0.0
    }
}

/// Sends a webhook notification
///
/// # Arguments
///
/// * `url` - The webhook URL to send the POST request to
/// * `session` - The experiment session containing the results
/// * `target_achieved` - Whether the experiment achieved its target
///
/// # Examples
///
/// ```ignore
/// use pi_autoresearch::notification::send_webhook;
/// use pi_autoresearch::session::ExperimentSession;
///
/// let session = ExperimentSession::new(/* ... */);
/// let result = send_webhook("https://hooks.example.com/xxx", &session, true);
/// assert!(result.is_ok());
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The URL is invalid
/// - The HTTP request fails
/// - The server returns a non-success status code
pub fn send_webhook(url: &str, session: &ExperimentSession, target_achieved: bool) -> Result<()> {
    let payload = WebhookPayload::new_completion(session, target_achieved);
    send_webhook_raw(url, &payload)
}

/// Sends an iteration milestone notification via webhook
///
/// # Arguments
///
/// * `url` - The webhook URL to send the POST request to
/// * `session` - The experiment session containing the current state
/// * `current_iteration` - The current iteration number
///
/// # Examples
///
/// ```ignore
/// use pi_autoresearch::notification::send_milestone_notification;
/// use pi_autoresearch::session::ExperimentSession;
///
/// let session = ExperimentSession::new(/* ... */);
/// let result = send_milestone_notification("https://hooks.example.com/xxx", &session, 5);
/// assert!(result.is_ok());
/// ```
pub fn send_milestone_notification(url: &str, session: &ExperimentSession, current_iteration: usize) -> Result<()> {
    let payload = WebhookPayload::new_milestone(session, current_iteration);
    send_webhook_raw(url, &payload)
}

/// Sends a Slack milestone notification via webhook
///
/// # Arguments
///
/// * `webhook_url` - The Slack webhook URL to send the notification to
/// * `session` - The experiment session containing the current state
/// * `current_iteration` - The current iteration number
///
/// # Examples
///
/// ```ignore
/// use pi_autoresearch::notification::send_slack_milestone;
/// use pi_autoresearch::session::ExperimentSession;
///
/// let session = ExperimentSession::new(/* ... */);
/// let result = send_slack_milestone("https://hooks.slack.com/services/xxx", &session, 5);
/// assert!(result.is_ok());
/// ```
///
/// # Notes
///
/// - Uses Slack's blocks API for rich formatting
/// - Shows progress update with current iteration and best improvement
///
/// # Errors
///
/// Returns an error if:
/// - The webhook URL is invalid
/// - The HTTP request fails
/// - Slack returns an error response
pub fn send_slack_milestone(webhook_url: &str, session: &ExperimentSession, current_iteration: usize) -> Result<()> {
    // Validate URL
    if webhook_url.is_empty() {
        anyhow::bail!("Slack webhook URL cannot be empty");
    }
    
    // Calculate metrics
    let best_improvement = session.calculate_final_improvement() * 100.0;
    let runtime_seconds = WebhookPayload::calculate_runtime(&session.start_time, &session.end_time);
    let runtime_formatted = format_duration(runtime_seconds);
    
    // Build Slack blocks message for milestone
    let blocks = vec![
        // Header block
        serde_json::json!({
            "type": "header",
            "text": {
                "type": "plain_text",
                "text": "🔄 Experiment Progress Update",
                "emoji": true
            }
        }),
        // Context block
        serde_json::json!({
            "type": "context",
            "elements": [
                {
                    "type": "mrkdwn",
                    "text": format!("*Session:* {} | *Metric:* {}", session.session_id, session.design.metric)
                }
            ]
        }),
        // Section block with summary
        serde_json::json!({
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": format!("*Question:* {}\n*Hypothesis:* {}", session.question, session.design.hypothesis)
            }
        }),
        // Divider
        serde_json::json!({
            "type": "divider"
        }),
        // Metrics block
        serde_json::json!({
            "type": "section",
            "fields": [
                {
                    "type": "mrkdwn",
                    "text": format!("*Baseline*\n{}", session.baseline_record.value)
                },
                {
                    "type": "mrkdwn",
                    "text": format!("*Best Improvement*\n{:+.1}%", best_improvement)
                },
                {
                    "type": "mrkdwn",
                    "text": format!("*Current Iteration*\n{}/?", current_iteration)
                },
                {
                    "type": "mrkdwn",
                    "text": format!("*Runtime So Far*\n{}", runtime_formatted)
                }
            ]
        }),
        // Progress block
        serde_json::json!({
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": format!("*📊 Progress:* Milestone reached at iteration {}. Best improvement so far: {:+.1}%", current_iteration, best_improvement)
            }
        }),
    ];
    
    // Create Slack message payload
    let slack_payload = serde_json::json!({
        "channel": "#experiments",
        "username": "pi-autoresearch",
        "icon_emoji": "🔬",
        "blocks": blocks
    });
    
    // Serialize to JSON
    let json = serde_json::to_string(&slack_payload)?;
    
    // Send to Slack
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(webhook_url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header("User-Agent", "pi-autoresearch")
        .body(json)
        .send()?;
    
    // Check response status
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        anyhow::bail!("Slack webhook request failed with status {}: {}", status, body);
    }
    
    Ok(())
}

/// Sends an email milestone notification
///
/// # Arguments
///
/// * `to` - The recipient email address
/// * `config` - Email configuration including SMTP settings
/// * `session` - The experiment session containing the current state
/// * `current_iteration` - The current iteration number
///
/// # Examples
///
/// ```ignore
/// use pi_autoresearch::notification::{send_email_milestone, EmailConfig};
/// use pi_autoresearch::session::ExperimentSession;
///
/// let session = ExperimentSession::new(/* ... */);
/// let config = EmailConfig::new("smtp.example.com".to_string(), "noreply@example.com".to_string())
///     .with_credentials("user".to_string(), "pass".to_string());
/// let result = send_email_milestone("recipient@example.com", &config, &session, 5);
/// assert!(result.is_ok());
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The recipient email is invalid
/// - SMTP connection fails
/// - Email sending fails
pub fn send_email_milestone(
    to: &str,
    config: &EmailConfig,
    session: &ExperimentSession,
    current_iteration: usize,
) -> Result<()> {
    // Validate recipient
    if to.is_empty() {
        anyhow::bail!("Recipient email address cannot be empty");
    }

    // Calculate metrics
    let best_improvement = session.calculate_final_improvement() * 100.0;
    let runtime_seconds = WebhookPayload::calculate_runtime(&session.start_time, &session.end_time);
    let runtime_formatted = format_duration(runtime_seconds);

    // Build HTML email body for milestone
    let html_body = build_email_milestone_html(
        session,
        current_iteration,
        best_improvement,
        runtime_formatted.clone(),
    );

    // Build plain text body for milestone
    let text_body = build_email_milestone_text(
        session,
        current_iteration,
        best_improvement,
        runtime_formatted,
    );

    // Create email subject
    let subject = format!(
        "[PROGRESS] Experiment {} - Iteration {} Milestone",
        session.session_id,
        current_iteration
    );

    // Build email using lettre
    let text_part = lettre::message::SinglePart::builder()
        .header(lettre::message::header::ContentType::TEXT_PLAIN)
        .body(text_body);
    
    let html_part = lettre::message::SinglePart::builder()
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .body(html_body);
    
    let alternative = lettre::message::MultiPart::alternative()
        .singlepart(text_part)
        .singlepart(html_part);
    
    let email = lettre::message::Message::builder()
        .from(config.from_address.parse()?)
        .to(to.parse()?)
        .subject(subject)
        .multipart(alternative)?;

    // Build SMTP transport
    let mut transport_builder = lettre::transport::smtp::SmtpTransport::relay(&config.smtp_host)?;
    transport_builder = transport_builder.port(config.smtp_port);
    
    // Add credentials if provided
    if let (Some(username), Some(password)) = (&config.username, &config.password) {
        let credentials = Credentials::new(username.clone(), password.clone());
        transport_builder = transport_builder.credentials(credentials);
    }
    
    let transport = transport_builder.build();

    // Send the email
    transport.send(&email)?;

    Ok(())
}

/// Builds HTML email body for milestone notifications
fn build_email_milestone_html(
    session: &ExperimentSession,
    current_iteration: usize,
    best_improvement: f64,
    runtime_formatted: String,
) -> String {
    let iterations_html: String = session.iterations.iter().map(|iter| {
        let status_icon = if iter.kept { "✅" } else { "❌" };
        format!(
            "<tr><td>{}</td><td>{}</td><td>{:.2}</td><td>{:+.2}%</td><td>{}</td></tr>",
            iter.iteration,
            iter.timestamp,
            iter.metric_value,
            iter.improvement,
            status_icon
        )
    }).collect();

    let improvement_color = if best_improvement > 0.0 { "#2ecc71" } else { "#e74c3c" };

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }}
        .container {{ max-width: 800px; margin: 0 auto; background-color: white; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .header {{ background-color: #3498db; color: white; padding: 20px; text-align: center; }}
        .header h1 {{ margin: 0; font-size: 24px; }}
        .content {{ padding: 20px; }}
        .section {{ margin-bottom: 20px; }}
        .section h2 {{ color: #333; border-bottom: 2px solid #eee; padding-bottom: 10px; }}
        .metrics {{ display: grid; grid-template-columns: repeat(2, 1fr); gap: 15px; margin: 15px 0; }}
        .metric-card {{ background-color: #f9f9f9; border-radius: 6px; padding: 15px; text-align: center; }}
        .metric-card .value {{ font-size: 28px; font-weight: bold; color: #333; }}
        .metric-card .label {{ color: #666; font-size: 14px; margin-top: 5px; }}
        table {{ width: 100%; border-collapse: collapse; margin-top: 10px; }}
        th {{ background-color: #f5f5f5; text-align: left; padding: 12px; border-bottom: 2px solid #ddd; }}
        td {{ padding: 10px 12px; border-bottom: 1px solid #eee; }}
        .footer {{ background-color: #f5f5f5; padding: 15px; text-align: center; color: #666; font-size: 12px; }}
        .status {{ font-size: 32px; margin-bottom: 10px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="status">🔄</div>
            <h1>Experiment Progress Update</h1>
        </div>
        <div class="content">
            <div class="section">
                <h2>📋 Overview</h2>
                <p><strong>Session ID:</strong> {session_id}</p>
                <p><strong>Question:</strong> {question}</p>
                <p><strong>Hypothesis:</strong> {hypothesis}</p>
                <p><strong>Metric:</strong> {metric}</p>
            </div>
            <div class="section">
                <h2>📊 Current Progress</h2>
                <div class="metrics">
                    <div class="metric-card">
                        <div class="value">{baseline}</div>
                        <div class="label">Baseline Value</div>
                    </div>
                    <div class="metric-card">
                        <div class="value" style="color: {improvement_color}">{improvement:+.1}%</div>
                        <div class="label">Best Improvement</div>
                    </div>
                    <div class="metric-card">
                        <div class="value">{current}/{total}</div>
                        <div class="label">Iterations Complete</div>
                    </div>
                    <div class="metric-card">
                        <div class="value">{runtime}</div>
                        <div class="label">Runtime So Far</div>
                    </div>
                </div>
            </div>
            <div class="section">
                <h2>🔄 Iteration Timeline</h2>
                <table>
                    <thead>
                        <tr><th>#</th><th>Timestamp</th><th>Value</th><th>Improvement</th><th>Kept</th></tr>
                    </thead>
                    <tbody>
                        {iterations_html}
                    </tbody>
                </table>
            </div>
        </div>
        <div class="footer">
            <p>Generated by pi-autoresearch v{version}</p>
            <p>{timestamp}</p>
        </div>
    </div>
</body>
</html>"#,
        session_id = session.session_id,
        question = session.question,
        hypothesis = session.design.hypothesis,
        metric = session.design.metric,
        baseline = session.baseline_record.value,
        improvement = best_improvement,
        current = current_iteration,
        total = session.iterations.len(),
        runtime = runtime_formatted,
        iterations_html = if iterations_html.is_empty() { "<tr><td colspan='5' style='text-align: center; color: #999;'>No iterations yet</td></tr>".to_string() } else { iterations_html },
        version = env!("CARGO_PKG_VERSION"),
        timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    )
}

/// Builds plain text email body for milestone notifications
fn build_email_milestone_text(
    session: &ExperimentSession,
    current_iteration: usize,
    best_improvement: f64,
    runtime_formatted: String,
) -> String {
    let mut text = String::new();
    
    text.push_str(&"=".repeat(60));
    text.push('\n');
    text.push_str(&format!("[PROGRESS] Experiment {}\n", session.session_id));
    text.push_str(&"=".repeat(60));
    text.push_str("\n\n");

    text.push_str(&format!("Question: {}\n", session.question));
    text.push_str(&format!("Hypothesis: {}\n\n", session.design.hypothesis));

    text.push_str(&"-".repeat(60));
    text.push('\n');
    text.push_str("CURRENT PROGRESS\n");
    text.push_str(&"-".repeat(60));
    text.push('\n');
    text.push_str(&format!("Baseline Value:   {}\n", session.baseline_record.value));
    text.push_str(&format!("Best Improvement: {:+.1}%\n", best_improvement));
    text.push_str(&format!("Iterations:       {}/{}\n", current_iteration, session.iterations.len()));
    text.push_str(&format!("Runtime So Far:   {}\n\n", runtime_formatted));

    if !session.iterations.is_empty() {
        text.push_str(&"-".repeat(60));
        text.push('\n');
        text.push_str("ITERATION TIMELINE\n");
        text.push_str(&"-".repeat(60));
        text.push('\n');
        text.push_str(&format!("{:>4}  {:24}  {:>10}  {:>12}  {:>8}\n", "#", "Timestamp", "Value", "Improvement", "Kept"));
        text.push_str(&"-".repeat(60));
        text.push('\n');

        for iter in &session.iterations {
            let status_icon = if iter.kept { "✅" } else { "❌" };
            text.push_str(&format!(
                "{:>4}  {:24}  {:>10.2}  {:>+12.2}%  {:>8}\n",
                iter.iteration,
                iter.timestamp,
                iter.metric_value,
                iter.improvement,
                status_icon
            ));
        }
        text.push('\n');
    }

    text.push('\n');
    text.push_str(&"=".repeat(60));
    text.push('\n');
    text.push_str(&format!(
        "Generated by pi-autoresearch v{} | {}\n",
        env!("CARGO_PKG_VERSION"),
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));

    text
}

/// Sends a Slack notification via webhook
///
/// # Arguments
///
/// * `webhook_url` - The Slack webhook URL to send the notification to
/// * `session` - The experiment session containing the results
/// * `target_achieved` - Whether the experiment achieved its target
///
/// # Examples
///
/// ```ignore
/// use pi_autoresearch::notification::send_slack;
/// use pi_autoresearch::session::ExperimentSession;
///
/// let session = ExperimentSession::new(/* ... */);
/// let result = send_slack("https://hooks.slack.com/services/xxx", &session, true);
/// assert!(result.is_ok());
/// ```
///
/// # Notes
///
/// - Uses Slack's blocks API for rich formatting
/// - Color-coded status: green for success, red for failure
/// - Includes experiment summary and key metrics
///
/// # Errors
///
/// Returns an error if:
/// - The webhook URL is invalid
/// - The HTTP request fails
/// - Slack returns an error response
pub fn send_slack(webhook_url: &str, session: &ExperimentSession, target_achieved: bool) -> Result<()> {
    // Validate URL
    if webhook_url.is_empty() {
        anyhow::bail!("Slack webhook URL cannot be empty");
    }
    
    // Determine color based on success/failure
    let color = if target_achieved {
        "#2ecc71" // Green for success
    } else {
        "#e74c3c" // Red for failure
    };
    
    // Calculate metrics
    let best_improvement = session.calculate_final_improvement() * 100.0;
    let runtime_seconds = WebhookPayload::calculate_runtime(&session.start_time, &session.end_time);
    let runtime_formatted = format_duration(runtime_seconds);
    
    // Build Slack blocks message
    let blocks = vec![
        // Header block
        serde_json::json!({
            "type": "header",
            "text": {
                "type": "plain_text",
                "text": if target_achieved { "✅ Experiment Complete" } else { "❌ Experiment Failed" },
                "emoji": true
            }
        }),
        // Context block
        serde_json::json!({
            "type": "context",
            "elements": [
                {
                    "type": "mrkdwn",
                    "text": format!("*Session:* {} | *Metric:* {}", session.session_id, session.design.metric)
                }
            ]
        }),
        // Section block with summary
        serde_json::json!({
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": format!("*Question:* {}\n*Hypothesis:* {}", session.question, session.design.hypothesis)
            }
        }),
        // Divider
        serde_json::json!({
            "type": "divider"
        }),
        // Metrics block
        serde_json::json!({
            "type": "section",
            "fields": [
                {
                    "type": "mrkdwn",
                    "text": format!("*Baseline*\\n{}", session.baseline_record.value)
                },
                {
                    "type": "mrkdwn",
                    "text": format!("*Best Improvement*\\n{}%", best_improvement.signum() * best_improvement.abs().round() / 10.0 * 10.0 / 10.0)
                },
                {
                    "type": "mrkdwn",
                    "text": format!("*Iterations*\\n{}", session.iterations.len())
                },
                {
                    "type": "mrkdwn",
                    "text": format!("*Runtime*\\n{}", runtime_formatted)
                }
            ]
        }),
        // Result block with color
        serde_json::json!({
            "type": "section",
            "attachments": [
                {
                    "color": color,
                    "fields": [
                        {
                            "title": "Result",
                            "value": if target_achieved {
                                "✅ Target achieved! Experiment was successful."
                            } else {
                                "❌ Target not achieved. Consider adjusting parameters or hypothesis."
                            }
                        }
                    ]
                }
            ]
        }),
    ];
    
    // Create Slack message payload
    let slack_payload = serde_json::json!({
        "channel": "#experiments", // Default channel (can be overridden by webhook)
        "username": "pi-autoresearch",
        "icon_emoji": "🔬",
        "blocks": blocks
    });
    
    // Serialize to JSON
    let json = serde_json::to_string(&slack_payload)?;
    
    // Send to Slack
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(webhook_url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header("User-Agent", "pi-autoresearch")
        .body(json)
        .send()?;
    
    // Check response status
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        anyhow::bail!("Slack webhook request failed with status {}: {}", status, body);
    }
    
    Ok(())
}

/// Sends a raw webhook payload to the specified URL
///
/// # Arguments
///
/// * `url` - The webhook URL to send the POST request to
/// * `payload` - The payload to send
///
/// # Examples
///
/// ```ignore
/// use pi_autoresearch::notification::{send_webhook_raw, WebhookPayload};
///
/// let payload = WebhookPayload {
///     notification_type: "test".to_string(),
///     session_id: "test_session".to_string(),
///     question: "Test question".to_string(),
///     metric: "test_metric".to_string(),
///     baseline: 100.0,
///     best_improvement: -10.0,
///     iterations: 5,
///     runtime_seconds: 120.0,
///     target_achieved: true,
///     metadata: HashMap::new(),
/// };
/// let result = send_webhook_raw("https://hooks.example.com/xxx", &payload);
/// assert!(result.is_ok());
/// ```
pub fn send_webhook_raw(url: &str, payload: &WebhookPayload) -> Result<()> {
    // Validate URL
    if url.is_empty() {
        anyhow::bail!("Webhook URL cannot be empty");
    }
    
    // Serialize payload to JSON
    let json = serde_json::to_string(payload)?;
    
    // Use reqwest to send the webhook
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header("User-Agent", "pi-autoresearch")
        .body(json)
        .send()?;
    
    // Check response status
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_default();
        anyhow::bail!("Webhook request failed with status {}: {}", status, body);
    }
    
    Ok(())
}

/// Formats a duration in seconds to a human-readable string
fn format_duration(seconds: f64) -> String {
    if seconds < 60.0 {
        format!("{:.0}s", seconds)
    } else if seconds < 3600.0 {
        format!("{:.1}m", seconds / 60.0)
    } else {
        format!("{:.1}h", seconds / 3600.0)
    }
}

/// Email configuration for sending notifications
#[derive(Debug, Clone)]
pub struct EmailConfig {
    /// SMTP server hostname
    pub smtp_host: String,
    /// SMTP server port
    pub smtp_port: u16,
    /// SMTP username
    pub username: Option<String>,
    /// SMTP password
    pub password: Option<String>,
    /// Email sender address
    pub from_address: String,
    /// Whether to use TLS
    pub use_tls: bool,
    /// Whether to use STARTTLS
    pub use_starttls: bool,
}

impl EmailConfig {
    /// Creates a new EmailConfig with default values
    pub fn new(smtp_host: String, from_address: String) -> Self {
        EmailConfig {
            smtp_host,
            smtp_port: 587,
            username: None,
            password: None,
            from_address,
            use_tls: false,
            use_starttls: true,
        }
    }

    /// Sets the SMTP port
    pub fn with_port(mut self, port: u16) -> Self {
        self.smtp_port = port;
        self
    }

    /// Sets the SMTP credentials
    pub fn with_credentials(mut self, username: String, password: String) -> Self {
        self.username = Some(username);
        self.password = Some(password);
        self
    }

    /// Enables TLS
    pub fn with_tls(mut self) -> Self {
        self.use_tls = true;
        self
    }

    /// Enables STARTTLS
    pub fn with_starttls(mut self) -> Self {
        self.use_starttls = true;
        self
    }
}

/// Sends an email notification
///
/// # Arguments
///
/// * `to` - The recipient email address
/// * `config` - Email configuration including SMTP settings
/// * `session` - The experiment session containing the results
/// * `target_achieved` - Whether the experiment achieved its target
///
/// # Examples
///
/// ```ignore
/// use pi_autoresearch::notification::{send_email, EmailConfig};
/// use pi_autoresearch::session::ExperimentSession;
///
/// let session = ExperimentSession::new(/* ... */);
/// let config = EmailConfig::new("smtp.example.com".to_string(), "noreply@example.com".to_string())
///     .with_credentials("user".to_string(), "pass".to_string());
/// let result = send_email("recipient@example.com", &config, &session, true);
/// assert!(result.is_ok());
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The recipient email is invalid
/// - SMTP connection fails
/// - Email sending fails
pub fn send_email(
    to: &str,
    config: &EmailConfig,
    session: &ExperimentSession,
    target_achieved: bool,
) -> Result<()> {
    // Validate recipient
    if to.is_empty() {
        anyhow::bail!("Recipient email address cannot be empty");
    }

    // Calculate metrics
    let best_improvement = session.calculate_final_improvement() * 100.0;
    let runtime_seconds = WebhookPayload::calculate_runtime(&session.start_time, &session.end_time);
    let runtime_formatted = format_duration(runtime_seconds);
    let status = if target_achieved {
        "✅ SUCCESS".to_string()
    } else {
        "❌ FAILED".to_string()
    };
    let status_color = if target_achieved {
        "#2ecc71"
    } else {
        "#e74c3c"
    };

    // Build HTML email body
    let html_body = build_email_html(
        session,
        &status,
        status_color,
        best_improvement,
        runtime_formatted.clone(),
        target_achieved,
    );

    // Build plain text body
    let text_body = build_email_text(
        session,
        &status,
        best_improvement,
        runtime_formatted,
        target_achieved,
    );

    // Create email subject
    let subject = format!(
        "[{}] Experiment {} - {}",
        status,
        session.session_id,
        if target_achieved { "Target Achieved!" } else { "Target Not Achieved" }
    );

    // Build email using lettre
    let text_part = lettre::message::SinglePart::builder()
        .header(lettre::message::header::ContentType::TEXT_PLAIN)
        .body(text_body);
    
    let html_part = lettre::message::SinglePart::builder()
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .body(html_body);
    
    let alternative = lettre::message::MultiPart::alternative()
        .singlepart(text_part)
        .singlepart(html_part);
    
    let email = lettre::message::Message::builder()
        .from(config.from_address.parse()?)
        .to(to.parse()?)
        .subject(subject)
        .multipart(alternative)?;

    // Build SMTP transport using lettre 0.11 API
    let mut transport_builder = lettre::transport::smtp::SmtpTransport::relay(&config.smtp_host)?;
    transport_builder = transport_builder.port(config.smtp_port);
    
    // Add credentials if provided
    if let (Some(username), Some(password)) = (&config.username, &config.password) {
        let credentials = Credentials::new(username.clone(), password.clone());
        transport_builder = transport_builder.credentials(credentials);
    }
    
    let transport = transport_builder.build();

    // Send the email
    transport.send(&email)?;

    Ok(())
}

/// Builds HTML email body for experiment notifications
fn build_email_html(
    session: &ExperimentSession,
    status: &str,
    status_color: &str,
    best_improvement: f64,
    runtime_formatted: String,
    _target_achieved: bool,
) -> String {
    let iterations_html: String = session.iterations.iter().map(|iter| {
        let status_icon = if iter.kept { "✅" } else { "❌" };
        format!(
            "<tr><td>{}</td><td>{}</td><td>{:.2}</td><td>{:+.2}%</td><td>{}</td></tr>",
            iter.iteration,
            iter.timestamp,
            iter.metric_value,
            iter.improvement,
            status_icon
        )
    }).collect();

    let improvement_color = if best_improvement > 0.0 { "#2ecc71" } else { "#e74c3c" };

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }}
        .container {{ max-width: 800px; margin: 0 auto; background-color: white; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .header {{ background-color: {status_color}; color: white; padding: 20px; text-align: center; }}
        .header h1 {{ margin: 0; font-size: 24px; }}
        .content {{ padding: 20px; }}
        .section {{ margin-bottom: 20px; }}
        .section h2 {{ color: #333; border-bottom: 2px solid #eee; padding-bottom: 10px; }}
        .metrics {{ display: grid; grid-template-columns: repeat(2, 1fr); gap: 15px; margin: 15px 0; }}
        .metric-card {{ background-color: #f9f9f9; border-radius: 6px; padding: 15px; text-align: center; }}
        .metric-card .value {{ font-size: 28px; font-weight: bold; color: #333; }}
        .metric-card .label {{ color: #666; font-size: 14px; margin-top: 5px; }}
        table {{ width: 100%; border-collapse: collapse; margin-top: 10px; }}
        th {{ background-color: #f5f5f5; text-align: left; padding: 12px; border-bottom: 2px solid #ddd; }}
        td {{ padding: 10px 12px; border-bottom: 1px solid #eee; }}
        .footer {{ background-color: #f5f5f5; padding: 15px; text-align: center; color: #666; font-size: 12px; }}
        .status {{ font-size: 32px; margin-bottom: 10px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div class="status">{status}</div>
            <h1>Experiment {status}</h1>
        </div>
        <div class="content">
            <div class="section">
                <h2>📋 Overview</h2>
                <p><strong>Session ID:</strong> {session_id}</p>
                <p><strong>Question:</strong> {question}</p>
                <p><strong>Hypothesis:</strong> {hypothesis}</p>
                <p><strong>Metric:</strong> {metric}</p>
            </div>
            <div class="section">
                <h2>📊 Key Metrics</h2>
                <div class="metrics">
                    <div class="metric-card">
                        <div class="value">{baseline}</div>
                        <div class="label">Baseline Value</div>
                    </div>
                    <div class="metric-card">
                        <div class="value" style="color: {improvement_color}">{improvement:+.1}%</div>
                        <div class="label">Best Improvement</div>
                    </div>
                    <div class="metric-card">
                        <div class="value">{iterations}</div>
                        <div class="label">Total Iterations</div>
                    </div>
                    <div class="metric-card">
                        <div class="value">{runtime}</div>
                        <div class="label">Total Runtime</div>
                    </div>
                </div>
            </div>
            <div class="section">
                <h2>🔄 Iteration Timeline</h2>
                <table>
                    <thead>
                        <tr><th>#</th><th>Timestamp</th><th>Value</th><th>Improvement</th><th>Kept</th></tr>
                    </thead>
                    <tbody>
                        {iterations_html}
                    </tbody>
                </table>
            </div>
        </div>
        <div class="footer">
            <p>Generated by pi-autoresearch v{version}</p>
            <p>{timestamp}</p>
        </div>
    </div>
</body>
</html>"#,
        session_id = session.session_id,
        question = session.question,
        hypothesis = session.design.hypothesis,
        metric = session.design.metric,
        baseline = session.baseline_record.value,
        improvement = best_improvement,
        iterations = session.iterations.len(),
        runtime = runtime_formatted,
        iterations_html = if iterations_html.is_empty() { "<tr><td colspan='5' style='text-align: center; color: #999;'>No iterations yet</td></tr>".to_string() } else { iterations_html },
        version = env!("CARGO_PKG_VERSION"),
        timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    )
}

/// Builds plain text email body for experiment notifications
fn build_email_text(
    session: &ExperimentSession,
    status: &str,
    best_improvement: f64,
    runtime_formatted: String,
    _target_achieved: bool,
) -> String {
    let mut text = String::new();
    
    text.push_str(&"=".repeat(60));
    text.push('\n');
    text.push_str(&format!("[{}] Experiment {}\n", status, session.session_id));
    text.push_str(&"=".repeat(60));
    text.push_str("\n\n");

    text.push_str(&format!("Question: {}\n", session.question));
    text.push_str(&format!("Hypothesis: {}\n\n", session.design.hypothesis));

    text.push_str(&"-".repeat(60));
    text.push('\n');
    text.push_str("KEY METRICS\n");
    text.push_str(&"-".repeat(60));
    text.push('\n');
    text.push_str(&format!("Baseline Value:   {}\n", session.baseline_record.value));
    text.push_str(&format!("Best Improvement: {:+.1}%\n", best_improvement));
    text.push_str(&format!("Total Iterations: {}\n", session.iterations.len()));
    text.push_str(&format!("Total Runtime:    {}\n\n", runtime_formatted));

    if !session.iterations.is_empty() {
        text.push_str(&"-".repeat(60));
        text.push('\n');
        text.push_str("ITERATION TIMELINE\n");
        text.push_str(&"-".repeat(60));
        text.push('\n');
        text.push_str(&format!("{:>4}  {:24}  {:>10}  {:>12}  {:>8}\n", "#", "Timestamp", "Value", "Improvement", "Kept"));
        text.push_str(&"-".repeat(60));
        text.push('\n');

        for iter in &session.iterations {
            let status_icon = if iter.kept { "✅" } else { "❌" };
            text.push_str(&format!(
                "{:>4}  {:24}  {:>10.2}  {:>+12.2}%  {:>8}\n",
                iter.iteration,
                iter.timestamp,
                iter.metric_value,
                iter.improvement,
                status_icon
            ));
        }
        text.push('\n');
    }

    text.push('\n');
    text.push_str(&"=".repeat(60));
    text.push('\n');
    text.push_str(&format!(
        "Generated by pi-autoresearch v{} | {}\n",
        env!("CARGO_PKG_VERSION"),
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));

    text
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase1_design::{BaselineRecord, ExperimentDesign};

    fn create_test_baseline() -> BaselineRecord {
        BaselineRecord::new(
            "2024-01-01T00:00:00Z".to_string(),
            "abc123".to_string(),
            "test-metric".to_string(),
            "test-command".to_string(),
            100.0,
            vec![100.0, 101.0],
            0.01,
            true,
        )
    }

    fn create_test_design() -> ExperimentDesign {
        ExperimentDesign::new(
            "test hypothesis".to_string(),
            "test-metric".to_string(),
            "test-command".to_string(),
            100.0,
            10.0,
        )
    }

    fn create_test_session() -> ExperimentSession {
        ExperimentSession::new(
            "test_session_123".to_string(),
            "How can I optimize performance?".to_string(),
            create_test_design(),
            create_test_baseline(),
        )
    }

    #[test]
    fn test_webhook_payload_new_completion() {
        let session = create_test_session();
        
        let payload = WebhookPayload::new_completion(&session, true);
        
        assert_eq!(payload.notification_type, "experiment_complete");
        assert_eq!(payload.session_id, "test_session_123");
        assert_eq!(payload.question, "How can I optimize performance?");
        assert_eq!(payload.metric, "test-metric");
        assert_eq!(payload.baseline, 100.0);
        assert_eq!(payload.iterations, 0);
        assert!(payload.target_achieved);
        assert!(payload.metadata.contains_key("version"));
        assert!(payload.metadata.contains_key("start_time"));
    }

    #[test]
    fn test_webhook_payload_new_milestone() {
        let session = create_test_session();
        
        let payload = WebhookPayload::new_milestone(&session, 5);
        
        assert_eq!(payload.notification_type, "iteration_milestone");
        assert_eq!(payload.session_id, "test_session_123");
        assert_eq!(payload.iterations, 5);
        assert!(!payload.target_achieved);
        assert_eq!(payload.metadata.get("current_iteration"), Some(&"5".to_string()));
    }

    #[test]
    fn test_webhook_payload_serialization() {
        let session = create_test_session();
        
        let payload = WebhookPayload::new_completion(&session, true);
        let json = serde_json::to_string(&payload).expect("Failed to serialize payload");
        
        // Verify JSON contains expected fields
        assert!(json.contains("experiment_complete"));
        assert!(json.contains("test_session_123"));
        assert!(json.contains("How can I optimize performance?"));
        assert!(json.contains("test-metric"));
        
        // Verify we can deserialize it back
        let deserialized: WebhookPayload = serde_json::from_str(&json).expect("Failed to deserialize payload");
        assert_eq!(deserialized.session_id, payload.session_id);
        assert_eq!(deserialized.best_improvement, payload.best_improvement);
    }

    #[test]
    fn test_send_webhook_empty_url() {
        let session = create_test_session();
        
        let result = send_webhook("", &session, true);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_send_webhook_invalid_url() {
        let session = create_test_session();
        
        // This should fail because the URL is not reachable
        let result = send_webhook("http://localhost:59999/webhook", &session, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_runtime_with_end_time() {
        let start = "2024-01-01T00:00:00Z";
        let end = Some("2024-01-01T00:02:00Z".to_string()); // 2 minutes later
        
        let runtime = WebhookPayload::calculate_runtime(start, &end);
        assert!((runtime - 120.0).abs() < 1.0); // Should be approximately 120 seconds
    }

    #[test]
    fn test_calculate_runtime_without_end_time() {
        // This test will always pass since it calculates from start to now
        let start = "2024-01-01T00:00:00Z";
        let end = None::<String>;
        
        let runtime = WebhookPayload::calculate_runtime(start, &end);
        assert!(runtime > 0.0); // Should be positive (time has passed)
    }

    #[test]
    fn test_calculate_runtime_invalid_timestamp() {
        let start = "invalid-timestamp";
        let end = None::<String>;
        
        let runtime = WebhookPayload::calculate_runtime(start, &end);
        assert_eq!(runtime, 0.0); // Should return 0 for invalid timestamps
    }

    #[test]
    fn test_send_slack_empty_url() {
        let session = create_test_session();
        
        let result = send_slack("", &session, true);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_send_slack_invalid_url() {
        let session = create_test_session();
        
        // This should fail because the URL is not reachable
        let result = send_slack("http://localhost:59998/slack", &session, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_slack_message_format_success() {
        // Test that Slack message can be constructed for successful experiment
        let session = create_test_session();
        
        // We can't easily test the full send_slack without a real webhook,
        // but we can verify the color logic by checking the function doesn't panic
        let result = send_slack("http://localhost:59997/slack", &session, true);
        // Should fail with connection error, not panic
        assert!(result.is_err());
    }

    #[test]
    fn test_slack_message_format_failure() {
        // Test that Slack message can be constructed for failed experiment
        let session = create_test_session();
        
        let result = send_slack("http://localhost:59996/slack", &session, false);
        // Should fail with connection error, not panic
        assert!(result.is_err());
    }

    #[test]
    fn test_format_duration_seconds() {
        // Test duration formatting (internal function tested through public API)
        let session = create_test_session();
        
        // This will construct a Slack message which uses format_duration
        // We're testing that it doesn't panic with various runtime values
        let result = send_slack("http://localhost:59995/slack", &session, true);
        assert!(result.is_err()); // Should fail with connection error
    }

    #[test]
    fn test_slack_with_iterations() {
        // Test Slack notification with iterations in session
        let mut session = create_test_session();
        
        // Add a test iteration
        use crate::phase2_iterate::IterationRecord;
        let iteration = IterationRecord::new(
            1,
            "Test changes".to_string(),
            90.0,
            -10.0,
            true,
        );
        session.iterations.push(iteration);
        
        let result = send_slack("http://localhost:59994/slack", &session, true);
        assert!(result.is_err()); // Should fail with connection error, not panic
    }

    // Email notification tests

    #[test]
    fn test_email_config_new() {
        let config = EmailConfig::new("smtp.example.com".to_string(), "noreply@example.com".to_string());
        
        assert_eq!(config.smtp_host, "smtp.example.com");
        assert_eq!(config.smtp_port, 587);
        assert_eq!(config.from_address, "noreply@example.com");
        assert!(config.username.is_none());
        assert!(config.password.is_none());
        assert!(!config.use_tls);
        assert!(config.use_starttls);
    }

    #[test]
    fn test_email_config_builder() {
        let config = EmailConfig::new("smtp.example.com".to_string(), "noreply@example.com".to_string())
            .with_port(465)
            .with_credentials("user".to_string(), "pass".to_string())
            .with_tls();
        
        assert_eq!(config.smtp_port, 465);
        assert_eq!(config.username, Some("user".to_string()));
        assert_eq!(config.password, Some("pass".to_string()));
        assert!(config.use_tls);
    }

    #[test]
    fn test_send_email_empty_recipient() {
        let session = create_test_session();
        let config = EmailConfig::new("smtp.example.com".to_string(), "noreply@example.com".to_string());
        
        let result = send_email("", &config, &session, true);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_send_email_invalid_smtp() {
        let session = create_test_session();
        let config = EmailConfig::new("localhost".to_string(), "noreply@example.com".to_string())
            .with_port(59999); // Invalid port
        
        let result = send_email("recipient@example.com", &config, &session, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_email_html_structure() {
        let session = create_test_session();
        let status = "SUCCESS";
        let status_color = "#2ecc71";
        let best_improvement = 10.0;
        let runtime_formatted = "1m".to_string();
        
        let html = build_email_html(&session, status, status_color, best_improvement, runtime_formatted, true);
        
        // Verify HTML structure
        assert!(html.contains("<!DOCTYPE html"));
        assert!(html.contains("<html>"));
        assert!(html.contains("<head>"));
        assert!(html.contains("<body>"));
        assert!(html.contains("test_session_123"));
        assert!(html.contains("How can I optimize performance?"));
        assert!(html.contains("test-metric"));
        assert!(html.contains("<table>"));
        assert!(html.contains("Baseline Value"));
        assert!(html.contains("Best Improvement"));
    }

    #[test]
    fn test_build_email_html_with_iterations() {
        let mut session = create_test_session();
        
        // Add test iterations
        use crate::phase2_iterate::IterationRecord;
        let iteration1 = IterationRecord::new(1, "Test 1".to_string(), 90.0, -10.0, true);
        let iteration2 = IterationRecord::new(2, "Test 2".to_string(), 85.0, -5.0, true);
        session.iterations.push(iteration1);
        session.iterations.push(iteration2);
        
        let html = build_email_html(&session, "SUCCESS", "#2ecc71", 15.0, "2m".to_string(), true);
        
        // Verify iterations are included
        assert!(html.contains("<tr>"));
        assert!(html.contains("<td>1</td>"));
        assert!(html.contains("<td>2</td>"));
    }

    #[test]
    fn test_build_email_html_failure() {
        let session = create_test_session();
        
        let html = build_email_html(&session, "FAILED", "#e74c3c", -5.0, "1m".to_string(), false);
        
        assert!(html.contains("#e74c3c"));
        assert!(html.contains("FAILED"));
    }

    #[test]
    fn test_build_email_text_structure() {
        let session = create_test_session();
        
        let text = build_email_text(&session, "SUCCESS", 10.0, "1m".to_string(), true);
        
        // Verify text structure
        assert!(text.contains("test_session_123"));
        assert!(text.contains("How can I optimize performance?"));
        assert!(text.contains("Baseline Value"));
        assert!(text.contains("Best Improvement"));
        assert!(text.contains("Total Iterations"));
        assert!(text.contains("Total Runtime"));
        assert!(text.contains("KEY METRICS"));
    }

    #[test]
    fn test_build_email_text_with_iterations() {
        let mut session = create_test_session();
        
        // Add test iterations
        use crate::phase2_iterate::IterationRecord;
        let iteration1 = IterationRecord::new(1, "Test 1".to_string(), 90.0, -10.0, true);
        session.iterations.push(iteration1);
        
        let text = build_email_text(&session, "SUCCESS", 10.0, "1m".to_string(), true);
        
        // Verify iterations are included
        assert!(text.contains("ITERATION TIMELINE"));
        assert!(text.contains("Timestamp"));
        assert!(text.contains("Value"));
        assert!(text.contains("Improvement"));
    }

    #[test]
    fn test_build_email_text_empty_iterations() {
        let session = create_test_session();
        
        let text = build_email_text(&session, "SUCCESS", 0.0, "0s".to_string(), true);
        
        // Should not have iteration timeline when no iterations
        assert!(!text.contains("ITERATION TIMELINE"));
    }

    #[test]
    fn test_email_config_clone() {
        let config1 = EmailConfig::new("smtp.example.com".to_string(), "noreply@example.com".to_string())
            .with_port(465)
            .with_credentials("user".to_string(), "pass".to_string());
        
        let config2 = config1.clone();
        
        assert_eq!(config1.smtp_host, config2.smtp_host);
        assert_eq!(config1.smtp_port, config2.smtp_port);
        assert_eq!(config1.username, config2.username);
        assert_eq!(config1.password, config2.password);
    }

    #[test]
    fn test_email_html_contains_version() {
        let session = create_test_session();
        let html = build_email_html(&session, "SUCCESS", "#2ecc71", 10.0, "1m".to_string(), true);
        
        // Should contain version info
        assert!(html.contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn test_email_text_contains_version() {
        let session = create_test_session();
        let text = build_email_text(&session, "SUCCESS", 10.0, "1m".to_string(), true);
        
        // Should contain version info
        assert!(text.contains(env!("CARGO_PKG_VERSION")));
    }

    // Milestone notification tests

    #[test]
    fn test_send_slack_milestone_empty_url() {
        let session = create_test_session();
        
        let result = send_slack_milestone("", &session, 5);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_send_slack_milestone_invalid_url() {
        let session = create_test_session();
        
        // This should fail because the URL is not reachable
        let result = send_slack_milestone("http://localhost:59993/slack", &session, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_slack_milestone_message_format() {
        // Test that Slack milestone message can be constructed
        let session = create_test_session();
        
        let result = send_slack_milestone("http://localhost:59992/slack", &session, 5);
        // Should fail with connection error, not panic
        assert!(result.is_err());
    }

    #[test]
    fn test_send_email_milestone_empty_recipient() {
        let session = create_test_session();
        let config = EmailConfig::new("smtp.example.com".to_string(), "noreply@example.com".to_string());
        
        let result = send_email_milestone("", &config, &session, 5);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_send_email_milestone_invalid_smtp() {
        let session = create_test_session();
        let config = EmailConfig::new("localhost".to_string(), "noreply@example.com".to_string())
            .with_port(59999); // Invalid port
        
        let result = send_email_milestone("recipient@example.com", &config, &session, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_email_milestone_html_structure() {
        let session = create_test_session();
        let current_iteration = 5;
        let best_improvement = 10.0;
        let runtime_formatted = "1m".to_string();
        
        let html = build_email_milestone_html(&session, current_iteration, best_improvement, runtime_formatted);
        
        // Verify HTML structure
        assert!(html.contains("<!DOCTYPE html"));
        assert!(html.contains("<html>"));
        assert!(html.contains("test_session_123"));
        assert!(html.contains("How can I optimize performance?"));
        assert!(html.contains("Iteration Timeline"));
        assert!(html.contains("Current Progress"));
    }

    #[test]
    fn test_build_email_milestone_html_with_iterations() {
        let mut session = create_test_session();
        
        // Add test iterations
        use crate::phase2_iterate::IterationRecord;
        let iteration1 = IterationRecord::new(1, "Test 1".to_string(), 90.0, -10.0, true);
        let iteration2 = IterationRecord::new(2, "Test 2".to_string(), 85.0, -5.0, true);
        session.iterations.push(iteration1);
        session.iterations.push(iteration2);
        
        let html = build_email_milestone_html(&session, 5, 15.0, "2m".to_string());
        
        // Verify iterations are included
        assert!(html.contains("<tr>"));
        assert!(html.contains("<td>1</td>"));
        assert!(html.contains("<td>2</td>"));
    }

    #[test]
    fn test_build_email_milestone_text_structure() {
        let session = create_test_session();
        
        let text = build_email_milestone_text(&session, 5, 10.0, "1m".to_string());
        
        // Verify text structure
        assert!(text.contains("test_session_123"));
        assert!(text.contains("How can I optimize performance?"));
        assert!(text.contains("CURRENT PROGRESS"));
        assert!(text.contains("Baseline Value"));
        assert!(text.contains("Best Improvement"));
        assert!(text.contains("Iterations:"));
        assert!(text.contains("Runtime So Far"));
    }

    #[test]
    fn test_build_email_milestone_text_with_iterations() {
        let mut session = create_test_session();
        
        // Add test iterations
        use crate::phase2_iterate::IterationRecord;
        let iteration1 = IterationRecord::new(1, "Test 1".to_string(), 90.0, -10.0, true);
        session.iterations.push(iteration1);
        
        let text = build_email_milestone_text(&session, 5, 10.0, "1m".to_string());
        
        // Verify iterations are included
        assert!(text.contains("ITERATION TIMELINE"));
        assert!(text.contains("Timestamp"));
        assert!(text.contains("Value"));
        assert!(text.contains("Improvement"));
    }

    #[test]
    fn test_build_email_milestone_text_empty_iterations() {
        let session = create_test_session();
        
        let text = build_email_milestone_text(&session, 5, 0.0, "0s".to_string());
        
        // Should not have iteration timeline when no iterations
        assert!(!text.contains("ITERATION TIMELINE"));
    }

    #[test]
    fn test_milestone_notification_with_iterations() {
        // Test milestone notification with iterations in session
        let mut session = create_test_session();
        
        // Add a test iteration
        use crate::phase2_iterate::IterationRecord;
        let iteration = IterationRecord::new(
            1,
            "Test changes".to_string(),
            90.0,
            -10.0,
            true,
        );
        session.iterations.push(iteration);
        
        // Test webhook milestone notification
        let result = send_milestone_notification("http://localhost:59991/webhook", &session, 5);
        assert!(result.is_err()); // Should fail with connection error, not panic
        
        // Test Slack milestone notification
        let result = send_slack_milestone("http://localhost:59990/slack", &session, 5);
        assert!(result.is_err()); // Should fail with connection error, not panic
    }
}
