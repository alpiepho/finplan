use rmcp::{ErrorData as McpError, model::*};
use serde_json::{Map, Value, json};

use finplan::data::portfolio_data::AssetTag;
use finplan::data::profiles_data::{ProfileData, ReturnProfileTag};
use finplan::data::ticker_profiles;

use super::{error_result, make_tool, text_result};
use crate::state::SharedState;

pub fn tools() -> Vec<Tool> {
    vec![make_tool(
        "map_tickers",
        "Auto-detect return profiles for tickers used in the portfolio. \
         For historical mode, maps tickers to historical presets. \
         For parametric mode, maps tickers to return profile definitions. \
         Call this AFTER set_portfolio and set_parameters.",
        json!({
            "type": "object",
            "properties": {
                "overrides": {
                    "type": "object",
                    "description": "Optional manual overrides: {\"TICKER\": \"Profile Name\"}",
                    "additionalProperties": { "type": "string" }
                }
            }
        }),
    )]
}

pub fn map_tickers(
    args: Map<String, Value>,
    state: &SharedState,
) -> Result<CallToolResult, McpError> {
    let mut st = state.lock().unwrap();

    let tickers = st.portfolio_tickers();
    if tickers.is_empty() {
        return error_result(
            "No tickers found in portfolio. Add investment accounts with assets first.",
        );
    }

    // Parse overrides
    let overrides: std::collections::HashMap<String, String> = args
        .get("overrides")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default();

    let is_historical = st
        .parameters
        .as_ref()
        .map(|p| p.returns_mode == finplan::data::parameters_data::ReturnsMode::Historical)
        .unwrap_or(true); // default historical

    let mut mapped = Vec::new();
    let mut unmapped = Vec::new();

    for ticker in &tickers {
        // Check for manual override first
        if let Some(profile_name) = overrides.get(ticker) {
            if is_historical {
                st.historical_assets.insert(
                    AssetTag(ticker.clone()),
                    ReturnProfileTag(profile_name.clone()),
                );
            } else {
                st.assets.insert(
                    AssetTag(ticker.clone()),
                    ReturnProfileTag(profile_name.clone()),
                );
            }
            mapped.push(format!("  {} → {} (manual override)", ticker, profile_name));
            continue;
        }

        if is_historical {
            // Try historical preset lookup
            if let Some((_preset_key, display_name)) =
                ticker_profiles::get_historical_suggestion(ticker)
            {
                st.historical_assets.insert(
                    AssetTag(ticker.clone()),
                    ReturnProfileTag(display_name.to_string()),
                );
                mapped.push(format!("  {} → {} (historical)", ticker, display_name));
            } else {
                unmapped.push(ticker.clone());
            }
        } else {
            // Try parametric profile lookup
            if let Some(suggestion) = ticker_profiles::get_suggestion(ticker) {
                let profile_name = suggestion.profile_name;
                // Add profile if not already present
                if !st.profiles.iter().any(|p| p.name.0 == profile_name) {
                    st.profiles.push(ProfileData {
                        name: ReturnProfileTag(profile_name.to_string()),
                        description: None,
                        profile: suggestion.profile_data.clone(),
                    });
                }
                st.assets.insert(
                    AssetTag(ticker.clone()),
                    ReturnProfileTag(profile_name.to_string()),
                );
                mapped.push(format!("  {} → {} (parametric)", ticker, profile_name));
            } else {
                unmapped.push(ticker.clone());
            }
        }
    }

    let mut result = String::new();
    if !mapped.is_empty() {
        result.push_str("Mapped tickers:\n");
        result.push_str(&mapped.join("\n"));
    }
    if !unmapped.is_empty() {
        if !result.is_empty() {
            result.push_str("\n\n");
        }
        result.push_str(&format!(
            "Unmapped tickers (need manual override): {}",
            unmapped.join(", ")
        ));
        result.push_str("\nUse the overrides parameter to map these, e.g.:\n");
        result.push_str("  overrides: {\"TICKER\": \"S&P 500\"}");
        if is_historical {
            result.push_str("\n\nAvailable historical presets:\n");
            for (_, name, desc) in ticker_profiles::HISTORICAL_PRESETS {
                result.push_str(&format!("  \"{}\" - {}\n", name, desc));
            }
        }
    }

    text_result(result)
}
