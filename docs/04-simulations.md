# Running Simulations & Analysis

This guide explains the different types of simulations in FinPlan and how to use them effectively.

## Types of Simulations

FinPlan supports three types of simulation runs:

1. **Single Run (Deterministic)** - Fast, uses median returns
2. **Monte Carlo (Probabilistic)** - Comprehensive, shows probability ranges
3. **Convergence Test** - Validation, ensures results are stable

## Single Run (Deterministic)

### What It Does

Runs one simulation using **median/middle-of-the-road market returns**. No randomness involved.

**Returns Used:**
- Takes historical returns for each asset
- Uses the 50th percentile (middle value)
- Same result every time you run it

### When to Use

- **Quick Testing**: Want to validate your scenario setup?
- **Learning**: Understanding cash flows before worrying about uncertainty
- **Prototyping**: Building a new scenario
- **Quick Checks**: Fast (< 1 second) way to see baseline projections

### How to Run

1. Go to **Scenario** tab (press `3`)
2. Press `r` (run)
3. Wait ~1 second
4. View results in **Results** tab (press `4`)

### Understanding Results

Single run shows:

- **One deterministic path** through the years
- **Year-by-year account values**
- **Taxes and expenses**
- **Investment returns** (all at median)

**Example Output:**
```
Year  Age  Net Worth   Income  Expenses  Taxes
2025   60  $800,000   $100K    $75K     $15K
2026   61  $850,000   $100K    $75K     $13K
2027   62  $900,000   $100K    $75K     $12K
```

### Limitations

- Doesn't show **probability of success**
- Doesn't show **what happens in bad markets**
- Doesn't show **best/worst case scenarios**
- **Not recommended for final decision-making**

Use Monte Carlo for actual retirement planning decisions.

## Monte Carlo Simulation

### What It Does

Runs **1000+ simulations** where:
- Each run uses different random market returns
- Returns are drawn from historical distributions
- Shows probability of success and outcome ranges

### Statistical Approach

For each asset class, FinPlan:

1. Analyzes **historical returns** (e.g., S&P 500 from 1926-2024)
2. Calculates **mean** (average return) and **standard deviation** (volatility)
3. Randomly samples from that distribution for each simulation
4. Each run might have great returns, bad returns, or anything between

**Example:**
```
S&P 500 Historical Data:
  Average Return: 10.5% per year
  Volatility (Std Dev): 18%

Run 1: Drew returns that averaged 15% → Plan succeeds with $1.2M
Run 2: Drew returns that averaged 8% → Plan succeeds with $900K
Run 3: Drew returns that averaged -5% (market crash) → Plan fails
...after 1000 runs...
Success Rate: 92% (920 runs succeeded)
```

### When to Use

- **Major Decisions**: Retiring in 5 years? Use Monte Carlo.
- **Risk Assessment**: Can you handle down markets?
- **Real Retirement Planning**: Accounts for uncertainty
- **Scenario Comparison**: Which retirement age is safer?

### How to Run

1. Go to **Scenario** tab (press `3`)
2. Press `m` (monte carlo)
3. Wait 30-60 seconds (computing 1000 simulations)
4. View results in **Results** tab (press `4`)

### Understanding Results

**Metrics Shown:**

| Metric | Meaning |
|--------|---------|
| **Success Rate** | % of simulations that didn't run out of money |
| **P5 Outcome** | 5th percentile: worst 5% of scenarios |
| **P50 Outcome** | 50th percentile: median, middle scenario |
| **P95 Outcome** | 95th percentile: best 5% of scenarios |
| **Mean Outcome** | Average across all 1000 simulations |

**Example:**
```
Monte Carlo Results (1000 runs):

Success Rate: 92%
  → Your plan succeeds in 920 out of 1000 scenarios

P5 (Worst Case):
  Final Net Worth: $50,000
  → Even in worst 5%, you have cushion

P50 (Median):
  Final Net Worth: $800,000
  → Most likely outcome

P95 (Best Case):
  Final Net Worth: $2,000,000
  → In great markets, you're very secure
```

### Percentile Views

In **Results** tab, press `v` to cycle between percentile views:

**P5 (Worst Case):**
- 5th percentile scenario
- Bad luck with markets
- Shows if your plan survives worst conditions
- Most conservative estimate

**P50 (Median):**
- Middle outcome
- 50% of runs better, 50% worse
- Most likely scenario
- Good baseline for planning

**P95 (Best Case):**
- 95th percentile scenario
- Good luck with markets
- Shows upside potential
- Don't plan on this

**Mean:**
- Average across all simulations
- Statistically useful
- May not be realistic for individual scenario

### Success Rate Interpretation

| Success Rate | Interpretation |
|--------------|-----------------|
| 95%+ | Excellent. Very confident you'll succeed. |
| 90-95% | Good. High confidence with small risk. |
| 80-90% | Moderate. Acceptable for most, some risk. |
| 70-80% | High Risk. Consider adjusting plan. |
| <70% | Not Recommended. Plan is likely to fail. |

**Guidelines:**
- **Retirement at 60-65**: Aim for 90%+
- **Late career changes**: 80%+ is acceptable
- **Very long retirements**: Want 95%+

## Convergence Test

### What It Does

Runs Monte Carlo **multiple times** to verify results are stable.

The concern: If you run Monte Carlo twice, do you get the same probability?

**Convergence test:**
1. Runs Monte Carlo 5 times
2. Each run does 1000 simulations
3. Compares success rates across runs
4. If all 5 runs show ~92% success, results are stable

### When to Use

- **Before Major Decisions**: Verify results are reliable
- **Edge Cases**: When success rate is near your threshold (85-95%)
- **Complex Scenarios**: Many events and parameters
- **Peer Review**: Showing results to advisors

### How to Run

1. Go to **Scenario** tab (press `3`)
2. Press `Shift+M` (convergence test)
3. Wait 2-3 minutes (5 × 1000 simulations)
4. See success rates from each run

### Understanding Results

**Example Output:**
```
Convergence Test Results:

Run 1: 92.1% success
Run 2: 91.8% success
Run 3: 92.0% success
Run 4: 92.3% success
Run 5: 91.9% success

Average: 92.0%
Std Dev: ±0.2%

✓ Results are stable and reliable
```

**Interpretation:**
- If all runs show similar percentages: **Results are reliable**
- If runs vary widely (90%, 85%, 95%): **Results are unstable**
  - Could mean: Small sample size, unstable scenario, or edge case

## Working with Results

### Viewing Your Simulation

After running a simulation (any type):

1. Go to **Results** tab (press `4`)
2. You see:
   - Net worth chart (top-left)
   - Account breakdown (top-right)
   - Ledger of transactions (bottom)

### Navigating Years

In the **Net Worth Chart**:

| Key | Action |
|-----|--------|
| `h` or `←` | Previous year |
| `l` or `→` | Next year |
| `Home` | First year |
| `End` | Last year |

The highlighted bar shows the year you're viewing. The ledger updates to show that year's details.

### Switching Display Modes

In **Results** tab:

| Key | Action |
|-----|--------|
| `$` | Toggle real $ (inflation-adjusted) vs nominal $ |
| `v` | Cycle percentile (P5, P50, P95, Mean) - Monte Carlo only |
| `g` | Toggle granularity (annual vs monthly) |
| `f` | Filter by account type |

**Real vs Nominal Dollars:**
- **Nominal**: Raw numbers (unadjusted)
- **Real**: Adjusted for inflation (all in 2024 dollars)

If inflation is 2.5%, $80,000 in 2045 real dollars = ~$130,000 nominal.

### Analyzing the Ledger

The ledger shows year-by-year breakdown:

| Column | Meaning |
|--------|---------|
| Year | Calendar year |
| Age | Your age (if not deceased) |
| Starting Balance | Net worth at start of year |
| Income | Money in (salary, transfers) |
| Expenses | Money out (living costs) |
| Taxes | Federal and state income taxes |
| Investment Returns | Gains/losses from market |
| Ending Balance | Net worth at end of year |

**What to Look For:**
- Does money ever hit zero? (Plan fails)
- When are taxes highest? (Good for tax planning)
- When do you spend down accounts? (See liquidation strategy)
- Any dramatic drops? (Market crash years)

## Simulation Workflow

### Testing a New Scenario

```
1. Create scenario
2. Set up accounts (Portfolio & Profiles tab)
3. Add events (Events tab)
4. Run single simulation with 'r'
   - Quick check, any obvious errors?
5. If OK, run Monte Carlo with 'm'
   - How likely is success?
6. If <90% success, iterate:
   - Reduce spending?
   - Work longer?
   - Increase asset allocation?
7. Once comfortable, save scenario
8. Run convergence test (optional)
```

### Comparing Scenarios

```
1. Create "Retire at 60"
2. Create "Retire at 62"
3. Create "Retire at 65"
4. Go to each and run Monte Carlo
5. Compare success rates:
   - 60: 75% success
   - 62: 90% success
   - 65: 97% success
6. Choose retirement age that matches your risk tolerance
```

### Sensitivity Analysis

Use the **Analysis** tab (press `5`) to test:

1. How does spending change success rate?
2. How does retirement age change outcomes?
3. What's the optimal withdrawal strategy?

See [Results Interpretation](05-results-interpretation.md) for details on analysis.

## Common Scenarios to Test

### Retirement Age Comparison

Create scenarios for ages 60, 62, 65, and 67:

1. Adjust start date or event trigger age
2. Run Monte Carlo on each
3. Compare success rates
4. Find your minimum acceptable age

**Example Comparison:**
```
Retire at 60:  75% success
Retire at 62:  88% success
Retire at 65:  95% success

If you want 90% confidence, retire at 65 or later.
```

### Spending Level Sensitivity

1. Test different annual spending amounts
2. Run Monte Carlo for each level
3. See how much you can spend safely

**Example:**
```
$50K spending:  98% success (very conservative)
$75K spending:  92% success (reasonable)
$100K spending: 78% success (risky)

Sweet spot: $75K annual spending
```

### Market Condition Edge Cases

1. Create "Recession" scenario (lower expected returns)
2. Create "Roaring 20s" scenario (higher returns)
3. Create "Stagflation" scenario (high inflation, low returns)

Run Monte Carlo on each to understand tail risks.

### Healthcare or Inheritance Scenarios

1. Create "With $200K inheritance at 70"
   - Add event: inheritance
2. Create "Major medical expense at 75"
   - Add event: expense
3. Compare vs baseline

See impact of major life events on success rate.

## Tips for Running Simulations

### Efficient Testing

- **Single run first**: Quick validation
- **Monte Carlo when ready**: Comprehensive analysis
- **Convergence test rarely**: Only for important decisions

### Interpreting Success Rates

- **80%+ is usually good** for most retirement plans
- **90%+ is very safe** for long retirements
- **Below 70%** means plan likely to fail

### When Results Don't Match Expectations

If you run a simulation and the results surprise you:

1. **Check your inputs**: Verify accounts, events, dates are correct
2. **Review the ledger**: See where money is actually going
3. **Check market returns**: Are assets properly mapped?
4. **Try single run first**: Easier to debug one path
5. **Add test events**: Manually verify calculations

### Performance Notes

Simulation times depend on scenario complexity:

- **Single run**: < 1 second
- **Monte Carlo (1000 runs)**: 30-60 seconds
- **Convergence test**: 2-3 minutes

If simulations are very slow, your scenario might be too complex (hundreds of events). Consider consolidating events.

---

**Next:** Read [Understanding Results](05-results-interpretation.md) to learn how to interpret charts and metrics.
