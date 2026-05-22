use rmcp::{ErrorData as McpError, model::*};
use serde_json::{Map, Value, json};

use finplan::data::events_data::*;
use finplan::data::portfolio_data::AssetTag;

use super::{make_tool, text_result};
use crate::defaults;
use crate::state::SharedState;

pub fn tools() -> Vec<Tool> {
    vec![
        make_tool(
            "add_income_event",
            "Add a recurring income event (salary, freelance, etc.). \
             Automatically creates a Repeating trigger with optional end event.",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Event name (e.g. \"Salary\")" },
                    "to_account": { "type": "string", "description": "Account receiving income (e.g. \"Checking\")" },
                    "amount": { "type": "number", "description": "Per-period income amount in dollars" },
                    "interval": { "type": "string", "enum": ["weekly", "biweekly", "monthly", "quarterly", "yearly"], "description": "Pay frequency (default: biweekly)" },
                    "gross": { "type": "boolean", "description": "Whether taxes are withheld (default: true)" },
                    "taxable": { "type": "boolean", "description": "Whether income is taxable (default: true)" },
                    "inflation_adjusted": { "type": "boolean", "description": "Adjust for inflation over time (default: true)" },
                    "end_event": { "type": "string", "description": "Event name that ends this income (e.g. \"Retirement\")" },
                    "start_age": { "type": "integer", "description": "Age to start income (optional)" }
                },
                "required": ["name", "to_account", "amount"]
            }),
        ),
        make_tool(
            "add_expense_event",
            "Add a recurring expense event (rent, living expenses, etc.).",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Event name (e.g. \"Living Expenses\")" },
                    "from_account": { "type": "string", "description": "Account paying the expense" },
                    "amount": { "type": "number", "description": "Per-period expense amount" },
                    "interval": { "type": "string", "enum": ["weekly", "biweekly", "monthly", "quarterly", "yearly"], "description": "Frequency (default: monthly)" },
                    "inflation_adjusted": { "type": "boolean", "description": "Adjust for inflation (default: true)" },
                    "end_event": { "type": "string", "description": "Event name that ends this expense" },
                    "start_event": { "type": "string", "description": "Event name that starts this expense" },
                    "start_age": { "type": "integer", "description": "Age to start expense (optional)" }
                },
                "required": ["name", "from_account", "amount"]
            }),
        ),
        make_tool(
            "add_retirement_event",
            "Add a retirement milestone event. This creates a once-trigger event at the given age \
             that other events can reference (e.g., income ends, spending begins).",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Event name (default: \"Retirement\")" },
                    "age": { "type": "integer", "description": "Retirement age (e.g. 65)" }
                },
                "required": ["age"]
            }),
        ),
        make_tool(
            "add_contribution_event",
            "Add a recurring investment contribution (401k, IRA, etc.).",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Event name (e.g. \"401k Contribution\")" },
                    "from_account": { "type": "string", "description": "Source cash account" },
                    "to_account": { "type": "string", "description": "Target investment account" },
                    "asset": { "type": "string", "description": "Ticker to buy (e.g. \"FXAIX\")" },
                    "amount": { "type": "number", "description": "Annual contribution amount" },
                    "interval": { "type": "string", "enum": ["monthly", "quarterly", "yearly"], "description": "Frequency (default: yearly)" },
                    "end_event": { "type": "string", "description": "Event that ends contributions (e.g. \"Retirement\")" }
                },
                "required": ["name", "from_account", "to_account", "asset", "amount"]
            }),
        ),
        make_tool(
            "add_sweep_event",
            "Add a post-retirement sweep/withdrawal event that liquidates from investment accounts \
             to fund spending. Uses the penalty_aware strategy by default.",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Event name (e.g. \"Post-Retirement Spending\")" },
                    "to_account": { "type": "string", "description": "Cash account to fund" },
                    "target_balance": { "type": "number", "description": "Target balance to maintain in the cash account" },
                    "interval": { "type": "string", "enum": ["monthly", "quarterly", "yearly"], "description": "Frequency (default: yearly)" },
                    "start_event": { "type": "string", "description": "Event that starts withdrawals (e.g. \"Retirement\")" },
                    "strategy": {
                        "type": "string",
                        "enum": ["penalty_aware", "tax_efficient", "tax_deferred_first", "tax_free_first", "pro_rata"],
                        "description": "Withdrawal strategy (default: penalty_aware)"
                    },
                    "gross": { "type": "boolean", "description": "Whether taxes are withheld (default: false)" },
                    "lot_method": {
                        "type": "string",
                        "enum": ["fifo", "lifo", "highest_cost", "lowest_cost", "average_cost"],
                        "description": "Lot selection method (default: fifo)"
                    }
                },
                "required": ["name", "to_account", "target_balance"]
            }),
        ),
        make_tool(
            "add_social_security_event",
            "Add a Social Security income event starting at a given age. \
             Can auto-estimate benefit from annual income.",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Event name (default: \"Social Security\")" },
                    "to_account": { "type": "string", "description": "Account receiving SS (default: \"Checking\")" },
                    "monthly_amount": { "type": "number", "description": "Monthly SS benefit amount" },
                    "annual_income": { "type": "number", "description": "Current annual income (used to estimate benefit if monthly_amount not given)" },
                    "start_age": { "type": "integer", "description": "Age to start (default: 67)" }
                }
            }),
        ),
        make_tool(
            "add_rmd_event",
            "Add Required Minimum Distributions starting at age 73.",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Event name (default: \"RMD\")" },
                    "destination": { "type": "string", "description": "Account receiving RMD proceeds (default: \"Checking\")" },
                    "start_age": { "type": "integer", "description": "Age RMDs begin (default: 73)" },
                    "lot_method": {
                        "type": "string",
                        "enum": ["fifo", "lifo", "highest_cost", "lowest_cost", "average_cost"],
                        "description": "Lot method (default: fifo)"
                    }
                }
            }),
        ),
        make_tool(
            "add_custom_event",
            "Add a fully custom event with explicit trigger and effects. \
             Use this for events that don't fit the other specialized tools. \
             The trigger and effects should be JSON objects matching the YAML schema.",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Event name" },
                    "description": { "type": "string", "description": "Optional description" },
                    "trigger": { "type": "object", "description": "Trigger definition (JSON matching YAML schema)" },
                    "effects": {
                        "type": "array",
                        "description": "Array of effect definitions (JSON matching YAML schema)",
                        "items": { "type": "object" }
                    },
                    "once": { "type": "boolean", "description": "Fire only once (default: false)" },
                    "enabled": { "type": "boolean", "description": "Whether event is active (default: true)" }
                },
                "required": ["name", "trigger", "effects"]
            }),
        ),
    ]
}

// ── Tool implementations ────────────────────────────────────────────────

pub fn add_income_event(
    args: Map<String, Value>,
    state: &SharedState,
) -> Result<CallToolResult, McpError> {
    let name = require_str(&args, "name")?;
    let to_account = require_str(&args, "to_account")?;
    let amount_val = require_f64(&args, "amount")?;

    let interval = parse_interval(
        args.get("interval")
            .and_then(|v| v.as_str())
            .unwrap_or("biweekly"),
    );
    let gross = args.get("gross").and_then(|v| v.as_bool()).unwrap_or(true);
    let taxable = args
        .get("taxable")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let inflation_adjusted = args
        .get("inflation_adjusted")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let amount = if inflation_adjusted {
        AmountData::InflationAdjusted {
            inner: Box::new(AmountData::Fixed { value: amount_val }),
        }
    } else {
        AmountData::Fixed { value: amount_val }
    };

    let end = args.get("end_event").and_then(|v| v.as_str()).map(|e| {
        Box::new(TriggerData::RelativeToEvent {
            event: EventTag(e.to_string()),
            offset: OffsetData::Months { value: 0 },
        })
    });

    let start = args.get("start_age").and_then(|v| v.as_u64()).map(|age| {
        Box::new(TriggerData::Age {
            years: age as u8,
            months: None,
        })
    });

    let trigger = TriggerData::Repeating {
        interval,
        start,
        end,
        max_occurrences: None,
    };

    let event = EventData {
        name: EventTag(name.clone()),
        description: None,
        trigger,
        effects: vec![EffectData::Income {
            to: AccountTag(to_account.clone()),
            amount,
            gross,
            taxable,
        }],
        once: false,
        enabled: true,
    };

    add_event(state, event)?;
    text_result(format!("Added income event \"{}\" → {}", name, to_account))
}

pub fn add_expense_event(
    args: Map<String, Value>,
    state: &SharedState,
) -> Result<CallToolResult, McpError> {
    let name = require_str(&args, "name")?;
    let from_account = require_str(&args, "from_account")?;
    let amount_val = require_f64(&args, "amount")?;

    let interval = parse_interval(
        args.get("interval")
            .and_then(|v| v.as_str())
            .unwrap_or("monthly"),
    );
    let inflation_adjusted = args
        .get("inflation_adjusted")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let amount = if inflation_adjusted {
        AmountData::InflationAdjusted {
            inner: Box::new(AmountData::Fixed { value: amount_val }),
        }
    } else {
        AmountData::Fixed { value: amount_val }
    };

    let end = args.get("end_event").and_then(|v| v.as_str()).map(|e| {
        Box::new(TriggerData::RelativeToEvent {
            event: EventTag(e.to_string()),
            offset: OffsetData::Months { value: 0 },
        })
    });

    let start = if let Some(event_name) = args.get("start_event").and_then(|v| v.as_str()) {
        Some(Box::new(TriggerData::RelativeToEvent {
            event: EventTag(event_name.to_string()),
            offset: OffsetData::Months { value: 0 },
        }))
    } else {
        args.get("start_age").and_then(|v| v.as_u64()).map(|age| {
            Box::new(TriggerData::Age {
                years: age as u8,
                months: None,
            })
        })
    };

    let trigger = TriggerData::Repeating {
        interval,
        start,
        end,
        max_occurrences: None,
    };

    let event = EventData {
        name: EventTag(name.clone()),
        description: None,
        trigger,
        effects: vec![EffectData::Expense {
            from: AccountTag(from_account.clone()),
            amount,
        }],
        once: false,
        enabled: true,
    };

    add_event(state, event)?;
    text_result(format!(
        "Added expense event \"{}\" from {}",
        name, from_account
    ))
}

pub fn add_retirement_event(
    args: Map<String, Value>,
    state: &SharedState,
) -> Result<CallToolResult, McpError> {
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Retirement")
        .to_string();
    let age = require_u64(&args, "age")? as u8;

    let event = EventData {
        name: EventTag(name.clone()),
        description: Some("Retirement milestone".into()),
        trigger: TriggerData::Age {
            years: age,
            months: None,
        },
        effects: vec![],
        once: true,
        enabled: true,
    };

    add_event(state, event)?;
    text_result(format!(
        "Added retirement event \"{}\" at age {}. Other events can reference this \
         as end_event/start_event.",
        name, age
    ))
}

pub fn add_contribution_event(
    args: Map<String, Value>,
    state: &SharedState,
) -> Result<CallToolResult, McpError> {
    let name = require_str(&args, "name")?;
    let from_account = require_str(&args, "from_account")?;
    let to_account = require_str(&args, "to_account")?;
    let asset = require_str(&args, "asset")?;
    let amount_val = require_f64(&args, "amount")?;

    let interval = parse_interval(
        args.get("interval")
            .and_then(|v| v.as_str())
            .unwrap_or("yearly"),
    );

    let end = args.get("end_event").and_then(|v| v.as_str()).map(|e| {
        Box::new(TriggerData::RelativeToEvent {
            event: EventTag(e.to_string()),
            offset: OffsetData::Months { value: 0 },
        })
    });

    let trigger = TriggerData::Repeating {
        interval,
        start: None,
        end,
        max_occurrences: None,
    };

    let event = EventData {
        name: EventTag(name.clone()),
        description: None,
        trigger,
        effects: vec![EffectData::AssetPurchase {
            from: AccountTag(from_account),
            to_account: AccountTag(to_account.clone()),
            asset: AssetTag(asset.clone()),
            amount: AmountData::Fixed { value: amount_val },
        }],
        once: false,
        enabled: true,
    };

    add_event(state, event)?;
    text_result(format!(
        "Added contribution event \"{}\" → {} in {}",
        name, asset, to_account
    ))
}

pub fn add_sweep_event(
    args: Map<String, Value>,
    state: &SharedState,
) -> Result<CallToolResult, McpError> {
    let name = require_str(&args, "name")?;
    let to_account = require_str(&args, "to_account")?;
    let target_balance = require_f64(&args, "target_balance")?;

    let interval = parse_interval(
        args.get("interval")
            .and_then(|v| v.as_str())
            .unwrap_or("yearly"),
    );

    let strategy = match args.get("strategy").and_then(|v| v.as_str()) {
        Some("tax_efficient") => WithdrawalStrategyData::TaxEfficient,
        Some("tax_deferred_first") => WithdrawalStrategyData::TaxDeferredFirst,
        Some("tax_free_first") => WithdrawalStrategyData::TaxFreeFirst,
        Some("pro_rata") => WithdrawalStrategyData::ProRata,
        _ => WithdrawalStrategyData::PenaltyAware,
    };

    let gross = args.get("gross").and_then(|v| v.as_bool()).unwrap_or(false);
    let lot_method = parse_lot_method(args.get("lot_method").and_then(|v| v.as_str()));

    let start = args.get("start_event").and_then(|v| v.as_str()).map(|e| {
        Box::new(TriggerData::RelativeToEvent {
            event: EventTag(e.to_string()),
            offset: OffsetData::Months { value: 0 },
        })
    });

    let trigger = TriggerData::Repeating {
        interval,
        start,
        end: None,
        max_occurrences: None,
    };

    let event = EventData {
        name: EventTag(name.clone()),
        description: None,
        trigger,
        effects: vec![EffectData::Sweep {
            to: AccountTag(to_account.clone()),
            amount: AmountData::TargetToBalance {
                target: target_balance,
            },
            strategy,
            gross,
            taxable: true,
            lot_method,
            exclude_accounts: vec![],
        }],
        once: false,
        enabled: true,
    };

    add_event(state, event)?;
    text_result(format!(
        "Added sweep event \"{}\" → {} (target: ${:.0})",
        name, to_account, target_balance
    ))
}

pub fn add_social_security_event(
    args: Map<String, Value>,
    state: &SharedState,
) -> Result<CallToolResult, McpError> {
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Social Security")
        .to_string();
    let to_account = args
        .get("to_account")
        .and_then(|v| v.as_str())
        .unwrap_or("Checking")
        .to_string();
    let start_age = args.get("start_age").and_then(|v| v.as_u64()).unwrap_or(67) as u8;

    let monthly_amount = if let Some(amt) = args.get("monthly_amount").and_then(|v| v.as_f64()) {
        amt
    } else if let Some(income) = args.get("annual_income").and_then(|v| v.as_f64()) {
        defaults::estimate_social_security(income)
    } else {
        2800.0 // reasonable default
    };

    let trigger = TriggerData::Repeating {
        interval: IntervalData::Monthly,
        start: Some(Box::new(TriggerData::Age {
            years: start_age,
            months: None,
        })),
        end: None,
        max_occurrences: None,
    };

    let event = EventData {
        name: EventTag(name.clone()),
        description: Some(format!("Social Security at age {}", start_age)),
        trigger,
        effects: vec![EffectData::Income {
            to: AccountTag(to_account.clone()),
            amount: AmountData::Fixed {
                value: monthly_amount,
            },
            gross: true,
            taxable: true,
        }],
        once: false,
        enabled: true,
    };

    add_event(state, event)?;
    text_result(format!(
        "Added Social Security \"{}\" at age {} → {} (${:.0}/month)",
        name, start_age, to_account, monthly_amount
    ))
}

pub fn add_rmd_event(
    args: Map<String, Value>,
    state: &SharedState,
) -> Result<CallToolResult, McpError> {
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("RMD")
        .to_string();
    let destination = args
        .get("destination")
        .and_then(|v| v.as_str())
        .unwrap_or("Checking")
        .to_string();
    let start_age = args.get("start_age").and_then(|v| v.as_u64()).unwrap_or(73) as u8;
    let lot_method = parse_lot_method(args.get("lot_method").and_then(|v| v.as_str()));

    let trigger = TriggerData::Repeating {
        interval: IntervalData::Yearly,
        start: Some(Box::new(TriggerData::Age {
            years: start_age,
            months: None,
        })),
        end: None,
        max_occurrences: None,
    };

    let event = EventData {
        name: EventTag(name.clone()),
        description: Some("Required Minimum Distributions".into()),
        trigger,
        effects: vec![EffectData::ApplyRmd {
            destination: AccountTag(destination.clone()),
            lot_method,
        }],
        once: false,
        enabled: true,
    };

    add_event(state, event)?;
    text_result(format!(
        "Added RMD event \"{}\" at age {} → {}",
        name, start_age, destination
    ))
}

pub fn add_custom_event(
    args: Map<String, Value>,
    state: &SharedState,
) -> Result<CallToolResult, McpError> {
    let name = require_str(&args, "name")?;
    let description = args
        .get("description")
        .and_then(|v| v.as_str())
        .map(String::from);
    let once = args.get("once").and_then(|v| v.as_bool()).unwrap_or(false);
    let enabled = args
        .get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    // Parse trigger from JSON
    let trigger_val = args
        .get("trigger")
        .ok_or_else(|| McpError::invalid_params("trigger is required", None))?;
    let trigger: TriggerData = serde_json::from_value(trigger_val.clone())
        .map_err(|e| McpError::invalid_params(format!("Invalid trigger: {}", e), None))?;

    // Parse effects from JSON
    let effects_val = args
        .get("effects")
        .ok_or_else(|| McpError::invalid_params("effects is required", None))?;
    let effects: Vec<EffectData> = serde_json::from_value(effects_val.clone())
        .map_err(|e| McpError::invalid_params(format!("Invalid effects: {}", e), None))?;

    let event = EventData {
        name: EventTag(name.clone()),
        description,
        trigger,
        effects,
        once,
        enabled,
    };

    add_event(state, event)?;
    text_result(format!("Added custom event \"{}\"", name))
}

// ── Helpers ─────────────────────────────────────────────────────────────

fn add_event(state: &SharedState, event: EventData) -> Result<(), McpError> {
    let mut st = state.lock().unwrap();

    // Check for duplicate
    if st.events.iter().any(|e| e.name == event.name) {
        return Err(McpError::invalid_params(
            format!("Event \"{}\" already exists", event.name.0),
            None,
        ));
    }

    st.events.push(event);
    Ok(())
}

fn require_str(args: &Map<String, Value>, key: &str) -> Result<String, McpError> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| McpError::invalid_params(format!("{} is required", key), None))
}

fn require_f64(args: &Map<String, Value>, key: &str) -> Result<f64, McpError> {
    args.get(key)
        .and_then(|v| v.as_f64())
        .ok_or_else(|| McpError::invalid_params(format!("{} is required", key), None))
}

fn require_u64(args: &Map<String, Value>, key: &str) -> Result<u64, McpError> {
    args.get(key)
        .and_then(|v| v.as_u64())
        .ok_or_else(|| McpError::invalid_params(format!("{} is required", key), None))
}

fn parse_interval(s: &str) -> IntervalData {
    match s {
        "weekly" => IntervalData::Weekly,
        "biweekly" => IntervalData::BiWeekly,
        "monthly" => IntervalData::Monthly,
        "quarterly" => IntervalData::Quarterly,
        "yearly" => IntervalData::Yearly,
        _ => IntervalData::Monthly,
    }
}

fn parse_lot_method(s: Option<&str>) -> LotMethodData {
    match s {
        Some("lifo") => LotMethodData::Lifo,
        Some("highest_cost") => LotMethodData::HighestCost,
        Some("lowest_cost") => LotMethodData::LowestCost,
        Some("average_cost") => LotMethodData::AverageCost,
        _ => LotMethodData::Fifo,
    }
}
