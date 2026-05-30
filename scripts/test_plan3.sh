#!/bin/bash
# Run Plan 3 (parameter sweep & sensitivity analysis) cargo integration tests
# with verbose output. Each group runs the matching subset and filters out
# cargo boilerplate so only test headers, values, and pass/fail lines are shown.
set -e
cd "$(dirname "$0")/.."

pass=0
fail=0

filter_output() {
    grep -E \
        "(^\[|^═|^  [^ ]|^test [a-z].*\.\.\.|FAILED|^test result: ok\. [1-9])" \
        2>/dev/null || true
}

run_group() {
    local label="$1"
    local filter="$2"
    echo ""
    echo "── $label ──"
    if cargo test -p finplan_mcp --test sweep_integration_test -- "$filter" --nocapture 2>&1 | filter_output; then
        pass=$((pass + 1))
    else
        fail=$((fail + 1))
    fi
}

echo "═══════════════════════════════════════════════════"
echo "  Plan 3: Parameter Sweep & Sensitivity  —  cargo tests"
echo "═══════════════════════════════════════════════════"

run_group "state initial values"                    "test_sweep_state_initial_values"
run_group "configure_sweep (updates state)"         "test_configure_sweep_updates_state"
run_group "configure_sweep (clamps iterations)"     "test_configure_sweep_clamps_iterations"
run_group "add_sweep_parameter (valid)"             "test_add_sweep_parameter_valid"
run_group "add_sweep_parameter (unknown event)"     "test_add_sweep_parameter_unknown_event"
run_group "add_sweep_parameter (inverted range)"    "test_add_sweep_parameter_inverted_range"
run_group "remove_sweep_parameter (by name)"        "test_remove_sweep_parameter_by_name"
run_group "run_sweep (no parameters error)"         "test_run_sweep_fails_without_parameters"
run_group "run_sweep (no portfolio error)"          "test_run_sweep_fails_without_portfolio"
run_group "run_sweep (1D shape)"                    "test_run_sweep_1d_returns_correct_shape"
run_group "run_sweep (stores results)"              "test_run_sweep_stores_results_in_state"
run_group "run_sweep (scenario change clears)"      "test_scenario_change_clears_sweep_results"
run_group "get_sensitivity (no sweep error)"        "test_get_sensitivity_fails_without_sweep"
run_group "get_sensitivity (sorted by impact)"      "test_get_sensitivity_returns_parameters"
run_group "get_sensitivity (unknown metric error)"  "test_get_sensitivity_unknown_metric"
run_group "get_sweep_curve (point count)"           "test_get_sweep_curve_returns_correct"
run_group "get_sweep_curve (threshold crossing)"    "test_get_sweep_curve_with_threshold"
run_group "get_sweep_curve (no sweep error)"        "test_get_sweep_curve_fails_without_sweep"
run_group "get_sweep_grid (1D error)"               "test_get_sweep_grid_fails_for_1d"
run_group "get_sweep_grid (matrix shape)"           "test_get_sweep_grid_returns_correct"
run_group "get_sweep_grid (target zone)"            "test_get_sweep_grid_target_zone"
run_group "get_interaction_matrix (1D error)"       "test_get_interaction_matrix_fails"
run_group "get_interaction_matrix (structure)"      "test_get_interaction_matrix_returns"
run_group "full 2D sweep workflow"                  "test_full_sweep_workflow"

echo ""
echo "═══════════════════════════════════════════════════"
if [ "$fail" -eq 0 ]; then
    echo "  All $((pass)) groups passed ✓"
else
    echo "  $fail group(s) FAILED, $pass passed"
    exit 1
fi
