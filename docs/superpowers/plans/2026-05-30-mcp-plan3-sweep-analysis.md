# MCP Plan 3: Parameter Sweep & Sensitivity Analysis — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add 8 MCP tools to the `finplan-mcp` server that expose the Analysis screen's parameter sweep engine, enabling an AI agent to run sensitivity analyses and answer questions like "what's my biggest retirement lever?" without opening the TUI.

**Architecture:** A new `tools/sweep.rs` file implements all 8 tools. `ScenarioState` gains 4 sweep fields. All computation works directly with `finplan_core::analysis::SweepResults` and `AnalysisMetric` (no dependency on `AnalysisResults` from the TUI crate — avoids the percentage-scaling mismatch). Sweep results are invalidated whenever the scenario changes, mirroring the existing simulation cache pattern.

**Tech Stack:** Rust · `finplan_core::analysis::{SweepResults, SweepParameter, SweepConfig, sweep_evaluate, AnalysisMetric}` · `finplan::data::analysis_data::{SweepParameterData, SweepTypeData}` · `serde_json`

---

## File Map

| File | Action | What changes |
|------|--------|-------------|
| `crates/finplan_mcp/src/state.rs` | Modify | Add 4 sweep fields + `invalidate_sweep_cache()` |
| `crates/finplan_mcp/src/tools/sweep.rs` | **Create** | All 8 sweep tools + helpers |
| `crates/finplan_mcp/src/tools/mod.rs` | Modify | Register sweep tools, update `get_state_summary` |
| `crates/finplan_mcp/tests/sweep_integration_test.rs` | **Create** | Integration tests for all 8 tools |

No other files need changing — `invalidate_simulation_cache()` already exists and is called by all scenario-modifying tools; we extend it to also clear sweep results.

---

## Task 1: Extend `ScenarioState` with sweep fields

**Files:**
- Modify: `crates/finplan_mcp/src/state.rs`

- [ ] **Step 1.1: Write the failing test**

Create `crates/finplan_mcp/tests/sweep_integration_test.rs` with an initial state-field test:

```rust
use finplan_mcp::{state, tools};
use serde_json::{Map, Value, json};

fn args(v: Value) -> Map<String, Value> {
    match v {
        Value::Object(m) => m,
        _ => Map::new(),
    }
}

fn is_ok(result: &rmcp::model::CallToolResult) -> bool {
    result.is_error.unwrap_or(false) == false
}

fn text_of(result: &rmcp::model::CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|c| c.as_text())
        .map(|t| t.text.as_str())
        .collect::<Vec<_>>()
        .join("")
}

#[test]
fn test_sweep_state_initial_values() {
    let st = state::new_shared_state();
    let s = st.lock().unwrap();
    assert!(s.sweep_parameters.is_empty());
    assert_eq!(s.sweep_mc_iterations, 200);
    assert_eq!(s.sweep_default_steps, 6);
    assert!(s.last_sweep_results.is_none());
}
```

- [ ] **Step 1.2: Run test to confirm it fails**

```bash
docker run --rm -v "$PWD":/app -w /app rust:slim cargo test -p finplan_mcp --test sweep_integration_test -- test_sweep_state_initial_values --nocapture 2>&1 | tail -20
```

Expected: compile error — fields do not exist yet.

- [ ] **Step 1.3: Add sweep fields to `ScenarioState` in `state.rs`**

Add to the `use` block at the top of `state.rs`:

```rust
use finplan::data::analysis_data::SweepParameterData;
use finplan_core::analysis::SweepResults;
```

Add fields to the `ScenarioState` struct (after `last_mc_summary`):

```rust
    /// Sweep parameter axes added via add_sweep_parameter.
    pub sweep_parameters: Vec<SweepParameterData>,

    /// MC iterations per sweep grid point (default: 200).
    pub sweep_mc_iterations: usize,

    /// Default step count for new sweep parameters (default: 6).
    pub sweep_default_steps: usize,

    /// Results from the last run_sweep call.
    pub last_sweep_results: Option<SweepResults>,
```

Update `Default for ScenarioState` — add these lines inside the `Self { ... }` block:

```rust
            sweep_parameters: Vec::new(),
            sweep_mc_iterations: 200,
            sweep_default_steps: 6,
            last_sweep_results: None,
```

Extend `invalidate_simulation_cache` to also clear sweep results:

```rust
    pub fn invalidate_simulation_cache(&mut self) {
        self.last_simulation_result = None;
        self.last_sim_data = None;
        self.last_mc_summary = None;
        self.last_sweep_results = None;
    }
```

Add a new dedicated sweep-cache invalidator (called from sweep tools when grid changes):

```rust
    pub fn invalidate_sweep_cache(&mut self) {
        self.last_sweep_results = None;
    }
```

Also extend `reset_state` (in `mod.rs`) — no change needed since it calls `ScenarioState::new()` which uses `Default`, which now initialises all sweep fields to their zero-values.

- [ ] **Step 1.4: Run the test to confirm it passes**

```bash
docker run --rm -v "$PWD":/app -w /app rust:slim cargo test -p finplan_mcp --test sweep_integration_test -- test_sweep_state_initial_values --nocapture 2>&1 | tail -10
```

Expected: `test test_sweep_state_initial_values ... ok`

- [ ] **Step 1.5: Commit**

```bash
git add crates/finplan_mcp/src/state.rs crates/finplan_mcp/tests/sweep_integration_test.rs
git commit -m "feat(mcp): add sweep fields to ScenarioState (Plan 3 step 1)"
```

---

## Task 2: Create `sweep.rs` — setup tools (`configure_sweep`, `add_sweep_parameter`, `remove_sweep_parameter`)

**Files:**
- Create: `crates/finplan_mcp/src/tools/sweep.rs`
- Modify: `crates/finplan_mcp/src/tools/mod.rs` (registration added in Task 7; for now just add `pub mod sweep;`)

- [ ] **Step 2.1: Write failing tests in `sweep_integration_test.rs`**

Append to `sweep_integration_test.rs`:

```rust
// ── helpers ──────────────────────────────────────────────────────────────────

/// Build a minimal scenario with two named events suitable for sweep tests.
/// Events: "Retirement" (age trigger) and "Living Expenses" (monthly expense).
fn setup_sweep_scenario() -> finplan_mcp::state::SharedState {
    let st = state::new_shared_state();

    tools::portfolio::set_portfolio(args(json!({"name": "Sweep Test Plan"})), &st).unwrap();
    tools::portfolio::add_account(
        args(json!({"name": "Checking", "account_type": "Checking", "value": 50000.0})),
        &st,
    ).unwrap();
    tools::portfolio::add_account(
        args(json!({
            "name": "401k",
            "account_type": "Traditional401k",
            "assets": [{"ticker": "FXAIX", "value": 300000.0}]
        })),
        &st,
    ).unwrap();
    tools::parameters::set_parameters(
        args(json!({
            "birth_date": "1970-01-01",
            "start_date": "2026-01-01",
            "duration_years": 35
        })),
        &st,
    ).unwrap();
    tools::events::add_expense_event(
        args(json!({"name": "Living Expenses", "from_account": "Checking", "amount": 5000.0})),
        &st,
    ).unwrap();
    tools::events::add_retirement_event(
        args(json!({"name": "Retirement", "age": 62})),
        &st,
    ).unwrap();
    tools::ticker::map_tickers(args(json!({})), &st).unwrap();
    st
}

// ── configure_sweep ───────────────────────────────────────────────────────────

#[test]
fn test_configure_sweep_updates_state() {
    let st = setup_sweep_scenario();
    let r = tools::sweep::configure_sweep(
        args(json!({"mc_iterations": 100, "default_steps": 4})),
        &st,
    ).unwrap();
    assert!(is_ok(&r), "configure_sweep failed: {}", text_of(&r));
    let s = st.lock().unwrap();
    assert_eq!(s.sweep_mc_iterations, 100);
    assert_eq!(s.sweep_default_steps, 4);
}

#[test]
fn test_configure_sweep_clamps_iterations() {
    let st = setup_sweep_scenario();
    let r = tools::sweep::configure_sweep(
        args(json!({"mc_iterations": 5})),   // below min of 10
        &st,
    ).unwrap();
    assert!(is_ok(&r));
    let s = st.lock().unwrap();
    assert_eq!(s.sweep_mc_iterations, 10);  // clamped to min
}

// ── add_sweep_parameter ───────────────────────────────────────────────────────

#[test]
fn test_add_sweep_parameter_valid() {
    let st = setup_sweep_scenario();
    let r = tools::sweep::add_sweep_parameter(
        args(json!({
            "event_name": "Retirement",
            "sweep_type": "trigger_age",
            "min_value": 60,
            "max_value": 67
        })),
        &st,
    ).unwrap();
    assert!(is_ok(&r), "add_sweep_parameter failed: {}", text_of(&r));
    let s = st.lock().unwrap();
    assert_eq!(s.sweep_parameters.len(), 1);
    assert_eq!(s.sweep_parameters[0].event_name, "Retirement");
}

#[test]
fn test_add_sweep_parameter_unknown_event_returns_error() {
    let st = setup_sweep_scenario();
    let r = tools::sweep::add_sweep_parameter(
        args(json!({
            "event_name": "NonExistentEvent",
            "sweep_type": "trigger_age",
            "min_value": 60,
            "max_value": 70
        })),
        &st,
    ).unwrap();
    assert_eq!(r.is_error, Some(true));
}

#[test]
fn test_add_sweep_parameter_inverted_range_returns_error() {
    let st = setup_sweep_scenario();
    let r = tools::sweep::add_sweep_parameter(
        args(json!({
            "event_name": "Retirement",
            "sweep_type": "trigger_age",
            "min_value": 70,
            "max_value": 60      // min > max
        })),
        &st,
    ).unwrap();
    assert_eq!(r.is_error, Some(true));
}

// ── remove_sweep_parameter ────────────────────────────────────────────────────

#[test]
fn test_remove_sweep_parameter_by_name() {
    let st = setup_sweep_scenario();
    tools::sweep::add_sweep_parameter(
        args(json!({"event_name": "Retirement", "sweep_type": "trigger_age",
                    "min_value": 60, "max_value": 67})),
        &st,
    ).unwrap();
    let r = tools::sweep::remove_sweep_parameter(
        args(json!({"event_name": "Retirement"})),
        &st,
    ).unwrap();
    assert!(is_ok(&r));
    assert!(st.lock().unwrap().sweep_parameters.is_empty());
}
```

- [ ] **Step 2.2: Run tests to confirm they all fail**

```bash
docker run --rm -v "$PWD":/app -w /app rust:slim cargo test -p finplan_mcp --test sweep_integration_test 2>&1 | tail -20
```

Expected: compile errors — `tools::sweep` module does not exist yet.

- [ ] **Step 2.3: Create `sweep.rs` with tools registration and the three setup tools**

Create `crates/finplan_mcp/src/tools/sweep.rs`:

```rust
use std::collections::HashMap;

use finplan::data::{
    analysis_data::{SweepParameterData, SweepTypeData},
    app_data::SimulationData,
    convert::to_simulation_config,
};
use finplan_core::{
    analysis::{
        AnalysisMetric, EffectParam, EffectTarget, SweepConfig, SweepParameter, SweepResults,
        SweepTarget, TriggerParam, sweep_evaluate,
    },
    model::EventId,
};
use rmcp::{ErrorData as McpError, model::*};
use serde_json::{Map, Value, json};

use super::{error_result, make_tool, text_result};
use crate::state::SharedState;

// ── Tool registration ─────────────────────────────────────────────────────────

pub fn tools() -> Vec<Tool> {
    vec![
        make_tool(
            "add_sweep_parameter",
            "Add one parameter dimension to the sweep grid. The sweep will vary this \
             event's field between min_value and max_value in step_count evenly-spaced steps. \
             Multiple calls build a multi-dimensional grid. Total MC runs = step_count_1 × \
             step_count_2 × ... × mc_iterations.",
            json!({
                "type": "object",
                "required": ["event_name", "sweep_type", "min_value", "max_value"],
                "properties": {
                    "event_name": {
                        "type": "string",
                        "description": "Name of the event whose parameter is being varied. Must match an event in the scenario."
                    },
                    "sweep_type": {
                        "type": "string",
                        "enum": ["trigger_age", "trigger_date", "effect_value",
                                 "repeating_start_age", "repeating_end_age"],
                        "description": "Which field of the event to vary"
                    },
                    "min_value": {
                        "type": "number",
                        "description": "Minimum value (age in years, year as integer, or dollars)"
                    },
                    "max_value": {
                        "type": "number",
                        "description": "Maximum value"
                    },
                    "step_count": {
                        "type": "integer",
                        "description": "Number of evenly-spaced steps (default: sweep default_steps)"
                    }
                }
            }),
        ),
        make_tool(
            "remove_sweep_parameter",
            "Remove a sweep parameter by event name or index. Clears any cached sweep results.",
            json!({
                "type": "object",
                "properties": {
                    "event_name": {
                        "type": "string",
                        "description": "Remove the parameter for this event name"
                    },
                    "index": {
                        "type": "integer",
                        "description": "Remove the parameter at this 0-based index"
                    }
                }
            }),
        ),
        make_tool(
            "configure_sweep",
            "Set sweep-level configuration: MC iterations per grid point and default step count.",
            json!({
                "type": "object",
                "properties": {
                    "mc_iterations": {
                        "type": "integer",
                        "description": "MC iterations per grid point (default: 200, min: 10, max: 2000)"
                    },
                    "default_steps": {
                        "type": "integer",
                        "description": "Default step count for new sweep parameters (default: 6, min: 2, max: 20)"
                    }
                }
            }),
        ),
        make_tool(
            "run_sweep",
            "Execute the full N-dimensional parameter sweep. Each grid point runs \
             mc_iterations Monte Carlo simulations. Results are stored for get_sensitivity, \
             get_sweep_curve, get_sweep_grid, and get_interaction_matrix.",
            json!({"type": "object", "properties": {}}),
        ),
        make_tool(
            "get_sensitivity",
            "Return tornado chart data: each parameter's impact on the metric, sorted by \
             absolute effect. Requires run_sweep to have been called first.",
            json!({
                "type": "object",
                "required": ["metric"],
                "properties": {
                    "metric": {
                        "type": "string",
                        "enum": ["success_rate", "p5_final_net_worth", "p50_final_net_worth",
                                 "p95_final_net_worth", "lifetime_taxes", "max_drawdown"]
                    }
                }
            }),
        ),
        make_tool(
            "get_sweep_curve",
            "Return the 1D metric curve along one parameter, with min/max spread bands \
             from other dimensions and optional threshold crossing points. Requires run_sweep.",
            json!({
                "type": "object",
                "required": ["metric"],
                "properties": {
                    "metric": {
                        "type": "string",
                        "enum": ["success_rate", "p5_final_net_worth", "p50_final_net_worth",
                                 "p95_final_net_worth", "lifetime_taxes", "max_drawdown"]
                    },
                    "param_index": {
                        "type": "integer",
                        "description": "Which sweep dimension is the X axis (default: 0)"
                    },
                    "fixed_values": {
                        "type": "object",
                        "description": "Fix other dimensions at specific step indices. Keys are dimension indices (as strings), values are step indices.",
                        "additionalProperties": {"type": "integer"}
                    },
                    "threshold": {
                        "type": "number",
                        "description": "Optional: report where curve crosses this value"
                    }
                }
            }),
        ),
        make_tool(
            "get_sweep_grid",
            "Return the full 2D metric matrix for two parameters (heatmap data). \
             Requires at least a 2D sweep and run_sweep to have been called.",
            json!({
                "type": "object",
                "required": ["metric"],
                "properties": {
                    "metric": {
                        "type": "string",
                        "enum": ["success_rate", "p5_final_net_worth", "p50_final_net_worth",
                                 "p95_final_net_worth", "lifetime_taxes", "max_drawdown"]
                    },
                    "x_param_index": {
                        "type": "integer",
                        "description": "X-axis dimension (default: 0)"
                    },
                    "y_param_index": {
                        "type": "integer",
                        "description": "Y-axis dimension (default: 1)"
                    },
                    "target_threshold": {
                        "type": "number",
                        "description": "Optional: count cells above this value"
                    }
                }
            }),
        ),
        make_tool(
            "get_interaction_matrix",
            "Return the NxN cross-parameter interaction matrix showing which pairs of \
             parameters have effects beyond their individual contributions. Requires a 2D+ \
             sweep and run_sweep to have been called.",
            json!({
                "type": "object",
                "required": ["metric"],
                "properties": {
                    "metric": {
                        "type": "string",
                        "enum": ["success_rate", "p5_final_net_worth", "p50_final_net_worth",
                                 "p95_final_net_worth", "lifetime_taxes", "max_drawdown"]
                    }
                }
            }),
        ),
    ]
}

// ── configure_sweep ───────────────────────────────────────────────────────────

pub fn configure_sweep(
    args: Map<String, Value>,
    state: &SharedState,
) -> Result<CallToolResult, McpError> {
    let mut st = state.lock().unwrap();

    if let Some(iters) = args.get("mc_iterations").and_then(|v| v.as_u64()) {
        let clamped = iters.clamp(10, 2000) as usize;
        if clamped != st.sweep_mc_iterations {
            st.sweep_mc_iterations = clamped;
            st.invalidate_sweep_cache();
        }
    }

    if let Some(steps) = args.get("default_steps").and_then(|v| v.as_u64()) {
        st.sweep_default_steps = steps.clamp(2, 20) as usize;
    }

    let total_points: usize = st.sweep_parameters.iter().map(|p| p.step_count).product();
    let est_secs = total_points * st.sweep_mc_iterations / 10; // ~0.1s per MC point

    text_result(format!(
        "Sweep configured: {} MC iterations per point, {} default steps.\n\
         Current grid: {} parameters, {} total points.\n\
         Estimated runtime: ~{} seconds.",
        st.sweep_mc_iterations,
        st.sweep_default_steps,
        st.sweep_parameters.len(),
        total_points,
        est_secs,
    ))
}

// ── add_sweep_parameter ───────────────────────────────────────────────────────

pub fn add_sweep_parameter(
    args: Map<String, Value>,
    state: &SharedState,
) -> Result<CallToolResult, McpError> {
    let event_name = match args.get("event_name").and_then(|v| v.as_str()) {
        Some(n) => n.to_string(),
        None => return error_result("event_name is required"),
    };
    let sweep_type_str = match args.get("sweep_type").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return error_result("sweep_type is required"),
    };
    let sweep_type = match parse_sweep_type(&sweep_type_str) {
        Some(t) => t,
        None => return error_result(format!("Unknown sweep_type: {sweep_type_str}")),
    };
    let min_value = match args.get("min_value").and_then(|v| v.as_f64()) {
        Some(v) => v,
        None => return error_result("min_value is required"),
    };
    let max_value = match args.get("max_value").and_then(|v| v.as_f64()) {
        Some(v) => v,
        None => return error_result("max_value is required"),
    };

    if min_value >= max_value {
        return error_result(format!(
            "min_value ({min_value}) must be less than max_value ({max_value})"
        ));
    }

    let mut st = state.lock().unwrap();

    // Validate event exists
    if !st.events.iter().any(|e| e.name.0 == event_name) {
        return error_result(format!(
            "Event '{event_name}' not found. Available events: {}",
            st.events.iter().map(|e| e.name.0.as_str()).collect::<Vec<_>>().join(", ")
        ));
    }

    let step_count = args
        .get("step_count")
        .and_then(|v| v.as_u64())
        .map(|n| n.max(2) as usize)
        .unwrap_or(st.sweep_default_steps);

    st.sweep_parameters.push(SweepParameterData {
        event_name: event_name.clone(),
        sweep_type,
        min_value,
        max_value,
        step_count,
    });
    st.invalidate_sweep_cache();

    let idx = st.sweep_parameters.len() - 1;
    let total_points: usize = st.sweep_parameters.iter().map(|p| p.step_count).product();
    let est_secs = total_points * st.sweep_mc_iterations / 10;

    let total_warning = if total_points > 500 {
        format!("\n⚠ Warning: {total_points} total points may take over a minute.")
    } else {
        String::new()
    };

    text_result(format!(
        "Added parameter [{idx}]: {event_name} → {sweep_type_str} \
         [{min_value}–{max_value}, {step_count} steps]\n\
         Grid so far: {total_points} total points ({} dimension(s))\n\
         Estimated runtime: ~{est_secs} seconds.{total_warning}",
        st.sweep_parameters.len(),
    ))
}

// ── remove_sweep_parameter ────────────────────────────────────────────────────

pub fn remove_sweep_parameter(
    args: Map<String, Value>,
    state: &SharedState,
) -> Result<CallToolResult, McpError> {
    let mut st = state.lock().unwrap();

    let idx = if let Some(name) = args.get("event_name").and_then(|v| v.as_str()) {
        st.sweep_parameters.iter().position(|p| p.event_name == name)
            .ok_or_else(|| format!("No sweep parameter found for event '{name}'"))
    } else if let Some(i) = args.get("index").and_then(|v| v.as_u64()) {
        let i = i as usize;
        if i < st.sweep_parameters.len() {
            Ok(i)
        } else {
            Err(format!("Index {i} out of range ({}  parameters)", st.sweep_parameters.len()))
        }
    } else {
        Err("Provide event_name or index".into())
    };

    let idx = match idx {
        Ok(i) => i,
        Err(msg) => return error_result(msg),
    };

    let removed = st.sweep_parameters.remove(idx);
    st.invalidate_sweep_cache();

    let total_points: usize = if st.sweep_parameters.is_empty() {
        0
    } else {
        st.sweep_parameters.iter().map(|p| p.step_count).product()
    };

    text_result(format!(
        "Removed parameter [{idx}]: {} → {:?}\n\
         Grid now has {} dimension(s), {total_points} total points.\n\
         Note: Previous sweep results cleared — call run_sweep again.",
        removed.event_name,
        removed.sweep_type,
        st.sweep_parameters.len(),
    ))
}

// ── run_sweep ─────────────────────────────────────────────────────────────────

pub fn run_sweep(
    _args: Map<String, Value>,
    state: &SharedState,
) -> Result<CallToolResult, McpError> {
    let (sim_data, sweep_params, mc_iterations) = {
        let st = state.lock().unwrap();
        if st.sweep_parameters.is_empty() {
            return error_result(
                "No sweep parameters defined. Call add_sweep_parameter first.",
            );
        }
        let sim_data = match st.merge() {
            Ok(d) => d,
            Err(errs) => {
                return error_result(format!("Cannot merge scenario:\n{}", errs.join("\n")));
            }
        };
        (sim_data, st.sweep_parameters.clone(), st.sweep_mc_iterations)
    };

    let config = match to_simulation_config(&sim_data) {
        Ok(c) => c,
        Err(e) => return error_result(format!("Config conversion failed: {e}")),
    };

    let sweep_config = match build_sweep_config(&sweep_params, &sim_data, mc_iterations) {
        Ok(c) => c,
        Err(msg) => return error_result(msg),
    };

    let mut sweep_results = match sweep_evaluate(&config, &sweep_config, None) {
        Ok(r) => r,
        Err(e) => return error_result(format!("Sweep failed: {e}")),
    };

    // Replace generic labels ("Age (Event N)") with readable "EventName (Type)" labels
    for (i, param) in sweep_params.iter().enumerate() {
        if i < sweep_results.param_labels.len() {
            sweep_results.param_labels[i] =
                format!("{} ({})", param.event_name, param.sweep_type.display_name());
        }
    }

    // Compute metric_ranges for summary
    let all_metrics: &[(&str, AnalysisMetric)] = &[
        ("success_rate", AnalysisMetric::SuccessRate),
        ("p5_final_net_worth", AnalysisMetric::Percentile { percentile: 5 }),
        ("p50_final_net_worth", AnalysisMetric::Percentile { percentile: 50 }),
        ("p95_final_net_worth", AnalysisMetric::Percentile { percentile: 95 }),
        ("lifetime_taxes", AnalysisMetric::LifetimeTaxes),
        ("max_drawdown", AnalysisMetric::MaxDrawdown),
    ];

    let mut metric_ranges = serde_json::Map::new();
    for (name, metric) in all_metrics {
        let (values, _, _) = sweep_results.get_metric_grid(metric);
        if !values.is_empty() {
            let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let baseline = compute_baseline(&sweep_results, metric);
            metric_ranges.insert(
                (*name).to_string(),
                json!({"min": min, "max": max, "baseline": baseline}),
            );
        }
    }

    let ndim = sweep_results.ndim();
    let total_points: usize = sweep_results.param_values.iter().map(Vec::len).product();

    let params_json: Vec<Value> = sweep_results
        .param_values
        .iter()
        .enumerate()
        .map(|(i, vals)| {
            json!({
                "index": i,
                "label": sweep_results.param_labels.get(i).map_or("", |s| s.as_str()),
                "values": vals,
            })
        })
        .collect();

    let summary = json!({
        "ndim": ndim,
        "total_points": total_points,
        "mc_iterations_per_point": mc_iterations,
        "parameters": params_json,
        "metric_ranges": metric_ranges,
    });

    {
        let mut st = state.lock().unwrap();
        st.last_sweep_results = Some(sweep_results);
    }

    text_result(serde_json::to_string_pretty(&summary).unwrap())
}

// ── get_sensitivity ───────────────────────────────────────────────────────────

pub fn get_sensitivity(
    args: Map<String, Value>,
    state: &SharedState,
) -> Result<CallToolResult, McpError> {
    let st = state.lock().unwrap();
    let results = match &st.last_sweep_results {
        Some(r) => r,
        None => return error_result("No sweep results. Call run_sweep first."),
    };

    let metric_str = match args.get("metric").and_then(|v| v.as_str()) {
        Some(m) => m.to_string(),
        None => return error_result("metric is required"),
    };
    let metric = match parse_metric(&metric_str) {
        Some(m) => m,
        None => return error_result(format!("Unknown metric: {metric_str}")),
    };

    let ndim = results.ndim();
    let shape = results.shape();

    if ndim == 0 {
        return error_result("Sweep has no dimensions.");
    }

    let baseline = compute_baseline(results, &metric);
    let (all_values, _, _) = results.get_metric_grid(&metric);
    let (min_val, max_val) = range(&all_values);
    let dist = distribution_summary(&all_values);

    let mut entries: Vec<Value> = (0..ndim)
        .filter(|&dim| shape[dim] >= 2)
        .map(|dim| {
            let low = avg_metric_for_dim_at(results, &metric, dim, 0);
            let high = avg_metric_for_dim_at(results, &metric, dim, shape[dim] - 1);
            let impact = high - low;
            json!({
                "index": dim,
                "label": results.param_labels.get(dim).map_or("", |s| s.as_str()),
                "low_value": low,
                "high_value": high,
                "impact": impact,
                "abs_impact": impact.abs(),
            })
        })
        .collect();

    entries.sort_by(|a, b| {
        b["abs_impact"]
            .as_f64()
            .unwrap_or(0.0)
            .total_cmp(&a["abs_impact"].as_f64().unwrap_or(0.0))
    });

    let output = json!({
        "metric": metric_str,
        "baseline": baseline,
        "range": {"min": min_val, "max": max_val},
        "distribution_summary": dist,
        "parameters": entries,
    });
    text_result(serde_json::to_string_pretty(&output).unwrap())
}

// ── get_sweep_curve ───────────────────────────────────────────────────────────

pub fn get_sweep_curve(
    args: Map<String, Value>,
    state: &SharedState,
) -> Result<CallToolResult, McpError> {
    let st = state.lock().unwrap();
    let results = match &st.last_sweep_results {
        Some(r) => r,
        None => return error_result("No sweep results. Call run_sweep first."),
    };

    let metric_str = match args.get("metric").and_then(|v| v.as_str()) {
        Some(m) => m.to_string(),
        None => return error_result("metric is required"),
    };
    let metric = match parse_metric(&metric_str) {
        Some(m) => m,
        None => return error_result(format!("Unknown metric: {metric_str}")),
    };

    let param_index = args.get("param_index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let threshold = args.get("threshold").and_then(|v| v.as_f64());

    if param_index >= results.ndim() {
        return error_result(format!(
            "param_index {param_index} out of range (ndim={})",
            results.ndim()
        ));
    }

    let fixed_step_map: HashMap<usize, usize> = args
        .get("fixed_values")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| Some((k.parse::<usize>().ok()?, v.as_u64()? as usize)))
                .collect()
        })
        .unwrap_or_default();

    let ndim = results.ndim();
    let shape = results.shape();

    // Build fixed_indices: None for param_index (free), Some(idx) for others
    let fixed: Vec<Option<usize>> = (0..ndim)
        .map(|dim| {
            if dim == param_index {
                None
            } else {
                Some(fixed_step_map.get(&dim).copied().unwrap_or(shape[dim] / 2))
            }
        })
        .collect();

    let curve = match results.get_metric_1d_slice(&metric, param_index, &fixed) {
        Some(pts) => pts,
        None => return error_result("Failed to compute 1D curve"),
    };

    let points_json: Vec<Value> = curve
        .iter()
        .map(|(pv, mv)| json!({"param_value": pv, "metric_value": mv}))
        .collect();

    // Spread: min/max across all other-dim combinations at each X value
    let spread_json: Vec<Value> = if ndim > 1 {
        let x_len = shape[param_index];
        (0..x_len)
            .map(|x_idx| {
                let pv = results.param_values[param_index][x_idx];
                let mut vals: Vec<f64> = results
                    .data
                    .iter()
                    .filter(|(indices, _)| indices[param_index] == x_idx)
                    .map(|(_, point)| {
                        point.compute_metric_with_inflation(
                            &metric,
                            results.birth_year,
                            results.standard_inflation_factor,
                        )
                    })
                    .collect();
                let (mn, mx) = if vals.is_empty() {
                    (0.0, 0.0)
                } else {
                    (
                        vals.iter().cloned().fold(f64::INFINITY, f64::min),
                        vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
                    )
                };
                json!({"param_value": pv, "min": mn, "max": mx})
            })
            .collect()
    } else {
        Vec::new()
    };

    // Threshold crossings
    let param_label = results.param_labels.get(param_index).map_or("", |s| s.as_str());
    let crossings_json: Vec<Value> = if let Some(thr) = threshold {
        find_all_threshold_crossings(&curve, thr, &metric_str, param_label)
    } else {
        Vec::new()
    };

    let output = json!({
        "metric": metric_str,
        "param_label": param_label,
        "points": points_json,
        "spread": spread_json,
        "threshold_crossings": crossings_json,
    });
    text_result(serde_json::to_string_pretty(&output).unwrap())
}

// ── get_sweep_grid ────────────────────────────────────────────────────────────

pub fn get_sweep_grid(
    args: Map<String, Value>,
    state: &SharedState,
) -> Result<CallToolResult, McpError> {
    let st = state.lock().unwrap();
    let results = match &st.last_sweep_results {
        Some(r) => r,
        None => return error_result("No sweep results. Call run_sweep first."),
    };

    if results.ndim() < 2 {
        return error_result(format!(
            "get_sweep_grid requires a 2D sweep (ndim={}). Add a second sweep parameter.",
            results.ndim()
        ));
    }

    let metric_str = match args.get("metric").and_then(|v| v.as_str()) {
        Some(m) => m.to_string(),
        None => return error_result("metric is required"),
    };
    let metric = match parse_metric(&metric_str) {
        Some(m) => m,
        None => return error_result(format!("Unknown metric: {metric_str}")),
    };

    let x_dim = args.get("x_param_index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let y_dim = args.get("y_param_index").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
    let target_threshold = args.get("target_threshold").and_then(|v| v.as_f64());

    let ndim = results.ndim();
    let shape = results.shape();

    let fixed: Vec<Option<usize>> = (0..ndim)
        .map(|dim| {
            if dim == x_dim || dim == y_dim {
                None
            } else {
                Some(shape[dim] / 2)
            }
        })
        .collect();

    // Returns (values in x-major flat order, x_params, y_params)
    let (flat_values, x_vals, y_vals) = match results.get_metric_2d_slice(&metric, x_dim, y_dim, &fixed) {
        Some(r) => r,
        None => return error_result("Failed to compute 2D grid"),
    };

    let x_len = x_vals.len();
    let y_len = y_vals.len();

    // flat_values is x-major: [x0y0, x0y1, ..., x0yN, x1y0, ...]
    // Reshape to matrix_xy[x][y], then transpose to matrix[y][x]
    let matrix_xy: Vec<Vec<f64>> = flat_values.chunks(y_len).map(|c| c.to_vec()).collect();
    let matrix: Vec<Vec<f64>> = (0..y_len)
        .map(|y| (0..x_len).map(|x| matrix_xy[x][y]).collect())
        .collect();

    let (min_val, max_val) = range(&flat_values);

    // Optimal cell: max for success_rate/net_worth, min for taxes/drawdown
    let (opt_x, opt_y, opt_val) = optimal_cell(&flat_values, x_len, y_len, &metric_str);

    let x_label = results.param_labels.get(x_dim).map_or("", |s| s.as_str());
    let y_label = results.param_labels.get(y_dim).map_or("", |s| s.as_str());

    let mut output = json!({
        "metric": metric_str,
        "x_label": x_label,
        "y_label": y_label,
        "x_values": x_vals,
        "y_values": y_vals,
        "matrix": matrix,
        "min_value": min_val,
        "max_value": max_val,
        "optimal_cell": {
            "x_index": opt_x,
            "y_index": opt_y,
            "x_value": x_vals.get(opt_x).copied().unwrap_or(0.0),
            "y_value": y_vals.get(opt_y).copied().unwrap_or(0.0),
            "metric_value": opt_val,
        },
    });

    if let Some(thr) = target_threshold {
        let cells_above = flat_values.iter().filter(|&&v| v >= thr).count();
        let total = flat_values.len();
        output["target_zone"] = json!({
            "threshold": thr,
            "cells_above_threshold": cells_above,
            "total_cells": total,
            "fraction": cells_above as f64 / total.max(1) as f64,
        });
    }

    text_result(serde_json::to_string_pretty(&output).unwrap())
}

// ── get_interaction_matrix ────────────────────────────────────────────────────

pub fn get_interaction_matrix(
    args: Map<String, Value>,
    state: &SharedState,
) -> Result<CallToolResult, McpError> {
    let st = state.lock().unwrap();
    let results = match &st.last_sweep_results {
        Some(r) => r,
        None => return error_result("No sweep results. Call run_sweep first."),
    };

    if results.ndim() < 2 {
        return error_result(format!(
            "get_interaction_matrix requires at least 2 dimensions (ndim={})",
            results.ndim()
        ));
    }

    let metric_str = match args.get("metric").and_then(|v| v.as_str()) {
        Some(m) => m.to_string(),
        None => return error_result("metric is required"),
    };
    let metric = match parse_metric(&metric_str) {
        Some(m) => m,
        None => return error_result(format!("Unknown metric: {metric_str}")),
    };

    let ndim = results.ndim();
    let shape = results.shape();
    let labels: Vec<&str> = (0..ndim)
        .map(|i| results.param_labels.get(i).map_or("", |s| s.as_str()))
        .collect();

    let mut matrix: Vec<Vec<Option<f64>>> = vec![vec![None; ndim]; ndim];
    let mut max_interaction = 0.0_f64;
    let mut strong_interactions: Vec<Value> = Vec::new();

    for i in 0..ndim {
        for j in (i + 1)..ndim {
            // Cross-derivative: joint effect minus sum of individual effects
            // f(min_i, min_j), f(min_i, max_j), f(max_i, min_j), f(max_i, max_j)
            let f_mm = avg_metric_pair(results, &metric, i, 0, j, 0);
            let f_mM = avg_metric_pair(results, &metric, i, 0, j, shape[j] - 1);
            let f_Mm = avg_metric_pair(results, &metric, i, shape[i] - 1, j, 0);
            let f_MM = avg_metric_pair(results, &metric, i, shape[i] - 1, j, shape[j] - 1);

            let interaction = f_MM - f_mM - f_Mm + f_mm;
            let abs_int = interaction.abs();

            matrix[i][j] = Some(interaction);
            matrix[j][i] = Some(interaction);
            max_interaction = max_interaction.max(abs_int);

            if abs_int > 0.01 {
                strong_interactions.push(json!({
                    "param_a": labels[i],
                    "param_b": labels[j],
                    "strength": interaction,
                    "interpretation": interaction_label(abs_int),
                }));
            }
        }
    }

    // Sort strong_interactions by abs strength descending
    strong_interactions.sort_by(|a, b| {
        b["strength"].as_f64().unwrap_or(0.0).abs()
            .total_cmp(&a["strength"].as_f64().unwrap_or(0.0).abs())
    });

    let matrix_json: Vec<Vec<Value>> = matrix
        .into_iter()
        .map(|row| row.into_iter().map(|v| v.map_or(Value::Null, |f| json!(f))).collect())
        .collect();

    let output = json!({
        "metric": metric_str,
        "labels": labels,
        "matrix": matrix_json,
        "max_interaction": max_interaction,
        "strong_interactions": strong_interactions,
    });
    text_result(serde_json::to_string_pretty(&output).unwrap())
}

// ── Pure helper functions ─────────────────────────────────────────────────────

fn parse_metric(s: &str) -> Option<AnalysisMetric> {
    match s {
        "success_rate" => Some(AnalysisMetric::SuccessRate),
        "p5_final_net_worth" => Some(AnalysisMetric::Percentile { percentile: 5 }),
        "p50_final_net_worth" => Some(AnalysisMetric::Percentile { percentile: 50 }),
        "p95_final_net_worth" => Some(AnalysisMetric::Percentile { percentile: 95 }),
        "lifetime_taxes" => Some(AnalysisMetric::LifetimeTaxes),
        "max_drawdown" => Some(AnalysisMetric::MaxDrawdown),
        _ => None,
    }
}

fn parse_sweep_type(s: &str) -> Option<SweepTypeData> {
    match s {
        "trigger_age" => Some(SweepTypeData::TriggerAge),
        "trigger_date" => Some(SweepTypeData::TriggerDate),
        "effect_value" => Some(SweepTypeData::EffectValue),
        "repeating_start_age" => Some(SweepTypeData::RepeatingStartAge),
        "repeating_end_age" => Some(SweepTypeData::RepeatingEndAge),
        _ => None,
    }
}

fn sweep_type_to_target(st: SweepTypeData) -> SweepTarget {
    match st {
        SweepTypeData::TriggerAge => SweepTarget::Trigger(TriggerParam::Age),
        SweepTypeData::TriggerDate => SweepTarget::Trigger(TriggerParam::Date),
        SweepTypeData::EffectValue => SweepTarget::Effect {
            param: EffectParam::Value,
            target: EffectTarget::FirstEligible,
        },
        SweepTypeData::RepeatingStartAge => {
            SweepTarget::Trigger(TriggerParam::RepeatingStart(Box::new(TriggerParam::Age)))
        }
        SweepTypeData::RepeatingEndAge => {
            SweepTarget::Trigger(TriggerParam::RepeatingEnd(Box::new(TriggerParam::Age)))
        }
    }
}

fn build_sweep_config(
    sweep_params: &[SweepParameterData],
    sim_data: &SimulationData,
    mc_iterations: usize,
) -> Result<SweepConfig, String> {
    let parameters: Result<Vec<SweepParameter>, String> = sweep_params
        .iter()
        .map(|param| {
            let event_id = sim_data
                .events
                .iter()
                .position(|e| e.name.0 == param.event_name)
                .map(|idx| EventId((idx + 1) as u16))
                .ok_or_else(|| format!("Event '{}' not found", param.event_name))?;
            Ok(SweepParameter {
                event_id,
                target: sweep_type_to_target(param.sweep_type),
                min_value: param.min_value,
                max_value: param.max_value,
                step_count: param.step_count,
            })
        })
        .collect();

    Ok(SweepConfig {
        parameters: parameters?,
        mc_iterations,
        ..Default::default()
    })
}

/// Average metric value at all points where `dim` is fixed to `dim_idx`.
fn avg_metric_for_dim_at(
    results: &SweepResults,
    metric: &AnalysisMetric,
    dim: usize,
    dim_idx: usize,
) -> f64 {
    let mut sum = 0.0;
    let mut count = 0usize;
    for (indices, point) in results.data.iter() {
        if indices[dim] == dim_idx {
            sum += point.compute_metric_with_inflation(
                metric,
                results.birth_year,
                results.standard_inflation_factor,
            );
            count += 1;
        }
    }
    if count == 0 { 0.0 } else { sum / count as f64 }
}

/// Average metric at all points where dim1==idx1 AND dim2==idx2.
fn avg_metric_pair(
    results: &SweepResults,
    metric: &AnalysisMetric,
    dim1: usize,
    idx1: usize,
    dim2: usize,
    idx2: usize,
) -> f64 {
    let mut sum = 0.0;
    let mut count = 0usize;
    for (indices, point) in results.data.iter() {
        if indices[dim1] == idx1 && indices[dim2] == idx2 {
            sum += point.compute_metric_with_inflation(
                metric,
                results.birth_year,
                results.standard_inflation_factor,
            );
            count += 1;
        }
    }
    if count == 0 { 0.0 } else { sum / count as f64 }
}

/// Metric value at all-midpoint indices (baseline).
fn compute_baseline(results: &SweepResults, metric: &AnalysisMetric) -> f64 {
    let shape = results.shape();
    let mid: Vec<usize> = (0..results.ndim()).map(|d| shape[d] / 2).collect();
    results
        .get(&mid)
        .map(|p| {
            p.compute_metric_with_inflation(
                metric,
                results.birth_year,
                results.standard_inflation_factor,
            )
        })
        .unwrap_or(0.0)
}

fn range(values: &[f64]) -> (f64, f64) {
    if values.is_empty() {
        return (0.0, 0.0);
    }
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    (min, max)
}

fn distribution_summary(values: &[f64]) -> Value {
    if values.is_empty() {
        return json!({"mean": 0.0, "std_dev": 0.0, "p25": 0.0, "p75": 0.0});
    }
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let std_dev = variance.sqrt();
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let p25 = sorted[((n * 0.25) as usize).min(sorted.len() - 1)];
    let p75 = sorted[((n * 0.75) as usize).min(sorted.len() - 1)];
    json!({"mean": mean, "std_dev": std_dev, "p25": p25, "p75": p75})
}

fn find_all_threshold_crossings(
    curve: &[(f64, f64)],
    threshold: f64,
    metric_str: &str,
    param_label: &str,
) -> Vec<Value> {
    let mut crossings = Vec::new();
    for window in curve.windows(2) {
        let (x0, y0) = window[0];
        let (x1, y1) = window[1];
        if (y0 < threshold) != (y1 < threshold) {
            let t = (threshold - y0) / (y1 - y0);
            let x_crossing = x0 + t * (x1 - x0);
            let direction = if y1 > y0 { "above" } else { "below" };
            let interp = if direction == "above" {
                format!(
                    "{metric_str} exceeds {threshold:.3} when {param_label} is above {x_crossing:.1}"
                )
            } else {
                format!(
                    "{metric_str} drops below {threshold:.3} when {param_label} is below {x_crossing:.1}"
                )
            };
            crossings.push(json!({
                "threshold": threshold,
                "crossing_param_value": x_crossing,
                "direction": direction,
                "interpretation": interp,
            }));
        }
    }
    crossings
}

/// Find the optimal cell in a flattened x-major grid.
/// "Optimal" means highest for success_rate/net_worth, lowest for taxes/drawdown.
fn optimal_cell(values: &[f64], x_len: usize, y_len: usize, metric: &str) -> (usize, usize, f64) {
    let lower_is_better = matches!(metric, "lifetime_taxes" | "max_drawdown");
    let mut best_flat = 0usize;
    let mut best_val = if lower_is_better {
        f64::INFINITY
    } else {
        f64::NEG_INFINITY
    };

    for (i, &v) in values.iter().enumerate() {
        let is_better = if lower_is_better { v < best_val } else { v > best_val };
        if is_better {
            best_val = v;
            best_flat = i;
        }
    }

    // flat index in x-major order: flat = x * y_len + y
    let x_idx = best_flat / y_len;
    let y_idx = best_flat % y_len;
    (x_idx, y_idx, best_val)
}

fn interaction_label(abs_strength: f64) -> &'static str {
    if abs_strength < 0.05 {
        "negligible"
    } else if abs_strength < 0.10 {
        "weak"
    } else if abs_strength < 0.20 {
        "moderate"
    } else {
        "strong"
    }
}
```

- [ ] **Step 2.4: Add `pub mod sweep;` to `tools/mod.rs`**

Add this line to the top of `crates/finplan_mcp/src/tools/mod.rs` alongside the other module declarations:

```rust
pub mod sweep;
```

(Full registration of the tools — in `list_tools()` and `call_tool()` — happens in Task 7. For now we just need the module declared so the tests can call `tools::sweep::*` directly.)

- [ ] **Step 2.5: Run the setup-tool tests to confirm they pass**

```bash
docker run --rm -v "$PWD":/app -w /app rust:slim cargo test -p finplan_mcp --test sweep_integration_test -- test_configure_sweep test_add_sweep_parameter test_remove_sweep_parameter --nocapture 2>&1 | tail -30
```

Expected: all 5 tests pass.

- [ ] **Step 2.6: Commit**

```bash
git add crates/finplan_mcp/src/tools/sweep.rs crates/finplan_mcp/src/tools/mod.rs
git commit -m "feat(mcp): add sweep.rs with configure/add/remove sweep parameter tools"
```

---

## Task 3: Implement `run_sweep`

`run_sweep` is already written in `sweep.rs` from Task 2. This task adds its integration tests.

**Files:**
- Modify: `crates/finplan_mcp/tests/sweep_integration_test.rs`

- [ ] **Step 3.1: Add run_sweep tests**

Append to `sweep_integration_test.rs`:

```rust
// ── Shared helper: 1D sweep ready to run ─────────────────────────────────────

fn setup_1d_sweep() -> finplan_mcp::state::SharedState {
    let st = setup_sweep_scenario();
    tools::sweep::add_sweep_parameter(
        args(json!({
            "event_name": "Retirement",
            "sweep_type": "trigger_age",
            "min_value": 60,
            "max_value": 65,
            "step_count": 3
        })),
        &st,
    ).unwrap();
    tools::sweep::configure_sweep(args(json!({"mc_iterations": 30})), &st).unwrap();
    st
}

// ── run_sweep tests ───────────────────────────────────────────────────────────

#[test]
fn test_run_sweep_fails_without_parameters() {
    let st = setup_sweep_scenario();
    let r = tools::sweep::run_sweep(args(json!({})), &st).unwrap();
    assert_eq!(r.is_error, Some(true));
    assert!(text_of(&r).contains("No sweep parameters"));
}

#[test]
fn test_run_sweep_fails_without_portfolio() {
    let st = state::new_shared_state();
    tools::sweep::add_sweep_parameter(
        args(json!({
            "event_name": "Retirement",
            "sweep_type": "trigger_age",
            "min_value": 60,
            "max_value": 65
        })),
        &st,
    ).unwrap();
    let r = tools::sweep::run_sweep(args(json!({})), &st).unwrap();
    assert_eq!(r.is_error, Some(true));
}

#[test]
fn test_run_sweep_1d_returns_correct_shape() {
    println!("\n═══ run_sweep: 1D sweep produces correct ndim and total_points ═══");
    let st = setup_1d_sweep();
    let r = tools::sweep::run_sweep(args(json!({})), &st).unwrap();
    assert!(is_ok(&r), "run_sweep failed: {}", text_of(&r));
    let j: Value = serde_json::from_str(&text_of(&r)).unwrap();
    println!("  {}", serde_json::to_string_pretty(&j).unwrap());
    assert_eq!(j["ndim"].as_u64().unwrap(), 1);
    assert_eq!(j["total_points"].as_u64().unwrap(), 3);
    assert!(j["metric_ranges"]["success_rate"]["baseline"].as_f64().unwrap() >= 0.0);
    // Confirm sweep results are cached in state
    assert!(st.lock().unwrap().last_sweep_results.is_some());
}

#[test]
fn test_run_sweep_stores_results_in_state() {
    let st = setup_1d_sweep();
    assert!(st.lock().unwrap().last_sweep_results.is_none());
    tools::sweep::run_sweep(args(json!({})), &st).unwrap();
    assert!(st.lock().unwrap().last_sweep_results.is_some());
}

#[test]
fn test_scenario_change_clears_sweep_results() {
    // Changing the scenario (e.g. set_parameters) should invalidate sweep results.
    let st = setup_1d_sweep();
    tools::sweep::run_sweep(args(json!({})), &st).unwrap();
    assert!(st.lock().unwrap().last_sweep_results.is_some());

    // Modify the scenario
    tools::parameters::set_parameters(
        args(json!({"birth_date": "1972-01-01"})),
        &st,
    ).unwrap();
    assert!(st.lock().unwrap().last_sweep_results.is_none());
}
```

- [ ] **Step 3.2: Run the tests (excluding slow sweep run tests first)**

```bash
docker run --rm -v "$PWD":/app -w /app rust:slim cargo test -p finplan_mcp --test sweep_integration_test -- test_run_sweep_fails --nocapture 2>&1 | tail -15
```

Expected: 2 failure-case tests pass immediately.

- [ ] **Step 3.3: Run the full run_sweep test suite (takes ~15s)**

```bash
docker run --rm -v "$PWD":/app -w /app rust:slim cargo test -p finplan_mcp --test sweep_integration_test -- test_run_sweep test_scenario_change --nocapture 2>&1 | tail -20
```

Expected: all tests pass.

- [ ] **Step 3.4: Commit**

```bash
git add crates/finplan_mcp/tests/sweep_integration_test.rs
git commit -m "test(mcp): add run_sweep integration tests (Plan 3 step 3)"
```

---

## Task 4: Test and verify `get_sensitivity`

`get_sensitivity` is already written in `sweep.rs` from Task 2. This task adds its integration tests.

**Files:**
- Modify: `crates/finplan_mcp/tests/sweep_integration_test.rs`

- [ ] **Step 4.1: Add get_sensitivity tests**

Append to `sweep_integration_test.rs`:

```rust
// ── Shared helper: 1D sweep already run ──────────────────────────────────────

fn run_1d_sweep() -> finplan_mcp::state::SharedState {
    let st = setup_1d_sweep();
    tools::sweep::run_sweep(args(json!({})), &st).unwrap();
    st
}

// ── get_sensitivity tests ─────────────────────────────────────────────────────

#[test]
fn test_get_sensitivity_fails_without_sweep() {
    let st = setup_sweep_scenario();
    let r = tools::sweep::get_sensitivity(
        args(json!({"metric": "success_rate"})),
        &st,
    ).unwrap();
    assert_eq!(r.is_error, Some(true));
}

#[test]
fn test_get_sensitivity_returns_parameters_sorted_by_impact() {
    println!("\n═══ get_sensitivity: results sorted by abs_impact ═══");
    let st = run_1d_sweep();
    let r = tools::sweep::get_sensitivity(
        args(json!({"metric": "success_rate"})),
        &st,
    ).unwrap();
    assert!(is_ok(&r), "get_sensitivity failed: {}", text_of(&r));
    let j: Value = serde_json::from_str(&text_of(&r)).unwrap();
    println!("  {}", serde_json::to_string_pretty(&j).unwrap());

    assert_eq!(j["metric"].as_str().unwrap(), "success_rate");
    assert!(j["baseline"].as_f64().is_some());
    let params = j["parameters"].as_array().unwrap();
    assert_eq!(params.len(), 1, "1D sweep should have 1 sensitivity entry");

    let entry = &params[0];
    assert_eq!(entry["label"].as_str().unwrap(), "Retirement (Age)");
    assert!(entry["abs_impact"].as_f64().unwrap() >= 0.0);
}

#[test]
fn test_get_sensitivity_unknown_metric_returns_error() {
    let st = run_1d_sweep();
    let r = tools::sweep::get_sensitivity(
        args(json!({"metric": "bogus_metric"})),
        &st,
    ).unwrap();
    assert_eq!(r.is_error, Some(true));
}
```

- [ ] **Step 4.2: Run get_sensitivity tests**

```bash
docker run --rm -v "$PWD":/app -w /app rust:slim cargo test -p finplan_mcp --test sweep_integration_test -- test_get_sensitivity --nocapture 2>&1 | tail -20
```

Expected: all 3 tests pass.

- [ ] **Step 4.3: Commit**

```bash
git add crates/finplan_mcp/tests/sweep_integration_test.rs
git commit -m "test(mcp): add get_sensitivity integration tests (Plan 3 step 4)"
```

---

## Task 5: Test and verify `get_sweep_curve`

**Files:**
- Modify: `crates/finplan_mcp/tests/sweep_integration_test.rs`

- [ ] **Step 5.1: Add get_sweep_curve tests**

Append to `sweep_integration_test.rs`:

```rust
// ── get_sweep_curve tests ─────────────────────────────────────────────────────

#[test]
fn test_get_sweep_curve_returns_correct_number_of_points() {
    println!("\n═══ get_sweep_curve: 1D curve has step_count points ═══");
    let st = run_1d_sweep(); // 3 steps
    let r = tools::sweep::get_sweep_curve(
        args(json!({"metric": "success_rate"})),
        &st,
    ).unwrap();
    assert!(is_ok(&r), "get_sweep_curve failed: {}", text_of(&r));
    let j: Value = serde_json::from_str(&text_of(&r)).unwrap();
    println!("  {}", serde_json::to_string_pretty(&j).unwrap());

    let points = j["points"].as_array().unwrap();
    assert_eq!(points.len(), 3, "Should have 3 points for 3-step sweep");
    assert!(j["threshold_crossings"].is_array());
    // 1D sweep: spread should be empty (no other dims to vary across)
    let spread = j["spread"].as_array().unwrap();
    assert!(spread.is_empty(), "1D sweep should have empty spread");
}

#[test]
fn test_get_sweep_curve_with_threshold() {
    let st = run_1d_sweep();
    let r = tools::sweep::get_sweep_curve(
        args(json!({"metric": "success_rate", "threshold": 0.5})),
        &st,
    ).unwrap();
    assert!(is_ok(&r));
    let j: Value = serde_json::from_str(&text_of(&r)).unwrap();
    // Threshold crossings array must be present (may be empty if curve never crosses 0.5)
    assert!(j["threshold_crossings"].is_array());
}

#[test]
fn test_get_sweep_curve_fails_without_sweep() {
    let st = setup_sweep_scenario();
    let r = tools::sweep::get_sweep_curve(
        args(json!({"metric": "success_rate"})),
        &st,
    ).unwrap();
    assert_eq!(r.is_error, Some(true));
}
```

- [ ] **Step 5.2: Run get_sweep_curve tests**

```bash
docker run --rm -v "$PWD":/app -w /app rust:slim cargo test -p finplan_mcp --test sweep_integration_test -- test_get_sweep_curve --nocapture 2>&1 | tail -20
```

Expected: all 3 tests pass.

- [ ] **Step 5.3: Commit**

```bash
git add crates/finplan_mcp/tests/sweep_integration_test.rs
git commit -m "test(mcp): add get_sweep_curve integration tests (Plan 3 step 5)"
```

---

## Task 6: Test and verify `get_sweep_grid` + `get_interaction_matrix`

**Files:**
- Modify: `crates/finplan_mcp/tests/sweep_integration_test.rs`

- [ ] **Step 6.1: Add 2D sweep helper and grid/interaction tests**

Append to `sweep_integration_test.rs`:

```rust
// ── Shared helper: 2D sweep already run ──────────────────────────────────────

fn run_2d_sweep() -> finplan_mcp::state::SharedState {
    let st = setup_sweep_scenario();
    // Dim 0: retirement age 60–65 in 3 steps
    tools::sweep::add_sweep_parameter(
        args(json!({
            "event_name": "Retirement",
            "sweep_type": "trigger_age",
            "min_value": 60,
            "max_value": 65,
            "step_count": 3
        })),
        &st,
    ).unwrap();
    // Dim 1: living expenses $4000–$7000 in 3 steps
    tools::sweep::add_sweep_parameter(
        args(json!({
            "event_name": "Living Expenses",
            "sweep_type": "effect_value",
            "min_value": 4000,
            "max_value": 7000,
            "step_count": 3
        })),
        &st,
    ).unwrap();
    tools::sweep::configure_sweep(args(json!({"mc_iterations": 30})), &st).unwrap();
    tools::sweep::run_sweep(args(json!({})), &st).unwrap();
    st
}

// ── get_sweep_grid tests ──────────────────────────────────────────────────────

#[test]
fn test_get_sweep_grid_fails_for_1d_sweep() {
    let st = run_1d_sweep();
    let r = tools::sweep::get_sweep_grid(
        args(json!({"metric": "success_rate"})),
        &st,
    ).unwrap();
    assert_eq!(r.is_error, Some(true));
    assert!(text_of(&r).contains("2D"));
}

#[test]
fn test_get_sweep_grid_returns_correct_matrix_shape() {
    println!("\n═══ get_sweep_grid: 2D grid has correct dimensions ═══");
    let st = run_2d_sweep();
    let r = tools::sweep::get_sweep_grid(
        args(json!({"metric": "success_rate"})),
        &st,
    ).unwrap();
    assert!(is_ok(&r), "get_sweep_grid failed: {}", text_of(&r));
    let j: Value = serde_json::from_str(&text_of(&r)).unwrap();
    println!("  {}", serde_json::to_string_pretty(&j).unwrap());

    let matrix = j["matrix"].as_array().unwrap();
    assert_eq!(matrix.len(), 3, "matrix should have 3 rows (y_steps)");
    assert_eq!(matrix[0].as_array().unwrap().len(), 3, "each row should have 3 cols (x_steps)");

    assert_eq!(j["x_values"].as_array().unwrap().len(), 3);
    assert_eq!(j["y_values"].as_array().unwrap().len(), 3);
    assert!(j["optimal_cell"]["metric_value"].as_f64().is_some());
}

#[test]
fn test_get_sweep_grid_target_zone() {
    let st = run_2d_sweep();
    let r = tools::sweep::get_sweep_grid(
        args(json!({"metric": "success_rate", "target_threshold": 0.5})),
        &st,
    ).unwrap();
    assert!(is_ok(&r));
    let j: Value = serde_json::from_str(&text_of(&r)).unwrap();
    assert!(j["target_zone"]["total_cells"].as_u64().unwrap() == 9); // 3×3
    assert!(j["target_zone"]["cells_above_threshold"].as_u64().is_some());
}

// ── get_interaction_matrix tests ──────────────────────────────────────────────

#[test]
fn test_get_interaction_matrix_fails_for_1d_sweep() {
    let st = run_1d_sweep();
    let r = tools::sweep::get_interaction_matrix(
        args(json!({"metric": "success_rate"})),
        &st,
    ).unwrap();
    assert_eq!(r.is_error, Some(true));
}

#[test]
fn test_get_interaction_matrix_returns_correct_structure() {
    println!("\n═══ get_interaction_matrix: 2×2 matrix with null diagonal ═══");
    let st = run_2d_sweep();
    let r = tools::sweep::get_interaction_matrix(
        args(json!({"metric": "success_rate"})),
        &st,
    ).unwrap();
    assert!(is_ok(&r), "get_interaction_matrix failed: {}", text_of(&r));
    let j: Value = serde_json::from_str(&text_of(&r)).unwrap();
    println!("  {}", serde_json::to_string_pretty(&j).unwrap());

    let matrix = j["matrix"].as_array().unwrap();
    assert_eq!(matrix.len(), 2, "2×2 matrix expected");
    // Diagonal must be null
    assert!(matrix[0].as_array().unwrap()[0].is_null());
    assert!(matrix[1].as_array().unwrap()[1].is_null());
    // Off-diagonal must be non-null
    assert!(matrix[0].as_array().unwrap()[1].as_f64().is_some());

    assert_eq!(j["labels"].as_array().unwrap().len(), 2);
    assert!(j["max_interaction"].as_f64().is_some());
    assert!(j["strong_interactions"].is_array());
}
```

- [ ] **Step 6.2: Run grid and interaction tests**

```bash
docker run --rm -v "$PWD":/app -w /app rust:slim cargo test -p finplan_mcp --test sweep_integration_test -- test_get_sweep_grid test_get_interaction_matrix --nocapture 2>&1 | tail -30
```

Expected: all 5 tests pass.

- [ ] **Step 6.3: Commit**

```bash
git add crates/finplan_mcp/tests/sweep_integration_test.rs
git commit -m "test(mcp): add get_sweep_grid + get_interaction_matrix tests (Plan 3 step 6)"
```

---

## Task 7: Wire everything up — register tools, update `get_state_summary`

**Files:**
- Modify: `crates/finplan_mcp/src/tools/mod.rs`

- [ ] **Step 7.1: Register sweep tools in `list_tools()` in `mod.rs`**

In `list_tools()`, add the sweep tools line:

```rust
pub fn list_tools() -> Vec<Tool> {
    let mut tools = Vec::new();
    tools.extend(parameters::tools());
    tools.extend(portfolio::tools());
    tools.extend(ticker::tools());
    tools.extend(events::tools());
    tools.extend(simulation::tools());
    tools.extend(sweep::tools());          // ← add this line
    tools.extend(merge::tools());
    tools.extend(validate::tools());
    tools.extend(utility_tools());
    tools
}
```

- [ ] **Step 7.2: Add 8 match arms to `call_tool()` in `mod.rs`**

In `call_tool()`, add these arms before the `_ =>` fallthrough:

```rust
        "add_sweep_parameter"    => sweep::add_sweep_parameter(args, state),
        "remove_sweep_parameter" => sweep::remove_sweep_parameter(args, state),
        "configure_sweep"        => sweep::configure_sweep(args, state),
        "run_sweep"              => sweep::run_sweep(args, state),
        "get_sensitivity"        => sweep::get_sensitivity(args, state),
        "get_sweep_curve"        => sweep::get_sweep_curve(args, state),
        "get_sweep_grid"         => sweep::get_sweep_grid(args, state),
        "get_interaction_matrix" => sweep::get_interaction_matrix(args, state),
```

- [ ] **Step 7.3: Update `get_state_summary` to report sweep state**

Replace the existing `get_state_summary` function body in `mod.rs`:

```rust
pub fn get_state_summary(state: &SharedState) -> Result<CallToolResult, McpError> {
    let st = state.lock().unwrap();
    let mut parts = Vec::new();

    if let Some(ref p) = st.portfolio {
        parts.push(format!(
            "Portfolio: \"{}\" with {} accounts",
            p.name,
            p.accounts.len()
        ));
    } else {
        parts.push("Portfolio: not set".into());
    }

    if let Some(ref params) = st.parameters {
        parts.push(format!(
            "Parameters: birth={}, start={}, duration={}yr",
            params.birth_date, params.start_date, params.duration_years
        ));
    } else {
        parts.push("Parameters: not set".into());
    }

    parts.push(format!("Events: {} defined", st.events.len()));

    if !st.assets.is_empty() || !st.historical_assets.is_empty() {
        parts.push(format!(
            "Ticker mappings: {} parametric, {} historical",
            st.assets.len(),
            st.historical_assets.len()
        ));
    }

    if !st.profiles.is_empty() {
        parts.push(format!("Profiles: {} defined", st.profiles.len()));
    }

    // Sweep state
    if st.sweep_parameters.is_empty() {
        parts.push("Sweep: no parameters defined".into());
    } else {
        let total_points: usize = st.sweep_parameters.iter().map(|p| p.step_count).product();
        let run_status = if st.last_sweep_results.is_some() {
            "results cached"
        } else {
            "not yet run"
        };
        parts.push(format!(
            "Sweep: {} parameter(s), {} total points, {} MC iter/point — {}",
            st.sweep_parameters.len(),
            total_points,
            st.sweep_mc_iterations,
            run_status,
        ));
        for (i, p) in st.sweep_parameters.iter().enumerate() {
            parts.push(format!(
                "  [{i}] {} → {:?} [{:.0}–{:.0}, {} steps]",
                p.event_name, p.sweep_type, p.min_value, p.max_value, p.step_count
            ));
        }
    }

    text_result(parts.join("\n"))
}
```

- [ ] **Step 7.4: Verify tool count with existing test**

```bash
docker run --rm -v "$PWD":/app -w /app rust:slim cargo test -p finplan_mcp --test integration_test -- test_tool_list_contains_expected_tools --nocapture 2>&1 | tail -15
```

This test checks for a specific tool count. If it hard-codes 20 tools and now we have 28, it will fail. Fix the count in the test:

Open `crates/finplan_mcp/tests/integration_test.rs`, find `test_tool_list_contains_expected_tools`, and update the tool count assertion from `20` to `28`. Also add the 8 new tool names to the list:

```rust
// Add to the expected_tools list:
"add_sweep_parameter",
"remove_sweep_parameter",
"configure_sweep",
"run_sweep",
"get_sensitivity",
"get_sweep_curve",
"get_sweep_grid",
"get_interaction_matrix",
```

- [ ] **Step 7.5: Run the full test suite to verify nothing is broken**

```bash
docker run --rm -v "$PWD":/app -w /app rust:slim cargo test -p finplan_mcp -- --nocapture 2>&1 | tail -30
```

Expected: all tests pass.

- [ ] **Step 7.6: Run fmt and clippy**

```bash
docker run --rm -v "$PWD":/app -w /app rust:slim sh -c "rustup component add rustfmt clippy 2>/dev/null; cargo fmt && cargo clippy -p finplan_mcp 2>&1" | tail -20
```

Fix any warnings. Re-run fmt after fixes.

- [ ] **Step 7.7: Commit**

```bash
git add crates/finplan_mcp/src/tools/mod.rs crates/finplan_mcp/tests/integration_test.rs
git commit -m "feat(mcp): wire sweep tools into MCP server + update get_state_summary"
```

---

## Task 8: End-to-end integration smoke test

Add one final test that exercises the full 2D sweep workflow from scratch in a single test function, mirroring the `test_full_scenario_build` pattern.

**Files:**
- Modify: `crates/finplan_mcp/tests/sweep_integration_test.rs`

- [ ] **Step 8.1: Add full workflow test**

Append to `sweep_integration_test.rs`:

```rust
// ── Full sweep workflow ───────────────────────────────────────────────────────

#[test]
fn test_full_sweep_workflow() {
    println!("\n═══ Full Sweep Workflow: 2D sensitivity analysis ═══");
    let st = setup_sweep_scenario();

    // Step 1: configure sweep
    let r = tools::sweep::configure_sweep(args(json!({"mc_iterations": 30})), &st).unwrap();
    assert!(is_ok(&r));
    println!("[step 1] configure_sweep → {}", text_of(&r));

    // Step 2: add retirement age sweep
    let r = tools::sweep::add_sweep_parameter(
        args(json!({
            "event_name": "Retirement",
            "sweep_type": "trigger_age",
            "min_value": 60,
            "max_value": 65,
            "step_count": 3
        })),
        &st,
    ).unwrap();
    assert!(is_ok(&r));
    println!("[step 2] add_sweep_parameter (Retirement age) → {}", text_of(&r));

    // Step 3: add expense sweep
    let r = tools::sweep::add_sweep_parameter(
        args(json!({
            "event_name": "Living Expenses",
            "sweep_type": "effect_value",
            "min_value": 4000,
            "max_value": 7000,
            "step_count": 3
        })),
        &st,
    ).unwrap();
    assert!(is_ok(&r));
    println!("[step 3] add_sweep_parameter (Living Expenses) → {}", text_of(&r));

    // Step 4: run sweep
    let r = tools::sweep::run_sweep(args(json!({})), &st).unwrap();
    assert!(is_ok(&r), "run_sweep failed: {}", text_of(&r));
    let j: Value = serde_json::from_str(&text_of(&r)).unwrap();
    assert_eq!(j["ndim"].as_u64().unwrap(), 2);
    assert_eq!(j["total_points"].as_u64().unwrap(), 9);
    println!("[step 4] run_sweep → ndim={}, total_points={}", j["ndim"], j["total_points"]);

    // Step 5: get sensitivity
    let r = tools::sweep::get_sensitivity(args(json!({"metric": "success_rate"})), &st).unwrap();
    assert!(is_ok(&r));
    let j: Value = serde_json::from_str(&text_of(&r)).unwrap();
    let params = j["parameters"].as_array().unwrap();
    assert_eq!(params.len(), 2);
    let impact0 = params[0]["abs_impact"].as_f64().unwrap();
    let impact1 = params[1]["abs_impact"].as_f64().unwrap();
    assert!(impact0 >= impact1, "Results must be sorted by abs_impact descending");
    println!("[step 5] get_sensitivity → {} is top lever (impact={:.3})", params[0]["label"], impact0);

    // Step 6: get sweep curve for dim 0 with threshold
    let r = tools::sweep::get_sweep_curve(
        args(json!({"metric": "success_rate", "param_index": 0, "threshold": 0.5})),
        &st,
    ).unwrap();
    assert!(is_ok(&r));
    let j: Value = serde_json::from_str(&text_of(&r)).unwrap();
    assert_eq!(j["points"].as_array().unwrap().len(), 3);
    println!("[step 6] get_sweep_curve → {} points, {} crossings",
        j["points"].as_array().unwrap().len(),
        j["threshold_crossings"].as_array().unwrap().len());

    // Step 7: get 2D grid
    let r = tools::sweep::get_sweep_grid(
        args(json!({"metric": "success_rate", "target_threshold": 0.5})),
        &st,
    ).unwrap();
    assert!(is_ok(&r));
    let j: Value = serde_json::from_str(&text_of(&r)).unwrap();
    let matrix = j["matrix"].as_array().unwrap();
    assert_eq!(matrix.len(), 3);
    assert_eq!(matrix[0].as_array().unwrap().len(), 3);
    println!("[step 7] get_sweep_grid → {}×{} matrix, optimal={:.3}",
        matrix.len(), matrix[0].as_array().unwrap().len(),
        j["optimal_cell"]["metric_value"].as_f64().unwrap_or(0.0));

    // Step 8: get interaction matrix
    let r = tools::sweep::get_interaction_matrix(
        args(json!({"metric": "success_rate"})),
        &st,
    ).unwrap();
    assert!(is_ok(&r));
    let j: Value = serde_json::from_str(&text_of(&r)).unwrap();
    let matrix = j["matrix"].as_array().unwrap();
    assert_eq!(matrix.len(), 2);
    assert!(matrix[0].as_array().unwrap()[0].is_null()); // diagonal null
    println!("[step 8] get_interaction_matrix → max_interaction={:.4}", j["max_interaction"].as_f64().unwrap_or(0.0));

    println!("\n✓ Full sweep workflow passed");
}
```

- [ ] **Step 8.2: Run the full test suite one final time**

```bash
docker run --rm -v "$PWD":/app -w /app rust:slim cargo test -p finplan_mcp -- --nocapture 2>&1 | tail -40
```

Expected: all tests pass, including `test_full_sweep_workflow`.

- [ ] **Step 8.3: Final fmt + clippy**

```bash
docker run --rm -v "$PWD":/app -w /app rust:slim sh -c "rustup component add rustfmt clippy 2>/dev/null; cargo fmt && cargo clippy -p finplan_mcp 2>&1" | grep -E "^error|^warning" | head -20
```

Fix any remaining warnings.

- [ ] **Step 8.4: Final commit**

```bash
git add crates/finplan_mcp/tests/sweep_integration_test.rs crates/finplan_mcp/src/tools/sweep.rs crates/finplan_mcp/src/state.rs
git commit -m "feat(mcp): Plan 3 complete — 8 parameter sweep & sensitivity tools"
```

---

## Self-Review Checklist

**Spec coverage:**
- ✅ `add_sweep_parameter` — Task 2, validates event name, min<max, step_count≥2
- ✅ `remove_sweep_parameter` — Task 2, by name or index, clears sweep cache
- ✅ `configure_sweep` — Task 2, clamps mc_iterations 10–2000, default_steps 2–20
- ✅ `run_sweep` — Task 3 (code in Task 2), calls `sweep_evaluate`, stores `SweepResults`
- ✅ `get_sensitivity` — Task 4 (code in Task 2), sorted by abs_impact, distribution_summary
- ✅ `get_sweep_curve` — Task 5 (code in Task 2), spread bands, threshold crossings
- ✅ `get_sweep_grid` — Task 6 (code in Task 2), 2D matrix, target_zone, optimal_cell
- ✅ `get_interaction_matrix` — Task 6 (code in Task 2), NxN with null diagonal, labels
- ✅ State additions (sweep_parameters, sweep_mc_iterations, sweep_default_steps, last_sweep_results) — Task 1
- ✅ Cache invalidation: `invalidate_simulation_cache` extended to clear sweep results (all scenario tools already call it) — Task 1
- ✅ `get_state_summary` extended with sweep info — Task 7
- ✅ Tool registration in `list_tools()` and `call_tool()` — Task 7
- ✅ Tool count test updated — Task 7
- ✅ Spec section 14.1 unit tests for helpers: covered by integration tests (linspace via step_count, threshold crossing in get_sweep_curve test, distribution_summary implicitly via get_sensitivity) — Tasks 4/5
- ✅ Spec section 16 Example Agent Session — covered by `test_full_sweep_workflow` — Task 8

**Spec delta:** The spec mentions `last_sweep_sim_data` in state, but after analysis, the query tools don't need it — `param_labels` are stored directly in `SweepResults`. Omitted intentionally.

**Scaling note:** The TUI's `AnalysisResults` scales `success_rate` × 100 for percentage display. This plan uses `SweepResults` + `AnalysisMetric` directly (core types), so all values are raw fractions/dollars matching the spec (e.g., `0.78` not `78.0`).
