# Feedback

## Priority 12: Experiment Comparison - Runtime Calculation Incomplete

**Issue**: The `calculate_session_runtime()` function is a placeholder that always returns 0.0.

**Current Implementation**:
```rust
fn calculate_session_runtime(session: &ExperimentSession) -> f64 {
    if session.end_time.is_some() {
        // Simple approximation: parse timestamps and calculate difference
        // For now, return a placeholder value
        0.0
    } else {
        0.0
    }
}
```

**Expected Behavior**:
- Parse `start_time` and `end_time` from the session
- Calculate the difference in seconds
- Return the actual runtime value

**Suggested Fix**:
Use chrono to parse the timestamps (which are already in ISO format) and calculate the difference:
```rust
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
```

**Task**: Mark as REVISE and implement proper runtime calculation.

