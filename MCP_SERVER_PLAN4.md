# MCP Server Plan 4 — Boldin-Compatible Planner Summary Export

> **Created**: 2026-05-20
> **Branch**: `feature/mcp`
> **Depends on**: MCP_SERVER_PLAN2.md (simulation tools — `run_monte_carlo` must run first)

---

## 1. Overview

[Boldin](https://boldin.com/) (formerly NewRetirement) is a popular retirement planning tool
that exports a multi-scenario CSV summary called the "Planner Summary." This plan adds an
`export_planner_summary` MCP tool that returns data in the same format, enabling direct
comparison between a finplan scenario and a Boldin analysis.

The tool runs on top of an already-completed Monte Carlo simulation (from `run_monte_carlo`
in MCP_SERVER_PLAN2.md) stored in the server's `last_mc_summary` state field.

---

## 2. Boldin CSV Format Analysis

The Boldin CSV (`planner_summary_YYYY-MM-DD.csv`) has the following structure:

```
Assumptions,Category,Item,[2026] age=65; spouse=62,...,[2062] age=101; spouse=98
Optimistic,Accounts,Brokerage,"$1,234,567.00",...
Optimistic,Accounts,Traditional IRA,...
...
Average,Accounts,Brokerage,...
...
Pessimistic,Accounts,Brokerage,...
```

### 2.1 Dimensions

| Dimension | Values | Count |
|-----------|--------|-------|
| Scenarios | Optimistic, Average, Pessimistic | 3 |
| Categories | Accounts, Income, Expense, Contributions, Withdrawals, Taxes | 6 |
| Items | Per-account or per-event-name rows within each category | varies |
| Years | One column per calendar year (e.g., 37 years: 2026–2062) | varies |

### 2.2 Categories and Their Items

**Accounts**
- One row per account (by name)
- Values = year-end total account balance

**Income**
- Interest
- Required Minimum Distribution
- Savings Drawdown *(proceeds from liquidating investments to cover expenses)*
- Work *(salary, wages)*
- Social Security
- Pension *(or other regular recurring non-work income)*
- Windfall *(one-time large income)*
- Primary Home Sale *(one-time real estate proceeds)*

**Expense**
- Medical
- Long Term Care
- Estimated Income Tax Payments *(estimated tax payments, not final bill)*
- General recurring *(ongoing lifestyle expenses)*
- Rent
- One-Time Expense
- IRMAA *(Medicare surcharge)*
- Capital Gains Tax Payments

**Contributions**
- One row per investment account with positive inflows
- Excess Income *(income that exceeds expenses, parked in default account)*

**Withdrawals**
- One row per investment account with outflows
- Shortfall *(expenses that exceeded available income, covered by cash overdraft)*

**Taxes**
- Federal Income Tax
- State Income Tax
- Federal Capital Gains Tax
- FICA
- Other State Taxes

### 2.3 Value Format

```
"$1,955,239.00"    (quoted when value contains comma)
$0.00              (unquoted when no comma)
```

Year column headers: `[YYYY] age=NN; spouse=NN` (or just `[YYYY] age=NN` if no spouse)

---

## 3. Mapping finplan Data → Boldin Categories

### 3.1 Scenario Mapping

| Boldin Scenario | finplan Source |
|-----------------|----------------|
| Optimistic      | P95 percentile run from `MonteCarloSummary.percentile_runs` |
| Average         | Mean result built from `MonteCarloSummary.mean_accumulators` |
| Pessimistic     | P5 percentile run from `MonteCarloSummary.percentile_runs` |

All three are already computed when `run_monte_carlo` completes with the default percentile
set `[0.05, 0.50, 0.95]` and `compute_mean: true`.

### 3.2 Accounts

Source: `SimulationResult.wealth_snapshots`

Filter `wealth_snapshots` to year-end snapshots only:
```rust
snapshots.iter()
    .filter(|s| s.date.month() == 12 && s.date.day() == s.date.days_in_month())
```

For each account (`AccountSnapshot`), the year-end `total_value()` is the Boldin account
balance. Account names come from the original `SimulationData.accounts` list, looked up by
`account_id`.

**Gap**: The mean result built by `MeanAccumulators::build_mean_snapshots()` stores averaged
account totals in `AccountSnapshotFlavor::Bank`, which loses per-account identity when
accounts have different IDs across iterations. Since MC runs the same SimulationData in
every iteration, account IDs are stable — this is safe.

### 3.3 Income

Source: `SimulationResult.ledger`

Walk ledger entries for each year, collecting entries where:

```rust
StateEvent::CashCredit { kind, .. } if matches!(
    kind,
    CashFlowKind::Income | CashFlowKind::RmdWithdrawal | CashFlowKind::LiquidationProceeds
)
```

Also:
```rust
StateEvent::RmdWithdrawal { .. }
```

Group by `(year, source_event_id → event_name)` using the event list from `SimulationData`.

**Boldin income row → finplan source mapping:**

| Boldin Row | finplan Signal |
|------------|----------------|
| Work | `kind: Income` + source event name contains "work", "salary", "job", "wages" — or event type is income event (use event YAML name) |
| Social Security | source event name contains "social security", "ss", "social_security" |
| Required Minimum Distribution | `StateEvent::RmdWithdrawal` OR `kind: RmdWithdrawal` |
| Savings Drawdown | `kind: LiquidationProceeds` (sweep/asset liquidation to fund expenses) |
| Interest | `StateEvent::CashAppreciation` (HYSA, money market interest) |
| Pension | source event name contains "pension" |
| Windfall | source event name contains "windfall" (or any one-time income event) |
| Primary Home Sale | source event name contains "home", "house", "real estate" |

**Note**: The categorization is name-based heuristics on event names from the scenario YAML.
A cleaner approach is to add an optional `boldin_category` field to events in the YAML, but
name matching covers the common cases.

**Fallback**: Any income not matching a known category is placed under "Other Income".

**Gap**: `MeanAccumulators` does not accumulate per-event ledger data — it only tracks
snapshot, tax, cash flow, and inflation means. The mean income breakdown by source requires
a separate accumulator pass or approximation from `YearlyCashFlowSummary.income` totals.
See §3.6 for the pragmatic solution.

### 3.4 Expense

Source: `SimulationResult.ledger`

```rust
StateEvent::CashDebit { kind: CashFlowKind::Expense, .. }
```

Group by `(year, source_event_id → event_name)`, same name-matching approach as income.

| Boldin Row | finplan Signal |
|------------|----------------|
| General recurring | Default fallback for expense events |
| Medical | event name contains "medical", "health", "doctor" |
| Long Term Care | event name contains "long term care", "ltc", "care" |
| Rent | event name contains "rent", "housing" |
| One-Time Expense | One-time events (non-repeating) |
| IRMAA | event name contains "irmaa", "medicare" |
| Estimated Income Tax Payments | Skip — use Taxes category instead |
| Capital Gains Tax Payments | Skip — use Taxes category instead |

### 3.5 Contributions and Withdrawals

Source: `SimulationResult.ledger`

**Contributions**:
- `CashCredit { kind: Contribution, to: account_id }` — group by account name
- Excess Income: income that flows to the "default" account as a `Contribution` when
  there is no explicit destination — approximated as `CashCredit { kind: Income }` to
  the checking/default account when income > expenses for that period

**Withdrawals**:
- `CashDebit { kind: LiquidationProceeds, from: account_id }` — group by account name
- `RmdWithdrawal { account_id }` — group by account name
- Shortfall: when `CashDebit` amount exceeds available cash, flagged by a warning in
  `SimulationResult.warnings`. Approximate as `net_cash_flow < 0` in `YearlyCashFlowSummary`.

### 3.6 Taxes

Source: `SimulationResult.yearly_taxes` (`TaxSummary` per year)

| Boldin Row | finplan Field |
|------------|---------------|
| Federal Income Tax | `TaxSummary.federal_tax` |
| State Income Tax | `TaxSummary.state_tax` |
| Federal Capital Gains Tax | Estimated: `TaxSummary.total_tax - federal_tax - state_tax` |
| FICA | **Gap**: not tracked separately — output $0 or estimate from income |
| Other State Taxes | **Gap**: not tracked — output $0 |

**Note**: `TaxSummary.total_tax` = federal + state + capital gains tax combined. The CG
tax component must be estimated. A reasonable approximation:
```
cg_tax ≈ capital_gains_income × effective_cg_rate
```
where `effective_cg_rate` is the long-term CG rate at the user's bracket (typically 0, 15%,
or 20%). This is an approximation until finplan tracks CG tax separately.

### 3.7 Pragmatic Mean Approximation

For the "Average" (mean) scenario, ledger-level per-source-event breakdowns are not
available from `MeanAccumulators` (which only tracks totals). The pragmatic approach:

1. **Accounts**: Use `MeanAccumulators::build_mean_snapshots()` — exact.
2. **Taxes**: Use `MeanAccumulators::build_mean_taxes()` — exact for totals.
3. **Income/Expense/Contributions/Withdrawals totals**: Use
   `MeanAccumulators::build_mean_cash_flows()` — exact for category totals.
4. **Income/Expense breakdowns by event**: Use P50 percentile run's ledger as an
   approximation for the mean breakdown. This is a reasonable proxy since the median run
   has similar category structure to the mean even if absolute values differ slightly.

---

## 4. New MCP Tool: `export_planner_summary`

### 4.1 Tool Registration

File: `crates/finplan_mcp/src/tools/export.rs` (new file)

```rust
pub fn tools() -> Vec<Tool> {
    vec![make_tool(
        "export_planner_summary",
        "Export simulation results in Boldin-compatible planner summary CSV format. \
         Requires a completed Monte Carlo simulation (call run_monte_carlo first). \
         Returns a CSV string with Optimistic/Average/Pessimistic scenarios across \
         Accounts, Income, Expense, Contributions, Withdrawals, and Taxes categories.",
        json!({
            "type": "object",
            "properties": {
                "format": {
                    "type": "string",
                    "enum": ["csv", "json"],
                    "default": "csv",
                    "description": "Output format. 'csv' = Boldin-compatible CSV text. 'json' = structured JSON with same data."
                },
                "age_at_start": {
                    "type": ["integer", "null"],
                    "description": "Primary person's age in the first simulation year. Auto-derived from parameters.birth_date if omitted. Override here only if you need a different value."
                },
                "spouse_age_at_start": {
                    "type": ["integer", "null"],
                    "description": "Spouse age in the first simulation year. Auto-derived from parameters.spouse_birth_date if set. Override here only if you need a different value."
                },
                "percentile_optimistic": {
                    "type": "number",
                    "default": 0.95,
                    "description": "Which MC percentile to use as the Optimistic scenario (default 0.95 = P95)."
                },
                "percentile_pessimistic": {
                    "type": "number",
                    "default": 0.05,
                    "description": "Which MC percentile to use as the Pessimistic scenario (default 0.05 = P5)."
                }
            },
            "required": []
        }),
    )]
}
```

### 4.2 Implementation Sketch

```rust
pub fn export_planner_summary(args: &Value, state: &SharedState) -> Result<CallToolResult, McpError> {
    let st = state.lock().unwrap();

    let mc_summary = st.last_mc_summary.as_ref()
        .ok_or_else(|| error("No Monte Carlo results available. Call run_monte_carlo first."))?;

    let sim_data = st.last_sim_data.as_ref()
        .ok_or_else(|| error("No simulation data available."))?;

    let format = args["format"].as_str().unwrap_or("csv");

    // Auto-derive ages from stored parameters.birth_date / spouse_birth_date if not overridden.
    let start_year = get_simulation_start_year(optimistic); // first year in results
    let age_at_start: i32 = if let Some(a) = args["age_at_start"].as_i64() {
        a as i32
    } else if let Some(ref params) = st.parameters {
        // birth_date is "YYYY-MM-DD"; compute age at start_year
        let birth_year = params.birth_date[..4].parse::<i32>().unwrap_or(1960);
        start_year - birth_year
    } else {
        return error_result("age_at_start is required when parameters.birth_date is not set");
    };
    let spouse_age: Option<i32> = if let Some(a) = args["spouse_age_at_start"].as_i64() {
        Some(a as i32)
    } else if let Some(ref params) = st.parameters {
        params.spouse_birth_date.as_deref().and_then(|d| {
            d[..4].parse::<i32>().ok().map(|birth_year| start_year - birth_year)
        })
    } else {
        None
    };
    let pct_opt = args["percentile_optimistic"].as_f64().unwrap_or(0.95);
    let pct_pes = args["percentile_pessimistic"].as_f64().unwrap_or(0.05);

    // Get the three scenario results
    let optimistic = mc_summary.get_percentile(pct_opt)
        .ok_or_else(|| error(format!("P{} run not available", pct_opt * 100.0)))?;
    let pessimistic = mc_summary.get_percentile(pct_pes)
        .ok_or_else(|| error(format!("P{} run not available", pct_pes * 100.0)))?;
    let average_result = mc_summary.get_mean_result()
        .ok_or_else(|| error("Mean result not available (run_monte_carlo with compute_mean=true)"))?;

    // Build year range from simulation snapshots
    let years: Vec<i16> = get_simulation_years(optimistic);

    // Build account name map: AccountId → name
    let account_names: HashMap<AccountId, String> = sim_data.accounts.iter()
        .map(|a| (a.id, a.name.clone()))
        .collect();

    // Build event name map: EventId → name
    let event_names: HashMap<EventId, String> = sim_data.events.iter()
        .map(|e| (e.id, e.name.clone()))
        .collect();

    // Build rows for each scenario
    let scenarios = [
        ("Optimistic", optimistic),
        ("Average", &average_result),
        ("Pessimistic", pessimistic),
    ];

    let mut all_rows: Vec<PlannerRow> = Vec::new();

    for (scenario_name, result) in &scenarios {
        // Accounts rows
        let account_rows = build_account_rows(result, &account_names, &years);
        // Income rows
        let income_rows = build_income_rows(result, &event_names, &account_names, &years, sim_data);
        // Expense rows
        let expense_rows = build_expense_rows(result, &event_names, &years, sim_data);
        // Contribution rows
        let contribution_rows = build_contribution_rows(result, &account_names, &years);
        // Withdrawal rows
        let withdrawal_rows = build_withdrawal_rows(result, &account_names, &years);
        // Tax rows
        let tax_rows = build_tax_rows(result, &years);

        for row in [account_rows, income_rows, expense_rows,
                    contribution_rows, withdrawal_rows, tax_rows].concat() {
            all_rows.push(PlannerRow {
                scenario: scenario_name.to_string(),
                ..row
            });
        }
    }

    match format {
        "json" => {
            let json_output = json!({ "rows": all_rows, "years": years });
            text_result(serde_json::to_string_pretty(&json_output).unwrap())
        }
        _ => {
            // CSV output
            let csv = render_boldin_csv(&all_rows, &years, age_at_start, spouse_age);
            text_result(csv)
        }
    }
}
```

### 4.3 Key Data Structures

```rust
struct PlannerRow {
    scenario: String,   // "Optimistic", "Average", "Pessimistic"
    category: String,   // "Accounts", "Income", etc.
    item: String,       // account name or income source name
    values: Vec<f64>,   // one value per year (indexed same as years vec)
}
```

### 4.4 CSV Rendering

```rust
fn render_boldin_csv(rows: &[PlannerRow], years: &[i16], age_at_start: i32,
                     spouse_age: Option<i32>) -> String {
    let mut out = String::from("Assumptions,Category,Item");

    // Year column headers
    for (i, &year) in years.iter().enumerate() {
        let age = age_at_start + i as i32;
        if let Some(sp_age) = spouse_age {
            let sp = sp_age + i as i32;
            out.push_str(&format!(",[{}] age={}; spouse={}", year, age, sp));
        } else {
            out.push_str(&format!(",[{}] age={}", year, age));
        }
    }
    out.push('\n');

    // Data rows
    for row in rows {
        out.push_str(&format!("{},{},{}", row.scenario, row.category, row.item));
        for &val in &row.values {
            let formatted = format!("${:.2}", val);
            if formatted.contains(',') {
                out.push_str(&format!(",\"{}\"", formatted));
            } else {
                out.push_str(&format!(",{}", formatted));
            }
        }
        out.push('\n');
    }

    out
}
```

---

## 5. Helper Functions

### 5.1 Account Rows

```rust
fn build_account_rows(result: &SimulationResult, account_names: &HashMap<AccountId, String>,
                      years: &[i16]) -> Vec<PlannerRow> {
    // Find year-end snapshots
    let year_end_snapshots: HashMap<i16, &WealthSnapshot> = result.wealth_snapshots.iter()
        .filter(|s| s.date.month() == 12 && s.date.day() == s.date.days_in_month())
        .map(|s| (s.date.year() as i16, s))
        .collect();

    // Collect account IDs from first snapshot
    let account_ids: Vec<AccountId> = result.wealth_snapshots.first()
        .map(|s| s.accounts.iter().map(|a| a.account_id).collect())
        .unwrap_or_default();

    account_ids.iter().map(|&acc_id| {
        let name = account_names.get(&acc_id)
            .cloned().unwrap_or_else(|| format!("Account {}", acc_id.0));
        let values: Vec<f64> = years.iter().map(|&year| {
            year_end_snapshots.get(&year)
                .and_then(|snap| snap.accounts.iter()
                    .find(|a| a.account_id == acc_id)
                    .map(|a| a.total_value()))
                .unwrap_or(0.0)
        }).collect();
        PlannerRow { scenario: String::new(), category: "Accounts".into(), item: name, values }
    }).collect()
}
```

### 5.2 Income Rows (Ledger-Based)

```rust
fn build_income_rows(result: &SimulationResult, event_names: &HashMap<EventId, String>,
                     account_names: &HashMap<AccountId, String>, years: &[i16],
                     sim_data: &SimulationData) -> Vec<PlannerRow> {
    // Build: item_name → year_index → amount
    let mut by_source: HashMap<String, Vec<f64>> = HashMap::new();
    let year_to_idx: HashMap<i16, usize> = years.iter().enumerate()
        .map(|(i, &y)| (y, i)).collect();

    for entry in &result.ledger {
        let year = entry.date.year() as i16;
        let idx = match year_to_idx.get(&year) { Some(&i) => i, None => continue };

        let (amount, item_name) = match &entry.event {
            StateEvent::CashCredit { amount, kind, to: _, .. } => {
                match kind {
                    CashFlowKind::Income => {
                        let name = entry.source_event
                            .and_then(|eid| event_names.get(&eid))
                            .map(|n| classify_income_name(n))
                            .unwrap_or_else(|| "Other Income".into());
                        (*amount, name)
                    }
                    CashFlowKind::LiquidationProceeds => (*amount, "Savings Drawdown".into()),
                    _ => continue,
                }
            }
            StateEvent::RmdWithdrawal { amount, .. } => (*amount, "Required Minimum Distribution".into()),
            StateEvent::CashAppreciation { previous_value, new_value, .. } => {
                (new_value - previous_value, "Interest".into())
            }
            _ => continue,
        };

        *by_source.entry(item_name).or_insert_with(|| vec![0.0; years.len()])
            .get_mut(idx).unwrap() += amount;
    }

    // Emit in Boldin order
    let ordered_items = ["Work", "Social Security", "Pension", "Required Minimum Distribution",
                         "Savings Drawdown", "Interest", "Windfall", "Primary Home Sale", "Other Income"];
    ordered_items.iter()
        .filter_map(|&name| by_source.remove(name).map(|values| {
            PlannerRow { scenario: String::new(), category: "Income".into(),
                         item: name.into(), values }
        }))
        .chain(by_source.into_iter().map(|(item, values)| {
            PlannerRow { scenario: String::new(), category: "Income".into(), item, values }
        }))
        .collect()
}

fn classify_income_name(event_name: &str) -> String {
    let lower = event_name.to_lowercase();
    if lower.contains("social security") || lower.contains("ss ") || lower.starts_with("ss_") {
        "Social Security".into()
    } else if lower.contains("pension") {
        "Pension".into()
    } else if lower.contains("windfall") {
        "Windfall".into()
    } else if lower.contains("home") || lower.contains("house") || lower.contains("real estate") {
        "Primary Home Sale".into()
    } else if lower.contains("work") || lower.contains("salary") || lower.contains("job") {
        "Work".into()
    } else {
        event_name.to_string()  // Use event name as-is
    }
}
```

### 5.3 Tax Rows

```rust
fn build_tax_rows(result: &SimulationResult, years: &[i16]) -> Vec<PlannerRow> {
    let year_to_idx: HashMap<i16, usize> = years.iter().enumerate()
        .map(|(i, &y)| (y, i)).collect();

    let mut federal = vec![0.0; years.len()];
    let mut state_tax = vec![0.0; years.len()];
    let mut cap_gains = vec![0.0; years.len()];

    for tax in &result.yearly_taxes {
        let idx = match year_to_idx.get(&tax.year) { Some(&i) => i, None => continue };
        federal[idx] = tax.federal_tax;
        state_tax[idx] = tax.state_tax;
        // Estimate CG tax: total - federal - state
        cap_gains[idx] = (tax.total_tax - tax.federal_tax - tax.state_tax).max(0.0);
    }

    vec![
        PlannerRow { scenario: String::new(), category: "Taxes".into(),
                     item: "Federal Income Tax".into(), values: federal },
        PlannerRow { scenario: String::new(), category: "Taxes".into(),
                     item: "State Income Tax".into(), values: state_tax },
        PlannerRow { scenario: String::new(), category: "Taxes".into(),
                     item: "Federal Capital Gains Tax".into(), values: cap_gains },
        PlannerRow { scenario: String::new(), category: "Taxes".into(),
                     item: "FICA".into(), values: vec![0.0; years.len()] }, // gap
        PlannerRow { scenario: String::new(), category: "Taxes".into(),
                     item: "Other State Taxes".into(), values: vec![0.0; years.len()] }, // gap
    ]
}
```

---

## 6. State Changes Required

The `export_planner_summary` tool requires that `run_monte_carlo` (from MCP_SERVER_PLAN2.md)
has already run and stored:
- `state.last_mc_summary: Option<MonteCarloSummary>`
- `state.last_sim_data: Option<SimulationData>`

No new state fields are needed.

---

## 7. Known Gaps and Approximations

| Gap | Boldin Field | Finplan Status | Workaround |
|-----|--------------|----------------|------------|
| FICA tax | FICA row | Not tracked | Output $0.00 |
| Other State Taxes | Other State Taxes | Not tracked | Output $0.00 |
| CG tax separate from income tax | Federal Capital Gains Tax | Bundled in `total_tax` | Estimate: `total - federal - state` |
| Per-source income breakdown for mean result | All Income rows | Ledger not averaged | Use P50 run's ledger as proxy |
| Shortfall labeling | Shortfall row | Tracked via warnings | Approximate from `net_cash_flow < 0` |
| Excess Income | Excess Income row | Not explicitly tracked | Approximate: `income - expenses` when positive, to default account |
| Event-name classification | All Income/Expense rows | Name-based heuristics | User should name events consistently |

---

## 8. File Changes Summary

| File | Change |
|------|--------|
| `crates/finplan_mcp/src/tools/export.rs` | New file: `export_planner_summary` tool |
| `crates/finplan_mcp/src/tools/mod.rs` | Add `mod export`, register tool and dispatcher |
| `crates/finplan_mcp/Cargo.toml` | No new dependencies needed |

---

## 9. Typical Workflow

```
AI Agent tool call sequence:
1. set_parameters { birth_date: "1961-06-01", spouse_birth_date: "1964-03-15", start_year: 2026, ... }
2. set_portfolio { ... }
3. add_income_event { name: "Work", ... }
4. add_social_security_event { name: "Social Security", ... }
5. add_social_security_event { name: "Spouse Social Security", ... }
6. add_expense_event { name: "General recurring", ... }
7. validate_scenario
8. run_monte_carlo { iterations: 1000, compute_mean: true }
9. export_planner_summary {}
   → ages auto-derived from birth_date / spouse_birth_date in stored parameters
   → returns CSV text matching Boldin format
```

---

## 10. Implementation Priority

This tool is self-contained once `run_monte_carlo` (MCP_SERVER_PLAN2.md) is implemented.
The ledger-walking and CSV rendering are straightforward. The main complexity is the
name-matching heuristics for income/expense categorization.

**Estimated effort**: 1–2 days of implementation.
**Highest-value deliverable**: The `export_planner_summary` CSV output, since it lets users
directly compare finplan's projections with their existing Boldin analysis.
