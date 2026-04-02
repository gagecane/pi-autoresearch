# PRD: Autoresearch CLI Wrapper for PI Coding Agent

## Overview

Build a CLI tool that orchestrates autonomous research experiments using a 3-phase framework inspired by karpathy's autoresearch. The tool wraps the PI coding agent to iteratively explore solutions, measure progress, and converge on optimal results.

---

## Phase 1: Experiment Design & Baseline Verification

### User Story 1.1: Auto-Generate Experiment Design
**As a** researcher  
**I want** the CLI to auto-generate an experiment design from a research question  
**So that** I can quickly start experiments without manual setup

**Acceptance Criteria:**
- Accept research question as CLI argument or stdin
- Analyze codebase context to propose:
  - Hypothesis statement
  - Success metric (quantitative, measurable)
  - Measurement methodology
  - Baseline value (current state)
- Output proposed design to stdout for review
- Support `--auto-approve` flag to skip confirmation

**Example:**
```bash
pi-autoresearch --question "Can we reduce memory usage in the image processing pipeline?"
```

**Output:**
```json
{
  "hypothesis": "Implementing lazy loading and object pooling will reduce memory usage by 30%",
  "metric": "peak_memory_mb",
  "measurement": "Run benchmark suite, capture peak RSS via /proc/self/status",
  "baseline": 512,
  "target_improvement": 0.30
}
```

### User Story 1.2: Baseline Verification
**As a** researcher  
**I want** the baseline metric automatically measured and verified  
**So that** I have a trusted reference point for improvement

**Acceptance Criteria:**
- Execute measurement methodology on current codebase state
- Record baseline value with timestamp and git commit hash
- Validate measurement is reproducible (run 2x, assert <5% variance)
- Store baseline in `autoresearch.jsonl` session file
- Fail fast if baseline cannot be established

---

## Phase 2: Iterative Experiment Loop with Stuck Detection

### User Story 2.1: Iterative Exploration Loop
**As a** researcher  
**I want** the CLI to loop through agent iterations, each proposing and testing changes  
**So that** it can autonomously explore the solution space

**Acceptance Criteria:**
- Each iteration:
  1. Invoke PI agent with current state + metric feedback
  2. Agent proposes changes (code modifications, config updates)
  3. Apply changes in isolated branch
  4. Measure metric
  5. Compare to best-so-far
  6. Keep changes if improved, revert if degraded
- Log each iteration to `autoresearch.jsonl`:
  ```json
  {
    "iteration": 3,
    "timestamp": "2024-01-15T10:30:00Z",
    "agent_action": " Implemented object pooling for image buffers",
    "metric_value": 421,
    "improvement": 0.18,
    "kept": true
  }
  ```
- Support `--max-iterations N` (default: 20)

### User Story 2.2: Multi-Layer Stuck Detection
**As a** researcher  
**I want** configurable guards to detect and handle stuck experiments  
**So that** resources aren't wasted on unproductive paths

**Acceptance Criteria:**

**Layer 1: Per-Iteration Time Limit**
- Configurable via `--iteration-timeout_minutes N` (default: 10)
- Kill agent process if exceeded
- Log as "timeout" failure, trigger retry with different approach

**Layer 2: No-Improvement Guard**
- Configurable via `--stall-limit N` (default: 5)
- Track consecutive iterations without improvement
- On stall limit reached:
  - Back off: reduce step size or change strategy
  - After 2 backoffs, abort experiment as stuck

**Layer 3: Total Runtime Limit**
- Configurable via `--total-timeout_minutes N` (default: 120)
- Hard kill on total experiment runtime
- Save best result before exit

**Layer 4: Confidence-Based Early Stop**
- Track metric variance across last N iterations
- If variance < threshold for M iterations → converged, exit early
- Configurable via `--convergence-threshold` and `--convergence-window`

### User Story 2.3: Live Progress Display
**As a** researcher  
**I want** real-time visibility into experiment progress  
**So that** I can monitor and intervene if needed

**Acceptance Criteria:**
- Inline status widget showing:
  ```
  [AutoResearch] Iter 7/20 | Best: 387MB (-24.5%) | Stall: 2/5 | Time: 12m/120m
  ```
- Update every iteration completion
- Support `--verbose` for per-iteration details
- Support `--quiet` for minimal output (just final result)

---

## Phase 3: Finalization & Merge

### User Story 3.1: Auto-Generate Merge PR
**As a** researcher  
**I want** successful experiments auto-merged with documentation  
**So that** improvements are immediately available

**Acceptance Criteria:**
- If best result beats baseline by target improvement:
  - Create git branch `autoresearch/<timestamp>`
  - Apply best-performing changes
  - Generate commit message with:
    - Metric improvement summary
    - Iteration count and runtime
    - Key changes that drove improvement
  - Create PR (or commit if no remote)
- Example commit message:
  ```
  [autoresearch] Reduce memory usage by 24.5% (512MB → 387MB)
  
  Iterations: 12/20 | Runtime: 47m | Convergence: achieved
  
  Key changes:
  - Implemented object pooling for image buffers (iter 3, -18%)
  - Added lazy loading for large files (iter 7, -6.5%)
  
  Metric: peak_memory_mb | Baseline: 512 | Best: 387
  ```

### User Story 3.2: Failed Experiment Reporting
**As a** researcher  
**I want** clear reporting when experiments fail to meet targets  
**So that** I can learn and adjust

**Acceptance Criteria:**
- If target not met:
  - Do NOT apply changes
  - Generate report showing:
    - Best improvement achieved (even if below target)
    - Where exploration got stuck
    - Recommendations for retry (different metrics, longer timeouts, etc.)
  - Exit code 1 (failure) but with useful output

---

## Configuration & CLI Interface

### User Story 4.1: CLI Arguments
**As a** user  
**I want** a simple CLI interface with sensible defaults  
**So that** I can run experiments with minimal setup

**Acceptance Criteria:**

```bash
pi-autoresearch \
  --question "RESEARCH_QUESTION" \
  --metric "METRIC_NAME" \
  --measure "MEASUREMENT_COMMAND" \
  --baseline BASELINE_VALUE \
  --target-improvement 0.30 \
  --max-iterations 20 \
  --iteration-timeout-minutes 10 \
  --total-timeout-minutes 120 \
  --stall-limit 5 \
  --convergence-threshold 0.01 \
  --convergence-window 3 \
  --auto-approve \
  --verbose
```

### User Story 4.2: Config File Support
**As a** power user  
**I want** to store default configurations  
**So that** I don't repeat flags

**Acceptance Criteria:**
- Read defaults from `~/.config/pi-autoresearch/config.json`
- CLI args override config file
- Support `--config PATH` for project-specific configs

---

## Session Persistence

### User Story 5.1: Session Logging
**As a** researcher  
**I want** all experiments logged to a session file  
**So that** I can review, resume, or analyze later

**Acceptance Criteria:**
- Append to `autoresearch.jsonl` (JSON Lines format)
- Each line = one experiment or iteration
- Support `--resume SESSION_ID` to continue from prior state
- Support `--history` to list prior experiments

---

## Integration Hooks

### User Story 6.1: Beads (bd) Integration
**As a** user of beads task tracking  
**I want** experiments to create/update beads issues  
**So that** work is tracked in my workflow

**Acceptance Criteria:**
- On experiment start: create bead issue with design
- On each iteration: update issue with progress
- On completion: close issue with results
- Configurable via `--beads-enabled`

**Acceptance Criteria:**
- Support reading task from `tasks/*.md` files
- Update task status on completion

---

## Technical Architecture

### Components

```
┌─────────────────────────────────────────────────┐
│              CLI Interface                      │
│  (arg parsing, config loading, output formatting)│
└──────────────────┬──────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────┐
│           Experiment Orchestrator               │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────┐│
│  │   Phase 1   │→ │   Phase 2    │→ │ Phase 3 ││
│  │  (Design)   │  │  (Iterate)   │  │ (Merge) ││
│  └─────────────┘  └──────┬───────┘  └─────────┘│
│                         │                      │
│                   ┌─────▼──────┐               │
│                   │ Stuck      │               │
│                   │ Detector   │               │
│                   └────────────┘               │
└──────────────────┬──────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────┐
│              PI Agent Wrapper                   │
│  (invokes pi coding agent with context)          │
└──────────────────┬──────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────┐
│            Metric Evaluator                     │
│  (executes measurement, returns value)           │
└─────────────────────────────────────────────────┘
```

### Key Files

```
pi-autoresearch/
├── src/
│   ├── cli.rs              # CLI interface
│   ├── orchestrator.rs     # Experiment orchestration
│   ├── phase1_design.rs    # Experiment design generation
│   ├── phase2_iterate.rs   # Iterative loop
│   ├── phase3_merge.rs     # Finalization
│   ├── stuck_detector.rs   # Stuck detection logic
│   ├── metric_evaluator.rs # Metric measurement
│   ├── pi_agent.rs         # PI agent wrapper
│   └── session.rs          # Session persistence
├── config/
│   └── default.json        # Default configuration
├── tasks/
│   └── prd-*.md           # PRD files (input)
└── autoresearch.jsonl      # Session log (output)
```

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Time to first experiment design | < 30 seconds |
| Baseline measurement accuracy | < 5% variance |
| Stuck detection accuracy | > 90% (no false positives) |
| Memory improvement on test cases | > 20% |
| User satisfaction (post-experiment survey) | > 4/5 |

---

## Open Questions

1. Should the tool support multiple metrics simultaneously (Pareto optimization)?
2. Should failed iterations be stored for later analysis (even if reverted)?
3. Should there be a "dry run" mode that simulates without applying changes?
4. How to handle non-monotonic metrics (where temporary degradation is acceptable)?

---

## References

- Original autoresearch: https://github.com/karpathy/autoresearch
- Current implementation as a pi extension: https://github.com/davebcn87/pi-autoresearch
- Beads (bd) task tracking: See AGENTS.md
- Ralph-tui: Task orchestration framework: https://github.com/subsy/ralph-tui