#!/bin/bash
# Performance regression checking script for pi-autoresearch
# Usage: ./scripts/check-benchmarks.sh [command] [options]
# Commands: save-baseline, check-regression, compare

set -e

BASELINE_FILE="benches/baseline.json"
DEFAULT_THRESHOLD=0.10  # 10% degradation threshold

show_help() {
    cat << EOF
Usage: $0 [command] [options]

Commands:
  save-baseline              Save current benchmark results as baseline
  check-regression           Check for performance regressions
  compare                    Compare current results with baseline
  update-baseline            Update baseline with current results

Options:
  --threshold <float>        Degradation threshold (default: 0.10 = 10%)
  --strict                   Fail on any regression (threshold = 0)
  --help                     Show this help message

Examples:
  $0 save-baseline
  $0 check-regression --threshold 0.15
  $0 check-regression --strict
  $0 compare

EOF
}

parse_args() {
    THRESHOLD=$DEFAULT_THRESHOLD
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --threshold)
                THRESHOLD="$2"
                shift 2
                ;;
            --strict)
                THRESHOLD=0
                shift
                ;;
            --help)
                show_help
                exit 0
                ;;
            *)
                COMMAND="$1"
                shift
                ;;
        esac
    done
}

save_baseline() {
    echo "Running benchmarks and saving baseline..."
    
    # Run benchmarks and capture output
    local output
    output=$(cargo bench --no-capture 2>&1) || {
        echo "Error: Benchmarks failed to run"
        echo "$output"
        exit 1
    }
    
    # Parse benchmark results
    local results="{"
    local first=true
    
    while IFS= read -r line; do
        if [[ $line =~ ^([a-z_]+)\s+time:\s+\[([0-9.]+)\ ]]; then
            local name="${BASH_REMATCH[1]}"
            local time="${BASH_REMATCH[2]}"
            
            if [ "$first" = true ]; then
                first=false
            else
                results+=","
            fi
            results+="\"$name\":$time"
        fi
    done <<< "$output"
    
    results+="}"
    
    # Save baseline
    echo "$results" | python3 -m json.tool > "$BASELINE_FILE" 2>/dev/null || echo "$results" > "$BASELINE_FILE"
    
    echo "Baseline saved to $BASELINE_FILE"
    echo "Threshold: ${THRESHOLD}%"
    echo ""
    echo "Baseline results:"
    cat "$BASELINE_FILE"
}

check_regression() {
    if [ ! -f "$BASELINE_FILE" ]; then
        echo "Error: No baseline found. Run '$0 save-baseline' first."
        exit 1
    fi
    
    echo "Running benchmarks and checking for regressions..."
    echo "Threshold: ${THRESHOLD}%"
    echo ""
    
    # Run benchmarks and capture output
    local output
    output=$(cargo bench --no-capture 2>&1) || {
        echo "Error: Benchmarks failed to run"
        echo "$output"
        exit 1
    }
    
    # Load baseline
    local baseline
    baseline=$(cat "$BASELINE_FILE")
    
    # Check each benchmark
    local failed=false
    local regressions=()
    
    while IFS= read -r line; do
        if [[ $line =~ ^([a-z_]+)\s+time:\s+\[([0-9.]+)\ ]]; then
            local name="${BASH_REMATCH[1]}"
            local current_time="${BASH_REMATCH[2]}"
            
            # Get baseline time
            local baseline_time
            baseline_time=$(echo "$baseline" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('$name', 0))" 2>/dev/null || echo "0")
            
            if [ "$baseline_time" != "0" ] && [ -n "$baseline_time" ]; then
                # Calculate degradation
                local degradation
                degradation=$(python3 -c "print(($current_time - $baseline_time) / $baseline_time)" 2>/dev/null || echo "0")
                
                # Check if degradation exceeds threshold
                local is_regression
                is_regression=$(python3 -c "print(1 if $degradation > $THRESHOLD else 0)" 2>/dev/null || echo "0")
                
                if [ "$is_regression" = "1" ]; then
                    failed=true
                    local pct
                    pct=$(python3 -c "print(f'{$degradation*100:.1f}%')")
                    regressions+=("REGRESSION: $name degraded by $pct (threshold: ${THRESHOLD}%)")
                else
                    local pct
                    pct=$(python3 -c "print(f'{$degradation*100:.1f}%')")
                    echo "OK: $name changed by $pct"
                fi
            else
                echo "WARNING: No baseline for $name"
            fi
        fi
    done <<< "$output"
    
    echo ""
    if [ "$failed" = true ]; then
        echo "FAILED: Performance regressions detected"
        echo ""
        for reg in "${regressions[@]}"; do
            echo "  - $reg"
        done
        echo ""
        echo "To update baseline, run: $0 update-baseline"
        exit 1
    else
        echo "SUCCESS: No performance regressions detected"
        exit 0
    fi
}

compare_results() {
    if [ ! -f "$BASELINE_FILE" ]; then
        echo "Error: No baseline found. Run '$0 save-baseline' first."
        exit 1
    fi
    
    echo "Comparing current benchmarks with baseline..."
    echo ""
    
    # Run benchmarks and capture output
    local output
    output=$(cargo bench --no-capture 2>&1) || {
        echo "Error: Benchmarks failed to run"
        exit 1
    }
    
    # Load baseline
    local baseline
    baseline=$(cat "$BASELINE_FILE")
    
    echo "Benchmark Results:"
    echo "=================="
    printf "%-35s %15s %15s %15s\n" "Benchmark" "Baseline" "Current" "Change"
    echo "-----------------------------------------------------------------------------"
    
    while IFS= read -r line; do
        if [[ $line =~ ^([a-z_]+)\s+time:\s+\[([0-9.]+)\ ]]; then
            local name="${BASH_REMATCH[1]}"
            local current_time="${BASH_REMATCH[2]}"
            
            # Get baseline time
            local baseline_time
            baseline_time=$(echo "$baseline" | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('$name', 'N/A'))" 2>/dev/null || echo "N/A")
            
            if [ "$baseline_time" != "N/A" ] && [ -n "$baseline_time" ]; then
                local change
                change=$(python3 -c "print(f'{$((float($current_time) - float($baseline_time)) / float($baseline_time) * 100):+.1f}%')" 2>/dev/null || echo "N/A")
                printf "%-35s %15s %15s %15s\n" "$name" "${baseline_time}s" "${current_time}s" "$change"
            else
                printf "%-35s %15s %15s %15s\n" "$name" "N/A" "${current_time}s" "N/A"
            fi
        fi
    done <<< "$output"
    
    echo ""
}

update_baseline() {
    echo "Updating baseline with current results..."
    save_baseline
    echo "Baseline updated successfully."
}

# Main
parse_args "$@"

case ${COMMAND:-help} in
    save-baseline)
        save_baseline
        ;;
    check-regression)
        check_regression
        ;;
    compare)
        compare_results
        ;;
    update-baseline)
        update_baseline
        ;;
    *)
        show_help
        exit 1
        ;;
esac

