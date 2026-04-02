use anyhow::Result;
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

/// Sends an iteration milestone notification
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
}
