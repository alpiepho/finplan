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

    let total_points: usize = if st.sweep_parameters.is_empty() {
        0
    } else {
        st.sweep_parameters.iter().map(|p| p.step_count).product()
    };
    let est_secs = total_points * st.sweep_mc_iterations / 10;

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

    if !st.events.iter().any(|e| e.name.0 == event_name) {
        let available: Vec<&str> = st.events.iter().map(|e| e.name.0.as_str()).collect();
        return error_result(format!(
            "Event '{event_name}' not found. Available events: {}",
            available.join(", ")
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
        format!("\nWarning: {total_points} total points may take over a minute.")
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

    let idx_result = if let Some(name) = args.get("event_name").and_then(|v| v.as_str()) {
        st.sweep_parameters
            .iter()
            .position(|p| p.event_name == name)
            .ok_or_else(|| format!("No sweep parameter found for event '{name}'"))
    } else if let Some(i) = args.get("index").and_then(|v| v.as_u64()) {
        let i = i as usize;
        if i < st.sweep_parameters.len() {
            Ok(i)
        } else {
            Err(format!(
                "Index {i} out of range ({} parameters)",
                st.sweep_parameters.len()
            ))
        }
    } else {
        Err("Provide event_name or index".into())
    };

    let idx = match idx_result {
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
            return error_result("No sweep parameters defined. Call add_sweep_parameter first.");
        }
        let sim_data = match st.merge() {
            Ok(d) => d,
            Err(errs) => {
                return error_result(format!("Cannot merge scenario:\n{}", errs.join("\n")));
            }
        };
        (
            sim_data,
            st.sweep_parameters.clone(),
            st.sweep_mc_iterations,
        )
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

    let all_metrics: &[(&str, AnalysisMetric)] = &[
        ("success_rate", AnalysisMetric::SuccessRate),
        (
            "p5_final_net_worth",
            AnalysisMetric::Percentile { percentile: 5 },
        ),
        (
            "p50_final_net_worth",
            AnalysisMetric::Percentile { percentile: 50 },
        ),
        (
            "p95_final_net_worth",
            AnalysisMetric::Percentile { percentile: 95 },
        ),
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
    let (min_val, max_val) = value_range(&all_values);
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

    let param_index = args
        .get("param_index")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
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

    let spread_json: Vec<Value> = if ndim > 1 {
        let x_len = shape[param_index];
        (0..x_len)
            .map(|x_idx| {
                let pv = results.param_values[param_index][x_idx];
                let vals: Vec<f64> = results
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

    let param_label = results
        .param_labels
        .get(param_index)
        .map_or("", |s| s.as_str());
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

    let x_dim = args
        .get("x_param_index")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    let y_dim = args
        .get("y_param_index")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as usize;
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

    let (flat_values, x_vals, y_vals) =
        match results.get_metric_2d_slice(&metric, x_dim, y_dim, &fixed) {
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

    let (min_val, max_val) = value_range(&flat_values);
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
            let f_min_min = avg_metric_pair(results, &metric, i, 0, j, 0);
            let f_min_max = avg_metric_pair(results, &metric, i, 0, j, shape[j] - 1);
            let f_max_min = avg_metric_pair(results, &metric, i, shape[i] - 1, j, 0);
            let f_max_max = avg_metric_pair(results, &metric, i, shape[i] - 1, j, shape[j] - 1);

            let interaction = f_max_max - f_min_max - f_max_min + f_min_min;
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

    strong_interactions.sort_by(|a, b| {
        b["strength"]
            .as_f64()
            .unwrap_or(0.0)
            .abs()
            .total_cmp(&a["strength"].as_f64().unwrap_or(0.0).abs())
    });

    let matrix_json: Vec<Vec<Value>> = matrix
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|v| v.map_or(Value::Null, |f| json!(f)))
                .collect()
        })
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

fn value_range(values: &[f64]) -> (f64, f64) {
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

fn optimal_cell(values: &[f64], _x_len: usize, y_len: usize, metric: &str) -> (usize, usize, f64) {
    let lower_is_better = matches!(metric, "lifetime_taxes" | "max_drawdown");
    let mut best_flat = 0usize;
    let mut best_val = if lower_is_better {
        f64::INFINITY
    } else {
        f64::NEG_INFINITY
    };

    for (i, &v) in values.iter().enumerate() {
        let is_better = if lower_is_better {
            v < best_val
        } else {
            v > best_val
        };
        if is_better {
            best_val = v;
            best_flat = i;
        }
    }

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
