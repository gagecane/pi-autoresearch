pub mod cli;
pub mod phase1_design;
pub mod phase2_iterate;
pub mod stuck_detector;
pub mod metric_evaluator;
pub mod pi_agent;
pub mod session;
pub mod export;
pub mod notification;

pub use cli::{Cli, ExportFormat, NotificationProvider, AuditLogFormat};
pub use phase1_design::{ExperimentDesign, BaselineRecord, generate_design};
pub use phase2_iterate::IterationRecord;
pub use stuck_detector::{StuckReason, StuckDetector, StuckDetectorConfig, IterationState};
pub use session::{ExperimentSession, SessionManager, SessionRecord, generate_session_id};
pub use pi_agent::{PiAgent, BranchManager, generate_uuid};
pub use notification::{send_webhook, send_milestone_notification, send_slack, send_slack_milestone, send_email, send_email_milestone, WebhookPayload, EmailConfig};

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // ==================== StuckReason Tests ====================

    #[test]
    fn test_stuck_reason_display_iteration_timeout() {
        let reason = StuckReason::IterationTimeout;
        let output = format!("{}", reason);
        assert!(output.contains("Iteration timeout exceeded"));
        assert!(output.contains("SUGGESTION"));
        assert!(output.contains("iteration_timeout_minutes"));
    }

    #[test]
    fn test_stuck_reason_display_stall_limit() {
        let reason = StuckReason::StallLimitReached;
        let output = format!("{}", reason);
        assert!(output.contains("No improvement after multiple iterations"));
        assert!(output.contains("SUGGESTION"));
        assert!(output.contains("stall_limit"));
    }

    #[test]
    fn test_stuck_reason_display_total_timeout() {
        let reason = StuckReason::TotalTimeout;
        let output = format!("{}", reason);
        assert!(output.contains("Total experiment timeout exceeded"));
        assert!(output.contains("SUGGESTION"));
        assert!(output.contains("total_timeout_minutes"));
    }

    #[test]
    fn test_stuck_reason_display_convergence() {
        let reason = StuckReason::ConvergenceAchieved;
        let output = format!("{}", reason);
        assert!(output.contains("Metric convergence achieved"));
        assert!(output.contains("NOTE"));
    }

    #[test]
    fn test_stuck_reason_display_max_iterations() {
        let reason = StuckReason::MaxIterationsReached;
        let output = format!("{}", reason);
        assert!(output.contains("Maximum iterations reached"));
        assert!(output.contains("SUGGESTION"));
        assert!(output.contains("max_iterations"));
    }

    #[test]
    fn test_stuck_reason_debug() {
        let reason = StuckReason::IterationTimeout;
        let debug_str = format!("{:?}", reason);
        assert!(debug_str.contains("IterationTimeout"));
    }

    #[test]
    fn test_stuck_reason_clone() {
        let reason = StuckReason::StallLimitReached;
        let cloned = reason.clone();
        assert_eq!(reason, cloned);
    }

    #[test]
    fn test_stuck_reason_partial_eq() {
        let reason1 = StuckReason::ConvergenceAchieved;
        let reason2 = StuckReason::ConvergenceAchieved;
        let reason3 = StuckReason::MaxIterationsReached;
        assert_eq!(reason1, reason2);
        assert_ne!(reason1, reason3);
    }

    // ==================== IterationState Tests ====================

    #[test]
    fn test_iteration_state_new() {
        let state = IterationState::new(100.0);
        assert_eq!(state.current_iteration, 0);
        assert_eq!(state.best_metric, 100.0);
        assert_eq!(state.best_iteration, 0);
        assert_eq!(state.consecutive_no_improvement, 0);
        assert_eq!(state.backoff_count, 0);
        assert!(state.recent_metrics.is_empty());
    }

    #[test]
    fn test_iteration_state_record_improvement() {
        let mut state = IterationState::new(100.0);
        state.record_improvement(1, 90.0);
        assert_eq!(state.current_iteration, 1);
        assert_eq!(state.best_metric, 90.0);
        assert_eq!(state.best_iteration, 1);
        assert_eq!(state.consecutive_no_improvement, 0);
        assert_eq!(state.backoff_count, 0);
        assert_eq!(state.recent_metrics.len(), 1);
        assert_eq!(state.recent_metrics[0], 90.0);
    }

    #[test]
    fn test_iteration_state_record_no_improvement() {
        let mut state = IterationState::new(100.0);
        state.record_no_improvement(1, 110.0);
        assert_eq!(state.current_iteration, 1);
        assert_eq!(state.best_metric, 100.0);
        assert_eq!(state.best_iteration, 0);
        assert_eq!(state.consecutive_no_improvement, 1);
        assert_eq!(state.recent_metrics.len(), 1);
        assert_eq!(state.recent_metrics[0], 110.0);
    }

    #[test]
    fn test_iteration_state_record_multiple_no_improvement() {
        let mut state = IterationState::new(100.0);
        state.record_no_improvement(1, 110.0);
        state.record_no_improvement(2, 115.0);
        state.record_no_improvement(3, 120.0);
        assert_eq!(state.consecutive_no_improvement, 3);
        assert_eq!(state.recent_metrics.len(), 3);
    }

    #[test]
    fn test_iteration_state_apply_backoff() {
        let mut state = IterationState::new(100.0);
        state.record_no_improvement(1, 110.0);
        state.record_no_improvement(2, 115.0);
        assert_eq!(state.consecutive_no_improvement, 2);
        assert_eq!(state.backoff_count, 0);
        state.apply_backoff();
        assert_eq!(state.consecutive_no_improvement, 0);
        assert_eq!(state.backoff_count, 1);
    }

    #[test]
    fn test_iteration_state_elapsed() {
        let state = IterationState::new(100.0);
        let elapsed = state.elapsed();
        assert!(elapsed.as_secs() >= 0);
    }

    #[test]
    fn test_iteration_state_clone() {
        let state = IterationState::new(100.0);
        let cloned = state.clone();
        assert_eq!(state.current_iteration, cloned.current_iteration);
        assert_eq!(state.best_metric, cloned.best_metric);
    }

    #[test]
    fn test_iteration_state_debug() {
        let state = IterationState::new(100.0);
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("IterationState"));
    }

    // ==================== StuckDetectorConfig Tests ====================

    #[test]
    fn test_stuck_detector_config_default() {
        let config = StuckDetectorConfig::default();
        assert_eq!(config.max_iterations, 20);
        assert_eq!(config.iteration_timeout_secs, 600);
        assert_eq!(config.total_timeout_secs, 7200);
        assert_eq!(config.stall_limit, 5);
        assert_eq!(config.convergence_threshold, 0.01);
        assert_eq!(config.convergence_window, 3);
    }

    #[test]
    fn test_stuck_detector_config_custom() {
        let config = StuckDetectorConfig {
            max_iterations: 10,
            iteration_timeout_secs: 300,
            total_timeout_secs: 3600,
            stall_limit: 3,
            convergence_threshold: 0.001,
            convergence_window: 5,
        };
        assert_eq!(config.max_iterations, 10);
        assert_eq!(config.iteration_timeout_secs, 300);
        assert_eq!(config.total_timeout_secs, 3600);
        assert_eq!(config.stall_limit, 3);
        assert_eq!(config.convergence_threshold, 0.001);
        assert_eq!(config.convergence_window, 5);
    }

    #[test]
    fn test_stuck_detector_config_clone() {
        let config = StuckDetectorConfig::default();
        let cloned = config.clone();
        assert_eq!(config.max_iterations, cloned.max_iterations);
        assert_eq!(config.stall_limit, cloned.stall_limit);
    }

    // ==================== StuckDetector Tests ====================

    #[test]
    fn test_stuck_detector_new() {
        let config = StuckDetectorConfig::default();
        let detector = StuckDetector::new(config.clone());
        // Can't directly access config field, but we can test the methods
        assert!(detector.check_max_iterations(21).is_some());
    }

    #[test]
    fn test_check_total_timeout_not_exceeded() {
        let config = StuckDetectorConfig { total_timeout_secs: 60, ..Default::default() };
        let detector = StuckDetector::new(config);
        let elapsed = Duration::from_secs(30);
        assert!(detector.check_total_timeout(elapsed).is_none());
    }

    #[test]
    fn test_check_total_timeout_exceeded() {
        let config = StuckDetectorConfig { total_timeout_secs: 60, ..Default::default() };
        let detector = StuckDetector::new(config);
        let elapsed = Duration::from_secs(120);
        assert_eq!(detector.check_total_timeout(elapsed), Some(StuckReason::TotalTimeout));
    }

    #[test]
    fn test_check_total_timeout_exact() {
        let config = StuckDetectorConfig { total_timeout_secs: 60, ..Default::default() };
        let detector = StuckDetector::new(config);
        let elapsed = Duration::from_secs(60);
        assert_eq!(detector.check_total_timeout(elapsed), Some(StuckReason::TotalTimeout));
    }

    #[test]
    fn test_check_iteration_timeout_not_exceeded() {
        let config = StuckDetectorConfig { iteration_timeout_secs: 60, ..Default::default() };
        let detector = StuckDetector::new(config);
        let elapsed = Duration::from_secs(30);
        assert!(detector.check_iteration_timeout(elapsed).is_none());
    }

    #[test]
    fn test_check_iteration_timeout_exceeded() {
        let config = StuckDetectorConfig { iteration_timeout_secs: 60, ..Default::default() };
        let detector = StuckDetector::new(config);
        let elapsed = Duration::from_secs(120);
        assert_eq!(detector.check_iteration_timeout(elapsed), Some(StuckReason::IterationTimeout));
    }

    #[test]
    fn test_check_max_iterations_not_reached() {
        let config = StuckDetectorConfig { max_iterations: 20, ..Default::default() };
        let detector = StuckDetector::new(config);
        assert!(detector.check_max_iterations(15).is_none());
    }

    #[test]
    fn test_check_max_iterations_reached() {
        let config = StuckDetectorConfig { max_iterations: 20, ..Default::default() };
        let detector = StuckDetector::new(config);
        assert_eq!(detector.check_max_iterations(20), Some(StuckReason::MaxIterationsReached));
    }

    #[test]
    fn test_check_max_iterations_exceeded() {
        let config = StuckDetectorConfig { max_iterations: 20, ..Default::default() };
        let detector = StuckDetector::new(config);
        assert_eq!(detector.check_max_iterations(25), Some(StuckReason::MaxIterationsReached));
    }

    #[test]
    fn test_check_convergence_not_enough_metrics() {
        let config = StuckDetectorConfig { convergence_window: 3, ..Default::default() };
        let detector = StuckDetector::new(config);
        let metrics = vec![100.0, 101.0];
        assert!(detector.check_convergence(&metrics).is_none());
    }

    #[test]
    fn test_check_convergence_achieved() {
        let config = StuckDetectorConfig {
            convergence_threshold: 0.01,
            convergence_window: 3,
            ..Default::default()
        };
        let detector = StuckDetector::new(config);
        let metrics = vec![100.0, 100.5, 100.3]; // variance < 1%
        assert_eq!(detector.check_convergence(&metrics), Some(StuckReason::ConvergenceAchieved));
    }

    #[test]
    fn test_check_convergence_not_achieved() {
        let config = StuckDetectorConfig {
            convergence_threshold: 0.01,
            convergence_window: 3,
            ..Default::default()
        };
        let detector = StuckDetector::new(config);
        let metrics = vec![100.0, 150.0, 120.0]; // high variance
        assert!(detector.check_convergence(&metrics).is_none());
    }

    #[test]
    fn test_check_convergence_exact_window_size() {
        let config = StuckDetectorConfig {
            convergence_threshold: 0.01,
            convergence_window: 3,
            ..Default::default()
        };
        let detector = StuckDetector::new(config);
        let metrics = vec![100.0, 100.1, 100.2];
        assert_eq!(detector.check_convergence(&metrics), Some(StuckReason::ConvergenceAchieved));
    }

    #[test]
    fn test_check_convergence_zero_values() {
        let config = StuckDetectorConfig {
            convergence_threshold: 0.01,
            convergence_window: 3,
            ..Default::default()
        };
        let detector = StuckDetector::new(config);
        let metrics = vec![0.0, 0.0, 0.0];
        assert_eq!(detector.check_convergence(&metrics), Some(StuckReason::ConvergenceAchieved));
    }

    #[test]
    fn test_check_stall_limit_not_reached() {
        let config = StuckDetectorConfig { stall_limit: 5, ..Default::default() };
        let detector = StuckDetector::new(config);
        assert!(detector.check_stall_limit(3, 2).is_none());
    }

    #[test]
    fn test_check_stall_limit_reached_with_backoff() {
        let config = StuckDetectorConfig { stall_limit: 5, ..Default::default() };
        let detector = StuckDetector::new(config);
        assert_eq!(detector.check_stall_limit(5, 2), Some(StuckReason::StallLimitReached));
    }

    #[test]
    fn test_check_stall_limit_reached_without_backoff() {
        let config = StuckDetectorConfig { stall_limit: 5, ..Default::default() };
        let detector = StuckDetector::new(config);
        assert!(detector.check_stall_limit(5, 1).is_none());
    }

    #[test]
    fn test_check_stall_limit_exceeded() {
        let config = StuckDetectorConfig { stall_limit: 5, ..Default::default() };
        let detector = StuckDetector::new(config);
        assert_eq!(detector.check_stall_limit(10, 2), Some(StuckReason::StallLimitReached));
    }

    #[test]
    fn test_should_backoff_true() {
        let config = StuckDetectorConfig { stall_limit: 5, ..Default::default() };
        let detector = StuckDetector::new(config);
        assert!(detector.should_backoff(5, 1));
    }

    #[test]
    fn test_should_backoff_false_not_enough_no_improvement() {
        let config = StuckDetectorConfig { stall_limit: 5, ..Default::default() };
        let detector = StuckDetector::new(config);
        assert!(!detector.should_backoff(3, 0));
    }

    #[test]
    fn test_should_backoff_false_max_backoff() {
        let config = StuckDetectorConfig { stall_limit: 5, ..Default::default() };
        let detector = StuckDetector::new(config);
        assert!(!detector.should_backoff(5, 2));
    }

    #[test]
    fn test_should_backoff_exact_stall_limit() {
        let config = StuckDetectorConfig { stall_limit: 5, ..Default::default() };
        let detector = StuckDetector::new(config);
        assert!(detector.should_backoff(5, 0));
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_full_stuck_detection_workflow() {
        let config = StuckDetectorConfig {
            max_iterations: 10,
            iteration_timeout_secs: 60,
            total_timeout_secs: 300,
            stall_limit: 3,
            convergence_threshold: 0.01,
            convergence_window: 3,
        };
        let detector = StuckDetector::new(config);
        let mut state = IterationState::new(100.0);

        // Simulate iterations with no improvement (reach stall_limit)
        for i in 1..=3 {
            state.record_no_improvement(i, 100.0 + (i as f64 * 10.0));
        }

        // Should trigger backoff (stall_limit reached, backoff_count < 2)
        assert!(detector.should_backoff(state.consecutive_no_improvement, state.backoff_count));
        state.apply_backoff();
        assert_eq!(state.consecutive_no_improvement, 0);
        assert_eq!(state.backoff_count, 1);

        // Continue with no improvement (reach stall_limit again)
        for i in 4..=6 {
            state.record_no_improvement(i, 100.0 + (i as f64 * 10.0));
        }

        // Should trigger backoff again
        assert!(detector.should_backoff(state.consecutive_no_improvement, state.backoff_count));
        state.apply_backoff();
        assert_eq!(state.backoff_count, 2);

        // Continue with no improvement (reach stall_limit a third time)
        for i in 7..=9 {
            state.record_no_improvement(i, 100.0 + (i as f64 * 10.0));
        }

        // Should now detect stall (stall_limit reached AND backoff_count >= 2)
        assert_eq!(
            detector.check_stall_limit(state.consecutive_no_improvement, state.backoff_count),
            Some(StuckReason::StallLimitReached)
        );
    }

    #[test]
    fn test_convergence_detection_workflow() {
        let config = StuckDetectorConfig {
            convergence_threshold: 0.02,
            convergence_window: 3,
            ..Default::default()
        };
        let detector = StuckDetector::new(config);
        let mut state = IterationState::new(100.0);

        // Simulate converging iterations
        state.record_improvement(1, 95.0);
        state.record_improvement(2, 94.8);
        state.record_improvement(3, 94.9);

        // Should detect convergence
        assert_eq!(
            detector.check_convergence(&state.recent_metrics),
            Some(StuckReason::ConvergenceAchieved)
        );
    }

    #[test]
    fn test_timeout_detection_workflow() {
        let config = StuckDetectorConfig {
            iteration_timeout_secs: 60,
            total_timeout_secs: 300,
            ..Default::default()
        };
        let detector = StuckDetector::new(config);

        // Test iteration timeout
        let iteration_elapsed = Duration::from_secs(120);
        assert_eq!(
            detector.check_iteration_timeout(iteration_elapsed),
            Some(StuckReason::IterationTimeout)
        );

        // Test total timeout
        let total_elapsed = Duration::from_secs(400);
        assert_eq!(
            detector.check_total_timeout(total_elapsed),
            Some(StuckReason::TotalTimeout)
        );
    }
}
