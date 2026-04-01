# Feedback for Priority 16: DOCS - Create Documentation

## Issues Found

### 1. Installation Instructions Incorrect (docs/README.md)
**Issue**: The installation command `cargo install pi-autoresearch` assumes the package is published to crates.io, but it's not (version 0.1.0, no publish configuration in Cargo.toml).

**Fix Required**: Update installation instructions to build from source:
```markdown
### Installation from Source

```bash
# Clone the repository
git clone <repository-url>
cd pi-autoresearch

# Build the project
cargo build --release

# Install locally
cargo install --path .
```
```

### 2. Missing Build Instructions
**Issue**: No documentation on how to build from source or run development version.

**Fix Required**: Add a "Development" section with:
- How to build from source
- How to run tests (`cargo test`)
- How to run in development mode (`cargo run -- <args>`)

### 3. Session File Format Could Be Clearer (docs/USAGE.md)
**Issue**: The session file format example shows a JSON object but doesn't clearly illustrate the JSONL format (one JSON object per line).

**Fix Required**: Update the example to show multiple records:
```markdown
## Session File Format

Experiments are saved in JSONL format (one JSON object per line):

```json
{"session_id": "uuid-1", "question": "...", "status": "completed", ...}
{"session_id": "uuid-2", "question": "...", "status": "in_progress", ...}
```
```

### 4. Missing Link to Main README (docs/README.md)
**Issue**: docs/README.md doesn't have a link back to the main README.md in the repository root.

**Fix Required**: Add at the top or bottom:
```markdown
> **Note**: This is the documentation for pi-autoresearch. For a quick overview, see the [main README](../README.md).
```

### 5. Clarify Metric Auto-Detection (docs/USAGE.md)
**Issue**: The metric auto-detection section is at the bottom and could be more prominent.

**Fix Required**: Move the "Metric Detection" section higher, perhaps after "Command-Line Options" or create a dedicated "Auto-Detection" section.

## Summary

The documentation is comprehensive and well-structured, but needs the following fixes:
1. ✅ Update installation instructions to build from source
2. ✅ Add development/build instructions
3. ✅ Clarify session file format with multi-line example
4. ✅ Add link to main README
5. ✅ Reorganize metric detection section for better visibility

**Status**: REVISE - Please address the issues above.

