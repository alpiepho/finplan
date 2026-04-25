/// Static text for MCP schema resources.
/// Each function returns a Markdown string documenting a section of the YAML format.

pub fn full_schema() -> &'static str {
    include_str!("../../../docs/08-scenario-yaml-reference.md")
}

pub fn example_yaml() -> &'static str {
    include_str!("../../../examples/example.yaml")
}

pub fn accounts_schema() -> &'static str {
    r#"# Account Types

FinPlan supports 13 account types in 3 categories.

## Investment Accounts (hold assets)

These accounts hold named assets (tickers) with dollar values.

| Type | YAML `type` | Fields |
|------|------------|--------|
| Brokerage | `Brokerage` | `assets: [{asset, value}]` |
| Traditional 401k | `Traditional401k` | `assets: [{asset, value}]` |
| Roth 401k | `Roth401k` | `assets: [{asset, value}]` |
| Traditional IRA | `TraditionalIRA` | `assets: [{asset, value}]` |
| Roth IRA | `RothIRA` | `assets: [{asset, value}]` |

Example:
```yaml
- name: Brokerage
  type: Brokerage
  assets:
    - asset: VTI
      value: 150000.0
    - asset: BND
      value: 50000.0
```

## Cash / Property Accounts

These accounts hold a single dollar value with an optional return profile.

| Type | YAML `type` | Fields |
|------|------------|--------|
| Checking | `Checking` | `value`, `return_profile` (optional) |
| Savings | `Savings` | `value`, `return_profile` (optional) |
| HSA | `HSA` | `value`, `return_profile` (optional) |
| Property | `Property` | `value`, `return_profile` (optional) |
| Collectible | `Collectible` | `value`, `return_profile` (optional) |

Example:
```yaml
- name: Savings
  type: Savings
  value: 45000.0
  return_profile: US T-Bills
```

## Debt Accounts

These accounts represent liabilities with a balance and interest rate.

| Type | YAML `type` | Fields |
|------|------------|--------|
| Mortgage | `Mortgage` | `balance`, `interest_rate` |
| Loan | `LoanDebt` | `balance`, `interest_rate` |
| Student Loan | `StudentLoanDebt` | `balance`, `interest_rate` |

Example:
```yaml
- name: Mortgage
  type: Mortgage
  balance: 350000.0
  interest_rate: 0.054
```
"#
}

pub fn events_schema() -> &'static str {
    r#"# Event Structure

Every event has this structure:

```yaml
- name: "Event Name"          # REQUIRED — unique identifier
  description: "..."          # optional
  trigger: { ... }            # REQUIRED — when this event fires
  effects:                    # REQUIRED — what happens when it fires
    - { type: Income, ... }
  once: false                 # default false — if true, fires only once
  enabled: true               # default true — if false, event is skipped
```

## Key Concepts

- **Trigger**: Determines *when* the event fires (date, age, balance threshold, repeating schedule, etc.)
- **Effects**: One or more actions that happen when the trigger fires (income, expense, transfer, etc.)
- **once**: If `true`, the event fires exactly once and is then disabled
- **enabled**: Set to `false` to skip the event without deleting it

Events reference accounts by name (must match an account in `portfolios.accounts`).
Events can reference other events by name for triggers like `RelativeToEvent` or effects like `TriggerEvent`.
"#
}

pub fn triggers_schema() -> &'static str {
    r#"# Trigger Types

## Date
Fire on a specific date.
```yaml
trigger:
  type: Date
  date: "2028-06-17"
```

## Age
Fire when the person reaches a specific age.
```yaml
trigger:
  type: Age
  years: 65
  months: 6    # optional
```

## RelativeToEvent
Fire relative to when another event fires.
```yaml
trigger:
  type: RelativeToEvent
  event: Retirement
  offset:
    unit: Months    # Days, Months, or Years
    value: 0
```

## AccountBalance
Fire when an account balance crosses a threshold.
```yaml
trigger:
  type: AccountBalance
  account: Checking
  threshold:
    comparison: LessThanOrEqual
    value: 5000.0
```

## AssetBalance
Fire when a specific asset's balance crosses a threshold.
```yaml
trigger:
  type: AssetBalance
  account: Brokerage
  asset: VTI
  threshold:
    comparison: GreaterThanOrEqual
    value: 100000.0
```

## NetWorth
Fire when total net worth crosses a threshold.
```yaml
trigger:
  type: NetWorth
  threshold:
    comparison: GreaterThanOrEqual
    value: 1000000.0
```

## And / Or
Combine multiple triggers with logical AND or OR.
```yaml
trigger:
  type: And
  conditions:
    - type: Age
      years: 65
    - type: AccountBalance
      account: 401k
      threshold:
        comparison: GreaterThanOrEqual
        value: 500000.0
```

## Repeating
Fire on a recurring schedule with optional start/end.
```yaml
trigger:
  type: Repeating
  interval: monthly          # weekly, biweekly, monthly, quarterly, yearly
  start:                     # optional — when to start repeating
    type: Age
    years: 30
  end:                       # optional — when to stop repeating
    type: RelativeToEvent
    event: Retirement
    offset:
      unit: Months
      value: 0
  max_occurrences: 360       # optional — max number of times to fire
```

## Manual
Only fires when triggered by a `TriggerEvent` effect from another event.
```yaml
trigger:
  type: Manual
```
"#
}

pub fn effects_schema() -> &'static str {
    r#"# Effect Types

## Income
Receive income into an account.
```yaml
- type: Income
  to: Checking
  amount: { type: Fixed, value: 5000.0 }
  gross: true       # default false — if true, federal+state taxes are withheld
  taxable: true     # default true — if true, amount is added to taxable income
```

## Expense
Pay an expense from an account.
```yaml
- type: Expense
  from: Checking
  amount: { type: InflationAdjusted, inner: { type: Fixed, value: 5500.0 } }
```

## AssetPurchase
Buy assets (shares) with cash.
```yaml
- type: AssetPurchase
  from: Checking
  to_account: 401k
  asset: FXAIX
  amount: { type: Fixed, value: 23500.0 }
```

## AssetSale
Sell assets from an account.
```yaml
- type: AssetSale
  from: Brokerage
  asset: VTI          # optional — if omitted, sells pro-rata
  amount: { type: Fixed, value: 10000.0 }
  gross: false
  lot_method: fifo    # fifo, lifo, highest_cost, lowest_cost, average_cost
```

## Sweep
Liquidate from multiple accounts to fund a target account.
```yaml
- type: Sweep
  to: Checking
  amount: { type: TargetToBalance, target: 80000.0 }
  strategy: penalty_aware   # penalty_aware, tax_efficient, tax_deferred_first, tax_free_first, pro_rata
  gross: false
  taxable: true
  lot_method: fifo
  exclude_accounts: []      # optional — accounts to skip
```

## TriggerEvent / PauseEvent / ResumeEvent / TerminateEvent
Control other events.
```yaml
- type: TriggerEvent
  event: "Buy House"
- type: PauseEvent
  event: "401k Contribution"
- type: ResumeEvent
  event: "401k Contribution"
- type: TerminateEvent
  event: "Old Salary"
```

## ApplyRmd
Apply Required Minimum Distributions from tax-deferred accounts.
```yaml
- type: ApplyRmd
  destination: Checking
  lot_method: fifo
```

## AdjustBalance
Directly adjust an account's balance.
```yaml
- type: AdjustBalance
  account: Mortgage
  amount: { type: Fixed, value: -350000.0 }
```

## CashTransfer
Transfer cash between accounts.
```yaml
- type: CashTransfer
  from: Checking
  to: Mortgage
  amount: { type: Fixed, value: 3000.0 }
```

## Random
Randomly trigger events based on probability.
```yaml
- type: Random
  probability: 0.05
  on_true: "Job Loss"
  on_false: null      # optional
```

## RsuVesting
RSU share vesting with optional sell-to-cover.
```yaml
- type: RsuVesting
  to: Brokerage
  asset: COMPANY
  units: 100.0
  sell_to_cover: true
  lot_method: fifo
```
"#
}

pub fn amounts_schema() -> &'static str {
    r#"# Amount Types

Amounts can be nested/composed. A bare number is treated as `Fixed`.

## Fixed
A fixed dollar amount.
```yaml
amount: 4500.0
# or explicitly:
amount: { type: Fixed, value: 4500.0 }
```

## InflationAdjusted
Wraps another amount to maintain purchasing power over time.
```yaml
amount:
  type: InflationAdjusted
  inner: { type: Fixed, value: 5500.0 }
```

## Scale
Multiply another amount by a factor (useful for percentages).
```yaml
amount:
  type: Scale
  multiplier: 0.04
  inner: { type: AccountBalance, account: "401k" }
```

## SourceBalance
Transfer the entire source account balance.
```yaml
amount: { type: SourceBalance }
```

## ZeroTargetBalance
Transfer enough to zero out the target balance (debt payoff).
```yaml
amount: { type: ZeroTargetBalance }
```

## TargetToBalance
Transfer enough to bring the target account to a specified balance.
```yaml
amount: { type: TargetToBalance, target: 80000.0 }
```

## AccountBalance
Reference an account's total balance as the amount.
```yaml
amount: { type: AccountBalance, account: "Checking" }
```

## AccountCashBalance
Reference an account's cash-only balance.
```yaml
amount: { type: AccountCashBalance, account: "Savings" }
```
"#
}

pub fn parameters_schema() -> &'static str {
    r#"# Parameters

The `parameters` section configures simulation timing, inflation, taxes, and returns mode.

```yaml
parameters:
  birth_date: "1985-06-15"       # REQUIRED — YYYY-MM-DD
  start_date: "2026-01-01"       # Simulation start date
  duration_years: 40             # How many years to simulate

  inflation:
    type: USHistorical           # None, Fixed, Normal, LogNormal, USHistorical
    distribution: lognormal      # For USHistorical: fixed, normal, lognormal

  tax_config:
    state_rate: 0.05             # State income tax rate (decimal)
    capital_gains_rate: 0.15     # Long-term capital gains rate
    federal_brackets: single2024 # single2024, married_joint2024, or custom

  returns_mode: historical       # parametric or historical
  historical_block_size: 5       # Block bootstrap size (historical mode)
  seed: null                     # Optional seed for reproducibility
```

## Inflation Types
- `None` — no inflation
- `Fixed { rate: 0.03 }` — constant annual inflation
- `Normal { mean: 0.03, std_dev: 0.01 }` — normally distributed
- `LogNormal { mean: 0.03, std_dev: 0.01 }` — log-normally distributed
- `USHistorical { distribution: lognormal }` — sample from historical US CPI data

## Federal Bracket Presets
- `single2024` — 2024 single filer brackets
- `married_joint2024` — 2024 married filing jointly brackets

## Returns Mode
- `parametric` — use profiles defined in `profiles:` section with `assets:` mappings
- `historical` — use bootstrap sampling from historical data via `historical_assets:` mappings
"#
}

pub fn profiles_schema() -> &'static str {
    r#"# Return Profiles

Return profiles define how asset returns are modeled in **parametric** mode.

```yaml
profiles:
  - name: "S&P 500"
    description: "US large cap equity"
    type: Normal
    mean: 0.1147
    std_dev: 0.1815

  - name: HYSA
    type: Fixed
    rate: 0.045
```

## Profile Types

| Type | Fields | Description |
|------|--------|-------------|
| `None` | — | No returns (0%) |
| `Fixed` | `rate` | Constant annual return |
| `Normal` | `mean`, `std_dev` | Normally distributed returns |
| `LogNormal` | `mean`, `std_dev` | Log-normally distributed returns |
| `StudentT` | `mean`, `scale`, `df` | Fat-tailed distribution |
| `RegimeSwitching` | `bull_mean`, `bull_std_dev`, `bear_mean`, `bear_std_dev`, `bull_to_bear_prob`, `bear_to_bull_prob` | Two-state market model |
| `Bootstrap` | `preset` | Sample from historical data |

## Bootstrap Presets
Available preset keys: `sp500`, `us_small_cap`, `us_tbills`, `us_long_bonds`, `intl_developed`, `emerging_markets`, `reits`, `gold`, `us_agg_bonds`, `us_corporate_bonds`, `tips`

## Asset Mappings

In **parametric** mode, map tickers to profile names:
```yaml
assets:
  VFIAX: "S&P 500"
  BND: "US Agg Bond"
```

In **historical** mode, map tickers to preset display names:
```yaml
historical_assets:
  VFIAX: "S&P 500"
  BND: "US Agg Bonds"
```
"#
}

pub fn analysis_schema() -> &'static str {
    r#"# Analysis Configuration

The `analysis` section controls Monte Carlo simulation and sweep parameters.

```yaml
analysis:
  mc_iterations: 500        # Number of Monte Carlo iterations per sweep point
  default_steps: 6           # Default number of steps for sweep parameters

  sweep_parameters:          # Parameters to sweep over
    - event_name: Retirement
      sweep_type: trigger_age
      min_value: 55.0
      max_value: 70.0
      step_count: 6

  selected_metrics:          # Metrics to compute
    - success_rate
    - p50_final_net_worth

  chart_configs:             # Result visualizations
    - chart_type: scatter_1d
      x_param_index: 0
      metric: success_rate
```

## Sweep Types
- `trigger_age` — sweep the age trigger of an event
- `trigger_date` — sweep the date trigger year
- `effect_value` — sweep the dollar amount of an effect
- `repeating_start_age` — sweep the start age of a repeating event
- `repeating_end_age` — sweep the end age of a repeating event

## Metrics
- `success_rate` — percentage of simulations that don't run out of money
- `p5_final_net_worth` through `p95_final_net_worth` — percentile final net worth
- `lifetime_taxes` — total taxes paid over simulation
- `max_drawdown` — maximum peak-to-trough net worth decline
- `net_worth_at_age { age: 65 }` — net worth at a specific age
"#
}

pub fn patterns_schema() -> &'static str {
    r#"# Common Scenario Patterns

## Basic Salary + Expenses
```yaml
events:
  - name: Salary
    trigger:
      type: Repeating
      interval: biweekly
      end:
        type: RelativeToEvent
        event: Retirement
        offset: { unit: Months, value: 0 }
    effects:
      - type: Income
        to: Checking
        amount:
          type: InflationAdjusted
          inner: { type: Fixed, value: 4500.0 }
        gross: true
        taxable: true

  - name: Living Expenses
    trigger:
      type: Repeating
      interval: monthly
    effects:
      - type: Expense
        from: Checking
        amount:
          type: InflationAdjusted
          inner: { type: Fixed, value: 5500.0 }
```

## 401k Contribution
```yaml
  - name: 401k Contribution
    trigger:
      type: Repeating
      interval: yearly
      end:
        type: RelativeToEvent
        event: Retirement
        offset: { unit: Months, value: 0 }
    effects:
      - type: AssetPurchase
        from: Checking
        to_account: 401k
        asset: FXAIX
        amount: { type: Fixed, value: 23500.0 }
```

## Retirement + Post-Retirement Spending
```yaml
  - name: Retirement
    trigger:
      type: Age
      years: 65
    once: true

  - name: Post-Retirement Spending
    trigger:
      type: Repeating
      interval: yearly
      start:
        type: RelativeToEvent
        event: Retirement
        offset: { unit: Months, value: 0 }
    effects:
      - type: Sweep
        to: Checking
        amount: { type: TargetToBalance, target: 80000.0 }
        strategy: penalty_aware
        gross: false
        taxable: true
        lot_method: fifo
```

## Social Security
```yaml
  - name: Social Security
    trigger:
      type: Repeating
      interval: monthly
      start:
        type: Age
        years: 67
    effects:
      - type: Income
        to: Checking
        amount: { type: Fixed, value: 2800.0 }
        gross: true
        taxable: true
```

## Required Minimum Distributions (RMD)
```yaml
  - name: RMD
    trigger:
      type: Repeating
      interval: yearly
      start:
        type: Age
        years: 73
    effects:
      - type: ApplyRmd
        destination: Checking
        lot_method: fifo
```

## Home Purchase with Mortgage
```yaml
  - name: Buy House
    trigger:
      type: Date
      date: "2028-06-17"
    effects:
      - type: AdjustBalance
        account: Mortgage
        amount: { type: Fixed, value: -480000.0 }
      - type: AdjustBalance
        account: House Value
        amount: { type: Fixed, value: 600000.0 }
      - type: Expense
        from: Checking
        amount: { type: Fixed, value: 120000.0 }
    once: true

  - name: Mortgage Payment
    trigger:
      type: Repeating
      interval: monthly
      start:
        type: RelativeToEvent
        event: Buy House
        offset: { unit: Months, value: 1 }
      end:
        type: AccountBalance
        account: Mortgage
        threshold:
          comparison: GreaterThanOrEqual
          value: 0.0
    effects:
      - type: CashTransfer
        from: Checking
        to: Mortgage
        amount: { type: Fixed, value: 3000.0 }
```
"#
}
