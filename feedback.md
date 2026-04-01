# Feedback Log

## Priority 23: TEST - Add Integration Test for Beads Flag
**Status**: REVISE
**Feedback**:

1. **Test Quality Issue in `test_beads_enabled_flag_recognized`**:
   - The assertion `assert!(stdout.contains("beads-enabled") || output.status.success())` is redundant
   - Since `output.status.success()` is already checked in the previous assertion, the `||` condition will always be true
   - This means the test doesn't actually verify that "beads-enabled" appears in the help output
   - **Fix**: Change to two separate assertions:
     ```rust
     assert!(output.status.success());
     let stdout = String::from_utf8_lossy(&output.stdout);
     assert!(stdout.contains("beads-enabled"), "Flag should be documented in help text");
     ```

2. **Consider Adding More Comprehensive Test**:
   - The current test only verifies the flag is recognized (doesn't cause CLI error)
   - Consider adding a test that verifies the `--beads-enabled` flag actually enables beads functionality
   - This could check that the `cli.beads_enabled` field is set to true when the flag is provided

**Action Required**: Fix the redundant assertion in `test_beads_enabled_flag_recognized` test

*No other active feedback - all previous feedback has been addressed*

