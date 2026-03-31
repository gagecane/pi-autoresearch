# Ralph Progress Log

This file tracks progress across iterations. Agents update this file
after each iteration and it's included in prompts for context.

## Codebase Patterns (Study These First)

*Add reusable patterns discovered during development here.*

---

## Codebase Patterns (Study These First)

**Stuck Detection Pattern**: Multi-layer guards with escalating severity:
- Layer 1 (fastest): Per-iteration timeout - kills individual slow iterations
- Layer 2 (medium): Stall limit with backoff - detects unproductive exploration, allows 2 recovery attempts
- Layer 3 (slower): Total runtime limit - hard cap on experiment duration
- Layer 4 (smartest): Convergence detection - exits early when metric stabilizes

**State Tracking Pattern**: Use `IterationState` struct to track all mutable experiment state in one place, passed through iterations for clean separation of concerns.

**Config File Pattern**: Load config from `~/.config/<app>/config.json` with `--config PATH` override. Use `Option<T>` fields in Config struct and merge with CLI args where CLI takes precedence (use `.or()` for Option types, `||` for bools).

---

## [2026-03-30] - pi-autoresearch-z1z.3
- US-2.1: Iterative Exploration Loop already implemented
- Files: `src/main.rs` (lines 350-536)
- Core functions: `run_iteration()`, `run_iterative_loop()`, `invoke_pi_agent()`
- Features:
  - Invokes PI agent with question, current state, and metric feedback
  - Applies changes in isolated branches (`autoresearch/iter-{uuid}`)
  - Measures metric and compares to best-so-far
  - Keeps changes if improved, reverts if degraded
  - Logs each iteration to `autoresearch.jsonl` with iteration number, timestamp, agent action, metric value, improvement, kept flag
  - Supports `--max-iterations N` (default: 20)
  - Live progress display: `[AutoResearch] Iter X/Y | Best: Z ({:+.1}%) | Stall: A/B | Time: C`
- **Learnings:**
  - Pattern: Iteration state tracked via `IterationState` struct with current iteration, best metric, best iteration number, and consecutive no-improvement counter
  - Pattern: Each iteration returns `(metric_value, agent_action, kept)` tuple for clean separation
  - Gotcha: Improvement calculated as `(best_metric - metric_value) / baseline_value` to get relative improvement percentage
---

## [2026-03-30] - pi-autoresearch-z1z.4
- US-2.2: Multi-Layer Stuck Detection implemented
- Files: `src/main.rs` (modified iteration loop, added `StuckReason` enum, extended `IterationState`)
- Features implemented:
  - Layer 1: Per-iteration timeout (`--iteration-timeout-minutes`, default 10)
  - Layer 2: Stall limit with backoff (`--stall-limit`, default 5, aborts after 2 backoffs)
  - Layer 3: Total runtime limit (`--total-timeout-minutes`, default 120)
  - Layer 4: Convergence-based early stop (`--convergence-threshold` default 0.01, `--convergence-window` default 3)
- Changes:
  - Added `StuckReason` enum for tracking termination cause
  - Extended `IterationState` with `backoff_count` and `recent_metrics` fields
  - Modified `run_iterative_loop()` to return `(ExperimentSession, Option<StuckReason>)`
  - Enhanced progress display to show backoff count and total timeout
- **Learnings:**
  - Pattern: Stuck detection layers ordered by responsiveness - fastest checks first
  - Pattern: Backoff strategy allows recovery from temporary stalls before giving up
  - Gotcha: Convergence detection requires tracking recent metrics in a sliding window
---

## [2026-03-30] - pi-autoresearch-z1z.5
- US-2.3: Live Progress Display implemented
- Files: `src/main.rs` (added `--verbose` and `--quiet` CLI flags, updated progress display logic)
- Features implemented:
  - Inline status widget already present from US-2.1: `[AutoResearch] Iter X/Y | Best: Z ({:+.1}%) | Stall: A/B | Time: C`
  - Updated every iteration completion
  - Added `--verbose` flag for per-iteration details (metric value, agent action summary)
  - Added `--quiet` flag for minimal output (only final result)
- Changes:
  - Added `verbose` and `quiet` fields to `Cli` struct
  - Wrapped all `eprintln!` calls in iterative loop with `!cli.quiet` checks
  - Added conditional verbose output after each iteration
- **Learnings:**
  - Pattern: Progress display was already implemented in US-2.1, only verbosity control needed
  - Gotcha: Using `cli.quiet` directly instead of scoped variable to avoid closure capture issues
---

## [2026-03-31] - pi-autoresearch-z1z.9
- US-4.2: Config File Support implemented
- Files: `src/main.rs` (added `Config` struct, `load_config()`, `merge_config()`, `--config` CLI flag)
- Features implemented:
  - Read defaults from `~/.config/pi-autoresearch/config.json`
  - CLI args override config file values
  - Support `--config PATH` for project-specific configs
- Changes:
  - Added `Config` struct with `Option<T>` fields matching CLI options
  - Added `load_config()` function with tilde expansion for default path
  - Added `merge_config()` function to merge CLI with config (CLI takes precedence)
  - Added `--config` CLI flag
  - Modified `run()` to load and merge config before processing
- **Learnings:**
  - Pattern: Use `Option<T>` in Config struct and `.or()` to merge with CLI values
  - Pattern: For boolean flags, use `cli_flag || config_flag.unwrap_or(false)`
  - Gotcha: Default values need special handling (e.g., `session_file == "autoresearch.jsonl"` check)
- Quality checks: cargo check passes, cargo test passes (19/19)
---

