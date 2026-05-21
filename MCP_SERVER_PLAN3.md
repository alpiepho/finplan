# FinPlan MCP Server — Plan 3: Parameter Sweep & Sensitivity Analysis

> **Branch**: `feature/mcp`
> **Crate**: `crates/finplan_mcp`
> **Depends on**: MCP_SERVER_PLAN1.md (scenario building), MCP_SERVER_PLAN2.md (simulation execution)
> **Goal**: Expose the Analysis screen's parameter sweep engine via MCP. An AI agent will be able to configure a multi-dimensional sweep grid, run it, and return structured sensitivity data — letting it answer "which knob matters most?" and "what's the optimal retirement age for this spending level?" without the user opening the TUI.

---

## Table of Contents

1. [Motivation](#1-motivation)
2. [New Tools Overview](#2-new-tools-overview)
3. [New Return Data Types](#3-new-return-data-types)
4. [Tool: `add_sweep_parameter`](#4-tool-add_sweep_parameter)
5. [Tool: `remove_sweep_parameter`](#5-tool-remove_sweep_parameter)
6. [Tool: `configure_sweep`](#6-tool-configure_sweep)
7. [Tool: `run_sweep`](#7-tool-run_sweep)
8. [Tool: `get_sensitivity`](#8-tool-get_sensitivity)
9. [Tool: `get_sweep_curve`](#9-tool-get_sweep_curve)
10. [Tool: `get_sweep_grid`](#10-tool-get_sweep_grid)
11. [Tool: `get_interaction_matrix`](#11-tool-get_interaction_matrix)
12. [State Changes](#12-state-changes)
13. [Implementation Details](#13-implementation-details)
14. [Testing Strategy](#14-testing-strategy)
15. [File-by-File Checklist](#15-file-by-file-checklist)
16. [Example Agent Session](#16-example-agent-session)
17. [Future Work](#17-future-work)

---

## 1. Motivation

After Plans 1 and 2, an AI agent can build a scenario and run a simulation. But users almost always have a follow-up question: *"What if I retire a few years earlier? What if I spend more?"*

The Analysis screen answers this by running the full Monte Carlo simulation across an N-dimensional grid of parameter values. For each combination it records several metrics (success rate, percentile net worths, lifetime taxes, max drawdown). From this grid it derives:

- A **tornado chart** — which parameters have the biggest impact on a chosen metric, ranked by absolute effect
- An **interaction matrix** — which pairs of parameters interact beyond their individual effects
- **1D curves** — how a metric changes as one parameter varies, with spread bands from other dimensions
- **2D heatmaps** — the metric surface over two parameters simultaneously

This plan adds 8 tools that expose this entire pipeline to an MCP caller, plus structured return types that let an LLM produce natural-language explanations of plan tradeoffs.

---

## 2. New Tools Overview

| Tool | Purpose |
|------|---------|
| `add_sweep_parameter` | Add one parameter axis to the sweep grid |
| `remove_sweep_parameter` | Remove a parameter by name or index |
| `configure_sweep` | Set MC iterations and default step count |
| `run_sweep` | Execute the full N-dimensional sweep |
| `get_sensitivity` | Tornado chart data — parameters ranked by impact |
| `get_sweep_curve` | 1D metric curve for one parameter (with spread bands) |
| `get_sweep_grid` | 2D metric matrix for two parameters (heatmap data) |
| `get_interaction_matrix` | NxN parameter interaction strengths |

All tools that read sweep results require `run_sweep` to have been called first. All tools that build the sweep grid require `set_portfolio` and `set_parameters` (Plan 1 prerequisites) to be set.

---

## 3. New Return Data Types

### 3.1 `SweepParameterInfo`

Describes one axis of the sweep grid.

```json
{
  "index": 0,
  "event_name": "Retirement",
  "sweep_type": "trigger_age",
  "min_value": 58,
  "max_value": 68,
  "step_count": 11,
  "values": [58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68]
}
```

### 3.2 `SweepSummary`

Top-level summary returned by `run_sweep`.

```json
{
  "ndim": 2,
  "total_points": 88,
  "mc_iterations_per_point": 200,
  "parameters": [
    { "index": 0, "label": "Retirement Age",           "values": [58, ..., 68] },
    { "index": 1, "label": "Living Expenses Amount",   "values": [60000, ..., 100000] }
  ],
  "metric_ranges": {
    "success_rate":        { "min": 0.42, "max": 0.94, "baseline": 0.78 },
    "p50_final_net_worth": { "min": 180000, "max": 2900000, "baseline": 1450000 },
    "lifetime_taxes":      { "min": 380000, "max": 920000, "baseline": 620000 }
  }
}
```

- `baseline`: metric value when all parameters are at their midpoint values.
- `metric_ranges`: computed for every metric, regardless of which were requested, since the sweep ran all of them.

### 3.3 `SensitivityResult`

Tornado chart data for a single metric.

```json
{
  "metric": "success_rate",
  "baseline": 0.78,
  "range": { "min": 0.42, "max": 0.94 },
  "distribution_summary": {
    "mean": 0.71,
    "std_dev": 0.14,
    "p25": 0.61,
    "p75": 0.83
  },
  "parameters": [
    {
      "index": 0,
      "label": "Retirement Age",
      "low_value": 0.52,
      "high_value": 0.91,
      "impact": 0.39,
      "abs_impact": 0.39
    },
    {
      "index": 1,
      "label": "Living Expenses Amount",
      "low_value": 0.89,
      "high_value": 0.62,
      "impact": -0.27,
      "abs_impact": 0.27
    }
  ]
}
```

- `low_value`: metric when parameter is at min (averaged across all other parameter combinations).
- `high_value`: metric when parameter is at max.
- `impact`: signed change (positive = higher param → better metric, negative = higher param → worse).
- `distribution_summary`: summary stats across all grid points — helps LLM describe shape ("most scenarios cluster around 70%").

### 3.4 `SweepCurveResult`

1D metric curve along one parameter.

```json
{
  "metric": "success_rate",
  "param_label": "Retirement Age",
  "points": [
    { "param_value": 58, "metric_value": 0.52 },
    { "param_value": 59, "metric_value": 0.58 },
    { "param_value": 63, "metric_value": 0.78 },
    { "param_value": 68, "metric_value": 0.91 }
  ],
  "spread": [
    { "param_value": 58, "min": 0.38, "max": 0.67 },
    { "param_value": 63, "min": 0.64, "max": 0.89 },
    { "param_value": 68, "min": 0.79, "max": 0.97 }
  ],
  "threshold_crossings": [
    { "threshold": 0.80, "crossing_param_value": 64.2, "direction": "above" }
  ]
}
```

- `spread`: min/max of the metric across all *other* parameter combinations at each X value. Omitted for 1D sweeps.
- `threshold_crossings`: where the curve crosses key thresholds (e.g., 80% success rate). The `crossing_param_value` is linearly interpolated between grid steps.

### 3.5 `SweepGridResult`

2D metric matrix for two parameters.

```json
{
  "metric": "success_rate",
  "x_label": "Retirement Age",
  "y_label": "Living Expenses Amount",
  "x_values": [58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68],
  "y_values": [60000, 70000, 80000, 90000, 100000],
  "matrix": [
    [0.91, 0.88, 0.84, 0.80, 0.75, 0.70, 0.64, 0.57, 0.51, 0.46, 0.42],
    [0.89, 0.85, 0.81, 0.76, 0.71, 0.66, 0.60, 0.54, 0.48, 0.43, 0.39],
    ...
  ],
  "min_value": 0.28,
  "max_value": 0.94,
  "optimal_cell": {
    "x_index": 10, "y_index": 0,
    "x_value": 68, "y_value": 60000,
    "metric_value": 0.94
  },
  "target_zone": {
    "threshold": 0.80,
    "cells_above_threshold": 22,
    "total_cells": 55
  }
}
```

- `matrix`: row-major `[y_index][x_index]`, matching TUI heatmap orientation (y_max at top).
- `optimal_cell`: grid location with the best metric value.
- `target_zone`: if the metric is `success_rate`, report how many cells exceed the 80% threshold (a common planning target).

### 3.6 `InteractionMatrixResult`

NxN parameter interaction strengths.

```json
{
  "metric": "success_rate",
  "labels": ["Retirement Age", "Living Expenses Amount"],
  "matrix": [
    [null, 0.12],
    [0.12, null]
  ],
  "max_interaction": 0.12,
  "strong_interactions": [
    {
      "param_a": "Retirement Age",
      "param_b": "Living Expenses Amount",
      "strength": 0.12,
      "interpretation": "moderate"
    }
  ]
}
```

- Diagonal cells are `null` (self-interaction is undefined).
- `strong_interactions`: pre-filtered list of pairs with non-trivial interaction (strength > 0.05), with a human-readable `interpretation` label (`"weak"`, `"moderate"`, `"strong"`).

---

## 4. Tool: `add_sweep_parameter`

Add one parameter dimension to the sweep grid. Multiple calls build a multi-dimensional grid. The total number of MC runs equals `step_count_1 × step_count_2 × ... × mc_iterations`.

### Input Schema

```json
{
  "type": "object",
  "required": ["event_name", "sweep_type", "min_value", "max_value"],
  "properties": {
    "event_name": {
      "type": "string",
      "description": "Name of the event whose parameter is being varied. Must match an event name in the current scenario."
    },
    "sweep_type": {
      "type": "string",
      "enum": ["trigger_age", "trigger_date", "effect_value", "repeating_start_age", "repeating_end_age"],
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
      "description": "Number of evenly-spaced steps between min and max (default: from configure_sweep default_steps, typically 6)"
    }
  }
}
```

### Output

```
Added parameter [0]: Retirement → trigger_age [58–68, 11 steps]
Grid so far: 11 total points (1 dimension)
Estimated runtime: ~22 seconds (11 points × 200 MC iterations)
```

Emit a runtime estimate using a rough heuristic (each MC point ≈ 0.1 seconds on typical hardware).

### Validation

- Error if `event_name` does not match any event in the current scenario state. Requires at least one `add_*_event` to have been called.
- Error if `min_value >= max_value`.
- Error if `step_count < 2`.
- Warn if total grid points (across all added dimensions) exceed 500 — that many MC runs may take over a minute.

---

## 5. Tool: `remove_sweep_parameter`

Remove a sweep parameter from the grid. Clears any existing sweep results (they are stale once the grid changes).

### Input Schema

```json
{
  "type": "object",
  "properties": {
    "event_name": {
      "type": "string",
      "description": "Remove the parameter for this event name"
    },
    "index": {
      "type": "integer",
      "description": "Remove the parameter at this index (0-based). Use if event_name is ambiguous."
    }
  }
}
```

Exactly one of `event_name` or `index` must be provided.

### Output

```
Removed parameter [0]: Retirement → trigger_age
Grid now has 1 dimension, 8 total points.
Note: Previous sweep results cleared — run run_sweep again.
```

---

## 6. Tool: `configure_sweep`

Set sweep-level configuration: MC iterations per grid point and the default step count for new parameters. These mirror the Configuration panel in the TUI.

### Input Schema

```json
{
  "type": "object",
  "properties": {
    "mc_iterations": {
      "type": "integer",
      "description": "Monte Carlo iterations per grid point (default: 200, min: 10, max: 2000). More iterations = more accurate but slower."
    },
    "default_steps": {
      "type": "integer",
      "description": "Default step count for new sweep parameters (default: 6, min: 2, max: 20)"
    }
  }
}
```

### Output

```
Sweep configured: 200 MC iterations per point, 6 default steps.
Current grid: 2 parameters, 88 total points.
Estimated runtime: ~18 seconds.
```

---

## 7. Tool: `run_sweep`

Execute the full N-dimensional parameter sweep. This is the long-running computation — each grid point runs `mc_iterations` Monte Carlo simulations. Results are stored in `ScenarioState` for subsequent query tools.

### Input Schema

```json
{
  "type": "object",
  "properties": {}
}
```

No inputs — uses the sweep parameters and configuration from prior calls.

### Output

Returns `SweepSummary`:

```json
{
  "ndim": 2,
  "total_points": 88,
  "mc_iterations_per_point": 200,
  "parameters": [
    { "index": 0, "label": "Retirement Age",         "values": [58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68] },
    { "index": 1, "label": "Living Expenses Amount",  "values": [60000, 70000, 80000, 90000, 100000] }
  ],
  "metric_ranges": {
    "success_rate":        { "min": 0.42, "max": 0.94, "baseline": 0.78 },
    "p50_final_net_worth": { "min": 180000, "max": 2900000, "baseline": 1450000 },
    "lifetime_taxes":      { "min": 380000, "max": 920000, "baseline": 620000 },
    "max_drawdown":        { "min": 0.08, "max": 0.61, "baseline": 0.29 }
  }
}
```

### Implementation Notes

1. Verify sweep parameters are defined — error if `sweep_parameters` is empty.
2. Call `state.merge()` to get `SimulationData`.
3. Call `to_simulation_config(&sim_data)` to get `SimulationConfig`.
4. Build a `SweepConfig` from `state.sweep_parameters` and `state.sweep_mc_iterations`.
5. Call `finplan_core::analysis::sweep_simulate(&config, &sweep_config)` to get `SweepResults`.
6. Build a `MonteCarloConfig` for each point with `iterations = sweep_mc_iterations`.
7. Store `SweepResults` in `state.last_sweep_results`.
8. Compute `metric_ranges` for all metrics (`SuccessRate`, `P5/P50/P95FinalNetWorth`, `LifetimeTaxes`, `MaxDrawdown`) using `sweep_results.get(&indices)` across all grid points.

### Core API

```rust
use finplan_core::analysis::{sweep_simulate, SweepConfig, SweepParameter, SweepResults};

// Build sweep config from state
let sweep_config = SweepConfig {
    parameters: state.sweep_parameters.iter().map(|p| {
        SweepParameter {
            event_id: resolve_event_id(&sim_data, &p.event_name),
            sweep_type: p.sweep_type.into(),
            values: linspace(p.min_value, p.max_value, p.step_count),
        }
    }).collect(),
    mc_config: MonteCarloConfig {
        iterations: state.sweep_mc_iterations,
        ..Default::default()
    },
};

let results: SweepResults = sweep_simulate(&config, &sweep_config);
state.last_sweep_results = Some(results);
state.last_sim_data = Some(sim_data);
```

### Error Handling

- Error if no sweep parameters have been added.
- Error if `merge()` fails (portfolio not set, etc.).
- Warn if estimated total iterations > 100,000 (500+ grid points × 200 iterations).

---

## 8. Tool: `get_sensitivity`

Return tornado chart data for a metric — each parameter's impact range, sorted by absolute effect. This is the most agent-useful tool in this plan.

### Input Schema

```json
{
  "type": "object",
  "required": ["metric"],
  "properties": {
    "metric": {
      "type": "string",
      "enum": ["success_rate", "p5_final_net_worth", "p50_final_net_worth",
               "p95_final_net_worth", "lifetime_taxes", "max_drawdown"],
      "description": "Metric to compute sensitivity for"
    }
  }
}
```

### Output

Returns `SensitivityResult`:

```json
{
  "metric": "success_rate",
  "baseline": 0.78,
  "range": { "min": 0.42, "max": 0.94 },
  "distribution_summary": {
    "mean": 0.71,
    "std_dev": 0.14,
    "p25": 0.61,
    "p75": 0.83
  },
  "parameters": [
    {
      "index": 0,
      "label": "Retirement Age",
      "low_value": 0.52,
      "high_value": 0.91,
      "impact": 0.39,
      "abs_impact": 0.39
    },
    {
      "index": 1,
      "label": "Living Expenses Amount",
      "low_value": 0.89,
      "high_value": 0.62,
      "impact": -0.27,
      "abs_impact": 0.27
    }
  ]
}
```

### Implementation Notes

Delegates directly to `AnalysisResults::compute_sensitivity()` (already implemented in `screen_state.rs`). The MCP layer only needs to:
1. Wrap `SweepResults` in `AnalysisResults`.
2. Call `results.compute_sensitivity(&metric)` → `Vec<SensitivityEntry>`.
3. Call `results.compute_baseline(&metric)` and `results.compute_metric_range(&metric)`.
4. Call `results.get_all_metric_values(&metric)` and compute `distribution_summary` (mean, std_dev, quartiles).
5. Serialize to JSON.

### Agent Use

The structured `impact` field lets an LLM generate natural-language explanations directly:

> "Retirement age is by far the biggest lever — moving from 58 to 68 shifts your success rate from 52% to 91% (a 39 percentage-point swing). Your living expenses are the second factor, but with less than half the impact: cutting from $100k to $60k annual spending adds 27 percentage points to your success rate."

---

## 9. Tool: `get_sweep_curve`

Return the 1D metric curve as a parameter varies, with min/max spread bands from other dimensions and threshold crossing points.

### Input Schema

```json
{
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
      "description": "Fix other dimensions at a specific step index. Keys are dimension indices (as strings), values are step indices. Omitted dimensions default to their midpoint.",
      "additionalProperties": { "type": "integer" }
    },
    "threshold": {
      "type": "number",
      "description": "Optional: report where the curve crosses this value (e.g., 0.80 for 80% success rate)"
    }
  }
}
```

### Output

Returns `SweepCurveResult`:

```json
{
  "metric": "success_rate",
  "param_label": "Retirement Age",
  "points": [
    { "param_value": 58, "metric_value": 0.52 },
    { "param_value": 60, "metric_value": 0.63 },
    { "param_value": 63, "metric_value": 0.78 },
    { "param_value": 65, "metric_value": 0.84 },
    { "param_value": 68, "metric_value": 0.91 }
  ],
  "spread": [
    { "param_value": 58, "min": 0.38, "max": 0.67 },
    { "param_value": 63, "min": 0.64, "max": 0.89 },
    { "param_value": 68, "min": 0.79, "max": 0.97 }
  ],
  "threshold_crossings": [
    {
      "threshold": 0.80,
      "crossing_param_value": 64.2,
      "direction": "above",
      "interpretation": "Success rate exceeds 80% when Retirement Age is above 64.2"
    }
  ]
}
```

### Threshold Crossing Calculation

For adjacent points where the metric crosses `threshold`, linearly interpolate:

```rust
fn find_threshold_crossing(points: &[(f64, f64)], threshold: f64) -> Option<(f64, &str)> {
    for window in points.windows(2) {
        let (x0, y0) = window[0];
        let (x1, y1) = window[1];
        if (y0 < threshold) != (y1 < threshold) {
            // Interpolate
            let t = (threshold - y0) / (y1 - y0);
            let x_crossing = x0 + t * (x1 - x0);
            let direction = if y1 > y0 { "above" } else { "below" };
            return Some((x_crossing, direction));
        }
    }
    None
}
```

---

## 10. Tool: `get_sweep_grid`

Return the full 2D metric matrix for two parameters — the data behind the heatmap chart.

### Input Schema

```json
{
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
      "description": "Optional: count cells above/below this value (e.g., 0.80 for success rate target zone)"
    }
  }
}
```

Requires at least a 2D sweep. Returns an error if `ndim < 2`.

### Output

Returns `SweepGridResult`:

```json
{
  "metric": "success_rate",
  "x_label": "Retirement Age",
  "y_label": "Living Expenses Amount",
  "x_values": [58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68],
  "y_values": [60000, 70000, 80000, 90000, 100000],
  "matrix": [
    [0.91, 0.88, 0.84, 0.80, 0.75, 0.70, 0.64, 0.57, 0.51, 0.46, 0.42],
    [0.87, 0.83, 0.79, 0.74, 0.69, 0.64, 0.58, 0.52, 0.46, 0.41, 0.37],
    [0.82, 0.78, 0.73, 0.68, 0.63, 0.58, 0.52, 0.46, 0.40, 0.36, 0.31],
    [0.76, 0.72, 0.67, 0.62, 0.57, 0.51, 0.45, 0.40, 0.34, 0.30, 0.28],
    [0.69, 0.65, 0.60, 0.55, 0.50, 0.44, 0.39, 0.34, 0.29, 0.28, 0.28]
  ],
  "min_value": 0.28,
  "max_value": 0.94,
  "optimal_cell": {
    "x_index": 0, "y_index": 0,
    "x_value": 58, "y_value": 60000,
    "metric_value": 0.94
  },
  "target_zone": {
    "threshold": 0.80,
    "cells_above_threshold": 12,
    "total_cells": 55,
    "fraction": 0.218
  }
}
```

The matrix follows TUI convention: `matrix[y_index][x_index]`, with `y_values[0]` being the first row. The agent should note that lower y-values (e.g., lower spending) appear first.

---

## 11. Tool: `get_interaction_matrix`

Return the NxN cross-parameter interaction matrix. Shows which pairs of parameters have effects beyond their individual contributions.

### Input Schema

```json
{
  "type": "object",
  "required": ["metric"],
  "properties": {
    "metric": {
      "type": "string",
      "enum": ["success_rate", "p5_final_net_worth", "p50_final_net_worth",
               "p95_final_net_worth", "lifetime_taxes", "max_drawdown"]
    }
  }
}
```

Requires at least a 2D sweep (`ndim >= 2`).

### Output

Returns `InteractionMatrixResult`:

```json
{
  "metric": "success_rate",
  "labels": ["Retirement Age", "Living Expenses Amount"],
  "matrix": [
    [null, 0.12],
    [0.12, null]
  ],
  "max_interaction": 0.12,
  "strong_interactions": [
    {
      "param_a": "Retirement Age",
      "param_b": "Living Expenses Amount",
      "strength": 0.12,
      "interpretation": "moderate"
    }
  ]
}
```

### Interaction Strength Labels

| Strength | Label |
|---------|-------|
| < 0.05 | `"negligible"` |
| 0.05–0.10 | `"weak"` |
| 0.10–0.20 | `"moderate"` |
| > 0.20 | `"strong"` |

### Implementation Notes

Delegates to `AnalysisResults::compute_interaction_matrix()` (already implemented in `screen_state.rs`). The method returns `Option<(Vec<Vec<f64>>, f64)>` — the matrix and its max absolute value.

---

## 12. State Changes

`ScenarioState` needs five new fields:

```rust
pub struct ScenarioState {
    // ... existing fields from Plans 1 & 2 ...

    /// Sweep parameter definitions (the grid axes).
    pub sweep_parameters: Vec<SweepParameterData>,

    /// MC iterations per sweep grid point.
    pub sweep_mc_iterations: usize,

    /// Default step count for new sweep parameters.
    pub sweep_default_steps: usize,

    /// Results from the last run_sweep call.
    pub last_sweep_results: Option<finplan_core::analysis::SweepResults>,

    /// SimulationData used to produce last_sweep_results (needed to resolve names).
    /// May already exist from Plan 2's last_sim_data — share if possible.
    pub last_sweep_sim_data: Option<finplan::data::app_data::SimulationData>,
}
```

**Default values:**
```rust
sweep_parameters: Vec::new(),
sweep_mc_iterations: 200,
sweep_default_steps: 6,
last_sweep_results: None,
last_sweep_sim_data: None,
```

**Cache invalidation:** `last_sweep_results` and `last_sweep_sim_data` are cleared whenever:
- Any `set_*` or `add_*` scenario tool is called (scenario changed)
- `add_sweep_parameter` or `remove_sweep_parameter` is called (grid changed)
- `configure_sweep` changes `mc_iterations` (results are stale)

`sweep_parameters`, `sweep_mc_iterations`, and `sweep_default_steps` are cleared by `reset_state` but are not cleared by scenario changes (the sweep configuration is independent of the scenario content).

---

## 13. Implementation Details

### 13.1 New File: `src/tools/sweep.rs`

All 8 tools live in one new file:

```
crates/finplan_mcp/src/tools/
├── simulation.rs    (Plan 2)
├── sweep.rs         ← NEW: all 8 sweep tools
```

Register in `src/tools/mod.rs`:

```rust
pub mod sweep;

// In list_tools():
tools.extend(sweep::tools());

// In call_tool():
"add_sweep_parameter"    => sweep::add_sweep_parameter(args, state),
"remove_sweep_parameter" => sweep::remove_sweep_parameter(args, state),
"configure_sweep"        => sweep::configure_sweep(args, state),
"run_sweep"              => sweep::run_sweep(args, state),
"get_sensitivity"        => sweep::get_sensitivity(args, state),
"get_sweep_curve"        => sweep::get_sweep_curve(args, state),
"get_sweep_grid"         => sweep::get_sweep_grid(args, state),
"get_interaction_matrix" => sweep::get_interaction_matrix(args, state),
```

### 13.2 Reusing TUI Analysis Logic

The `AnalysisResults` struct in `crates/finplan/src/state/screen_state.rs` already implements:
- `compute_sensitivity(metric)` → `Vec<SensitivityEntry>`
- `compute_baseline(metric)` → `f64`
- `compute_metric_range(metric)` → `(f64, f64)`
- `get_all_metric_values(metric)` → `Vec<f64>`
- `get_1d_metric_data_for_config(metric, dim, fixed)` → `(Vec<f64>, Vec<f64>)`
- `get_1d_metric_spread_across_other_dims(metric, dim)` → `(Vec<f64>, Vec<f64>, Vec<f64>)`
- `get_2d_metric_matrix_for_config(metric, x_dim, y_dim, fixed)` → `Option<(Vec<Vec<f64>>, f64, f64)>`
- `compute_interaction_matrix(metric)` → `Option<(Vec<Vec<f64>>, f64)>`

The MCP sweep tools should wrap `SweepResults` in `AnalysisResults` and call these methods directly, rather than re-implementing the computation logic.

```rust
fn to_analysis_results(sweep_results: &SweepResults) -> AnalysisResults {
    AnalysisResults::new(sweep_results.clone())
}
```

### 13.3 Resolving Event Names to IDs

`add_sweep_parameter` takes an `event_name` string. The core sweep engine needs an `EventId`. Resolution:

```rust
fn resolve_event_id(sim_data: &SimulationData, event_name: &str) -> Option<EventId> {
    sim_data.events.iter().enumerate()
        .find(|(_, e)| e.name.0 == event_name)
        .map(|(i, _)| EventId((i + 1) as u16))
}
```

Validate at `add_sweep_parameter` time (before `run_sweep`) so the error is surfaced early. Event resolution must be re-checked at `run_sweep` time in case events were modified after parameters were added.

### 13.4 Building `SweepConfig` from State

```rust
fn build_sweep_config(
    state: &ScenarioState,
    sim_data: &SimulationData,
) -> Result<SweepConfig, String> {
    if state.sweep_parameters.is_empty() {
        return Err("No sweep parameters defined. Call add_sweep_parameter first.".into());
    }

    let parameters = state.sweep_parameters.iter().map(|p| {
        let event_id = resolve_event_id(sim_data, &p.event_name)
            .ok_or_else(|| format!("Event '{}' not found in scenario", p.event_name))?;

        let values = linspace(p.min_value, p.max_value, p.step_count);

        Ok(SweepParameter {
            event_id,
            sweep_type: p.sweep_type.into(),
            values,
        })
    }).collect::<Result<Vec<_>, String>>()?;

    Ok(SweepConfig {
        parameters,
        mc_config: MonteCarloConfig {
            iterations: state.sweep_mc_iterations,
            ..Default::default()
        },
    })
}

fn linspace(min: f64, max: f64, steps: usize) -> Vec<f64> {
    if steps <= 1 { return vec![min]; }
    (0..steps).map(|i| min + (max - min) * i as f64 / (steps - 1) as f64).collect()
}
```

### 13.5 Computing `distribution_summary`

For `get_sensitivity`, compute summary stats over the full distribution of metric values across all grid points:

```rust
fn distribution_summary(values: &[f64]) -> DistributionSummary {
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let std_dev = variance.sqrt();

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    DistributionSummary {
        mean,
        std_dev,
        p25: sorted[(n * 0.25) as usize],
        p75: sorted[(n * 0.75) as usize],
    }
}
```

### 13.6 Performance Considerations

- `run_sweep` is synchronous and CPU-bound. For a 10×10 grid with 200 MC iterations each, this is 20,000 simulations — potentially 20–60 seconds.
- Do not run in a background thread. MCP's stdio transport has no streaming, so the call must complete before a response is sent.
- Recommended agent guidance: before calling `run_sweep`, call `configure_sweep` with a modest `mc_iterations` (100–200) and warn the user it may take a moment.
- The `get_state_summary` tool (Plan 1) should be updated to also report sweep state:
  ```
  Sweep: 2 parameters, 88 grid points (not yet run)
  ```

---

## 14. Testing Strategy

### 14.1 Unit Tests

**Test: `linspace` helper**
```rust
#[test]
fn test_linspace_basic() {
    let v = linspace(58.0, 68.0, 11);
    assert_eq!(v.len(), 11);
    assert!((v[0] - 58.0).abs() < 0.001);
    assert!((v[10] - 68.0).abs() < 0.001);
    assert!((v[5] - 63.0).abs() < 0.001);
}
```

**Test: `threshold_crossing` interpolation**
```rust
#[test]
fn test_threshold_crossing() {
    let points = vec![(58.0, 0.72), (63.0, 0.78), (65.0, 0.82), (68.0, 0.91)];
    let crossing = find_threshold_crossing(&points, 0.80);
    assert!(crossing.is_some());
    let (x, dir) = crossing.unwrap();
    assert!((x - 64.0).abs() < 1.0); // should cross near 64
    assert_eq!(dir, "above");
}
```

**Test: `distribution_summary` stats**
```rust
#[test]
fn test_distribution_summary_uniform() {
    let values: Vec<f64> = (0..=100).map(|i| i as f64 / 100.0).collect();
    let summary = distribution_summary(&values);
    assert!((summary.mean - 0.5).abs() < 0.01);
    assert!((summary.p25 - 0.25).abs() < 0.02);
    assert!((summary.p75 - 0.75).abs() < 0.02);
}
```

### 14.2 Integration Tests

**Test: `add_sweep_parameter` validates event name**
```rust
#[test]
fn test_add_sweep_parameter_unknown_event() {
    let state = setup_test_scenario_state(); // has "Retirement" event
    let args = json!({
        "event_name": "NonExistentEvent",
        "sweep_type": "trigger_age",
        "min_value": 60,
        "max_value": 70
    });
    let result = add_sweep_parameter(as_map(args), &state).unwrap();
    assert_eq!(result.is_error, Some(true));
}
```

**Test: `run_sweep` 1D produces correct shape**
```rust
#[test]
fn test_run_sweep_1d() {
    let state = setup_and_configure_1d_sweep(); // retirement age 60–65, 6 steps, 50 MC iter
    let result = run_sweep(Default::default(), &state).unwrap();
    let json: Value = serde_json::from_str(result_text(&result)).unwrap();
    assert_eq!(json["ndim"].as_u64().unwrap(), 1);
    assert_eq!(json["total_points"].as_u64().unwrap(), 6);
    assert!(json["metric_ranges"]["success_rate"]["baseline"].as_f64().unwrap() > 0.0);
}
```

**Test: `get_sensitivity` returns sorted results**
```rust
#[test]
fn test_sensitivity_sorted_by_impact() {
    let state = setup_and_run_2d_sweep(); // retirement age + expenses
    let args = json!({ "metric": "success_rate" });
    let result = get_sensitivity(as_map(args), &state).unwrap();
    let json: Value = serde_json::from_str(result_text(&result)).unwrap();
    let params = json["parameters"].as_array().unwrap();
    assert!(params.len() == 2);
    // First entry should have higher abs_impact than second
    let impact0 = params[0]["abs_impact"].as_f64().unwrap();
    let impact1 = params[1]["abs_impact"].as_f64().unwrap();
    assert!(impact0 >= impact1);
}
```

**Test: `get_sweep_grid` requires 2D sweep**
```rust
#[test]
fn test_get_sweep_grid_requires_2d() {
    let state = setup_and_run_1d_sweep();
    let args = json!({ "metric": "success_rate" });
    let result = get_sweep_grid(as_map(args), &state).unwrap();
    assert_eq!(result.is_error, Some(true));
}
```

**Test: `get_sweep_curve` threshold crossing**
```rust
#[test]
fn test_sweep_curve_threshold_crossing() {
    let state = setup_and_run_1d_sweep(); // success rate should cross 0.80 somewhere
    let args = json!({ "metric": "success_rate", "param_index": 0, "threshold": 0.80 });
    let result = get_sweep_curve(as_map(args), &state).unwrap();
    let json: Value = serde_json::from_str(result_text(&result)).unwrap();
    // Either there's a crossing or there isn't — just check the field is present
    assert!(json["threshold_crossings"].is_array());
}
```

### 14.3 Manual Integration Test Extension

Extend `scripts/test_mcp.sh` with sweep tool calls after the merge step:

```bash
echo '{"jsonrpc":"2.0","id":19,"method":"tools/call","params":{"name":"add_sweep_parameter","arguments":{"event_name":"Retirement","sweep_type":"trigger_age","min_value":60,"max_value":68,"step_count":5}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":20,"method":"tools/call","params":{"name":"configure_sweep","arguments":{"mc_iterations":50}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":21,"method":"tools/call","params":{"name":"run_sweep","arguments":{}}}'
sleep 10  # sweep takes longer
echo '{"jsonrpc":"2.0","id":22,"method":"tools/call","params":{"name":"get_sensitivity","arguments":{"metric":"success_rate"}}}'
sleep $S
echo '{"jsonrpc":"2.0","id":23,"method":"tools/call","params":{"name":"get_sweep_curve","arguments":{"metric":"success_rate","threshold":0.8}}}'
```

---

## 15. File-by-File Checklist

### New Files

- [ ] `crates/finplan_mcp/src/tools/sweep.rs`
  - [ ] `tools()` — register 8 tools
  - [ ] `add_sweep_parameter(args, state)` — add one sweep axis
  - [ ] `remove_sweep_parameter(args, state)` — remove a sweep axis
  - [ ] `configure_sweep(args, state)` — set iterations and default steps
  - [ ] `run_sweep(args, state)` — execute the sweep, store results
  - [ ] `get_sensitivity(args, state)` — tornado chart data
  - [ ] `get_sweep_curve(args, state)` — 1D metric curve with spread and crossings
  - [ ] `get_sweep_grid(args, state)` — 2D metric matrix
  - [ ] `get_interaction_matrix(args, state)` — NxN interaction strengths
  - [ ] `build_sweep_config(state, sim_data)` — helper
  - [ ] `linspace(min, max, steps)` — helper
  - [ ] `to_analysis_results(sweep_results)` — helper
  - [ ] `distribution_summary(values)` — helper
  - [ ] `find_threshold_crossing(points, threshold)` — helper
  - [ ] `interaction_strength_label(strength)` — helper

### Modified Files

- [ ] `crates/finplan_mcp/src/tools/mod.rs`
  - [ ] Add `pub mod sweep;`
  - [ ] Add 8 tool entries to `list_tools()`
  - [ ] Add 8 match arms to `call_tool()`

- [ ] `crates/finplan_mcp/src/state.rs`
  - [ ] Add `sweep_parameters: Vec<SweepParameterData>`
  - [ ] Add `sweep_mc_iterations: usize` (default: 200)
  - [ ] Add `sweep_default_steps: usize` (default: 6)
  - [ ] Add `last_sweep_results: Option<SweepResults>`
  - [ ] Add `last_sweep_sim_data: Option<SimulationData>`
  - [ ] Initialize all new fields in `ScenarioState::default()`
  - [ ] Clear sweep results in `invalidate_simulation_cache()`
  - [ ] Add `pub fn invalidate_sweep_cache(&mut self)` (clears only sweep results)
  - [ ] Clear all fields in `reset_state`

- [ ] `crates/finplan_mcp/src/tools/mod.rs` `get_state_summary()`
  - [ ] Extend to report sweep parameters and last sweep run status

- [ ] Each Plan 1/2 tool that modifies the scenario (set_parameters, add_account, etc.):
  - [ ] Also call `state.lock().unwrap().invalidate_sweep_cache()`

### New Test Files

- [ ] `crates/finplan_mcp/tests/sweep_tests.rs`
  - [ ] Unit tests for helpers (`linspace`, `threshold_crossing`, `distribution_summary`)
  - [ ] Integration tests for all 8 sweep tools

---

## 16. Example Agent Session

This extends the Plan 2 session (Sarah's Retirement Plan, MC run complete) to include sensitivity analysis.

### Step 11: Add sweep parameters

```json
// Agent calls: add_sweep_parameter
{
  "event_name": "Retirement",
  "sweep_type": "trigger_age",
  "min_value": 57, "max_value": 67, "step_count": 11
}
// Response: "Added parameter [0]: Retirement → trigger_age [57–67, 11 steps]. Grid: 11 points."

// Agent calls: add_sweep_parameter
{
  "event_name": "Living Expenses",
  "sweep_type": "effect_value",
  "min_value": 50000, "max_value": 90000, "step_count": 5
}
// Response: "Added parameter [1]: Living Expenses → effect_value [$50K–$90K, 5 steps]. Grid: 55 points."
```

### Step 12: Configure and run sweep

```json
// Agent calls: configure_sweep
{ "mc_iterations": 100 }
// Response: "Configured: 100 MC iterations per point. 55 total points. Estimated runtime: ~28 seconds."

// Agent calls: run_sweep
{}
// Response: {
//   "ndim": 2, "total_points": 55,
//   "metric_ranges": {
//     "success_rate": { "min": 0.31, "max": 0.96, "baseline": 0.74 },
//     "p50_final_net_worth": { "min": 120000, "max": 3100000, "baseline": 1380000 }
//   }
// }
```

### Step 13: Get sensitivity

```json
// Agent calls: get_sensitivity
{ "metric": "success_rate" }
```

Response:
```json
{
  "baseline": 0.74,
  "parameters": [
    { "label": "Retirement Age", "impact": 0.42, "low_value": 0.49, "high_value": 0.91 },
    { "label": "Living Expenses Amount", "impact": -0.31, "low_value": 0.88, "high_value": 0.57 }
  ]
}
```

**Agent explanation to user:**
> "I ran 55 scenarios varying your retirement age (57–67) and annual living expenses ($50K–$90K). Here's what I found:
>
> **Retirement age is your biggest lever.** Moving from 57 to 67 shifts your success rate from 49% to 91% — a 42-point swing. This makes sense: each extra year you work means more savings, lower withdrawal period, and higher Social Security.
>
> **Spending matters less than timing, but still significantly.** Cutting annual expenses from $90K to $50K adds 31 points to your success rate. That's meaningful, but roughly 75% of the impact of simply working two more years.
>
> **Bottom line:** At your current spending of $80K/year, retiring at 63 instead of 60 would move you from ~67% to ~80% success rate. Want me to find the exact retirement age that hits 80%?"

### Step 14: Find the threshold crossing

```json
// Agent calls: get_sweep_curve
{
  "metric": "success_rate",
  "param_index": 0,
  "fixed_values": { "1": 3 },  // fix spending at its midpoint ($70K)
  "threshold": 0.80
}
```

Response:
```json
{
  "threshold_crossings": [
    {
      "threshold": 0.80,
      "crossing_param_value": 62.4,
      "direction": "above",
      "interpretation": "Success rate exceeds 80% when Retirement Age is above 62.4"
    }
  ]
}
```

**Agent:** "At $70K annual spending, you need to work until **age 62–63** to hit an 80% success rate."

---

## 17. Future Work

Explicitly out of scope for this plan:

1. **Optimization tools** — `find_optimal_retirement_age(target_success_rate)` using the existing `optimization/` binary search and Nelder-Mead. Would call into `finplan_core::optimization::optimize()` rather than a manual sweep grid.

2. **Adaptive grid** — Run a coarse sweep first (3 steps per dimension), identify the high-gradient regions, then refine only those regions. More accurate with fewer total runs.

3. **Additional sweep types** — Currently supports age, date, and amount fields. Could extend to: inflation rate, state tax rate, MC iterations (for convergence analysis), or account starting balance.

4. **Named metric at age** — `NetWorthAtAge { age }` is supported by the core but not yet exposed in this plan's metric enum. Add as `"net_worth_at_age"` with an `age` parameter.

5. **Sweep result export** — `export_sweep(format: "csv" | "json")` — return the full grid as tabular data for use in external tools (Excel, Python).

6. **Progress streaming** — For large sweeps, stream progress as each batch of grid points completes. Requires MCP SSE transport (not available with stdio).
