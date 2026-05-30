#!/bin/bash
# Run Plan 2 (simulation & results) cargo integration tests with verbose output.
# Each group runs the matching subset and filters out cargo boilerplate so only
# the test headers, values, and pass/fail lines are shown.
set -e
cd "$(dirname "$0")/.."

pass=0
fail=0

# Filter cargo's boilerplate noise; keep only meaningful lines:
#   ═══ …        — test section headers (from println!)
#   "  key=val"  — indented test output (from println!) — starts with exactly 2 spaces
#   test … ok / FAILED  — individual test result line
#   test result: ok. N passed  — group summary (only when N > 0)
#
# Excluded: Compiling / Finished / Running / Doc-tests / "running N tests"
filter_output() {
    grep -E \
        "(^═|^  [^ ]|^test [a-z].*\.\.\.|FAILED|^test result: ok\. [1-9])" \
        2>/dev/null || true
}

run_group() {
    local label="$1"
    local filter="$2"
    echo ""
    echo "── $label ──"
    if cargo test -p finplan_mcp -- "$filter" --nocapture 2>&1 | filter_output; then
        pass=$((pass + 1))
    else
        fail=$((fail + 1))
    fi
}

echo "═══════════════════════════════════════════════════"
echo "  Plan 2: Simulation & Results  —  cargo tests"
echo "═══════════════════════════════════════════════════"

run_group "run_simulation (structure)"         "test_run_simulation_basic"
run_group "run_simulation (year count)"        "test_run_simulation_year_count"
run_group "run_simulation (year fields)"       "test_run_simulation_year_fields"
run_group "run_simulation (spouse_age null)"   "test_run_simulation_spouse_age_null"
run_group "run_simulation (spouse_age set)"    "test_run_simulation_spouse_age_appears"
run_group "run_simulation (state cached)"      "test_run_simulation_caches"
run_group "run_simulation (cashflows ≠ 0)"    "test_run_simulation_produces"
run_group "run_simulation (net worth grows)"   "test_run_simulation_net_worth"
run_group "run_simulation (seeds differ)"      "test_run_simulation_different_seeds"
run_group "run_simulation (seed reproducible)" "test_run_simulation_same_seed"
run_group "run_monte_carlo (success rate)"     "test_run_monte_carlo_success_rate"
run_group "run_monte_carlo (percentile keys)"  "test_run_monte_carlo_percentile_runs"
run_group "run_monte_carlo (percentile order)" "test_run_monte_carlo_percentile_ordering"
run_group "run_monte_carlo (state cached)"     "test_run_monte_carlo_stores"
run_group "get_account_snapshot (no-run err)"  "test_get_account_snapshot_requires"
run_group "get_account_snapshot (owner field)" "test_get_account_snapshot_returns"
run_group "get_account_snapshot (real values)" "test_get_account_snapshot_real"
run_group "get_ledger (no-run err)"            "test_get_ledger_requires"
run_group "get_ledger (income filter)"         "test_get_ledger_income"
run_group "get_ledger (expense filter)"        "test_get_ledger_expense"
run_group "get_ledger (year filter)"           "test_get_ledger_year"
run_group "get_ledger (pagination)"            "test_get_ledger_pagination"
run_group "cache invalidation"                 "test_simulation_cache_cleared"

echo ""
echo "═══════════════════════════════════════════════════"
if [ "$fail" -eq 0 ]; then
    echo "  All $((pass)) groups passed ✓"
else
    echo "  $fail group(s) FAILED, $pass passed"
    exit 1
fi
