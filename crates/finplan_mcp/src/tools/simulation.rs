use std::collections::HashMap;

use finplan::data::{app_data::SimulationData, convert::to_simulation_config};
use finplan_core::model::{
    AccountId, AccountSnapshotFlavor, AssetId, CashFlowKind, EventId, MonteCarloConfig, Person,
    SimulationResult, StateEvent,
};
use finplan_core::simulation::{monte_carlo_simulate_with_config, simulate};
use rmcp::{ErrorData as McpError, model::*};
use serde_json::{Map, Value, json};

use super::{error_result, make_tool, text_result};
use crate::state::SharedState;

pub fn tools() -> Vec<Tool> {
    vec![
        make_tool(
            "run_simulation",
            "Run a single deterministic simulation on the current scenario and return \
             a year-by-year summary (income, expenses, contributions, withdrawals, taxes, \
             net worth). Results are cached for use by get_account_snapshot and get_ledger.",
            json!({
                "type": "object",
                "properties": {
                    "seed": {
                        "type": "integer",
                        "description": "Random seed for reproducible results (default: 0)"
                    }
                }
            }),
        ),
        make_tool(
            "run_monte_carlo",
            "Run Monte Carlo simulation and return success rate, percentile statistics, \
             and per-percentile year-by-year curves. Results are cached for use by \
             get_account_snapshot, get_ledger, and export_planner_summary.",
            json!({
                "type": "object",
                "properties": {
                    "iterations": {
                        "type": "integer",
                        "description": "Number of MC iterations (default: 500, max: 2000)"
                    },
                    "percentiles": {
                        "type": "array",
                        "items": { "type": "number" },
                        "description": "Percentiles to compute (default: [0.05, 0.50, 0.95])"
                    },
                    "seed": {
                        "type": "integer",
                        "description": "Optional seed for reproducible results"
                    }
                }
            }),
        ),
        make_tool(
            "get_account_snapshot",
            "Return per-account balances for a specific simulation year. \
             Requires run_simulation or run_monte_carlo to have been called first.",
            json!({
                "type": "object",
                "required": ["year"],
                "properties": {
                    "year": {
                        "type": "integer",
                        "description": "Calendar year to inspect (e.g. 2045)"
                    },
                    "real": {
                        "type": "boolean",
                        "description": "Return inflation-adjusted values (default: false)"
                    }
                }
            }),
        ),
        make_tool(
            "get_ledger",
            "Return filtered transaction ledger entries from the last simulation run. \
             Requires run_simulation or run_monte_carlo to have been called first.",
            json!({
                "type": "object",
                "properties": {
                    "year": {
                        "type": "integer",
                        "description": "Filter to a specific calendar year (omit for all years)"
                    },
                    "filter": {
                        "type": "string",
                        "enum": ["all", "income", "expense", "contribution", "withdrawal", "tax"],
                        "description": "Filter by category (default: all)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max entries to return (default: 100)"
                    },
                    "offset": {
                        "type": "integer",
                        "description": "Skip this many entries (for pagination)"
                    }
                }
            }),
        ),
    ]
}

// ── run_simulation ────────────────────────────────────────────────────────────

pub fn run_simulation(
    args: Map<String, Value>,
    state: &SharedState,
) -> Result<CallToolResult, McpError> {
    let sim_data = {
        let st = state.lock().unwrap();
        match st.merge() {
            Ok(d) => d,
            Err(errs) => {
                return error_result(format!("Cannot run simulation:\n{}", errs.join("\n")));
            }
        }
    };

    let config = match to_simulation_config(&sim_data) {
        Ok(c) => c,
        Err(e) => return error_result(format!("Config conversion failed: {e}")),
    };

    let seed = args.get("seed").and_then(|v| v.as_u64()).unwrap_or(0);

    let result = match simulate(&config, seed) {
        Ok(r) => r,
        Err(e) => return error_result(format!("Simulation failed: {e}")),
    };

    let years = build_year_summaries(&result, &sim_data);
    let summary = build_simulation_summary(&result, &years);

    {
        let mut st = state.lock().unwrap();
        st.last_simulation_result = Some(result);
        st.last_sim_data = Some(sim_data);
        st.last_mc_summary = None;
    }

    let output = json!({ "summary": summary, "years": years });
    text_result(serde_json::to_string_pretty(&output).unwrap())
}

// ── run_monte_carlo ───────────────────────────────────────────────────────────

pub fn run_monte_carlo(
    args: Map<String, Value>,
    state: &SharedState,
) -> Result<CallToolResult, McpError> {
    let sim_data = {
        let st = state.lock().unwrap();
        match st.merge() {
            Ok(d) => d,
            Err(errs) => {
                return error_result(format!("Cannot run simulation:\n{}", errs.join("\n")));
            }
        }
    };

    let config = match to_simulation_config(&sim_data) {
        Ok(c) => c,
        Err(e) => return error_result(format!("Config conversion failed: {e}")),
    };

    let iterations = args
        .get("iterations")
        .and_then(|v| v.as_u64())
        .map(|n| n.min(2000) as usize)
        .unwrap_or(500);

    let percentiles = args
        .get("percentiles")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_f64()).collect::<Vec<_>>())
        .unwrap_or_else(|| vec![0.05, 0.50, 0.95]);

    let seed = args.get("seed").and_then(|v| v.as_u64());

    let mc_config = MonteCarloConfig {
        iterations,
        percentiles,
        compute_mean: true,
        seed,
        ..Default::default()
    };

    let mc_summary = match monte_carlo_simulate_with_config(&config, &mc_config) {
        Ok(s) => s,
        Err(e) => return error_result(format!("Monte Carlo failed: {e}")),
    };

    // Build output: stats + per-percentile year curves
    let stats = build_mc_stats_json(&mc_summary.stats);

    let mut percentile_runs = serde_json::Map::new();
    for (pct, result) in &mc_summary.percentile_runs {
        let years = build_year_summaries(result, &sim_data);
        let summary = build_simulation_summary(result, &years);
        let key = format!("p{}", (*pct * 100.0) as u32);
        percentile_runs.insert(key, json!({ "summary": summary, "years": years }));
    }

    // Store P50 as the default result for get_account_snapshot / get_ledger
    let p50_result = mc_summary
        .get_percentile(0.50)
        .or_else(|| mc_summary.percentile_runs.first().map(|(_, r)| r))
        .cloned();

    {
        let mut st = state.lock().unwrap();
        st.last_simulation_result = p50_result;
        st.last_sim_data = Some(sim_data);
        st.last_mc_summary = Some(mc_summary);
    }

    let output = json!({
        "stats": stats,
        "percentile_runs": percentile_runs
    });
    text_result(format!(
        "Completed {} iterations.\n{}",
        iterations,
        serde_json::to_string_pretty(&output).unwrap()
    ))
}

fn build_mc_stats_json(stats: &finplan_core::model::MonteCarloStats) -> Value {
    json!({
        "num_iterations": stats.num_iterations,
        "success_rate": stats.success_rate,
        "mean_final_net_worth": stats.mean_final_net_worth,
        "std_dev_final_net_worth": stats.std_dev_final_net_worth,
        "min_final_net_worth": stats.min_final_net_worth,
        "max_final_net_worth": stats.max_final_net_worth,
        "percentile_values": stats.percentile_values.iter().map(|(p, v)| {
            json!({ "percentile": p, "net_worth": v })
        }).collect::<Vec<_>>(),
        "converged": stats.converged,
    })
}

// ── get_account_snapshot ──────────────────────────────────────────────────────

pub fn get_account_snapshot(
    args: Map<String, Value>,
    state: &SharedState,
) -> Result<CallToolResult, McpError> {
    let st = state.lock().unwrap();

    let result = match &st.last_simulation_result {
        Some(r) => r,
        None => {
            return error_result(
                "No simulation results available. Call run_simulation or run_monte_carlo first.",
            );
        }
    };
    let sim_data = st.last_sim_data.as_ref().unwrap();

    let year = match args.get("year").and_then(|v| v.as_i64()) {
        Some(y) => y as i32,
        None => return error_result("year is required"),
    };
    let real = args.get("real").and_then(|v| v.as_bool()).unwrap_or(false);

    // Find year-end snapshot for the requested year
    let snapshot = result
        .wealth_snapshots
        .iter()
        .rev()
        .find(|s| s.date.year() as i32 == year);

    let snapshot = match snapshot {
        Some(s) => s,
        None => return error_result(format!("No snapshot data for year {year}")),
    };

    // Compute inflation factor for the year
    let start_year = parse_year(&sim_data.parameters.start_date);
    let year_idx = (year - start_year).max(0) as usize;
    let inflation = result
        .cumulative_inflation
        .get(year_idx)
        .copied()
        .unwrap_or(1.0);

    let net_worth: f64 = snapshot.accounts.iter().map(|a| a.total_value()).sum();
    let net_worth_out = if real {
        net_worth / inflation
    } else {
        net_worth
    };

    // Build account map: AccountId → (name, type_str, owner)
    let account_meta = build_account_meta(sim_data);

    let accounts: Vec<Value> = snapshot
        .accounts
        .iter()
        .map(|acc| {
            let raw_value = acc.total_value();
            let value = if real {
                raw_value / inflation
            } else {
                raw_value
            };
            let (name, type_str, owner) = account_meta
                .get(&acc.account_id)
                .map(|(n, t, o)| (n.as_str(), t.as_str(), o.as_str()))
                .unwrap_or(("Unknown", "Unknown", "primary"));

            let mut obj = json!({
                "name": name,
                "type": type_str,
                "owner": owner,
                "value": value,
            });

            // Add per-asset breakdown for investment accounts
            if let AccountSnapshotFlavor::Investment { cash, assets } = &acc.flavor {
                let asset_meta = build_asset_meta(sim_data, acc.account_id);
                let mut asset_list: Vec<Value> = assets
                    .iter()
                    .map(|(asset_id, &asset_val)| {
                        let ticker = asset_meta
                            .get(asset_id)
                            .map(|s| s.as_str())
                            .unwrap_or("unknown");
                        let v = if real {
                            asset_val / inflation
                        } else {
                            asset_val
                        };
                        json!({ "ticker": ticker, "value": v })
                    })
                    .collect();
                asset_list.sort_by(|a, b| a["ticker"].as_str().cmp(&b["ticker"].as_str()));
                let cash_val = if real { cash / inflation } else { *cash };
                if cash_val.abs() > 0.01 {
                    asset_list.push(json!({ "ticker": "cash", "value": cash_val }));
                }
                obj["assets"] = Value::Array(asset_list);
            }

            obj
        })
        .collect();

    let output = json!({
        "year": year,
        "net_worth": net_worth_out,
        "inflation_factor": inflation,
        "accounts": accounts,
    });
    text_result(serde_json::to_string_pretty(&output).unwrap())
}

// ── get_ledger ────────────────────────────────────────────────────────────────

pub fn get_ledger(
    args: Map<String, Value>,
    state: &SharedState,
) -> Result<CallToolResult, McpError> {
    let st = state.lock().unwrap();

    let result = match &st.last_simulation_result {
        Some(r) => r,
        None => {
            return error_result(
                "No simulation results available. Call run_simulation or run_monte_carlo first.",
            );
        }
    };
    let sim_data = st.last_sim_data.as_ref().unwrap();

    let filter_year = args.get("year").and_then(|v| v.as_i64()).map(|y| y as i32);
    let filter_cat = args.get("filter").and_then(|v| v.as_str()).unwrap_or("all");
    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
    let offset = args.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

    let account_meta = build_account_meta(sim_data);
    let event_names = build_event_names(sim_data);

    // Collect matching entries
    let mut entries: Vec<Value> = Vec::new();
    for entry in &result.ledger {
        let year = entry.date.year() as i32;
        if let Some(fy) = filter_year {
            if year != fy {
                continue;
            }
        }

        let (category, amount, account_name, description) =
            match classify_ledger_entry(&entry.event, &account_meta) {
                Some(t) => t,
                None => continue,
            };

        if filter_cat != "all" && category != filter_cat {
            continue;
        }

        let event_name = entry
            .source_event
            .and_then(|eid| event_names.get(&eid))
            .map(|s| s.as_str())
            .unwrap_or("");

        entries.push(json!({
            "date": entry.date.to_string(),
            "event": event_name,
            "description": description,
            "amount": amount,
            "account": account_name,
            "category": category,
        }));
    }

    let total_matching = entries.len();
    let returned_entries: Vec<Value> = entries.into_iter().skip(offset).take(limit).collect();
    let returned = returned_entries.len();

    let output = json!({
        "filter": filter_cat,
        "year": filter_year,
        "total_matching": total_matching,
        "returned": returned,
        "entries": returned_entries,
    });
    text_result(serde_json::to_string_pretty(&output).unwrap())
}

// ── Helper: year summaries ────────────────────────────────────────────────────

fn build_year_summaries(result: &SimulationResult, sim_data: &SimulationData) -> Vec<Value> {
    let birth_year = parse_year(&sim_data.parameters.birth_date);
    let spouse_birth_year = sim_data
        .parameters
        .spouse_birth_date
        .as_deref()
        .and_then(|s| s[..4].parse::<i32>().ok());
    let start_year = parse_year(&sim_data.parameters.start_date);

    // Build net worth map: last snapshot per calendar year wins (same logic as TUI).
    // Year-end snapshots are NOT guaranteed to fall on Dec 31 — the simulation
    // takes a snapshot at the last event checkpoint before the year rolls over.
    let mut year_nw: HashMap<i32, f64> = HashMap::new();
    for snap in &result.wealth_snapshots {
        let nw: f64 = snap.accounts.iter().map(|a| a.total_value()).sum();
        year_nw.insert(snap.date.year() as i32, nw);
    }

    result
        .yearly_cash_flows
        .iter()
        .map(|cf| {
            let year = cf.year as i32;
            let year_idx = (year - start_year).max(0) as usize;
            let inflation = result
                .cumulative_inflation
                .get(year_idx)
                .copied()
                .unwrap_or(1.0);
            let nw = year_nw.get(&year).copied().unwrap_or(0.0);
            let taxes = result
                .yearly_taxes
                .iter()
                .find(|t| t.year == cf.year)
                .map(|t| t.total_tax)
                .unwrap_or(0.0);

            let mut obj = json!({
                "year": year,
                "age": year - birth_year,
                "net_worth": nw,
                "real_net_worth": nw / inflation,
                "income": cf.income,
                "real_income": cf.income / inflation,
                "expenses": cf.expenses,
                "real_expenses": cf.expenses / inflation,
                "withdrawals": cf.withdrawals,
                "contributions": cf.contributions,
                "taxes": taxes,
            });

            if let Some(sby) = spouse_birth_year {
                obj["spouse_age"] = json!(year - sby);
            } else {
                obj["spouse_age"] = Value::Null;
            }

            obj
        })
        .collect()
}

// ── Helper: simulation summary ────────────────────────────────────────────────

fn build_simulation_summary(result: &SimulationResult, years: &[Value]) -> Value {
    let final_nw = years
        .last()
        .and_then(|y| y["net_worth"].as_f64())
        .unwrap_or(0.0);
    let final_real_nw = years
        .last()
        .and_then(|y| y["real_net_worth"].as_f64())
        .unwrap_or(0.0);

    let ruin_year = years
        .iter()
        .find(|y| y["net_worth"].as_f64().unwrap_or(0.0) < 0.0)
        .and_then(|y| y["year"].as_i64());

    let (peak_nw, peak_year) = years
        .iter()
        .max_by(|a, b| {
            a["net_worth"]
                .as_f64()
                .unwrap_or(0.0)
                .partial_cmp(&b["net_worth"].as_f64().unwrap_or(0.0))
                .unwrap()
        })
        .map(|y| {
            (
                y["net_worth"].as_f64().unwrap_or(0.0),
                y["year"].as_i64().unwrap_or(0),
            )
        })
        .unwrap_or((0.0, 0));

    let warnings: Vec<Value> = result
        .warnings
        .iter()
        .map(|w| {
            json!({
                "date": w.date.to_string(),
                "message": w.message,
            })
        })
        .collect();

    json!({
        "final_net_worth": final_nw,
        "final_real_net_worth": final_real_nw,
        "duration_years": years.len(),
        "final_year": years.last().and_then(|y| y["year"].as_i64()).unwrap_or(0),
        "peak_net_worth": peak_nw,
        "peak_net_worth_year": peak_year,
        "ruin_year": ruin_year,
        "warnings": warnings,
    })
}

// ── Helper: account / asset metadata ─────────────────────────────────────────

/// Returns AccountId → (name, type_display, owner_str)
fn build_account_meta(sim_data: &SimulationData) -> HashMap<AccountId, (String, String, String)> {
    sim_data
        .portfolios
        .accounts
        .iter()
        .enumerate()
        .map(|(idx, acc)| {
            let id = AccountId((idx + 1) as u16);
            let owner_str = match acc.owner {
                Person::Primary => "primary".to_string(),
                Person::Spouse => "spouse".to_string(),
            };
            let type_str = acc.account_type.display_name().to_string();
            (id, (acc.name.clone(), type_str, owner_str))
        })
        .collect()
}

/// Returns AssetId → ticker string for a specific account
fn build_asset_meta(sim_data: &SimulationData, account_id: AccountId) -> HashMap<AssetId, String> {
    let idx = account_id.0 as usize - 1;
    let Some(acc) = sim_data.portfolios.accounts.get(idx) else {
        return HashMap::new();
    };
    let Some(inv) = acc.account_type.as_investment() else {
        return HashMap::new();
    };
    inv.assets
        .iter()
        .enumerate()
        .map(|(i, asset_val)| (AssetId((i + 1) as u16), asset_val.asset.0.clone()))
        .collect()
}

/// Returns EventId → event name string
fn build_event_names(sim_data: &SimulationData) -> HashMap<EventId, String> {
    sim_data
        .events
        .iter()
        .enumerate()
        .map(|(idx, e)| (EventId((idx + 1) as u16), e.name.0.clone()))
        .collect()
}

// ── Helper: ledger classification ─────────────────────────────────────────────

/// Returns Some((category, amount, account_name, description)) or None to skip the entry.
fn classify_ledger_entry(
    event: &StateEvent,
    account_meta: &HashMap<AccountId, (String, String, String)>,
) -> Option<(&'static str, f64, String, String)> {
    let account_name = |id: AccountId| -> String {
        account_meta
            .get(&id)
            .map(|(n, _, _)| n.clone())
            .unwrap_or_else(|| format!("account_{}", id.0))
    };

    match event {
        StateEvent::CashCredit { to, amount, kind } => {
            let acc = account_name(*to);
            let (cat, desc) = match kind {
                CashFlowKind::Income => ("income", format!("Income → {acc}")),
                CashFlowKind::Contribution => ("contribution", format!("Contribution → {acc}")),
                CashFlowKind::LiquidationProceeds => {
                    ("withdrawal", format!("Liquidation proceeds → {acc}"))
                }
                CashFlowKind::Transfer => ("contribution", format!("Transfer → {acc}")),
                _ => return None,
            };
            Some((cat, *amount, acc, desc))
        }
        StateEvent::CashDebit { from, amount, kind } => {
            let acc = account_name(*from);
            let (cat, desc) = match kind {
                CashFlowKind::Expense => ("expense", format!("Expense ← {acc}")),
                CashFlowKind::LiquidationProceeds => ("withdrawal", format!("Liquidation ← {acc}")),
                CashFlowKind::InvestmentPurchase => {
                    ("contribution", format!("Investment purchase ← {acc}"))
                }
                CashFlowKind::Transfer => ("withdrawal", format!("Transfer ← {acc}")),
                _ => return None,
            };
            Some((cat, *amount, acc, desc))
        }
        StateEvent::RmdWithdrawal {
            account_id,
            actual_amount,
            ..
        } => {
            let acc = account_name(*account_id);
            Some((
                "withdrawal",
                *actual_amount,
                acc.clone(),
                format!("RMD withdrawal from {acc}"),
            ))
        }
        StateEvent::AssetPurchase {
            account_id,
            cost_basis,
            ..
        } => {
            let acc = account_name(*account_id);
            Some((
                "contribution",
                *cost_basis,
                acc.clone(),
                format!("Asset purchase in {acc}"),
            ))
        }
        StateEvent::IncomeTax {
            federal_tax,
            state_tax,
            ..
        } => Some((
            "tax",
            federal_tax + state_tax,
            String::new(),
            "Income tax".to_string(),
        )),
        StateEvent::LongTermCapitalGainsTax {
            federal_tax,
            state_tax,
            ..
        } => Some((
            "tax",
            federal_tax + state_tax,
            String::new(),
            "Long-term capital gains tax".to_string(),
        )),
        StateEvent::ShortTermCapitalGainsTax {
            federal_tax,
            state_tax,
            ..
        } => Some((
            "tax",
            federal_tax + state_tax,
            String::new(),
            "Short-term capital gains tax".to_string(),
        )),
        StateEvent::EarlyWithdrawalPenalty { penalty_amount, .. } => Some((
            "tax",
            *penalty_amount,
            String::new(),
            "Early withdrawal penalty".to_string(),
        )),
        _ => None,
    }
}

// ── Misc helpers ──────────────────────────────────────────────────────────────

fn parse_year(date_str: &str) -> i32 {
    date_str[..4.min(date_str.len())]
        .parse::<i32>()
        .unwrap_or(2025)
}
