# FinPlan MCP Server — Plan 2: Simulation & Results

> **Branch**: `feature/mcp`
> **Crate**: `crates/finplan_mcp`
> **Depends on**: MCP_SERVER_PLAN1.md (Phase 1–7, substantially complete)
> **Goal**: Extend the MCP server to *execute* simulations and *return results* — transforming it from a scenario builder into a full scenario builder + evaluator. An AI agent will be able to run a simulation, interpret the output, and explain the plan's health to the user without them opening the TUI.

---

## Table of Contents

1. [Motivation](#1-motivation)
2. [New Tools Overview](#2-new-tools-overview)
3. [New Return Data Types](#3-new-return-data-types)
4. [Tool: `run_simulation`](#4-tool-run_simulation)
5. [Tool: `run_monte_carlo`](#5-tool-run_monte_carlo)
6. [Tool: `get_account_snapshot`](#6-tool-get_account_snapshot)
7. [Tool: `get_ledger`](#7-tool-get_ledger)
8. [State Changes](#8-state-changes)
9. [Implementation Details](#9-implementation-details)
10. [Testing Strategy](#10-testing-strategy)
11. [File-by-File Checklist](#11-file-by-file-checklist)
12. [Example Agent Session](#12-example-agent-session)
13. [Future Work](#13-future-work)

---

## 1. Motivation

After Plan 1, an AI agent can:
- Build a complete scenario (portfolio, parameters, events)
- Validate the scenario YAML
- Export the finished YAML file

What it **cannot** do is answer the user's actual question: *"Will my plan work?"*

The Results screen in the TUI (screen `[4]`) provides four views of simulation output:

| TUI Panel | What it shows |
|-----------|--------------|
| Net Worth Chart | Bar chart of net worth by year/age |
| Account Breakdown | Per-account balances for a selected year |
| Yearly Breakdown | Per-year: income, withdrawals, contributions, expenses, taxes, net worth |
| Ledger | Full chronological transaction log |

This plan adds four MCP tools that expose this same data to an AI agent, plus enriches the Monte Carlo return with the most important planning signal: **success rate**.

---

## 2. New Tools Overview

| Tool | Purpose | Maps to TUI action |
|------|---------|-------------------|
| `run_simulation` | Single deterministic simulation run | `[r]` key |
| `run_monte_carlo` | Monte Carlo with aggregate stats + percentile curves | `[m]` key |
| `get_account_snapshot` | Per-account balances for a specific year | Account Breakdown panel |
| `get_ledger` | Filtered transaction ledger for a year/range | Ledger panel |

All four tools operate on the accumulated `ScenarioState` built by the Plan 1 tools. They require at minimum `set_portfolio` and `set_parameters` to have been called first.

---

## 3. New Return Data Types

### 3.1 `YearSummary`

Per-year cash flow and net worth summary. Returned in `run_simulation` and `run_monte_carlo`.

```json
{
  "year": 2045,
  "age": 57,
  "net_worth": 1850000.0,
  "real_net_worth": 890000.0,
  "income": 145000.0,
  "real_income": 69800.0,
  "expenses": 72000.0,
  "real_expenses": 34600.0,
  "withdrawals": 0.0,
  "contributions": 23500.0,
  "taxes": 36200.0
}
```

### 3.2 `SimulationSummary`

Top-level facts derived from a single simulation run.

```json
{
  "final_net_worth": 2450000.0,
  "final_real_net_worth": 1180000.0,
  "duration_years": 40,
  "final_year": 2066,
  "peak_net_worth": 2810000.0,
  "peak_net_worth_year": 2058,
  "ruin_year": null,
  "warnings": [
    {
      "date": "2031-06-01",
      "event": "401k Contribution",
      "message": "Insufficient funds in Checking — effect skipped"
    }
  ]
}
```

- `ruin_year`: the first year `net_worth < 0`, or `null` if the plan stays solvent.
- `peak_net_worth_year`: the year net worth is highest (often just before retirement spending ramps up).
- `warnings`: non-fatal simulation events (skipped effects, iteration limit hits).

### 3.3 `MonteCarloStats` (enriched)

```json
{
  "num_iterations": 500,
  "success_rate": 0.87,
  "mean_final_net_worth": 2100000.0,
  "std_dev_final_net_worth": 680000.0,
  "min_final_net_worth": -120000.0,
  "max_final_net_worth": 5800000.0,
  "percentile_values": [
    { "percentile": 0.05, "net_worth": 420000.0 },
    { "percentile": 0.50, "net_worth": 1950000.0 },
    { "percentile": 0.95, "net_worth": 4100000.0 }
  ],
  "converged": true
}
```

### 3.4 `AccountSnapshotResult`

Per-account balances for a specific year.

```json
{
  "year": 2045,
  "net_worth": 1043000.0,
  "accounts": [
    { "name": "Checking", "type": "Checking", "value": 28000.0 },
    {
      "name": "401k", "type": "Traditional401k", "value": 890000.0,
      "assets": [{ "ticker": "FXAIX", "value": 890000.0 }]
    },
    { "name": "Roth IRA", "type": "RothIRA", "value": 310000.0 },
    { "name": "Mortgage", "type": "Mortgage", "value": -185000.0 }
  ]
}
```

### 3.5 `LedgerResult`

Filtered ledger entries for a year or date range.

```json
{
  "filter": "income",
  "year": 2045,
  "total_entries": 28,
  "entries": [
    {
      "date": "2045-01-15",
      "event": "Salary",
      "description": "Income → Checking",
      "amount": 5769.23,
      "account": "Checking",
      "category": "income"
    }
  ]
}
```

---

## 4. Tool: `run_simulation`

Execute a single deterministic simulation on the current scenario state and store the result for use by `get_account_snapshot` and `get_ledger`.

### Input Schema

```json
{
  "type": "object",
  "properties": {
    "returns_mode": {
      "type": "string",
      "enum": ["historical", "parametric"],
      "description": "Override the returns mode from parameters (optional)"
    }
  }
}
```

All inputs are optional — if omitted, the scenario's configured parameters are used.

### Output

Returns `SimulationSummary` + `YearSummary[]`:

```json
{
  "summary": {
    "final_net_worth": 2450000.0,
    "final_real_net_worth": 1180000.0,
    "duration_years": 40,
    "final_year": 2066,
    "peak_net_worth": 2810000.0,
    "peak_net_worth_year": 2058,
    "ruin_year": null,
    "warnings": []
  },
  "years": [
    { "year": 2026, "age": 38, "net_worth": 520000.0, ... },
    ...
  ]
}
```

### Implementation Notes

1. Call `state.merge()` to get `SimulationData`.
2. Call `finplan::data::convert::to_simulation_config(&sim_data)` to get a `SimulationConfig`.
3. Call `finplan_core::simulation::simulate(&config)` to get a core `SimulationResult`.
4. Derive `YearSummary` rows from `result.yearly_cash_flows` and `result.wealth_snapshots`.
5. Derive `SimulationSummary` from the year series (find ruin year, peak, etc.).
6. Store the full core `SimulationResult` in `ScenarioState.last_simulation_result` for use by `get_account_snapshot` and `get_ledger`.

```rust
// Pseudocode
pub fn run_simulation(args: Map<String, Value>, state: &SharedState)
    -> Result<CallToolResult, McpError>
{
    let sim_data = state.lock().unwrap().merge()?;
    let config = to_simulation_config(&sim_data)?;
    let result = simulate(&config);
    let years = build_year_summaries(&result, &sim_data);
    let summary = build_simulation_summary(&result, &years);

    state.lock().unwrap().last_simulation_result = Some(result);
    state.lock().unwrap().last_sim_data = Some(sim_data);

    let output = json!({ "summary": summary, "years": years });
    text_result(serde_json::to_string_pretty(&output).unwrap())
}
```

---

## 5. Tool: `run_monte_carlo`

Run Monte Carlo simulation on the current scenario state.

### Input Schema

```json
{
  "type": "object",
  "properties": {
    "iterations": {
      "type": "integer",
      "description": "Number of MC iterations (default: 500, max: 2000)"
    },
    "percentiles": {
      "type": "array",
      "items": { "type": "number" },
      "description": "Percentiles to compute yearly curves for (default: [0.05, 0.50, 0.95])"
    },
    "seed": {
      "type": "integer",
      "description": "Optional random seed for reproducible results"
    }
  }
}
```

### Output

Returns `MonteCarloStats` + one `YearSummary[]` per requested percentile:

```json
{
  "stats": {
    "num_iterations": 500,
    "success_rate": 0.87,
    "mean_final_net_worth": 2100000.0,
    "std_dev_final_net_worth": 680000.0,
    "min_final_net_worth": -120000.0,
    "max_final_net_worth": 5800000.0,
    "percentile_values": [
      { "percentile": 0.05, "net_worth": 420000.0 },
      { "percentile": 0.50, "net_worth": 1950000.0 },
      { "percentile": 0.95, "net_worth": 4100000.0 }
    ],
    "converged": true
  },
  "percentile_runs": {
    "p5":  { "summary": { "final_net_worth": 420000, "ruin_year": 2058, ... }, "years": [...] },
    "p50": { "summary": { "final_net_worth": 1950000, "ruin_year": null, ... }, "years": [...] },
    "p95": { "summary": { "final_net_worth": 4100000, "ruin_year": null, ... }, "years": [...] }
  }
}
```

### Implementation Notes

1. Build `SimulationConfig` same as `run_simulation`.
2. Build `MonteCarloConfig` from tool arguments (iterations, percentiles, seed).
3. Call `finplan_core::simulation::monte_carlo_simulate_with_config(&config, &mc_config)`.
4. The core returns `MonteCarloSummary` which contains `percentile_runs: Vec<(f64, SimulationResult)>` and `MonteCarloStats`.
5. Build `YearSummary[]` for each requested percentile from its `SimulationResult`.
6. Store the P50 `SimulationResult` in `ScenarioState.last_simulation_result` so that `get_account_snapshot` and `get_ledger` work against the median scenario by default.

### Success Rate Interpretation

Surface the `success_rate` prominently — it is the single most important number in the output. The agent should present it to the user with context:

| Success Rate | Interpretation |
|-------------|----------------|
| ≥ 0.90 | Strong plan — resilient across most market conditions |
| 0.75–0.89 | Acceptable — consider small adjustments (retire later, spend less) |
| 0.50–0.74 | Fragile — significant risk of running short |
| < 0.50 | Plan needs revision |

---

## 6. Tool: `get_account_snapshot`

Return per-account balances for a specific simulation year. Requires `run_simulation` or `run_monte_carlo` to have been called first.

### Input Schema

```json
{
  "type": "object",
  "required": ["year"],
  "properties": {
    "year": {
      "type": "integer",
      "description": "Calendar year to inspect (e.g., 2045)"
    },
    "real": {
      "type": "boolean",
      "description": "Return inflation-adjusted (real) values (default: false)"
    }
  }
}
```

### Output

Returns `AccountSnapshotResult`:

```json
{
  "year": 2045,
  "net_worth": 1043000.0,
  "inflation_factor": 2.08,
  "accounts": [
    { "name": "Checking", "type": "Checking", "value": 28000.0 },
    {
      "name": "401k",
      "type": "Traditional401k",
      "value": 890000.0,
      "assets": [{ "ticker": "FXAIX", "value": 890000.0 }]
    },
    { "name": "Roth IRA", "type": "RothIRA", "value": 310000.0 },
    { "name": "Mortgage", "type": "Mortgage", "value": -185000.0 }
  ]
}
```

### Implementation Notes

1. Look up `state.last_simulation_result` — error if not present.
2. Find the last `WealthSnapshot` whose `date.year() == year` in `result.wealth_snapshots`.
3. For each `AccountSnapshot` in the snapshot, resolve the account name from the scenario's portfolio account list (by index → `AccountId`).
4. For investment accounts, include the per-asset breakdown from `AccountSnapshotFlavor::Investment { assets }`.
5. If `real == true`, divide all values by `result.cumulative_inflation[year_index]`.

---

## 7. Tool: `get_ledger`

Return filtered ledger entries from the last simulation run.

### Input Schema

```json
{
  "type": "object",
  "properties": {
    "year": {
      "type": "integer",
      "description": "Filter to a specific calendar year (optional — omit for all years)"
    },
    "filter": {
      "type": "string",
      "enum": ["all", "income", "expense", "contribution", "withdrawal", "tax"],
      "description": "Filter by transaction category (default: 'all')"
    },
    "limit": {
      "type": "integer",
      "description": "Maximum number of entries to return (default: 100)"
    },
    "offset": {
      "type": "integer",
      "description": "Skip this many entries before returning (for pagination)"
    }
  }
}
```

### Output

Returns `LedgerResult`:

```json
{
  "filter": "income",
  "year": 2045,
  "total_matching": 28,
  "returned": 28,
  "entries": [
    {
      "date": "2045-01-15",
      "event": "Salary",
      "description": "Income → Checking",
      "amount": 5769.23,
      "account": "Checking",
      "category": "income"
    },
    {
      "date": "2045-02-01",
      "event": "Social Security",
      "description": "SS income → Checking",
      "amount": 2800.00,
      "account": "Checking",
      "category": "income"
    }
  ]
}
```

### Ledger Entry Categories

Map `StateEvent` / `LedgerEntry` kinds to human-readable categories:

| Ledger entry type | Category |
|-------------------|----------|
| `Income` effect | `income` |
| `Expense` effect | `expense` |
| `AssetPurchase` effect | `contribution` |
| `AssetSale` / `Sweep` liquidation | `withdrawal` |
| `CashTransfer` to investment | `contribution` |
| `CashTransfer` from investment | `withdrawal` |
| Federal/state/capital gains tax | `tax` |
| Everything else | `other` |

### Implementation Notes

1. Look up `state.last_simulation_result` — error if not present.
2. Iterate `result.ledger` (Vec<LedgerEntry>).
3. Apply year filter and category filter.
4. Apply `limit` and `offset` for pagination.
5. Resolve account names and event names from the stored `SimulationData`.

---

## 8. State Changes

`ScenarioState` in `src/state.rs` needs two new optional fields to hold simulation results between tool calls:

```rust
pub struct ScenarioState {
    // ... existing fields from Plan 1 ...

    /// Result of the last run_simulation or run_monte_carlo (P50) call.
    /// Used by get_account_snapshot and get_ledger.
    pub last_simulation_result: Option<finplan_core::model::SimulationResult>,

    /// The SimulationData used to produce last_simulation_result.
    /// Needed to resolve account names and event names in get_ledger/get_account_snapshot.
    pub last_sim_data: Option<finplan::data::app_data::SimulationData>,
}
```

Both fields are cleared by `reset_state`. They are also cleared (set to `None`) whenever the scenario changes (i.e., whenever any `set_*` or `add_*` tool is called), so stale results are not returned after the scenario is modified.

### Stale Result Detection

Add a helper to each `set_*` and `add_*` tool handler that clears cached results:

```rust
fn invalidate_simulation_cache(state: &SharedState) {
    let mut st = state.lock().unwrap();
    st.last_simulation_result = None;
    st.last_sim_data = None;
}
```

`get_account_snapshot` and `get_ledger` should return a clear error if called before any simulation:
```
"No simulation results available. Call run_simulation or run_monte_carlo first."
```

---

## 9. Implementation Details

### 9.1 Dependency Access

The simulation functions are in `finplan_core`, which `finplan_mcp` already indirectly accesses through the `finplan` crate dependency. The relevant functions are:

```rust
// Single run
use finplan_core::simulation::simulate;
// simulate(config: &SimulationConfig) -> SimulationResult

// Monte Carlo
use finplan_core::simulation::monte_carlo_simulate_with_config;
// monte_carlo_simulate_with_config(config: &SimulationConfig, mc_config: &MonteCarloConfig) -> MonteCarloSummary

// Config conversion
use finplan::data::convert::to_simulation_config;
// to_simulation_config(data: &SimulationData) -> Result<SimulationConfig, ConvertError>
```

### 9.2 Building `YearSummary` from Core Results

The TUI derives its year rows by iterating `result.yearly_cash_flows` (one `YearlyCashFlowSummary` per year) and cross-referencing `result.wealth_snapshots` for net worth at year end. The same approach applies for MCP:

```rust
fn build_year_summaries(
    result: &SimulationResult,
    sim_data: &SimulationData,
) -> Vec<YearSummary> {
    let birth_year = sim_data.parameters.birth_date.year();

    result.yearly_cash_flows.iter().zip(
        result.cumulative_inflation.iter()
    ).map(|(cf, &inflation)| {
        let snapshot_nw = result.wealth_snapshots.iter()
            .rfind(|s| s.date.year() == cf.year as i32)
            .map(|s| s.accounts.iter().map(|a| a.total_value()).sum::<f64>())
            .unwrap_or(0.0);

        YearSummary {
            year: cf.year as i32,
            age: (cf.year as i32 - birth_year) as u8,
            net_worth: snapshot_nw,
            real_net_worth: snapshot_nw / inflation,
            income: cf.income,
            real_income: cf.income / inflation,
            expenses: cf.expenses,
            real_expenses: cf.expenses / inflation,
            withdrawals: cf.withdrawals,
            contributions: cf.contributions,
            taxes: result.yearly_taxes.iter()
                .find(|t| t.year == cf.year)
                .map(|t| t.total_tax())
                .unwrap_or(0.0),
        }
    }).collect()
}
```

### 9.3 Building `SimulationSummary`

```rust
fn build_simulation_summary(
    result: &SimulationResult,
    years: &[YearSummary],
) -> SimulationSummary {
    let ruin_year = years.iter()
        .find(|y| y.net_worth < 0.0)
        .map(|y| y.year);

    let (peak_nw, peak_year) = years.iter()
        .max_by(|a, b| a.net_worth.partial_cmp(&b.net_worth).unwrap())
        .map(|y| (y.net_worth, y.year))
        .unwrap_or((0.0, 0));

    SimulationSummary {
        final_net_worth: years.last().map(|y| y.net_worth).unwrap_or(0.0),
        final_real_net_worth: years.last().map(|y| y.real_net_worth).unwrap_or(0.0),
        duration_years: years.len(),
        final_year: years.last().map(|y| y.year).unwrap_or(0),
        peak_net_worth: peak_nw,
        peak_net_worth_year: peak_year,
        ruin_year,
        warnings: result.warnings.iter().map(|w| WarningOutput {
            date: w.date.to_string(),
            event: w.event_id.map(|id| id.to_string()),
            message: w.message.clone(),
        }).collect(),
    }
}
```

### 9.4 New File: `src/tools/simulation.rs`

All four tools should live in a new file to keep `events.rs` and other Plan 1 tool files unchanged:

```
crates/finplan_mcp/src/tools/
├── simulation.rs    ← NEW: run_simulation, run_monte_carlo,
│                          get_account_snapshot, get_ledger
```

Register in `src/tools/mod.rs`:
```rust
pub mod simulation;
// Add to list_tools():
tools.extend(simulation::tools());
// Add to call_tool():
"run_simulation"       => simulation::run_simulation(args, state),
"run_monte_carlo"      => simulation::run_monte_carlo(args, state),
"get_account_snapshot" => simulation::get_account_snapshot(args, state),
"get_ledger"           => simulation::get_ledger(args, state),
```

### 9.5 Performance Considerations

- Monte Carlo with 500 iterations typically takes 1–5 seconds on modern hardware for a 40-year simulation. This is within MCP's timeout window.
- Do not run in a background thread — MCP tool calls are synchronous. If performance becomes an issue, suggest the user reduce `iterations`.
- The `simulate()` function is CPU-bound. Consider a brief progress indicator in the return text for Monte Carlo: "Completed 500 iterations."

---

## 10. Testing Strategy

### 10.1 Unit Tests for Helper Functions

Test `build_year_summaries` and `build_simulation_summary` with a known `SimulationResult`:

```rust
#[test]
fn test_ruin_year_detection() {
    let years = vec![
        YearSummary { year: 2040, net_worth: 100000.0, .. },
        YearSummary { year: 2041, net_worth: -5000.0, .. },
    ];
    let summary = build_simulation_summary_from_years(&years);
    assert_eq!(summary.ruin_year, Some(2041));
}

#[test]
fn test_peak_net_worth() {
    // ...
}

#[test]
fn test_year_summaries_real_values() {
    // Verify nominal / inflation_factor == real_value
}
```

### 10.2 Integration Tests

**Test: `run_simulation` produces valid output**

```rust
#[test]
fn test_run_simulation_basic() {
    let state = setup_test_scenario(); // portfolio + parameters + salary + expenses + retirement
    let result = run_simulation(Default::default(), &state).unwrap();
    let json: Value = serde_json::from_str(result_text(&result)).unwrap();
    assert!(json["summary"]["final_net_worth"].as_f64().unwrap() > 0.0);
    assert!(json["years"].as_array().unwrap().len() > 0);
}
```

**Test: `run_monte_carlo` returns success_rate in [0, 1]**

```rust
#[test]
fn test_monte_carlo_success_rate() {
    let state = setup_test_scenario();
    let args = json!({"iterations": 50}); // small for test speed
    let result = run_monte_carlo(as_map(args), &state).unwrap();
    let json: Value = serde_json::from_str(result_text(&result)).unwrap();
    let sr = json["stats"]["success_rate"].as_f64().unwrap();
    assert!((0.0..=1.0).contains(&sr));
}
```

**Test: `get_account_snapshot` requires prior run**

```rust
#[test]
fn test_get_account_snapshot_without_run() {
    let state = new_shared_state();
    let args = json!({"year": 2040});
    let result = get_account_snapshot(as_map(args), &state).unwrap();
    assert_eq!(result.is_error, Some(true));
}
```

**Test: `get_ledger` income filter**

```rust
#[test]
fn test_get_ledger_income_filter() {
    let state = setup_and_run_test_scenario();
    let args = json!({"filter": "income", "limit": 10});
    let result = get_ledger(as_map(args), &state).unwrap();
    let json: Value = serde_json::from_str(result_text(&result)).unwrap();
    let entries = json["entries"].as_array().unwrap();
    for entry in entries {
        assert_eq!(entry["category"].as_str().unwrap(), "income");
    }
}
```

### 10.3 Manual Integration Test Extension

Extend `scripts/test_mcp.sh` with the new tool calls after the existing merge/validate sequence:

```bash
echo '{"jsonrpc":"2.0","id":15,"method":"tools/call","params":{"name":"run_simulation","arguments":{}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":16,"method":"tools/call","params":{"name":"run_monte_carlo","arguments":{"iterations":50}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":17,"method":"tools/call","params":{"name":"get_account_snapshot","arguments":{"year":2035}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":18,"method":"tools/call","params":{"name":"get_ledger","arguments":{"filter":"income","limit":5}}}'
```

---

## 11. File-by-File Checklist

### New Files

- [ ] `crates/finplan_mcp/src/tools/simulation.rs`
  - [ ] `tools()` — register 4 tools
  - [ ] `run_simulation(args, state)` — single run
  - [ ] `run_monte_carlo(args, state)` — MC run
  - [ ] `get_account_snapshot(args, state)` — per-account balances
  - [ ] `get_ledger(args, state)` — filtered ledger
  - [ ] `build_year_summaries(result, sim_data)` — helper
  - [ ] `build_simulation_summary(result, years)` — helper
  - [ ] `build_account_snapshot(result, sim_data, year, real)` — helper
  - [ ] `build_ledger_entries(result, sim_data, year, filter, limit, offset)` — helper

### Modified Files

- [ ] `crates/finplan_mcp/src/tools/mod.rs`
  - [ ] Add `pub mod simulation;`
  - [ ] Add 4 tool entries to `list_tools()`
  - [ ] Add 4 match arms to `call_tool()`

- [ ] `crates/finplan_mcp/src/state.rs`
  - [ ] Add `last_simulation_result: Option<finplan_core::model::SimulationResult>`
  - [ ] Add `last_sim_data: Option<SimulationData>`
  - [ ] Initialize both to `None` in `ScenarioState::default()`
  - [ ] Clear both in `reset_state`
  - [ ] Add `pub fn invalidate_simulation_cache(&mut self)`

- [ ] Each Plan 1 tool that modifies scenario state (set_parameters, set_portfolio, add_account, map_tickers, add_* events):
  - [ ] Call `state.lock().unwrap().invalidate_simulation_cache()` at the end of each successful tool call

### New Test Files

- [ ] `crates/finplan_mcp/tests/simulation_tests.rs`
  - [ ] Unit tests for `build_year_summaries`, `build_simulation_summary`
  - [ ] Integration tests for all 4 new tools

---

## 12. Example Agent Session

This extends the Plan 1 example session (Sarah's Retirement Plan) to include simulation and results retrieval.

### Step 9: Run Monte Carlo

```json
// Agent calls: run_monte_carlo
{ "iterations": 500, "percentiles": [0.05, 0.50, 0.95] }
```

Response:
```json
{
  "stats": {
    "num_iterations": 500,
    "success_rate": 0.84,
    "mean_final_net_worth": 1850000,
    "percentile_values": [
      { "percentile": 0.05, "net_worth": -42000 },
      { "percentile": 0.50, "net_worth": 1620000 },
      { "percentile": 0.95, "net_worth": 4200000 }
    ]
  },
  "percentile_runs": {
    "p5":  { "summary": { "final_net_worth": -42000, "ruin_year": 2071 }, "years": [...] },
    "p50": { "summary": { "final_net_worth": 1620000, "ruin_year": null }, "years": [...] },
    "p95": { "summary": { "final_net_worth": 4200000, "ruin_year": null }, "years": [...] }
  }
}
```

**Agent interpretation for user:**
> "Your plan has an 84% success rate across 500 market simulations. In the median scenario, you end with $1.6M at age 100. In a bad market (bottom 5%), you run out of money in 2071 at age 83 — 16 years before your plan ends. You may want to consider retiring at 62 instead of 60, or reducing post-retirement spending by 5–10%."

### Step 10: Inspect retirement year account snapshot

```json
// Agent calls: get_account_snapshot
{ "year": 2048 }  // Sarah's retirement year (age 60)
```

Response:
```json
{
  "year": 2048,
  "net_worth": 1240000,
  "accounts": [
    { "name": "Chase Checking", "type": "Checking", "value": 18000 },
    { "name": "Ally Savings", "type": "Savings", "value": 62000 },
    { "name": "Fidelity Brokerage", "type": "Brokerage", "value": 310000 },
    { "name": "Company 401k", "type": "Traditional401k", "value": 680000,
      "assets": [{ "ticker": "FXAIX", "value": 680000 }] },
    { "name": "Roth IRA", "type": "RothIRA", "value": 170000 }
  ]
}
```

**Agent interpretation for user:**
> "At retirement in 2048, you'll have about $1.24M total. Most of it ($680k) is in your 401k. Note that early withdrawals from the 401k before age 59½ incur a 10% penalty — the withdrawal strategy is set to 'penalty_aware' which will prioritize the brokerage and Roth accounts first."

---

## 13. Future Work

These are explicitly out of scope for this plan but natural next steps:

1. **Year-by-year success rate** — "What fraction of MC runs are still solvent at age 75/80/85?" This requires tracking ruin years across all MC iterations, not just the percentile runs. Requires a new accumulator in the core.

2. **What-if comparison** — `compare_scenarios(scenario_a_yaml, scenario_b_yaml)` — run both and return a side-by-side summary showing which is better and by how much.

3. **Parameter sensitivity** — `sensitivity_sweep(parameter, min, max, steps)` — vary a single parameter (retirement age, spending level) and show how success rate changes. Wraps the existing `analysis/` sweep infrastructure.

4. **Optimal retirement age finder** — `find_optimal_retirement_age(target_success_rate)` — binary-search the retirement age that achieves a target success rate. Wraps the existing `optimization/` binary search.

5. **CSV/JSON export** — `export_results(format)` — return the full yearly table as CSV or structured JSON for external tools.

6. **Async/streaming MC** — Stream MC progress back to the caller as iterations complete (requires MCP SSE transport, not stdio).
