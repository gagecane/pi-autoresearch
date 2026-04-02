# Feedback

## Priority 92.1: CLI - Add Export Flags - REVISE

**Issues Found:**

1. **Unused functions in main.rs**: The functions `get_export_format()` (line 985) and `get_export_path()` (line 1009) are defined but never used in main.rs. These generate compiler warnings:
   ```
   warning: function `get_export_format` is never used
   warning: function `get_export_path` is never used
   ```
   
   **Recommendation**: These functions duplicate the functionality already available as methods on `Cli` in cli.rs (`cli.get_export_format()` and `cli.get_export_path()`). Either:
   - Remove these standalone functions from main.rs since Cli methods provide the same functionality
   - Or use them when implementing the actual export logic in subsequent tasks

2. **Duplicate ExportFormat enum**: The `ExportFormat` enum is defined in both `src/cli.rs` (lines 5-30) and `src/main.rs` (lines 16-42). This is code duplication.
   
   **Recommendation**: 
   - Keep `ExportFormat` in `src/cli.rs` (where it belongs as part of the CLI definition)
   - Export it from the lib.rs module
   - Import it in main.rs with `use pi_autoresearch::cli::ExportFormat;`
   - Remove the duplicate definition from main.rs

**Action Required**: Mark task as REVISE after addressing these issues to eliminate code duplication and compiler warnings.

