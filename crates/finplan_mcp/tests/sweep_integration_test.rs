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
