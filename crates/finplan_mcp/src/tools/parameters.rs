use rmcp::{ErrorData as McpError, model::*};
use serde_json::{Map, Value, json};

use finplan::data::parameters_data::{
    DistributionType, FederalBracketsPreset, InflationData, ParametersData, ReturnsMode,
    TaxConfigData,
};

use super::{make_tool, text_result};
use crate::defaults;
use crate::state::SharedState;

pub fn tools() -> Vec<Tool> {
    vec![make_tool(
        "set_parameters",
        "Set simulation parameters: birth date, start date, duration, inflation, taxes, and returns mode. \
         Call this early in scenario construction.",
        json!({
            "type": "object",
            "properties": {
                "birth_date": {
                    "type": "string",
                    "description": "Birth date in YYYY-MM-DD format (e.g. \"1985-06-15\")"
                },
                "start_date": {
                    "type": "string",
                    "description": "Simulation start date YYYY-MM-DD (default: \"2026-01-01\")"
                },
                "duration_years": {
                    "type": "integer",
                    "description": "Simulation duration in years (default: 30)"
                },
                "inflation_type": {
                    "type": "string",
                    "enum": ["none", "fixed", "normal", "lognormal", "us_historical"],
                    "description": "Inflation model (default: \"us_historical\")"
                },
                "inflation_rate": {
                    "type": "number",
                    "description": "For fixed inflation, the annual rate (e.g. 0.03)"
                },
                "inflation_mean": {
                    "type": "number",
                    "description": "For normal/lognormal inflation, the mean"
                },
                "inflation_std_dev": {
                    "type": "number",
                    "description": "For normal/lognormal inflation, the std dev"
                },
                "state_abbreviation": {
                    "type": "string",
                    "description": "US state abbreviation for tax rate lookup (e.g. \"CA\", \"TX\")"
                },
                "state_rate": {
                    "type": "number",
                    "description": "Direct state income tax rate (overrides state_abbreviation)"
                },
                "capital_gains_rate": {
                    "type": "number",
                    "description": "Long-term capital gains rate (default: 0.15)"
                },
                "federal_brackets": {
                    "type": "string",
                    "enum": ["single2024", "married_joint2024"],
                    "description": "Federal tax bracket preset (default: \"single2024\")"
                },
                "returns_mode": {
                    "type": "string",
                    "enum": ["parametric", "historical"],
                    "description": "Returns mode (default: \"historical\")"
                },
                "historical_block_size": {
                    "type": "integer",
                    "description": "Block bootstrap size for historical returns (default: 5)"
                },
                "spouse_birth_date": {
                    "type": "string",
                    "description": "Spouse's date of birth in YYYY-MM-DD format. Optional. Required for SpouseAge event triggers.",
                    "pattern": "^\\d{4}-\\d{2}-\\d{2}$"
                },
                "seed": {
                    "type": "integer",
                    "description": "Random seed for reproducibility (optional)"
                }
            },
            "required": ["birth_date"]
        }),
    )]
}

pub fn set_parameters(
    args: Map<String, Value>,
    state: &SharedState,
) -> Result<CallToolResult, McpError> {
    let birth_date = args
        .get("birth_date")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("birth_date is required", None))?
        .to_string();

    let start_date = args
        .get("start_date")
        .and_then(|v| v.as_str())
        .unwrap_or("2026-01-01")
        .to_string();

    let duration_years = args
        .get("duration_years")
        .and_then(|v| v.as_u64())
        .unwrap_or(30) as usize;

    // Build inflation
    let inflation = match args.get("inflation_type").and_then(|v| v.as_str()) {
        Some("none") => InflationData::None,
        Some("fixed") => {
            let rate = args
                .get("inflation_rate")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.03);
            InflationData::Fixed { rate }
        }
        Some("normal") => {
            let mean = args
                .get("inflation_mean")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.03);
            let std_dev = args
                .get("inflation_std_dev")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.01);
            InflationData::Normal { mean, std_dev }
        }
        Some("lognormal") => {
            let mean = args
                .get("inflation_mean")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.03);
            let std_dev = args
                .get("inflation_std_dev")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.01);
            InflationData::LogNormal { mean, std_dev }
        }
        _ => InflationData::USHistorical {
            distribution: DistributionType::LogNormal,
        },
    };

    // Build tax config
    let state_rate = if let Some(rate) = args.get("state_rate").and_then(|v| v.as_f64()) {
        rate
    } else if let Some(abbr) = args.get("state_abbreviation").and_then(|v| v.as_str()) {
        defaults::state_tax_rate(abbr)
    } else {
        0.05
    };

    let capital_gains_rate = args
        .get("capital_gains_rate")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.15);

    let federal_brackets = match args.get("federal_brackets").and_then(|v| v.as_str()) {
        Some("married_joint2024") => FederalBracketsPreset::MarriedJoint2024,
        _ => FederalBracketsPreset::Single2024,
    };

    let returns_mode = match args.get("returns_mode").and_then(|v| v.as_str()) {
        Some("parametric") => ReturnsMode::Parametric,
        _ => ReturnsMode::Historical,
    };

    let historical_block_size = args
        .get("historical_block_size")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize);

    let seed = args.get("seed").and_then(|v| v.as_u64());

    let spouse_birth_date = args
        .get("spouse_birth_date")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let params = ParametersData {
        birth_date: birth_date.clone(),
        spouse_birth_date,
        start_date: start_date.clone(),
        duration_years,
        inflation,
        tax_config: TaxConfigData {
            state_rate,
            capital_gains_rate,
            federal_brackets,
        },
        returns_mode,
        historical_block_size,
        seed,
    };

    let mut st = state.lock().unwrap();
    st.parameters = Some(params);

    text_result(format!(
        "Parameters set:\n  Birth: {}\n  Start: {}\n  Duration: {} years\n  Returns: {:?}\n  State tax: {:.1}%",
        birth_date,
        start_date,
        duration_years,
        returns_mode,
        state_rate * 100.0
    ))
}
