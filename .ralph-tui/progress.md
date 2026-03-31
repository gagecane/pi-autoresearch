# Ralph Progress Log

This file tracks progress across iterations. Agents update this file
after each iteration and it's included in prompts for context.

## Codebase Patterns (Study These First)

- **JSON Lines Session Logging**: Use `serde_json::to_string()` + `writeln!` to append records to `.jsonl` files. Each record type (`BaselineRecord`, `IterationRecord`, `ExperimentSession`) serializes independently.
- **Session Resume Pattern**: Store `session_id` in each `ExperimentSession`. Use `find_session_by_id()` to locate prior state, then merge new iterations with offset calculation (`max_old_iter + new_iter`).
- **Untagged Enum for Polymorphic Records**: Use `#[serde(untagged)]` enum (`SessionRecord`) to parse mixed JSONL files containing different record types.

---

## [2026-03-31] - pi-autoresearch-z1z.10
- Verified US-5.1 (Session Logging) was already fully implemented
- Files reviewed: `src/main.rs` (lines 67-69, 369-387, 411-425, 1022-1078, 1080-1122, 1124-1136, 1186-1294)
- **Learnings:**
  - JSON Lines format: `save_to_session_file()` and `log_iteration()` append records to `autoresearch.jsonl`
  - Session types: `BaselineRecord`, `IterationRecord`, `ExperimentSession` all serialize to JSONL
  - Resume functionality: `find_session_by_id()` + resume logic merges old/new iterations
  - History listing: `list_history()` parses JSONL and displays experiment summary
  - CLI flags: `--session-file`, `--resume`, `--history` all implemented
  - Tests pass sequentially (--test-threads=1) - parallel test isolation issue is pre-existing

---

