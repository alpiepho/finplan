# FinPlan MCP Server — Implementation Plan

> **Branch**: `feature/mcp`
> **Crate**: `crates/finplan_mcp`
> **Goal**: Expose the FinPlan scenario YAML schema, provide guided scenario construction tools, and validate completed scenarios — all over the Model Context Protocol (MCP) so an external AI agent can build a complete scenario file from statement data + user interview answers.

---

## Table of Contents

1. [Overview & Data Flow](#1-overview--data-flow)
2. [Crate Setup](#2-crate-setup)
3. [MCP Resources](#3-mcp-resources)
4. [MCP Tools](#4-mcp-tools)
5. [MCP Prompts](#5-mcp-prompts)
6. [Interview Engine Design](#6-interview-engine-design)
7. [Ticker-to-Profile Mapping](#7-ticker-to-profile-mapping)
8. [Merge & Assembly Logic](#8-merge--assembly-logic)
9. [Validation Pipeline](#9-validation-pipeline)
10. [Testing Strategy](#10-testing-strategy)
11. [File-by-File Implementation Checklist](#11-file-by-file-implementation-checklist)
12. [Example End-to-End Session](#12-example-end-to-end-session)
13. [Future Work](#13-future-work)

---

## 1. Overview & Data Flow

```
┌──────────────────────┐      ┌──────────────────────┐
│  Statements MCP      │      │  AI Agent            │
│  (bank/brokerage     │      │  (Claude, GPT, etc.) │
│   PDF parser)        │      │                      │
└──────────┬───────────┘      └──────────┬───────────┘
           │                             │
           │ accounts + holdings JSON    │ interview answers
           │                             │
           ▼                             ▼
     ┌─────────────────────────────────────────┐
     │         finplan_mcp  (this server)      │
     │                                         │
     │  Resources:                             │
     │    • schema://full        (YAML ref)    │
     │    • schema://accounts    (account types)│
     │    • schema://events      (event types) │
     │    • schema://parameters  (param fields)│
     │    • schema://profiles    (return profs)│
     │    • schema://amounts     (amount types)│
     │    • schema://triggers    (trigger types)│
     │    • schema://analysis    (analysis cfg)│
     │    • schema://example     (example.yaml)│
     │    • schema://patterns    (common combos)│
     │                                         │
     │  Tools:                                 │
     │    • set_parameters       → YAML section│
     │    • add_income_event     → event YAML  │
     │    • add_expense_event    → event YAML  │
     │    • add_contribution     → event YAML  │
     │    • add_retirement       → event YAML  │
     │    • add_social_security  → event YAML  │
     │    • add_rmd              → event YAML  │
     │    • add_home_purchase    → events YAML │
     │    • add_custom_event     → event YAML  │
     │    • set_portfolio        → portfolio   │
     │    • map_tickers          → profiles +  │
     │                            assets YAML  │
     │    • merge_scenario       → full YAML   │
     │    • validate_scenario    → errors/ok   │
     │    • get_ticker_info      → profile data│
     │                                         │
     │  Prompts:                               │
     │    • build_scenario       (guided)      │
     │    • quick_retirement     (template)    │
     │                                         │
     └─────────────────────────────────────────┘
                      │
                      ▼
              scenario.yaml  (complete, validated)
```

### Intended Workflow

1. **Statement parsing MCP** extracts account names, types, balances, and asset holdings from bank/brokerage PDFs → produces a `portfolio` JSON blob.
2. **AI agent** calls `set_portfolio` on **finplan_mcp** with that portfolio data.
3. **AI agent** reads `schema://` resources to understand the YAML format.
4. **AI agent** calls interview-style tools (`set_parameters`, `add_income_event`, etc.) with user-provided answers or inferred defaults.
5. **AI agent** calls `map_tickers` to auto-map holdings to return profiles using the built-in ticker database.
6. **AI agent** calls `merge_scenario` to assemble the full YAML.
7. **AI agent** calls `validate_scenario` to check for errors and fix them.
8. **AI agent** delivers the validated YAML file to the user.

---

## 2. Crate Setup

### 2.1 Workspace Addition

Add to the root `Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
    "crates/finplan",
    "crates/finplan_core",
    "crates/finplan_mcp",        # ← NEW
]
```

### 2.2 New Crate: `crates/finplan_mcp/Cargo.toml`

```toml
[package]
name = "finplan_mcp"
version = "0.1.0"
edition = "2024"
description = "MCP server for FinPlan scenario YAML construction"

[[bin]]
name = "finplan-mcp"
path = "src/main.rs"

[dependencies]
# Internal
finplan = { path = "../finplan" }

# MCP SDK
rmcp = { version = "0.17", features = [
    "server",
    "transport-io",
] }

# Async runtime
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { workspace = true }
serde_json = "1"
serde_saphyr = "0.0.8"

# JSON Schema generation (required by rmcp #[tool] macro)
schemars = "1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[dev-dependencies]
tokio-test = "0.4"
pretty_assertions = "1"
```

### 2.3 Directory Layout

```
crates/finplan_mcp/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point: stdio transport setup
│   ├── server.rs            # MCP ServerHandler impl + tool/prompt routers
│   ├── resources.rs         # Schema resource definitions
│   ├── tools/
│   │   ├── mod.rs           # Tool router aggregate
│   │   ├── parameters.rs    # set_parameters tool
│   │   ├── portfolio.rs     # set_portfolio tool
│   │   ├── events.rs        # add_*_event tools (income, expense, contribution, etc.)
│   │   ├── retirement.rs    # add_retirement, add_social_security, add_rmd tools
│   │   ├── home_purchase.rs # add_home_purchase tool
│   │   ├── custom_event.rs  # add_custom_event tool
│   │   ├── ticker.rs        # map_tickers, get_ticker_info tools
│   │   ├── merge.rs         # merge_scenario tool
│   │   └── validate.rs      # validate_scenario tool
│   ├── prompts.rs           # MCP prompt definitions
│   ├── schema_text.rs       # Static schema documentation strings
│   ├── interview.rs         # Interview defaults & question metadata
│   ├── state.rs             # Server state: accumulated scenario sections
│   └── defaults.rs          # Sensible default values table
└── tests/
    ├── integration_test.rs  # Full end-to-end scenario build
    ├── tool_tests.rs        # Individual tool unit tests
    ├── resource_tests.rs    # Resource content tests
    └── validation_tests.rs  # Validation pipeline tests
```

---

## 3. MCP Resources

Resources provide **read-only documentation** the AI agent can fetch to understand the YAML format. They are registered on the server and returned via `resources/list` and `resources/read`.

### 3.1 Resource Definitions

| URI | Name | Description |
|-----|------|-------------|
| `schema://full` | Full YAML Reference | Complete scenario YAML reference (derived from `docs/08-scenario-yaml-reference.md`) |
| `schema://accounts` | Account Types | All 13 account types with fields and examples |
| `schema://events` | Event Structure | Event structure: triggers, effects, once, enabled |
| `schema://triggers` | Trigger Types | All 10 trigger types with fields and YAML examples |
| `schema://effects` | Effect Types | All 14 effect types with fields and YAML examples |
| `schema://amounts` | Amount Types | All 8 amount types with nesting examples |
| `schema://parameters` | Parameters | Parameter fields: dates, inflation, taxes, returns_mode |
| `schema://profiles` | Return Profiles | 7 profile types (Fixed, Normal, LogNormal, StudentT, RegimeSwitching, Bootstrap) + presets |
| `schema://analysis` | Analysis Config | MC iterations, sweep parameters, metrics, chart configs |
| `schema://example` | Example Scenario | The complete `examples/example.yaml` contents |
| `schema://patterns` | Common Patterns | Reusable event pattern templates (salary, retirement spending, 401k, SS, RMD, home purchase) |

### 3.2 Implementation Notes

- Resources should be **static text** compiled into the binary (use `include_str!` for the example YAML and doc file, or generate text from the data structures).
- Each resource returns `Content::text(...)` with well-formatted Markdown.
- The `schema://accounts` resource should list every variant of `AccountType` with its inner struct fields, derived from `portfolio_data.rs`.

### 3.3 Account Types Resource Content

The `schema://accounts` resource must document these 13 types:

```
INVESTMENT ACCOUNTS (hold assets):
  Brokerage      — Taxable investment account. Fields: assets[{asset, value}]
  Traditional401k — Tax-deferred employer plan. Fields: assets[{asset, value}]
  Roth401k       — After-tax employer plan. Fields: assets[{asset, value}]
  TraditionalIRA — Tax-deferred individual. Fields: assets[{asset, value}]
  RothIRA        — Tax-free individual. Fields: assets[{asset, value}]

CASH/PROPERTY ACCOUNTS:
  Checking       — Cash account. Fields: value, return_profile (optional)
  Savings        — Cash/HYSA. Fields: value, return_profile (optional)
  HSA            — Health savings. Fields: value (treated as cash currently)
  Property       — Real estate. Fields: value, return_profile (optional)
  Collectible    — Art, gold, etc. Fields: value, return_profile (optional)

DEBT ACCOUNTS:
  Mortgage       — Home loan. Fields: balance, interest_rate
  LoanDebt       — Personal/other. Fields: balance, interest_rate
  StudentLoanDebt — Student loans. Fields: balance, interest_rate
```

### 3.4 Trigger Types Resource Content

```
Date            — { type: Date, date: "YYYY-MM-DD" }
Age             — { type: Age, years: u8, months?: u8 }
RelativeToEvent — { type: RelativeToEvent, event: "EventName", offset: { unit: Days|Months|Years, value: i32 } }
AccountBalance  — { type: AccountBalance, account: "Name", threshold: { comparison: LessThanOrEqual|GreaterThanOrEqual, value: f64 } }
AssetBalance    — { type: AssetBalance, account: "Name", asset: "TICKER", threshold: {...} }
NetWorth        — { type: NetWorth, threshold: { comparison: ..., value: ... } }
And             — { type: And, conditions: [trigger, trigger, ...] }
Or              — { type: Or, conditions: [trigger, trigger, ...] }
Repeating       — { type: Repeating, interval: weekly|biweekly|monthly|quarterly|yearly, start?: trigger, end?: trigger, max_occurrences?: u32 }
Manual          — { type: Manual } — only fired by TriggerEvent effects
```

### 3.5 Effect Types Resource Content

```
Income          — { type: Income, to: "account", amount: {...}, gross: bool, taxable: bool }
Expense         — { type: Expense, from: "account", amount: {...} }
AssetPurchase   — { type: AssetPurchase, from: "account", to_account: "account", asset: "TICKER", amount: {...} }
AssetSale       — { type: AssetSale, from: "account", asset?: "TICKER", amount: {...}, gross: bool, lot_method: fifo|lifo|highest_cost|lowest_cost|average_cost }
Sweep           — { type: Sweep, to: "account", amount: {...}, strategy: penalty_aware|tax_efficient|tax_deferred_first|tax_free_first|pro_rata, gross: bool, taxable: bool, lot_method: ..., exclude_accounts?: ["name"] }
TriggerEvent    — { type: TriggerEvent, event: "EventName" }
PauseEvent      — { type: PauseEvent, event: "EventName" }
ResumeEvent     — { type: ResumeEvent, event: "EventName" }
TerminateEvent  — { type: TerminateEvent, event: "EventName" }
ApplyRmd        — { type: ApplyRmd, destination: "account", lot_method: ... }
AdjustBalance   — { type: AdjustBalance, account: "account", amount: {...} }
CashTransfer    — { type: CashTransfer, from: "account", to: "account", amount: {...} }
Random          — { type: Random, probability: f64, on_true: "EventName", on_false?: "EventName" }
RsuVesting      — { type: RsuVesting, to: "account", asset: "TICKER", units: f64, sell_to_cover: bool, lot_method: ... }
```

### 3.6 Amount Types Resource Content

```
Fixed               — { type: Fixed, value: 4500.0 } (or bare float: 4500.0)
InflationAdjusted   — { type: InflationAdjusted, inner: <amount> }
Scale               — { type: Scale, multiplier: 0.04, inner: <amount> }
SourceBalance       — { type: SourceBalance }
ZeroTargetBalance   — { type: ZeroTargetBalance }
TargetToBalance     — { type: TargetToBalance, target: 130000.0 }
AccountBalance      — { type: AccountBalance, account: "Checking" }
AccountCashBalance  — { type: AccountCashBalance, account: "Savings" }
```

---

## 4. MCP Tools

Each tool accepts structured JSON parameters (with JSON Schema from `schemars`) and returns either YAML text or structured JSON. The AI agent accumulates sections by calling tools sequentially, then merges them.

### 4.1 Tool: `set_parameters`

Sets the simulation parameters section.

**Input Schema:**

```json
{
  "birth_date": "1985-06-15",           // REQUIRED — YYYY-MM-DD
  "start_date": "2026-01-01",           // default: current year Jan 1
  "duration_years": 40,                 // default: 30
  "state_tax_rate": 0.05,               // default: 0.05
  "capital_gains_rate": 0.15,           // default: 0.15
  "federal_brackets": "single2024",     // default: "single2024", options: "single2024", "married_joint2024"
  "inflation_type": "us_historical",    // default: "us_historical", options: "us_historical", "fixed", "none"
  "inflation_rate": null,               // only used if inflation_type == "fixed"
  "returns_mode": "historical",         // default: "historical", options: "historical", "parametric"
  "historical_block_size": 5            // default: 5, only used if returns_mode == "historical"
}
```

**Output:** YAML string for the `parameters:` section + confirmation text.

**Defaults Logic:**
- If `start_date` is omitted, use January 1st of the current year.
- If `duration_years` is omitted, calculate `(100 - current_age)` years, minimum 30.
- If `returns_mode` is omitted, default to `"historical"` (more realistic for most users).

### 4.2 Tool: `add_income_event`

Adds a recurring income event (salary, freelance, etc.).

**Input Schema:**

```json
{
  "name": "Bi-Weekly Salary",                  // REQUIRED
  "description": "Primary employment income",  // optional
  "to_account": "Checking",                    // REQUIRED — destination account name
  "amount": 4500.0,                            // REQUIRED — per-period amount
  "interval": "biweekly",                      // default: "biweekly", options: weekly|biweekly|monthly|quarterly|yearly
  "inflation_adjusted": true,                  // default: true
  "gross": true,                               // default: true
  "taxable": true,                             // default: true
  "start_age": null,                           // optional — start at this age
  "end_event": "Retirement",                   // optional — end when this event fires
  "end_age": null                              // optional — end at this age (alternative to end_event)
}
```

**Output:** YAML string for one `events:` list entry.

### 4.3 Tool: `add_expense_event`

Adds a recurring expense event.

**Input Schema:**

```json
{
  "name": "Living Expenses",                    // REQUIRED
  "description": "Monthly living costs",        // optional
  "from_account": "Checking",                   // REQUIRED
  "amount": 5500.0,                             // REQUIRED
  "interval": "monthly",                        // default: "monthly"
  "inflation_adjusted": true,                   // default: true
  "start_age": null,                            // optional
  "end_age": null,                              // optional
  "end_event": null                             // optional
}
```

**Output:** YAML string for one `events:` list entry.

### 4.4 Tool: `add_contribution`

Adds a recurring investment contribution (401k, IRA, etc.).

**Input Schema:**

```json
{
  "name": "401k Contribution",                  // REQUIRED
  "from_account": "Checking",                   // REQUIRED — source of cash
  "to_account": "401k",                         // REQUIRED — investment account
  "asset": "FXAIX",                             // REQUIRED — ticker to buy
  "amount": 24500.0,                            // REQUIRED — annual amount
  "interval": "yearly",                         // default: "yearly"
  "end_event": "Retirement",                    // default: "Retirement"
  "employer_match": null,                       // optional: { "rate": 0.50, "limit": 6.0 } — match 50% up to 6% of salary
  "end_age": null                               // optional
}
```

**Output:** YAML string for one (or two, if employer match) event entries.

**Special Behavior:** If `employer_match` is provided, generate a second event named `"{name} Employer Match"` with the matching amount and same schedule.

### 4.5 Tool: `add_retirement`

Adds the retirement marker event and optional post-retirement spending.

**Input Schema:**

```json
{
  "retirement_age": 65,                         // REQUIRED
  "event_name": "Retirement",                   // default: "Retirement"
  "annual_spending": 100000.0,                  // optional — post-retirement annual spending target
  "spending_account": "Checking",               // default: "Checking"
  "withdrawal_strategy": "penalty_aware",       // default: "penalty_aware"
  "lot_method": "fifo"                          // default: "fifo"
}
```

**Output:** YAML string for 1-2 events:
1. The marker event: `Retirement` at the given age (once: true)
2. If `annual_spending` is provided: a `Yearly Spend Post-Retirement` event with a Sweep effect using `TargetToBalance` amount, starting relative to the retirement event.

### 4.6 Tool: `add_social_security`

Adds Social Security income.

**Input Schema:**

```json
{
  "claim_age": 67,                              // default: 67 (full retirement age)
  "monthly_benefit": 2000.0,                    // REQUIRED — estimated monthly benefit
  "to_account": "Checking",                     // default: "Checking"
  "taxable": true                               // default: true
}
```

**Output:** YAML string for one repeating monthly income event starting at the given age.

### 4.7 Tool: `add_rmd`

Adds Required Minimum Distribution event.

**Input Schema:**

```json
{
  "start_age": 73,                              // default: 73
  "destination": "Checking",                    // default: "Checking"
  "lot_method": "fifo"                          // default: "fifo"
}
```

**Output:** YAML string for one repeating yearly ApplyRmd event.

### 4.8 Tool: `add_home_purchase`

Adds a home purchase with mortgage. Generates multiple coordinated events.

**Input Schema:**

```json
{
  "purchase_date": "2028-06-17",                // REQUIRED — YYYY-MM-DD
  "home_value": 600000.0,                       // REQUIRED
  "down_payment": 120000.0,                     // REQUIRED
  "mortgage_rate": 0.054,                       // REQUIRED — annual rate as decimal
  "monthly_payment": 3000.0,                    // REQUIRED — monthly mortgage payment
  "down_payment_source": "Checking"             // default: "Checking"
}
```

**Output:** YAML string with:
1. Two new accounts: `Mortgage` (Debt) and `House Value` (Property)
2. `Buy House` event with AdjustBalance effects for mortgage balance and house value
3. `Mortgage Payment` event with CashTransfer, starting relative to Buy House, ending when mortgage balance ≤ 0

### 4.9 Tool: `add_custom_event`

A flexible tool for creating any event that doesn't fit the templates above.

**Input Schema:**

```json
{
  "name": "Medicare Part B",                    // REQUIRED
  "description": "Medicare premiums",           // optional
  "trigger": {                                  // REQUIRED — raw trigger object
    "type": "Repeating",
    "interval": "monthly",
    "start": { "type": "Age", "years": 65 }
  },
  "effects": [                                  // REQUIRED — raw effects array
    {
      "type": "Expense",
      "from": "Checking",
      "amount": { "type": "Fixed", "value": 174.70 }
    }
  ],
  "once": false,                                // default: false
  "enabled": true                               // default: true
}
```

**Output:** YAML string for the event, after validating the trigger and effect structures.

### 4.10 Tool: `set_portfolio`

Sets the portfolio section (accounts and holdings). This is typically called with data from the statements-parsing MCP.

**Input Schema:**

```json
{
  "name": "My Retirement Plan",                 // REQUIRED
  "description": "Generated from statements",   // optional
  "accounts": [                                 // REQUIRED
    {
      "name": "Checking",
      "type": "Checking",
      "value": 20000.0,
      "return_profile": null
    },
    {
      "name": "Brokerage",
      "type": "Brokerage",
      "assets": [
        { "asset": "VFIAX", "value": 200000.0 },
        { "asset": "VTIAX", "value": 33000.0 }
      ]
    },
    {
      "name": "401k",
      "type": "Traditional401k",
      "assets": [
        { "asset": "FXAIX", "value": 42000.0 }
      ]
    },
    {
      "name": "Mortgage",
      "type": "Mortgage",
      "balance": 350000.0,
      "interest_rate": 0.054
    }
  ]
}
```

**Output:** Confirmation + list of all tickers found (so agent knows what to map).

### 4.11 Tool: `map_tickers`

Auto-maps asset tickers to return profiles using the built-in ticker database (`ticker_profiles.rs`). For unknown tickers, returns a list that needs manual mapping.

**Input Schema:**

```json
{
  "tickers": ["VFIAX", "VTIAX", "FXAIX", "PLTR"],  // optional — if omitted, uses tickers from set_portfolio
  "returns_mode": "historical",                       // default: from parameters, or "historical"
  "custom_mappings": {                                // optional — manual overrides
    "PLTR": "S&P 500"
  }
}
```

**Output:** JSON with:
- `profiles`: generated `profiles:` YAML section (only for parametric mode)
- `assets`: generated `assets:` YAML section (for parametric mode)
- `historical_assets`: generated `historical_assets:` YAML section (for historical mode)
- `unmapped`: list of tickers with no known mapping (need user input)
- `suggestions`: for unmapped tickers, suggested similar profiles

**Built-in Ticker Database** (from `ticker_profiles.rs`):

| Category | Tickers | Parametric Profile | Historical Preset |
|----------|---------|-------------------|-------------------|
| US Total Market | VTI, VTSAX, ITOT, SPTM, SCHB, FSKAX, FZROX | Normal(0.1147, 0.1815) | S&P 500 |
| S&P 500 | VOO, SPY, IVV, VFIAX, FXAIX, SWPPX | Normal(0.1147, 0.1815) | S&P 500 |
| US Small Cap | VB, IJR, SCHA, VBR, IWM, VIOO, VSMAX | Normal(0.1477, 0.2780) | US Small Cap |
| US Agg Bond | BND, AGG, VBTLX, SCHZ, FBND, FXNAX | Normal(0.0312, 0.0469) | US Agg Bonds |
| Intl Developed | VXUS, VEA, EFA, IXUS, IEFA, SWISX, FSPSX | Normal(0.0778, 0.1883) | Intl Developed |
| Emerging Markets | VWO, IEMG, EEM, SCHE, VEMAX | Normal(0.1073, 0.3475) | Emerging Markets |
| REITs | VNQ, IYR, SCHH, FREL, VGSLX, RWR | Normal(0.0828, 0.1959) | REITs |
| Money Market | VGSH, SHV, BIL, VMFXX, SPAXX, FDRXX, SGOV | Normal(0.0342, 0.0305) | US T-Bills |
| Long-Term Treasury | TLT, VGLT, EDV, SPTL, ZROZ | Normal(0.0477, 0.0701) | US Long Bonds |
| TIPS | TIP, VTIP, SCHP, STIP, FIPDX | Normal(0.0359, 0.0607) | TIPS |
| US Corporate Bond | LQD, VCIT, IGIB, SPIB, VCSH | Normal(0.0441, 0.0698) | US Corp Bonds |
| Gold | GLD, IAU, SGOL, GLDM | Normal(0.1317, 0.1734) | Gold |

Historical presets available for `historical_assets` mapping:
- `S&P 500`, `US Small Cap`, `US T-Bills`, `US Long Bonds`, `Intl Developed`, `Emerging Markets`, `REITs`, `Gold`, `US Agg Bonds`, `US Corp Bonds`, `TIPS`

### 4.12 Tool: `merge_scenario`

Assembles all accumulated sections into one complete scenario YAML.

**Input Schema:**

```json
{
  "validate": true                              // default: true — run validation after merge
}
```

**Output:** The complete scenario YAML string. If `validate` is true and errors are found, returns the YAML + a list of validation errors.

**Assembly Order:**
1. `portfolios:` (from `set_portfolio`)
2. `profiles:` (from `map_tickers`, parametric mode only)
3. `assets:` (from `map_tickers`, parametric mode only)
4. `historical_assets:` (from `map_tickers`, historical mode only)
5. `asset_prices:` (for any tickers with explicit prices)
6. `events:` (accumulated from all `add_*` tools, in order added)
7. `parameters:` (from `set_parameters`)
8. `analysis:` (default config with sensible MC iterations)

### 4.13 Tool: `validate_scenario`

Validates a scenario YAML string (either from `merge_scenario` or provided directly).

**Input Schema:**

```json
{
  "yaml": "portfolios:\n  name: ..."           // REQUIRED — full YAML string
}
```

**Output:** JSON with:
- `valid`: boolean
- `errors`: array of `{ section, message, help }` objects (same as `ValidationError` from `validator.rs`)
- `warnings`: array of non-fatal suggestions

**Validation checks** (delegates to existing `validate_scenario()` in `validator.rs`):
1. Portfolio name not empty, ≥ 1 account, no duplicate account names
2. All profile references resolve (assets → profiles, historical_assets → presets)
3. All event references resolve (accounts and events exist)
4. Date formats are valid (YYYY-MM-DD)
5. Duration > 0
6. MC iterations ≥ 10

### 4.14 Tool: `get_ticker_info`

Look up a single ticker in the built-in database.

**Input Schema:**

```json
{
  "ticker": "VFIAX"                             // REQUIRED
}
```

**Output:** JSON with:
- `known`: boolean
- `category`: e.g., "S&P 500"
- `parametric_profile`: { type, mean, std_dev }
- `historical_preset`: e.g., "S&P 500"
- `similar_tickers`: other tickers in the same category

---

## 5. MCP Prompts

Prompts provide pre-built message templates the AI agent can use to guide conversations with the end user.

### 5.1 Prompt: `build_scenario`

A guided multi-step prompt for building a complete scenario.

```json
{
  "name": "build_scenario",
  "description": "Guided interview to build a complete FinPlan scenario YAML file. Walks through parameters, income, expenses, retirement, and other life events.",
  "arguments": [
    {
      "name": "portfolio_json",
      "description": "JSON portfolio data from statements parser (accounts + holdings). If not provided, portfolio will need to be set manually.",
      "required": false
    }
  ]
}
```

**Prompt Template:**

```
You are building a FinPlan retirement planning scenario. Follow these steps:

1. PARAMETERS: Ask the user for:
   - Date of birth (REQUIRED)
   - State of residence (to determine tax rate)
   - Filing status (single or married)
   - How many years to simulate (suggest: to age 95)

2. INCOME: For each income source:
   - Source name, amount per paycheck, frequency
   - Will it end at retirement?

3. EXPENSES: Monthly living expenses
   - Current monthly spending
   - Any planned large expenses?

4. RETIREMENT:
   - Target retirement age
   - Expected annual spending in retirement
   - Social Security claim age and estimated benefit

5. CONTRIBUTIONS:
   - 401k/IRA contributions and target funds
   - Any employer match?

6. Call the appropriate tools with the answers.
7. Call merge_scenario and validate_scenario.
8. Return the final YAML to the user.
```

### 5.2 Prompt: `quick_retirement`

A template for users who just want a basic retirement plan with minimal inputs.

```json
{
  "name": "quick_retirement",
  "description": "Generate a basic retirement scenario with minimal inputs. Good for getting started quickly.",
  "arguments": [
    {
      "name": "birth_date",
      "description": "Date of birth (YYYY-MM-DD)",
      "required": true
    },
    {
      "name": "retirement_age",
      "description": "Target retirement age",
      "required": false
    },
    {
      "name": "annual_income",
      "description": "Current gross annual income",
      "required": true
    },
    {
      "name": "monthly_expenses",
      "description": "Current monthly expenses",
      "required": true
    }
  ]
}
```

---

## 6. Interview Engine Design

The interview engine provides sensible defaults so the AI agent can fill in gaps without asking the user 50 questions.

### 6.1 Default Values Table

| Field | Default | Rationale |
|-------|---------|-----------|
| `start_date` | Jan 1 of current year | Most natural starting point |
| `duration_years` | `100 - current_age` (min 30) | Simulate to age 100 |
| `inflation_type` | `USHistorical` | Most realistic |
| `inflation_distribution` | `lognormal` | Better tail behavior |
| `returns_mode` | `historical` | More realistic than parametric |
| `historical_block_size` | `5` | Balance between correlation and diversity |
| `state_tax_rate` | `0.05` (5%) | Median US state rate |
| `capital_gains_rate` | `0.15` | Most common bracket |
| `federal_brackets` | `single2024` | Conservative assumption |
| `retirement_age` | `65` | Standard full retirement age |
| `social_security_age` | `67` | Full benefit age |
| `rmd_start_age` | `73` | Current IRS rule |
| `income_interval` | `biweekly` | Most common US pay schedule |
| `expense_interval` | `monthly` | Natural budgeting period |
| `contribution_interval` | `yearly` | Simplest for 401k limits |
| `withdrawal_strategy` | `penalty_aware` | Avoids early penalties |
| `lot_method` | `fifo` | Standard approach |
| `mc_iterations` | `500` | Good accuracy/speed tradeoff |
| `income_gross` | `true` | Most salaries are gross |
| `income_taxable` | `true` | Most income is taxable |
| `inflation_adjusted` | `true` for expenses, income | Maintain purchasing power |

### 6.2 State Tax Rate Lookup

Provide a helper that maps US state abbreviation → tax rate:

```rust
pub fn state_tax_rate(state: &str) -> f64 {
    match state.to_uppercase().as_str() {
        "AK" | "FL" | "NV" | "NH" | "SD" | "TN" | "TX" | "WA" | "WY" => 0.0,
        "CA" => 0.0930,
        "NY" => 0.0685,
        "NJ" => 0.0897,
        "IL" => 0.0495,
        "PA" => 0.0307,
        "OH" => 0.0399,
        "GA" => 0.0549,
        "NC" => 0.0450,
        "MA" => 0.0500,
        "VA" => 0.0575,
        "CO" => 0.0440,
        "MN" => 0.0985,
        "OR" => 0.0990,
        "HI" => 0.1100,
        // ... add all 50 states
        _ => 0.05, // default fallback
    }
}
```

### 6.3 Social Security Estimator

If the user doesn't know their SS benefit, provide a rough estimate:

```
Monthly SS ≈ (annual_income × 0.40) / 12   # for income up to ~$70k
Monthly SS ≈ $2,500 + (annual_income - 70000) × 0.15 / 12   # for higher income
Maximum (2024): ~$4,873/month at age 70
```

This is just a rough guide — recommend the user check ssa.gov for their actual estimate.

---

## 7. Ticker-to-Profile Mapping

### 7.1 Design Decision

The ticker-to-profile mapping logic already exists in `finplan/src/data/ticker_profiles.rs`. The MCP crate should **re-use this module directly** by depending on the `finplan` crate (which is a library + binary).

The `finplan` crate already exposes:
- `PROFILE_CATEGORIES` — parametric profile data for 12 asset categories with ~80+ tickers
- `HISTORICAL_PRESETS` — 11 historical bootstrap presets
- `get_suggestion(ticker)` → `Option<TickerMatch>` for parametric lookup
- `get_historical_suggestion(ticker)` → historical preset key + display name
- `is_known_ticker(ticker)` → bool

### 7.2 Mapping Logic for `map_tickers` Tool

```
For each ticker found in portfolio holdings:
  1. Try get_suggestion(ticker) for parametric profile
  2. Try get_historical_suggestion(ticker) for historical preset
  3. If returns_mode == "historical":
       → Add to historical_assets: { TICKER: "Display Name" }
     If returns_mode == "parametric":
       → Add the profile to profiles[] (deduplicated by name)
       → Add to assets: { TICKER: "Profile Name" }
  4. If ticker not found → add to unmapped list
  5. Apply any custom_mappings overrides
```

### 7.3 Cash Account Return Profiles

For Checking/Savings accounts with `return_profile`, use:
- `"US T-Bills"` as the default historical mapping
- A `Fixed { rate: 0.045 }` profile named "HYSA" for parametric mode savings accounts

---

## 8. Merge & Assembly Logic

### 8.1 Server State

The server maintains mutable state accumulating scenario sections across tool calls:

```rust
pub struct ScenarioState {
    /// Portfolio section (from set_portfolio)
    pub portfolio: Option<PortfolioData>,

    /// Parameters section (from set_parameters)
    pub parameters: Option<ParametersData>,

    /// Accumulated events (from add_* tools)
    pub events: Vec<EventData>,

    /// Return profiles (from map_tickers, parametric mode)
    pub profiles: Vec<ProfileData>,

    /// Asset-to-profile mappings (parametric)
    pub assets: HashMap<AssetTag, ReturnProfileTag>,

    /// Asset-to-historical-preset mappings
    pub historical_assets: HashMap<AssetTag, ReturnProfileTag>,

    /// Explicit asset prices
    pub asset_prices: HashMap<AssetTag, f64>,

    /// Tracking errors
    pub asset_tracking_errors: HashMap<AssetTag, f64>,

    /// Analysis config
    pub analysis: AnalysisConfigData,
}
```

State is behind `Arc<Mutex<ScenarioState>>` for thread safety.

### 8.2 Merge Algorithm

```rust
pub fn merge(&self) -> Result<SimulationData, Vec<String>> {
    let portfolio = self.portfolio.clone()
        .ok_or_else(|| vec!["Portfolio not set. Call set_portfolio first.".into()])?;
    let parameters = self.parameters.clone()
        .unwrap_or_default();

    Ok(SimulationData {
        portfolios: portfolio,
        profiles: self.profiles.clone(),
        assets: self.assets.clone(),
        historical_assets: self.historical_assets.clone(),
        asset_prices: self.asset_prices.clone(),
        asset_tracking_errors: self.asset_tracking_errors.clone(),
        events: self.events.clone(),
        parameters,
        analysis: self.analysis.clone(),
    })
}
```

### 8.3 YAML Serialization

Use the same `serde_saphyr` serializer the TUI app uses:

```rust
let sim_data = state.merge()?;
let yaml = serde_saphyr::to_string(&sim_data)?;
```

This ensures output is byte-for-byte compatible with what the TUI app expects to read.

---

## 9. Validation Pipeline

### 9.1 Validation Function Access

The validation logic lives in `finplan/src/data/validator.rs`. Since `finplan_mcp` depends on the `finplan` crate, it can call `validate_scenario()` directly:

```rust
use finplan::data::validator::{validate_scenario, ValidationError};

pub fn validate(yaml: &str) -> Result<(), Vec<ValidationError>> {
    let data: SimulationData = serde_saphyr::from_str(yaml)
        .map_err(|e| vec![ValidationError {
            section: "YAML".into(),
            message: format!("Parse error: {}", e),
            help: Some("Check YAML syntax".into()),
        }])?;
    validate_scenario(&data)
}
```

### 9.2 Conversion Check

After validation passes, also try the full conversion to catch deeper issues:

```rust
use finplan::data::convert::{to_simulation_config, ConvertError};

pub fn deep_validate(yaml: &str) -> Result<(), Vec<String>> {
    let data: SimulationData = serde_saphyr::from_str(yaml)?;
    validate_scenario(&data)?;
    to_simulation_config(&data)?;
    Ok(())
}
```

### 9.3 Error Format Returned to Agent

```json
{
  "valid": false,
  "errors": [
    {
      "section": "events[3].effects[0].to",
      "message": "References unknown account 'Savings2'",
      "help": "Account 'Savings2' is not defined in portfolios.accounts"
    }
  ],
  "warnings": [
    {
      "section": "parameters",
      "message": "No inflation configured — amounts will not be inflation-adjusted"
    }
  ]
}
```

---

## 10. Testing Strategy

### 10.1 Unit Tests (per tool)

Each tool module should have tests that:
1. Call the tool with valid inputs → verify output is valid YAML
2. Call the tool with minimal inputs (use defaults) → verify defaults applied correctly
3. Call the tool with invalid inputs → verify appropriate error message

**Example test for `add_income_event`:**

```rust
#[test]
fn test_add_income_defaults() {
    let event = build_income_event(IncomeParams {
        name: "Salary".into(),
        to_account: "Checking".into(),
        amount: 4500.0,
        ..Default::default()
    });

    assert_eq!(event.name.0, "Salary");
    // Default interval should be biweekly
    match &event.trigger {
        TriggerData::Repeating { interval, .. } => {
            assert_eq!(*interval, IntervalData::BiWeekly);
        }
        _ => panic!("Expected Repeating trigger"),
    }
    // Default should be inflation-adjusted
    match &event.effects[0] {
        EffectData::Income { amount, gross, taxable, .. } => {
            assert!(amount.is_inflation_adjusted());
            assert!(*gross);
            assert!(*taxable);
        }
        _ => panic!("Expected Income effect"),
    }
}
```

### 10.2 Integration Tests

**Test: Full scenario build from scratch**

```rust
#[tokio::test]
async fn test_full_scenario_build() {
    let server = FinplanMcpServer::new();

    // 1. Set portfolio
    server.set_portfolio(/* checking + brokerage + 401k */).await;

    // 2. Set parameters
    server.set_parameters(ParameterParams {
        birth_date: "1985-06-15".into(),
        ..Default::default()
    }).await;

    // 3. Add events
    server.add_income_event(/* salary */).await;
    server.add_expense_event(/* living expenses */).await;
    server.add_contribution(/* 401k */).await;
    server.add_retirement(/* age 65 */).await;
    server.add_social_security(/* age 67, $2000/mo */).await;
    server.add_rmd(/* defaults */).await;

    // 4. Map tickers
    server.map_tickers(/* auto */).await;

    // 5. Merge and validate
    let result = server.merge_scenario(true).await;
    assert!(result.valid);

    // 6. Verify the YAML parses correctly
    let sim: SimulationData = serde_saphyr::from_str(&result.yaml).unwrap();
    assert_eq!(sim.events.len(), 6); // salary, expenses, 401k, retirement, SS, RMD
    assert!(!sim.historical_assets.is_empty());
}
```

**Test: Round-trip with example.yaml**

```rust
#[test]
fn test_example_yaml_validates() {
    let yaml = include_str!("../../../examples/example.yaml");
    let result = validate(yaml);
    assert!(result.is_ok(), "example.yaml should validate: {:?}", result);
}
```

**Test: Merge produces loadable YAML**

```rust
#[test]
fn test_merged_yaml_loads_in_tui() {
    let yaml = build_test_scenario();
    let sim: SimulationData = serde_saphyr::from_str(&yaml).unwrap();
    let config = to_simulation_config(&sim).unwrap();
    // Verify it has the right number of accounts, events, etc.
    assert!(config.accounts().len() > 0);
}
```

### 10.3 Resource Tests

```rust
#[test]
fn test_schema_resources_not_empty() {
    let resources = list_resources();
    assert!(resources.len() >= 10);
    for resource in &resources {
        let content = read_resource(&resource.uri);
        assert!(!content.is_empty(), "Resource {} is empty", resource.uri);
    }
}
```

### 10.4 MCP Protocol Tests

```rust
#[tokio::test]
async fn test_mcp_server_starts() {
    // Verify the server starts and responds to initialize
    let server = FinplanMcpServer::new();
    let info = server.get_info();
    assert!(info.capabilities.tools.is_some());
    assert!(info.capabilities.resources.is_some());
    assert!(info.capabilities.prompts.is_some());
}

#[tokio::test]
async fn test_tool_list() {
    let server = FinplanMcpServer::new();
    let tools = server.list_tools();
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(tool_names.contains(&"set_parameters"));
    assert!(tool_names.contains(&"add_income_event"));
    assert!(tool_names.contains(&"merge_scenario"));
    assert!(tool_names.contains(&"validate_scenario"));
}
```

---

## 11. File-by-File Implementation Checklist

### Phase 1: Scaffolding

- [ ] Create `crates/finplan_mcp/Cargo.toml`
- [ ] Add `"crates/finplan_mcp"` to workspace `Cargo.toml` members
- [ ] Create `src/main.rs` — tokio main, stdio transport, server startup
- [ ] Create `src/server.rs` — `FinplanMcpServer` struct with `ServerHandler` impl
- [ ] Create `src/state.rs` — `ScenarioState` struct with `Arc<Mutex<>>` wrapper
- [ ] Verify `cargo build -p finplan_mcp` compiles

### Phase 2: Resources

- [ ] Create `src/resources.rs` — implement resource listing and reading
- [ ] Create `src/schema_text.rs` — static strings for each schema resource
- [ ] Wire resources into `ServerHandler::list_resources` and `read_resource`
- [ ] Write `tests/resource_tests.rs`

### Phase 3: Core Tools

- [ ] Create `src/tools/mod.rs` — aggregate tool router
- [ ] Create `src/tools/parameters.rs` — `set_parameters` tool
- [ ] Create `src/tools/portfolio.rs` — `set_portfolio` tool
- [ ] Create `src/tools/ticker.rs` — `map_tickers` + `get_ticker_info` tools
- [ ] Create `src/tools/merge.rs` — `merge_scenario` tool
- [ ] Create `src/tools/validate.rs` — `validate_scenario` tool
- [ ] Write `tests/tool_tests.rs` for core tools

### Phase 4: Event Tools

- [ ] Create `src/tools/events.rs` — `add_income_event`, `add_expense_event` tools
- [ ] Create `src/tools/retirement.rs` — `add_retirement`, `add_social_security`, `add_rmd` tools
- [ ] Create `src/tools/home_purchase.rs` — `add_home_purchase` tool
- [ ] Create `src/tools/custom_event.rs` — `add_custom_event` tool
- [ ] Create `src/defaults.rs` — default values and state tax lookup
- [ ] Create `src/interview.rs` — interview question metadata
- [ ] Add `src/tools/contribution.rs` — `add_contribution` tool (with employer match)
- [ ] Write event tool tests

### Phase 5: Prompts

- [ ] Create `src/prompts.rs` — `build_scenario` and `quick_retirement` prompts
- [ ] Wire prompts into `ServerHandler::list_prompts` and `get_prompt`

### Phase 6: Integration Testing

- [ ] Write `tests/integration_test.rs` — full end-to-end scenario build
- [ ] Write `tests/validation_tests.rs` — validation edge cases
- [ ] Test with `example.yaml` round-trip
- [ ] Test with Claude Desktop or MCP Inspector

### Phase 7: Polish

- [ ] Add `--help` to the binary with `clap`
- [ ] Add logging with `tracing`
- [ ] Update `finplan/README.md` with MCP server section
- [ ] Add MCP server config example for Claude Desktop (`claude_desktop_config.json`)
- [ ] Add Docker support for the MCP server binary

---

## 12. Example End-to-End Session

This shows the sequence of MCP tool calls an AI agent would make to build a complete scenario from statement data + user interview.

### Step 1: Agent receives portfolio from statements MCP

```json
// Agent calls: set_portfolio
{
  "name": "Sarah's Retirement Plan",
  "accounts": [
    { "name": "Chase Checking", "type": "Checking", "value": 15000 },
    { "name": "Ally Savings", "type": "Savings", "value": 45000 },
    { "name": "Fidelity Brokerage", "type": "Brokerage", "assets": [
      { "asset": "VTI", "value": 150000 },
      { "asset": "VXUS", "value": 50000 },
      { "asset": "BND", "value": 30000 }
    ]},
    { "name": "Company 401k", "type": "Traditional401k", "assets": [
      { "asset": "FXAIX", "value": 200000 }
    ]},
    { "name": "Roth IRA", "type": "RothIRA", "assets": [
      { "asset": "VOO", "value": 75000 }
    ]}
  ]
}
```

Response: "Portfolio set with 5 accounts. Tickers found: VTI, VXUS, BND, FXAIX, VOO"

### Step 2: Agent sets parameters from user interview

```json
// Agent calls: set_parameters
{
  "birth_date": "1988-03-22",
  "state_tax_rate": 0.0,
  "federal_brackets": "single2024",
  "returns_mode": "historical"
}
```

Response: Parameters set. Duration auto-calculated as 62 years (to age 100). Start date: 2026-01-01.

### Step 3: Agent adds income

```json
// Agent calls: add_income_event
{
  "name": "Software Engineer Salary",
  "to_account": "Chase Checking",
  "amount": 5769.23,
  "interval": "biweekly",
  "end_event": "Retirement"
}
```

### Step 4: Agent adds expenses

```json
// Agent calls: add_expense_event
{
  "name": "Living Expenses",
  "from_account": "Chase Checking",
  "amount": 6000.0
}
```

### Step 5: Agent adds 401k contribution

```json
// Agent calls: add_contribution
{
  "name": "401k Contribution",
  "from_account": "Chase Checking",
  "to_account": "Company 401k",
  "asset": "FXAIX",
  "amount": 23500.0,
  "end_event": "Retirement"
}
```

### Step 6: Agent adds retirement events

```json
// Agent calls: add_retirement
{ "retirement_age": 60, "annual_spending": 80000, "spending_account": "Chase Checking" }

// Agent calls: add_social_security
{ "claim_age": 67, "monthly_benefit": 2800, "to_account": "Chase Checking" }

// Agent calls: add_rmd
{}  // all defaults
```

### Step 7: Agent maps tickers

```json
// Agent calls: map_tickers
{ "returns_mode": "historical" }
```

Response:
```json
{
  "historical_assets": {
    "VTI": "S&P 500",
    "VXUS": "Intl Developed",
    "BND": "US Agg Bonds",
    "FXAIX": "S&P 500",
    "VOO": "S&P 500"
  },
  "unmapped": []
}
```

### Step 8: Agent merges and validates

```json
// Agent calls: merge_scenario
{ "validate": true }
```

Response: Complete YAML + `{ "valid": true }`.

### Final Output

```yaml
portfolios:
  name: Sarah's Retirement Plan
  accounts:
    - name: Chase Checking
      type: Checking
      value: 15000.0
    - name: Ally Savings
      type: Savings
      value: 45000.0
    - name: Fidelity Brokerage
      type: Brokerage
      assets:
        - asset: VTI
          value: 150000.0
        - asset: VXUS
          value: 50000.0
        - asset: BND
          value: 30000.0
    - name: Company 401k
      type: Traditional401k
      assets:
        - asset: FXAIX
          value: 200000.0
    - name: Roth IRA
      type: RothIRA
      assets:
        - asset: VOO
          value: 75000.0

historical_assets:
  VTI: S&P 500
  VXUS: Intl Developed
  BND: US Agg Bonds
  FXAIX: S&P 500
  VOO: S&P 500

events:
  - name: Software Engineer Salary
    trigger:
      type: Repeating
      interval: biweekly
      end:
        type: RelativeToEvent
        event: Retirement
        offset:
          unit: Months
          value: 0
    effects:
      - type: Income
        to: Chase Checking
        amount:
          type: InflationAdjusted
          inner:
            type: Fixed
            value: 5769.23
        gross: true
        taxable: true
    once: false
    enabled: true

  - name: Living Expenses
    trigger:
      type: Repeating
      interval: monthly
    effects:
      - type: Expense
        from: Chase Checking
        amount:
          type: InflationAdjusted
          inner:
            type: Fixed
            value: 6000.0
    once: false
    enabled: true

  - name: 401k Contribution
    trigger:
      type: Repeating
      interval: yearly
      end:
        type: RelativeToEvent
        event: Retirement
        offset:
          unit: Months
          value: 0
    effects:
      - type: AssetPurchase
        from: Chase Checking
        to_account: Company 401k
        asset: FXAIX
        amount:
          type: Fixed
          value: 23500.0
    once: false
    enabled: true

  - name: Retirement
    trigger:
      type: Age
      years: 60
    once: true
    enabled: true

  - name: Yearly Spend Post-Retirement
    trigger:
      type: Repeating
      interval: yearly
      start:
        type: RelativeToEvent
        event: Retirement
        offset:
          unit: Months
          value: 0
    effects:
      - type: Sweep
        to: Chase Checking
        amount:
          type: TargetToBalance
          target: 80000.0
        strategy: penalty_aware
        gross: false
        taxable: true
        lot_method: fifo
    once: false
    enabled: true

  - name: Social Security
    trigger:
      type: Repeating
      interval: monthly
      start:
        type: Age
        years: 67
    effects:
      - type: Income
        to: Chase Checking
        amount:
          type: Fixed
          value: 2800.0
        gross: true
        taxable: true
    once: false
    enabled: true

  - name: RMD
    trigger:
      type: Repeating
      interval: yearly
      start:
        type: Age
        years: 73
    effects:
      - type: ApplyRmd
        destination: Chase Checking
        lot_method: fifo
    once: false
    enabled: true

parameters:
  birth_date: "1988-03-22"
  start_date: "2026-01-01"
  duration_years: 62
  inflation:
    type: USHistorical
    distribution: lognormal
  tax_config:
    state_rate: 0.0
    capital_gains_rate: 0.15
    federal_brackets: single2024
  returns_mode: historical
  historical_block_size: 5

analysis:
  mc_iterations: 500
  default_steps: 6
```

---

## 13. Future Work

1. **SSE/HTTP Transport** — Allow the statements MCP server to call finplan_mcp directly over HTTP instead of requiring an orchestrating agent.
2. **Scenario Comparison** — Tool to diff two scenarios and explain the differences.
3. **Simulation Execution** — Run Monte Carlo simulation directly from the MCP server and return summary statistics (success rate, median net worth, etc.).
4. **Interactive Refinement** — After seeing simulation results, tools to tweak parameters ("What if I retire at 62 instead of 65?").
5. **Batch Account Import** — Support CSV/JSON batch import formats from common brokerages (Fidelity, Vanguard, Schwab export formats).
6. **Template Library** — Pre-built scenario templates: "FIRE at 40", "Traditional retirement at 65", "Single income with kids", "Dual income no kids".

---

## Appendix A: MCP Server Configuration for Claude Desktop

```json
{
  "mcpServers": {
    "finplan": {
      "command": "/path/to/finplan-mcp",
      "args": [],
      "env": {}
    }
  }
}
```

Or if running from the project directory:

```json
{
  "mcpServers": {
    "finplan": {
      "command": "cargo",
      "args": ["run", "-p", "finplan_mcp"],
      "cwd": "/path/to/finplan",
      "env": {}
    }
  }
}
```

## Appendix B: Data Type Quick Reference

### Account Types → Inner Structs

| Type | Inner | Fields |
|------|-------|--------|
| Checking | Property | `value: f64`, `return_profile: Option<String>` |
| Savings | Property | `value: f64`, `return_profile: Option<String>` |
| HSA | Property | `value: f64`, `return_profile: Option<String>` |
| Property | Property | `value: f64`, `return_profile: Option<String>` |
| Collectible | Property | `value: f64`, `return_profile: Option<String>` |
| Brokerage | AssetAccount | `assets: Vec<{asset: String, value: f64}>` |
| Traditional401k | AssetAccount | `assets: Vec<{asset: String, value: f64}>` |
| Roth401k | AssetAccount | `assets: Vec<{asset: String, value: f64}>` |
| TraditionalIRA | AssetAccount | `assets: Vec<{asset: String, value: f64}>` |
| RothIRA | AssetAccount | `assets: Vec<{asset: String, value: f64}>` |
| Mortgage | Debt | `balance: f64`, `interest_rate: f64` |
| LoanDebt | Debt | `balance: f64`, `interest_rate: f64` |
| StudentLoanDebt | Debt | `balance: f64`, `interest_rate: f64` |

### Interval Options

`never`, `weekly`, `biweekly`, `monthly`, `quarterly`, `yearly`

### Withdrawal Strategies

`penalty_aware` (default), `tax_efficient`, `tax_deferred_first`, `tax_free_first`, `pro_rata`

### Lot Methods

`fifo` (default), `lifo`, `highest_cost`, `lowest_cost`, `average_cost`

### Federal Bracket Presets

`single2024`, `married_joint2024`, `custom { brackets: [{threshold, rate}] }`

### Inflation Types

`None`, `Fixed { rate }`, `Normal { mean, std_dev }`, `LogNormal { mean, std_dev }`, `USHistorical { distribution: fixed|normal|lognormal }`

### Return Profile Types

`None`, `Fixed { rate }`, `Normal { mean, std_dev }`, `LogNormal { mean, std_dev }`, `StudentT { mean, scale, df }`, `RegimeSwitching { bull_mean, bull_std_dev, bear_mean, bear_std_dev, bull_to_bear_prob, bear_to_bull_prob }`, `Bootstrap { preset }`

### Historical Preset Keys

`sp500`, `us_small_cap`, `us_tbills`, `us_long_bonds`, `intl_developed`, `emerging_markets`, `reits`, `gold`, `us_agg_bonds`, `us_corporate_bonds`, `tips`
