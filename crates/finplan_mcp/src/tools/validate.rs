use rmcp::{ErrorData as McpError, model::*};
use serde_json::json;

use finplan::data::validator;

use super::{error_result, make_tool, text_result};
use crate::state::SharedState;

pub fn tools() -> Vec<Tool> {
    vec![make_tool(
        "validate_scenario",
        "Validate the current accumulated scenario state. Checks for missing fields, \
         invalid references, duplicate names, and other issues. \
         Returns validation errors with help text, or confirms the scenario is valid.",
        json!({
            "type": "object",
            "properties": {}
        }),
    )]
}

pub fn validate_scenario(state: &SharedState) -> Result<CallToolResult, McpError> {
    let st = state.lock().unwrap();

    // First try to merge
    let sim_data = match st.merge() {
        Ok(data) => data,
        Err(errors) => {
            return error_result(format!(
                "Cannot validate — incomplete scenario:\n{}",
                errors.join("\n")
            ));
        }
    };

    // Run validator
    match validator::validate_scenario(&sim_data) {
        Ok(()) => text_result(
            "✅ Scenario is valid! No errors found.\n\n\
             You can now call merge_scenario to get the final YAML output.",
        ),
        Err(errors) => {
            let mut msg = format!("❌ Found {} validation error(s):\n\n", errors.len());
            for (i, err) in errors.iter().enumerate() {
                msg.push_str(&format!("{}. [{}] {}\n", i + 1, err.section, err.message));
                if let Some(ref help) = err.help {
                    msg.push_str(&format!("   Help: {}\n", help));
                }
            }
            error_result(msg)
        }
    }
}
