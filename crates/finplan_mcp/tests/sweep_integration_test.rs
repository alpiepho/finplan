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

// ── helpers ──────────────────────────────────────────────────────────────────

/// Build a minimal scenario with two named events suitable for sweep tests.
/// Events: "Retirement" (age trigger) and "Living Expenses" (monthly expense).
fn setup_sweep_scenario() -> finplan_mcp::state::SharedState {
    let st = state::new_shared_state();

    tools::portfolio::set_portfolio(args(json!({"name": "Sweep Test Plan"})), &st).unwrap();
    tools::portfolio::add_account(
        args(json!({"name": "Checking", "account_type": "Checking", "value": 50000.0})),
        &st,
    )
    .unwrap();
    tools::portfolio::add_account(
        args(json!({
            "name": "401k",
            "account_type": "Traditional401k",
            "assets": [{"ticker": "FXAIX", "value": 300000.0}]
        })),
        &st,
    )
    .unwrap();
    tools::parameters::set_parameters(
        args(json!({
            "birth_date": "1970-01-01",
            "start_date": "2026-01-01",
            "duration_years": 35
        })),
        &st,
    )
    .unwrap();
    tools::events::add_expense_event(
        args(json!({"name": "Living Expenses", "from_account": "Checking", "amount": 5000.0})),
        &st,
    )
    .unwrap();
    tools::events::add_retirement_event(args(json!({"name": "Retirement", "age": 62})), &st)
        .unwrap();
    tools::ticker::map_tickers(args(json!({})), &st).unwrap();
    st
}

// ── configure_sweep ───────────────────────────────────────────────────────────

#[test]
fn test_configure_sweep_updates_state() {
    let st = setup_sweep_scenario();
    let r =
        tools::sweep::configure_sweep(args(json!({"mc_iterations": 100, "default_steps": 4})), &st)
            .unwrap();
    assert!(is_ok(&r), "configure_sweep failed: {}", text_of(&r));
    let s = st.lock().unwrap();
    assert_eq!(s.sweep_mc_iterations, 100);
    assert_eq!(s.sweep_default_steps, 4);
}

#[test]
fn test_configure_sweep_clamps_iterations() {
    let st = setup_sweep_scenario();
    let r = tools::sweep::configure_sweep(
        args(json!({"mc_iterations": 5})), // below min of 10
        &st,
    )
    .unwrap();
    assert!(is_ok(&r));
    let s = st.lock().unwrap();
    assert_eq!(s.sweep_mc_iterations, 10); // clamped to min
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
    )
    .unwrap();
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
    )
    .unwrap();
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
    )
    .unwrap();
    assert_eq!(r.is_error, Some(true));
}

// ── remove_sweep_parameter ────────────────────────────────────────────────────

#[test]
fn test_remove_sweep_parameter_by_name() {
    let st = setup_sweep_scenario();
    tools::sweep::add_sweep_parameter(
        args(
            json!({"event_name": "Retirement", "sweep_type": "trigger_age",
                    "min_value": 60, "max_value": 67}),
        ),
        &st,
    )
    .unwrap();
    let r = tools::sweep::remove_sweep_parameter(args(json!({"event_name": "Retirement"})), &st)
        .unwrap();
    assert!(is_ok(&r));
    assert!(st.lock().unwrap().sweep_parameters.is_empty());
}

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
    )
    .unwrap();
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
    // Add a sweep param directly to state to bypass event validation
    {
        use finplan::data::analysis_data::{SweepParameterData, SweepTypeData};
        let mut s = st.lock().unwrap();
        s.sweep_parameters.push(SweepParameterData {
            event_name: "Retirement".to_string(),
            sweep_type: SweepTypeData::TriggerAge,
            min_value: 60.0,
            max_value: 65.0,
            step_count: 3,
        });
    }
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
    println!("  ndim={}, total_points={}", j["ndim"], j["total_points"]);
    assert_eq!(j["ndim"].as_u64().unwrap(), 1);
    assert_eq!(j["total_points"].as_u64().unwrap(), 3);
    assert!(
        j["metric_ranges"]["success_rate"]["baseline"]
            .as_f64()
            .unwrap()
            >= 0.0
    );
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
    println!("\n═══ scenario change clears cached sweep results ═══");
    let st = setup_1d_sweep();
    tools::sweep::run_sweep(args(json!({})), &st).unwrap();
    assert!(st.lock().unwrap().last_sweep_results.is_some());

    // Modify the scenario — should invalidate sweep results
    tools::parameters::set_parameters(args(json!({"birth_date": "1972-01-01"})), &st).unwrap();
    assert!(st.lock().unwrap().last_sweep_results.is_none());
}

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
    let r = tools::sweep::get_sensitivity(args(json!({"metric": "success_rate"})), &st).unwrap();
    assert_eq!(r.is_error, Some(true));
}

#[test]
fn test_get_sensitivity_returns_parameters_sorted_by_impact() {
    println!("\n[get_sensitivity] results sorted by abs_impact");
    let st = run_1d_sweep();
    let r = tools::sweep::get_sensitivity(args(json!({"metric": "success_rate"})), &st).unwrap();
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
    // success_rate is a fraction 0-1, not 0-100
    let baseline = j["baseline"].as_f64().unwrap();
    assert!(
        (0.0..=1.0).contains(&baseline),
        "baseline should be 0-1 fraction, got {baseline}"
    );
}

#[test]
fn test_get_sensitivity_unknown_metric_returns_error() {
    let st = run_1d_sweep();
    let r = tools::sweep::get_sensitivity(args(json!({"metric": "bogus_metric"})), &st).unwrap();
    assert_eq!(r.is_error, Some(true));
}

// ── get_sweep_curve tests ─────────────────────────────────────────────────────

#[test]
fn test_get_sweep_curve_returns_correct_number_of_points() {
    println!("\n[get_sweep_curve] 1D curve has step_count points");
    let st = run_1d_sweep(); // 3 steps
    let r = tools::sweep::get_sweep_curve(args(json!({"metric": "success_rate"})), &st).unwrap();
    assert!(is_ok(&r), "get_sweep_curve failed: {}", text_of(&r));
    let j: Value = serde_json::from_str(&text_of(&r)).unwrap();
    println!("  {}", serde_json::to_string_pretty(&j).unwrap());

    let points = j["points"].as_array().unwrap();
    assert_eq!(points.len(), 3, "Should have 3 points for 3-step sweep");
    assert!(j["threshold_crossings"].is_array());
    // 1D sweep: spread should be empty (no other dims to vary across)
    let spread = j["spread"].as_array().unwrap();
    assert!(spread.is_empty(), "1D sweep should have empty spread");
    // Each point should have param_value and metric_value
    let first = &points[0];
    assert!(first["param_value"].as_f64().is_some());
    assert!(first["metric_value"].as_f64().is_some());
    // success_rate values should be fractions 0-1
    for pt in points {
        let mv = pt["metric_value"].as_f64().unwrap();
        assert!(
            (0.0..=1.0).contains(&mv),
            "success_rate should be 0-1 fraction, got {mv}"
        );
    }
}

#[test]
fn test_get_sweep_curve_with_threshold() {
    let st = run_1d_sweep();
    let r = tools::sweep::get_sweep_curve(
        args(json!({"metric": "success_rate", "threshold": 0.5})),
        &st,
    )
    .unwrap();
    assert!(is_ok(&r));
    let j: Value = serde_json::from_str(&text_of(&r)).unwrap();
    // threshold_crossings array must be present (may be empty if curve never crosses 0.5)
    assert!(j["threshold_crossings"].is_array());
}

#[test]
fn test_get_sweep_curve_fails_without_sweep() {
    let st = setup_sweep_scenario();
    let r = tools::sweep::get_sweep_curve(args(json!({"metric": "success_rate"})), &st).unwrap();
    assert_eq!(r.is_error, Some(true));
}

// ── Shared helper: 2D sweep already run ──────────────────────────────────────

fn run_2d_sweep() -> finplan_mcp::state::SharedState {
    let st = setup_sweep_scenario();
    // Dim 0: retirement age 60-65 in 3 steps
    tools::sweep::add_sweep_parameter(
        args(json!({
            "event_name": "Retirement",
            "sweep_type": "trigger_age",
            "min_value": 60,
            "max_value": 65,
            "step_count": 3
        })),
        &st,
    )
    .unwrap();
    // Dim 1: living expenses $4000-$7000 in 3 steps
    tools::sweep::add_sweep_parameter(
        args(json!({
            "event_name": "Living Expenses",
            "sweep_type": "effect_value",
            "min_value": 4000,
            "max_value": 7000,
            "step_count": 3
        })),
        &st,
    )
    .unwrap();
    tools::sweep::configure_sweep(args(json!({"mc_iterations": 30})), &st).unwrap();
    tools::sweep::run_sweep(args(json!({})), &st).unwrap();
    st
}

// ── get_sweep_grid tests ──────────────────────────────────────────────────────

#[test]
fn test_get_sweep_grid_fails_for_1d_sweep() {
    let st = run_1d_sweep();
    let r = tools::sweep::get_sweep_grid(args(json!({"metric": "success_rate"})), &st).unwrap();
    assert_eq!(r.is_error, Some(true));
    assert!(text_of(&r).contains("2D"));
}

#[test]
fn test_get_sweep_grid_returns_correct_matrix_shape() {
    println!("\n[get_sweep_grid] 2D grid has correct dimensions");
    let st = run_2d_sweep();
    let r = tools::sweep::get_sweep_grid(args(json!({"metric": "success_rate"})), &st).unwrap();
    assert!(is_ok(&r), "get_sweep_grid failed: {}", text_of(&r));
    let j: Value = serde_json::from_str(&text_of(&r)).unwrap();
    println!("  optimal={:?}", j["optimal_cell"]["metric_value"]);

    let matrix = j["matrix"].as_array().unwrap();
    assert_eq!(matrix.len(), 3, "matrix should have 3 rows (y_steps)");
    assert_eq!(
        matrix[0].as_array().unwrap().len(),
        3,
        "each row should have 3 cols (x_steps)"
    );
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
    )
    .unwrap();
    assert!(is_ok(&r));
    let j: Value = serde_json::from_str(&text_of(&r)).unwrap();
    assert_eq!(j["target_zone"]["total_cells"].as_u64().unwrap(), 9); // 3x3
    assert!(j["target_zone"]["cells_above_threshold"].as_u64().is_some());
}

// ── get_interaction_matrix tests ──────────────────────────────────────────────

#[test]
fn test_get_interaction_matrix_fails_for_1d_sweep() {
    let st = run_1d_sweep();
    let r =
        tools::sweep::get_interaction_matrix(args(json!({"metric": "success_rate"})), &st).unwrap();
    assert_eq!(r.is_error, Some(true));
}

#[test]
fn test_get_interaction_matrix_returns_correct_structure() {
    println!("\n[get_interaction_matrix] 2x2 matrix with null diagonal");
    let st = run_2d_sweep();
    let r =
        tools::sweep::get_interaction_matrix(args(json!({"metric": "success_rate"})), &st).unwrap();
    assert!(is_ok(&r), "get_interaction_matrix failed: {}", text_of(&r));
    let j: Value = serde_json::from_str(&text_of(&r)).unwrap();
    println!("  max_interaction={}", j["max_interaction"]);

    let matrix = j["matrix"].as_array().unwrap();
    assert_eq!(matrix.len(), 2, "2x2 matrix expected");
    // Diagonal must be null
    assert!(matrix[0].as_array().unwrap()[0].is_null());
    assert!(matrix[1].as_array().unwrap()[1].is_null());
    // Off-diagonal must be non-null
    assert!(matrix[0].as_array().unwrap()[1].as_f64().is_some());
    assert_eq!(j["labels"].as_array().unwrap().len(), 2);
    assert!(j["max_interaction"].as_f64().is_some());
    assert!(j["strong_interactions"].is_array());
}
