# Ralph Progress Log

This file tracks progress across iterations. Agents update this file
after each iteration and it's included in prompts for context.

## Codebase Patterns (Study These First)

- **Failure Reporting Pattern**: When target not met, use `FailureReport` struct with `best_improvement`, `stuck_reason`, and `recommendations` fields. Generate recommendations via `generate_failure_recommendations()` based on stuck reason type.
- **Exit Code Convention**: Return `std::process::exit(1)` for failed experiments, but always print useful failure report first via `print_failure_report()`.
- **No-Apply-on-Failure**: The `finalize_experiment()` function creates `FinalizationResult` with `success: false` and `failure_report: Some(...)` when target not met, preventing any git branch/commit operations.

---

## [2026-03-31] - pi-autoresearch-z1z.7
- Verified US-3.2 (Failed Experiment Reporting) was already fully implemented
- Files reviewed: `src/main.rs` (lines 669-733, 735-782, 996-1018, 1138-1181)
- **Learnings:**
  - Feature complete: `FailureReport` struct, `generate_failure_recommendations()`, `print_failure_report()`
  - Exit code 1 returned when target not met (line 1461)
  - No changes applied to codebase on failure (git branch creation skipped)
  - Recommendations are contextual based on `StuckReason` enum variant

---

