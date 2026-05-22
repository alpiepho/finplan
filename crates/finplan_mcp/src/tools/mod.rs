pub mod events;
pub mod merge;
pub mod parameters;
pub mod portfolio;
pub mod ticker;
pub mod validate;

use rmcp::{ErrorData as McpError, model::*};
use serde_json::{Map, Value};

use crate::state::SharedState;

/// Return the list of all available tools.
pub fn list_tools() -> Vec<Tool> {
    let mut tools = Vec::new();
    tools.extend(parameters::tools());
    tools.extend(portfolio::tools());
    tools.extend(ticker::tools());
    tools.extend(events::tools());
    tools.extend(merge::tools());
    tools.extend(validate::tools());
    tools.extend(utility_tools());
    tools
}

fn utility_tools() -> Vec<Tool> {
    vec![
        make_tool(
            "get_state_summary",
            "Get a summary of the current accumulated scenario state: portfolio name, \
             account count, parameter settings, event count, and ticker mappings.",
            serde_json::json!({"type": "object", "properties": {}}),
        ),
        make_tool(
            "reset_state",
            "Clear all accumulated scenario state and start fresh. \
             Use this to begin building a new scenario.",
            serde_json::json!({"type": "object", "properties": {}}),
        ),
    ]
}

/// Dispatch a tool call by name.
pub async fn call_tool(
    name: &str,
    args: Map<String, Value>,
    state: &SharedState,
) -> Result<CallToolResult, McpError> {
    match name {
        "set_parameters" => parameters::set_parameters(args, state),
        "set_portfolio" => portfolio::set_portfolio(args, state),
        "add_account" => portfolio::add_account(args, state),
        "map_tickers" => ticker::map_tickers(args, state),
        "add_income_event" => events::add_income_event(args, state),
        "add_expense_event" => events::add_expense_event(args, state),
        "add_retirement_event" => events::add_retirement_event(args, state),
        "add_contribution_event" => events::add_contribution_event(args, state),
        "add_sweep_event" => events::add_sweep_event(args, state),
        "add_social_security_event" => events::add_social_security_event(args, state),
        "add_rmd_event" => events::add_rmd_event(args, state),
        "add_custom_event" => events::add_custom_event(args, state),
        "merge_scenario" => merge::merge_scenario(state),
        "validate_scenario" => validate::validate_scenario(state),
        "get_state_summary" => get_state_summary(state),
        "reset_state" => reset_state(state),
        _ => Err(McpError::invalid_params(
            format!("Unknown tool: {}", name),
            None,
        )),
    }
}

/// Helper to build a successful text result.
pub fn text_result(text: impl Into<String>) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult {
        content: vec![Content::text(text.into())],
        is_error: None,
        structured_content: None,
        meta: None,
    })
}

/// Helper to build an error text result (is_error = true).
pub fn error_result(text: impl Into<String>) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult {
        content: vec![Content::text(text.into())],
        is_error: Some(true),
        structured_content: None,
        meta: None,
    })
}

/// Helper to make a Tool with a JSON Schema object.
pub fn make_tool(name: &'static str, description: &'static str, schema: Value) -> Tool {
    Tool::new(name, description, json_object(schema))
}

fn json_object(v: Value) -> serde_json::Map<String, Value> {
    match v {
        Value::Object(m) => m,
        _ => {
            let mut m = serde_json::Map::new();
            m.insert("type".into(), Value::String("object".into()));
            m
        }
    }
}

/// Get a summary of the current state.
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

    text_result(parts.join("\n"))
}

/// Reset all accumulated state.
pub fn reset_state(state: &SharedState) -> Result<CallToolResult, McpError> {
    let mut st = state.lock().unwrap();
    *st = crate::state::ScenarioState::new();
    text_result("State reset. All accumulated scenario data cleared.")
}
