# Ralph Progress Log

This file tracks progress across iterations. Agents update this file
after each iteration and it's included in prompts for context.

## Codebase Patterns (Study These First)

- **JSON Lines Session Logging**: Use `serde_json::to_string()` + `writeln!` to append records to `.jsonl` files. Each record type (`BaselineRecord`, `IterationRecord`, `ExperimentSession`) serializes independently.
- **Session Resume Pattern**: Store `session_id` in each `ExperimentSession`. Use `find_session_by_id()` to locate prior state, then merge new iterations with offset calculation (`max_old_iter + new_iter`).
- **Untagged Enum for Polymorphic Records**: Use `#[serde(untagged)]` enum (`SessionRecord`) to parse mixed JSONL files containing different record types.
- **External CLI Integration**: Wrap external CLI tools (like `bd`) using `std::process::Command::new().args([...]).output()`. Parse stdout to extract IDs from `"Created <id>"` output format. Always check `output.status.success()` before processing.
- **Ralph-TUI Integration Pattern**: Create integration struct with `enabled` flag and task file path. Output pipe-delimited status (`RALPH_STATUS|key=value`) for parsing, write status to `.ralph-tui/status.json`, and mark task complete/failed on experiment end.

---

## [2026-03-31] - pi-autoresearch-z1z.10
- Verified US-5.1 (Session Logging) was already fully implemented
- Files reviewed: `src/main.rs` (lines 67-69, 369-387, 411-425, 1080-1122, 1124-1136, 1186-1294)
- **Learnings:**
  - JSON Lines format: `save_to_session_file()` and `log_iteration()` append records to `autoresearch.jsonl`
  - Session types: `BaselineRecord`, `IterationRecord`, `ExperimentSession` all serialize to JSONL
  - Resume functionality: `find_session_by_id()` + resume logic merges old/new iterations
  - History listing: `list_history()` parses JSONL and displays experiment summary
  - CLI flags: `--session-file`, `--resume`, `--history` all implemented
  - Tests pass sequentially (--test-threads=1) - parallel test isolation issue is pre-existing

---

## [2026-03-31] - pi-autoresearch-z1z.11
- Implemented US-6.1 (Beads/bd Integration)
- Files changed: `src/main.rs`
- Added `BeadsIntegration` struct with methods: `create_experiment_bead()`, `update_bead_progress()`, `close_bead()`
- Added `--beads-enabled` CLI flag
- Integration points:
  - Create bead on experiment start with design details
  - Update bead with `bd note` on each iteration
  - Close bead with `bd close --reason` on completion
- **Learnings:**
  - `std::process::Command` for external CLI integration
  - Parsing `bd create` output to extract bead ID from "Created <id>" format
  - Using `Option<BeadsIntegration>` to pass through `run_iterative_loop`
  - Tests pass sequentially (--test-threads=1)

---

## [2026-03-31] - pi-autoresearch-z1z.12
- Implemented US-6.2 (Ralph-TUI Compatibility)
- Files changed: `src/main.rs`, `tasks/example-task.md` (sample)
- Added `RalphTuiIntegration` struct with methods:
  - `read_task_from_file()` - reads task description from `tasks/*.md` files
  - `output_ralph_status()` - outputs ralph-tui compatible status format
  - `update_task_status()` - writes status to `.ralph-tui/status.json`
  - `mark_task_complete()` - marks task as completed/failed on experiment end
- Added CLI flags: `--ralph-tui-enabled`, `--ralph-task-file`
- Integration points:
  - Reads research question from task file if `--question` not provided
  - Outputs `RALPH_STATUS|...` format on each iteration for ralph-tui parsing
  - Writes status.json to `.ralph-tui/` directory for status tracking
  - Marks task complete/failed on experiment completion
- **Learnings:**
  - Task ID extracted from filename (e.g., `tasks/prd-123.md` → `prd-123`)
  - Status output uses pipe-delimited format for easy parsing
  - Status file stored in `.ralph-tui/status.json` for ralph-tui to read
  - Follows same pattern as `BeadsIntegration` for consistency
  - Tests pass sequentially (--test-threads=1)

---


