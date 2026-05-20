# Scenario YAML Reference

Complete reference for all parameters in FinPlan scenario files. Use this to understand the structure and create custom scenarios or modify existing ones.

## Overview

A scenario file defines:
- **Portfolio**: Your accounts and assets
- **Return Profiles**: How different investments perform (historical or simulated)
- **Assets**: Which stocks/funds you own and their return profiles
- **Events**: Income, expenses, transfers, and other life changes
- **Parameters**: Simulation settings (dates, taxes, inflation)
- **Analysis**: Sensitivity analysis configuration

## File Structure

```yaml
portfolios:
  name: string
  description: string (optional)
  accounts: [...]
  
profiles: [...]
assets: {...}
historical_assets: {...}
asset_prices: {...}
asset_tracking_errors: {...}

events: [...]

parameters:
  birth_date: YYYY-MM-DD
  start_date: YYYY-MM-DD
  duration_years: number
  inflation: {...}
  tax_config: {...}
  returns_mode: string
  historical_block_size: number

analysis:
  mc_iterations: number
  default_steps: number
  sweep_parameters: [...]
  selected_metrics: [...]
  chart_configs: [...]
```

## Portfolios Section

### Portfolio (Top-Level)

```yaml
portfolios:
  name: My Retirement Plan
  description: Sample retirement planning scenario
  accounts: [...]
```

**Fields:**
- `name` (string): Display name for your scenario
- `description` (string): Optional description
- `accounts` (array): List of accounts (checking, investment, loans, etc.)

### Accounts

#### Account Structure

```yaml
accounts:
  - name: Checking
    type: Checking
    description: Main checking account (optional)
    value: 20000.0
    return_profile: US T-Bills  # optional for non-investment accounts
    assets: [...]  # for investment accounts only
```

#### Account Types

| Type | Description | Can Hold Assets? | Notes |
|------|-------------|------------------|-------|
| `Checking` | Checking account | No | Cash, no return profile needed |
| `Savings` | Savings/HYSA | No | Cash, higher yields (set via return profile if desired) |
| `Taxable` / `Brokerage` | Regular brokerage | Yes | Capital gains taxed yearly |
| `Traditional401k` | Employer 401(k) | Yes | Tax-deferred, required minimum distributions (RMD) at 73 |
| `Roth401k` | Roth 401(k) | Yes | Tax-free growth, no RMD |
| `TraditionalIRA` | Traditional IRA | Yes | Tax-deferred |
| `RothIRA` | Roth IRA | Yes | Tax-free growth |
| `HSA` | Health Savings Account | Yes | Triple tax-free (contributions, growth, withdrawals) |
| `Mortgage` | Mortgage loan | No | Liability account, use `balance` and `interest_rate` |
| `StudentLoanDebt` | Student loan | No | Liability account |
| `LoanDebt` | Personal/other loans | No | Liability account |
| `Property` | Real estate | No | Illiquid asset, doesn't grow (unless mapped to return profile) |
| `Collectible` | Art, jewelry, etc. | No | Illiquid asset |

**Account Fields:**

- `name` (string): Account identifier
- `type` (string): Account type (see table above)
- `description` (string, optional): Notes about the account
- `value` (number): Current balance (for cash accounts)
- `balance` (number): Current balance (for loan accounts)
- `interest_rate` (number, optional): For mortgage/loan accounts (as decimal, e.g., 0.054 = 5.4%)
- `return_profile` (string, optional): Name of return profile for growth
- `assets` (array, optional): Holdings in investment accounts

#### Assets in Accounts

```yaml
accounts:
  - name: Brokerage
    type: Brokerage
    assets:
      - asset: VFIAX
        value: 200000.0
      - asset: VTIAX
        value: 33000.0
```

**Fields:**
- `asset` (string): Ticker symbol (must exist in `assets` section)
- `value` (number): Current market value

## Profiles Section

Return profiles define how investments perform. Three types: Fixed, Normal, and LogNormal.

### Fixed Rate Profile

```yaml
profiles:
  - name: HYSA
    description: High-yield savings account
    type: Fixed
    rate: 0.045  # 4.5% annual return
```

**Use for:**
- Cash accounts with guaranteed returns
- Fixed-income investments (bonds, CDs, money market)

### Normal Distribution Profile

```yaml
profiles:
  - name: S&P 500
    description: Historical S&P 500 returns
    type: Normal
    mean: 0.095668
    std_dev: 0.165244
```

**Use for:**
- Stock market simulations
- Assumes returns follow normal (bell curve) distribution

**Fields:**
- `mean` (number): Expected annual return
- `std_dev` (number): Annual volatility (standard deviation)

### LogNormal Distribution Profile

```yaml
profiles:
  - name: S&P Log Normal
    type: LogNormal
    mean: 0.055
    std_dev: 0.161
```

**Use for:**
- Stock markets with better mathematical properties (prevents negative returns if std_dev is low)
- More realistic for long-term projections

**Fields:**
- `mean` (number): Expected annual return
- `std_dev` (number): Annual volatility

### Regime Switching Profile

```yaml
profiles:
  - name: S&P 500 Regime Switching (Fat Tails)
    description: Bull/bear regime switching with fat-tailed distributions
    type: RegimeSwitching
    bull_mean: 0.12
    bull_std_dev: 0.092951600308978
    bear_mean: -0.05
    bear_std_dev: 0.17041126723312636
    bull_to_bear_prob: 0.15      # Probability of switching from bull to bear
    bear_to_bull_prob: 0.4       # Probability of switching from bear to bull
```

**Use for:**
- Market conditions that switch between bull and bear markets
- More realistic for long multi-decade plans

**Fields:**
- `bull_mean` / `bear_mean` (number): Expected returns in each regime
- `bull_std_dev` / `bear_std_dev` (number): Volatility in each regime
- `bull_to_bear_prob` (number): Probability (0-1) of switching from bull to bear in a year
- `bear_to_bull_prob` (number): Probability (0-1) of switching from bear to bull in a year

## Assets Section

Maps ticker symbols to return profiles for all future simulations.

```yaml
assets:
  VFIAX: S&P 500 Regime Switching (Fat Tails)
  VTIAX: S&P Log Normal
  GOOGL: S&P Log Normal
```

**Use:** Define expected returns for stocks you own.

## Historical Assets Section

Maps ticker symbols to historical return profiles (when `returns_mode: historical`).

```yaml
historical_assets:
  VFIAX: S&P 500
  FXAIX: S&P 500
  VTIAX: Intl Developed
```

**Use:** Reference historical data for backtesting or conservative projections. See available profiles in historical data.

## Asset Prices Section

Current market prices for assets (optional, for informational purposes).

```yaml
asset_prices:
  PLTR: 100.0
```

## Asset Tracking Errors Section

Optional tracking error/deviation from index (for sophisticated analyses).

```yaml
asset_tracking_errors:
  PLTR: 0.2
```

## Events Section

Define life events: income, expenses, transfers, purchases, portfolio changes.

### Event Structure

```yaml
events:
  - name: Bi-Weekly Salary
    description: Primary income (optional)
    trigger: {...}          # When does this event occur?
    effects: [...]          # What does it do?
    once: false            # One-time event?
    enabled: true
```

**Fields:**
- `name` (string): Event identifier
- `description` (string, optional): What this event does
- `trigger` (object): When the event fires
- `effects` (array): Actions when event fires
- `once` (boolean): If true, fires only once, then never again
- `enabled` (boolean): If false, event is skipped in simulations

### Trigger Types

#### Age Trigger

```yaml
trigger:
  type: Age
  years: 67
```

Fires when you reach a specific age.

#### Date Trigger

```yaml
trigger:
  type: Date
  date: 2028-06-17
```

Fires on a specific calendar date.

#### Repeating Trigger

```yaml
trigger:
  type: Repeating
  interval: biweekly    # Options: weekly, biweekly, monthly, quarterly, yearly
  start:
    type: Age
    years: 25
  end:
    type: RelativeToEvent
    event: Retirement
    offset:
      unit: Months
      value: 0
```

Fires repeatedly at intervals.

**Start/End Options:**
- `type: Age` with `years: 30` - Start/end at a specific age
- `type: Date` with `date: 2028-06-17` - Start/end on a date
- `type: RelativeToEvent` with `event: EventName` and offset - Start/end relative to another event
- `type: AccountBalance` - Start/end when account balance meets a threshold

#### Account Balance Trigger

```yaml
trigger:
  type: AccountBalance
  account: Checking
  threshold:
    comparison: LessThanOrEqual
    value: 20000.0
```

Fires when account balance meets a condition.

**Comparison Options:**
- `LessThan`
- `LessThanOrEqual`
- `Equal`
- `GreaterThanOrEqual`
- `GreaterThan`

### Effect Types

#### Income Effect

```yaml
effects:
  - type: Income
    to: Checking
    amount:
      type: Fixed
      value: 4500.0
    gross: true          # If true, amount is before taxes
    taxable: true        # If true, subject to income tax
```

Add money (salary, bonus, gift, Social Security, etc.).

**Fields:**
- `to` (string): Destination account
- `amount` (object): How much (see Amount Types below)
- `gross` (boolean): true = amount is before taxes, false = amount is net
- `taxable` (boolean): true = subject to income tax, false = tax-free

#### Expense Effect

```yaml
effects:
  - type: Expense
    from: Checking
    amount:
      type: Fixed
      value: 5500.0
```

Remove money (living expenses, medical bills, etc.).

**Fields:**
- `from` (string): Source account
- `amount` (object): How much

#### Asset Purchase Effect

```yaml
effects:
  - type: AssetPurchase
    from: Checking
    to_account: 401k
    asset: FXAIX
    amount:
      type: Fixed
      value: 24500.0
```

Buy assets (401k contribution, stock purchase, etc.).

**Fields:**
- `from` (string): Account to draw cash from
- `to_account` (string): Account to buy assets in
- `asset` (string): Ticker symbol to buy
- `amount` (object): Dollar amount to invest

#### Sweep Effect

```yaml
effects:
  - type: Sweep
    to: Checking
    amount:
      type: TargetToBalance
      target: 130000.0
    strategy: penalty_aware  # Options: penalty_aware, fifo, tax_efficient
    gross: false            # Before/after taxes?
    taxable: true          # Is the withdrawal taxable?
    lot_method: fifo        # Options: fifo, lifo, highest_cost, lowest_cost, average_cost
```

Withdraw money from investment accounts intelligently (for retirement spending).

**Strategies:**
- `penalty_aware` (default): Before age 59.5 uses taxable → tax-free → tax-deferred; after 59.5 uses tax-efficient ordering
- `tax_efficient`: Taxable first, then tax-deferred, then tax-free
- `tax_deferred_first`: Draws from tax-deferred accounts first
- `tax_free_first`: Draws from tax-free accounts first
- `pro_rata`: Withdraws proportionally from all accounts

**Lot Methods:**
- `fifo` (default): Sell oldest-purchased lots first
- `lifo`: Sell newest-purchased lots first
- `highest_cost`: Sell highest-cost lots first (minimizes capital gains)
- `lowest_cost`: Sell lowest-cost lots first (maximizes capital gains)
- `average_cost`: Use average cost basis

#### Cash Transfer Effect

```yaml
effects:
  - type: CashTransfer
    from: Checking
    to: Mortgage
    amount:
      type: Fixed
      value: 3000.0
```

Move cash between accounts.

#### Adjust Balance Effect

```yaml
effects:
  - type: AdjustBalance
    account: House Value
    amount:
      type: Fixed
      value: 600000.0
```

Set (not adjust) an account's balance. Useful for events like "buy house."

#### Apply RMD Effect

```yaml
effects:
  - type: ApplyRmd
    destination: Checking
    lot_method: fifo
```

Required Minimum Distribution from tax-deferred accounts (automatic at age 73).

### Amount Types

#### Fixed Amount

```yaml
amount:
  type: Fixed
  value: 4500.0
```

Fixed dollar amount, never changes.

#### Inflation-Adjusted Amount

```yaml
amount:
  type: InflationAdjusted
  inner:
    type: Fixed
    value: 4500.0
```

Fixed amount adjusted annually by inflation. The `inner` amount is the base.

#### Percentage of Balance

```yaml
amount:
  type: PercentageOfBalance
  account: Brokerage
  percentage: 0.05
```

Withdraw/deposit a percentage of an account's balance.

#### Target to Balance

```yaml
amount:
  type: TargetToBalance
  target: 130000.0
```

Withdraw/deposit enough to bring account to a target balance.

## Parameters Section

Simulation settings: dates, taxes, inflation, return assumptions.

```yaml
parameters:
  birth_date: 1997-03-16
  start_date: 2026-01-01
  duration_years: 62
  inflation: {...}
  tax_config: {...}
  returns_mode: historical
  historical_block_size: 5
```

### Date Parameters

- `birth_date` (YYYY-MM-DD): Your birth date (used for age-based events)
- `start_date` (YYYY-MM-DD): When the simulation starts
- `duration_years` (number): How many years to simulate

### Inflation

```yaml
inflation:
  type: USHistorical
  distribution: lognormal
```

**Types:**
- `USHistorical`: Use actual historical U.S. inflation data
- `Fixed`: Fixed inflation rate (alternative: `type: Fixed, rate: 0.025`)

**Distribution:**
- `lognormal`: More realistic for long-term projections

### Tax Configuration

```yaml
tax_config:
  federal_brackets: single2024    # Filing status & year
  state_rate: 0.05                # Flat state income tax rate
  capital_gains_rate: 0.2         # Long-term capital gains tax
```

**Federal Bracket Options:**
- `single2024`: Single filer 2024 brackets
- `married2024`: Married filing jointly 2024
- `married_separate2024`: Married filing separately 2024
- `head_of_household2024`: Head of household 2024

(Years may vary; 2024 is the latest available)

**Capital Gains Rate:**
- Applied to long-term capital gains (assets held > 1 year)
- Varies by total income:
  - 0% for low income
  - 15% for middle income
  - 20% for high income
- Set to the rate applicable to your situation

### Returns Mode

```yaml
returns_mode: historical    # Options: historical, profile
```

**Options:**
- `historical`: Use historical return data from `historical_assets`
- `profile`: Use simulated returns from `assets` section

### Historical Block Size

```yaml
historical_block_size: 5
```

When using historical returns, block size (in years) for resampling. Larger blocks = more realistic correlation between assets.

## Analysis Section

Configure sensitivity analysis and parameter sweeps.

```yaml
analysis:
  mc_iterations: 1000
  default_steps: 5
  sweep_parameters: [...]
  selected_metrics: [...]
  chart_configs: [...]
```

### Monte Carlo Iterations

```yaml
mc_iterations: 1000
```

Number of simulations to run for Monte Carlo analysis. Higher = more accurate but slower.

### Default Steps

```yaml
default_steps: 5
```

Default number of values to test for each sweep parameter.

### Sweep Parameters

```yaml
sweep_parameters:
  - event_name: Retirement
    sweep_type: trigger_age
    min_value: 35.0
    max_value: 50.0
    step_count: 6
  
  - event_name: Living Expenses
    sweep_type: effect_value
    min_value: 4000.0
    max_value: 10500.0
    step_count: 6
```

Define parameters to test in sensitivity analysis.

**Sweep Types:**
- `trigger_age`: Test different ages for age-based triggers
- `effect_value`: Test different amounts for income/expense effects

**Fields:**
- `event_name` (string): Name of event to vary
- `sweep_type` (string): What to vary (trigger_age or effect_value)
- `min_value` (number): Minimum value to test
- `max_value` (number): Maximum value to test
- `step_count` (number): How many values to test (e.g., 6 = test min, max, and 4 values in between)

### Selected Metrics

```yaml
selected_metrics:
  - success_rate
  - p50_final_net_worth
  - p95_final_net_worth
```

**Available Metrics:**
- `success_rate`: Percentage of simulations that don't run out of money
- `p5_final_net_worth`: 5th percentile final wealth
- `p50_final_net_worth`: Median final wealth
- `p95_final_net_worth`: 95th percentile final wealth
- `mean_final_net_worth`: Average final wealth
- `lifetime_taxes`: Total taxes paid over simulation

### Chart Configurations

```yaml
chart_configs:
  - chart_type: scatter1_d
    x_param_index: 1
    metric: success_rate
    color_scheme: viridis
  
  - chart_type: heatmap2_d
    x_param_index: 0
    y_param_index: 1
    metric: p50_final_net_worth
    color_scheme: cividis
```

**Chart Types:**
- `scatter1_d`: 1D scatter plot (one parameter vs. metric)
- `heatmap2_d`: 2D heatmap (two parameters vs. metric)

**Fields:**
- `x_param_index` / `y_param_index` (number): Index into `sweep_parameters` array (0-based)
- `metric` (string): Which metric to display
- `color_scheme` (string): Color scheme (viridis, cividis, etc.)

## Common Patterns

### Salary to Retirement

```yaml
events:
  - name: Main Salary
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
        to: Checking
        amount:
          type: InflationAdjusted
          inner:
            type: Fixed
            value: 4500.0
        gross: true
        taxable: true
    enabled: true
```

### Spending in Retirement

```yaml
events:
  - name: Retirement Spending
    trigger:
      type: Repeating
      interval: monthly
      start:
        type: RelativeToEvent
        event: Retirement
        offset:
          unit: Months
          value: 0
    effects:
      - type: Sweep
        to: Checking
        amount:
          type: TargetToBalance
          target: 100000.0
        strategy: penalty_aware
        gross: false
        taxable: true
        lot_method: fifo
    enabled: true
```

### Annual 401k Contribution

```yaml
events:
  - name: Annual 401k
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
        from: Checking
        to_account: 401k
        asset: FXAIX
        amount:
          type: Fixed
          value: 24500.0
    enabled: true
```

### Social Security at 67

```yaml
events:
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
        amount:
          type: Fixed
          value: 2000.0
        gross: true
        taxable: true
    enabled: true
```

### Required Minimum Distribution (RMD) at 73

```yaml
events:
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
    enabled: true
```

## Tips

1. **Start simple**: Create a basic scenario with accounts, then add events gradually
2. **Use inflation-adjusted amounts** for repeating expenses to be realistic
3. **Mark income as `taxable: true`** unless you know it's tax-free (Roth conversions, gifts)
4. **Use `penalty_aware` sweep strategy** for retirement withdrawals to avoid early withdrawal penalties
5. **Test multiple scenarios** with different retirement ages or spending levels using analysis sweeps
6. **Keep `historical_block_size` small (3-5)** for computational speed, larger (10+) for better correlations
7. **Validate your dates**: Ensure start date >= birth date, and duration makes sense for your age

