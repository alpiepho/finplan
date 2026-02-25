# Understanding Results & Analysis

This guide helps you interpret FinPlan's charts, metrics, and analysis features.

## Results Tab Overview

The **Results** tab shows your simulation output with:

1. **Net Worth Projection Chart** - Your wealth over time
2. **Account Breakdown** - Money distributed by account type
3. **Ledger** - Detailed year-by-year transactions
4. **Status Indicators** - Key metrics

## Reading the Net Worth Chart

### Typical Chart Display

Here's what you'll see in the Results tab when viewing a Monte Carlo simulation:

```
NET WORTH PROJECTION (2045) (P50 Nominal)

        Net Worth ($)
    $2.0M │
    $1.8M │                                    P5 scenario
          │                                   (worst 5%)
    $1.6M │    ████                          P50 scenario
          │    ████ ████                     (median)
    $1.4M │    ████ ████ ████               P95 scenario
          │    ████ ████ ████ ████          (best 5%)
    $1.2M │    ████ ████ ████ ████ ████
          │    ████ ████ ████ ████ ████ ████
    $1.0M │    ████ ████ ████ ████ ████ ████ ████
          │    ████ ████ ████ ████ ████ ████ ████ ████
    $800K │    ████ ████ ████ ████ ████ ████ ████ ████
          │    ████ ████ ████ ████ ████ ████ ████ ████ ████
    $600K │    ████ ████ ████ ████ ████ ████ ████ ████ ████ ████
          │    ████ ████ ████ ████ ████ ████ ████ ████ ████ ████
    $400K │    ████ ████ ████ ████ ████ ████ ████ ████ ████ ████
          │    ████ ████ ████ ████ ████ ████ ████ ████ ████ ████
    $200K │    ████ ████ ████ ████ ████ ████ ████ ████ ████ ████
          │
        $0 └─────────────────────────────────────────────────
             2025  2030  2035  2040  2045  2050  2055

Colors represent account types:
  Green  = Cash (Checking, Savings)
  Blue   = Tax-Deferred (401k, IRA)
  Purple = Tax-Free (Roth)
  Yellow = Taxable (Brokerage)
  Red    = Debt (negative)
```

### Chart Layout

```
┌─────────────────────────────────────┐
│  NET WORTH PROJECTION (2025) (Nominal)│
├─────────────────────────────────────┤
│     ███                             │
│     ███ ███                         │
│ ███ ███ ███ ███                     │
│ ███ ███ ███ ███ ███                 │
│ ███ ███ ███ ███ ███ ███             │
├─────────────────────────────────────┤
│ 2025  2027  2029  2031  2033  2035  │
└─────────────────────────────────────┘

Legend:
▓▓▓ Green   = Cash (Checking, Savings)
▓▓▓ Blue    = Tax-Deferred (401k, IRA)
▓▓▓ Purple  = Tax-Free (Roth)
▓▓▓ Yellow  = Taxable (Brokerage)
▓▓▓ Red     = Debt (Negative)
```

### Interpreting Heights

- **Tall bar** = High net worth that year
- **Short bar** = Lower net worth
- **Declining bars** = Spending down your wealth (expected in retirement)
- **Flat bars** = Stable wealth (income covering expenses)

### Color Composition

Each bar is stacked in segments:

```
$2.0M ┌─────┐      Total: $2M
      │Green│ $100K (Cash)
$1.5M ├─────┤
      │ Blue│ $700K (401k)
$1.0M ├─────┤
      │Purpl│ $600K (Roth)
$0.5M ├─────┤
      │Yello│ $600K (Taxable)
    0 └─────┘
```

The proportion of colors shows how much money is in each account type.

**What's Good:**
- Tax-free (purple) growing over time
- Tax-deferred (blue) in early years, then drawn down
- Taxable (yellow) keeping a cushion

**What's Concerning:**
- All wealth in one account type (lack of diversification)
- Red sections (debt) growing
- Rapid bar decline (spending too much)

## Reading Single Run Results

### Example: Single Deterministic Run

```
Year  Age  Net Worth   Income   Expenses  Taxes   Returns
2025   60  $800,000   $120K    $75K      $15K    +$50K
2026   61  $805,000   $120K    $75K      $14K    +$45K
2027   62  $810,000    $75K    $75K      $12K    +$45K   ← Transition year
2028   63  $820,000     $0K    $75K       $8K    +$48K   ← Drawing down accounts
2029   64  $835,000     $0K    $75K       $6K    +$50K
...continues for 40 years...
2065   100 $150,000     $0K    $75K       $0K    -$10K
2066   101 -$50,000 ← PLAN FAILS (money runs out in year 41)
```

### What to Look For

**Success Indicators:**
- ✓ Net worth stays positive entire period
- ✓ Decreases predictably as you spend down
- ✓ Taxes decline as income ends
- ✓ Still have money at 100+ years old

**Warning Signs:**
- ✗ Net worth hits zero or negative (plan fails)
- ✗ Wild year-to-year swings (indicates problem)
- ✗ Rapid decline in early years (spending too much)
- ✗ Account type runs dry before others (liquidation issue)

### Single Run Limitations

A single run shows **only one path**. Reality is random:
- Markets might crash early (bad scenario)
- Markets might soar (good scenario)
- You might live longer than expected
- Major health costs might appear

**Always follow up with Monte Carlo** for real retirement decisions.

## Reading Monte Carlo Results

### Success Rate

The most important metric from Monte Carlo.

```
1000 simulations run with different market returns:

Succeeded: 920 simulations (92% success rate)
Failed:     80 simulations (8% failure rate)
```

**Interpretation:**
- **92% success** = In 92 out of 100 alternate universes, your plan works
- **8% failure** = In 8 out of 100, you run out of money

### Success Rate Benchmarks

| Range | Interpretation | Recommendation |
|-------|-----------------|-----------------|
| 95%+ | Excellent. Very safe. | Retire comfortably |
| 90-95% | Good. High confidence. | Safe choice |
| 80-90% | Moderate. Acceptable risk. | OK for most |
| 70-80% | High risk. Significant chance of failure. | Consider adjusting |
| <70% | Very risky. Plan likely to fail. | Modify plan |

**Financial Rules of Thumb:**
- **Traditional**: Aim for 95%+ for 30-year retirement
- **Modern**: 90% is acceptable for most people
- **Aggressive**: 80% if you have flexibility (can reduce spending)

### Percentile Outcomes

When viewing Monte Carlo results, you see different percentiles:

**P5 (5th Percentile - Worst Case):**
```
Final Net Worth: $50,000

Worst 5% of scenarios end here. Even in very bad luck with markets,
you have a cushion. This is your downside protection.
```

**P50 (50th Percentile - Median):**
```
Final Net Worth: $800,000

Middle outcome. Half the runs do better, half do worse.
This is the most likely scenario. Plan for this.
```

**P95 (95th Percentile - Best Case):**
```
Final Net Worth: $2,000,000

Best 5% of scenarios end here. With lucky market timing,
you're very wealthy. Don't plan for this.
```

**Mean (Average):**
```
Final Net Worth: $750,000

Mathematical average across all 1000 runs.
Often not realistic for individual scenarios.
```

### Example Monte Carlo Output

```
Monte Carlo Analysis (1000 simulations)

Success Rate: 88%

Final Net Worth (at age 100):
  P5 (worst 5%):  $100,000
  P50 (median):   $600,000
  P95 (best 5%):  $1,800,000
  Mean (avg):     $700,000

Max Drawdown (worst year-to-year decline):
  P5:   -45% (stocks crash, bad year)
  P50:  -25% (normal market correction)
  P95:  -10% (minor market decline)

Lifetime Taxes Paid:
  P5:   $400,000 (lower income years = fewer taxes)
  P50:  $600,000 (steady taxes throughout)
  P95:  $750,000 (higher income, more taxes)
```

### Interpreting Percentiles

**Safe Plan Indicators:**
- P5 outcome is still positive (cushion against bad luck)
- P50 outcome is comfortable for your lifestyle
- P95 outcome shows potential upside

**Example Good Plan:**
```
Success: 92%
P5: $200,000
P50: $800,000
P95: $2,200,000

Analysis: Even worst-case has cushion. Median is strong.
Confidence: High
```

**Example Concerning Plan:**
```
Success: 72%
P5: -$50,000 (runs out of money!)
P50: $300,000
P95: $1,500,000

Analysis: 28% chance of failure. Worst case is bad.
Confidence: Low - need to adjust
```

## Account Breakdown Panel

Shows how your wealth is distributed by account type:

```
Cash        $100K  ▓▓▓░░░░░░░  (5%)
Tax-Deferred $700K  ▓▓▓▓▓▓▓░░░  (35%)
Tax-Free     $600K  ▓▓▓▓▓▓░░░░  (30%)
Taxable      $600K  ▓▓▓▓▓▓░░░░  (30%)
Debt           $0   ░░░░░░░░░░  (0%)
─────────────────────────────────────
Total      $2.0M
```

### Analyzing the Breakdown

**Good distributions at different life stages:**

**Early Retirement (Age 60-65):**
- Keep 2-3 years expenses in cash
- 30-40% in tax-deferred (401k phase)
- 20-30% in tax-free (Roth)
- 30-40% in taxable (flexibility)

**Mid-Retirement (Age 75-85):**
- Keep 1-2 years expenses in cash
- Less in tax-deferred (drawing it down)
- More in tax-free (preserve assets)
- Adjust taxable as needed

**Late Retirement (Age 90+):**
- Mostly spent down
- Some tax-free remaining (Roth)
- Debt should be zero
- Declining but positive

### Red Flags

- **All cash**: No growth potential (inflation problem)
- **All in one account**: Inflexible (can't optimize taxes)
- **Growing debt**: Mortgages should decrease
- **Negative total**: Plan failed (money ran out)

## Ledger Panel

Detailed year-by-year breakdown:

```
Year Age Start    Income  Expenses Taxes Investment End
      Balance                             Returns    Balance
2025 60  $800K   $120K   $75K     $15K  +$50K      $880K
2026 61  $880K   $120K   $75K     $14K  +$45K      $956K
2027 62  $956K    $75K   $75K     $12K  +$48K      $997K
2028 63  $997K     $0K   $75K      $8K  +$50K      $964K (declining)
2029 64  $964K     $0K   $75K      $6K  +$48K      $931K
```

### Key Columns

**Starting Balance:**
- Net worth at start of year
- Should match previous year's ending balance

**Income:**
- All money coming in (salary, Social Security, gifts, transfers)
- Usually drops at retirement

**Expenses:**
- Living costs, planned purchases
- Should be relatively stable (unless you modeled change)

**Taxes:**
- Federal and state income taxes paid that year
- Depends on income level and account type

**Investment Returns:**
- Gains/losses from market for that year
- Positive in good years, negative in down markets
- Only for Monte Carlo (single run always uses median)

**Ending Balance:**
- Net worth at end of year
- Formula: Start + Income - Expenses - Taxes + Returns

### Using the Ledger

**Find the year money runs out:**
- Scroll through ledger
- Look for negative net worth
- See what caused it (too much spending, bad market, etc.)

**Identify high-tax years:**
- Look for large "Taxes" column values
- These might be conversion years or high-income years
- Could optimize with tax strategies

**See portfolio growth:**
- Compare starting to ending balances
- Investment returns should compound
- Declining balance is normal in retirement

**Verify events fire correctly:**
- Look for expected income/expense jumps
- Social Security should appear at expected age
- Major expenses should align with planned events

## Charts and Visualization

### What Different Chart Shapes Mean

**Steady Decline (Expected):**
```
$2.0M ┐
      │
$1.5M │
      │
$1.0M │
      │
$500K │
      │
    0 └─ ─── ─── ─── ─── ─── ─── ───
      Age 60  70  80  90 100

✓ Healthy: Spending down savings at expected rate
  You're living off savings as planned
```

**Cliff (Danger):**
```
$2.0M ┐  ███ ███ ███ ███ ███
      │
$1.5M │
      │
$1.0M │
      │
$500K │
      │
    0 └─ ─── ─── ─── ─── ─── ─── ─── ─── ───   ╲╲ RUNS OUT!
      Age 60  70  80  90 100  110

✗ Bad: Sudden drop to zero (money runs out)
  Plan fails in this scenario
```

**Plateau (Good):**
```
$2.0M ┐
      │
$1.5M ││
      │   ╱╱│
$1.0M │  ╱╱│
      │ ╱╱│
$500K │╱╱│
      ││
    0 └──────┴──────────────────
      Age 60  80               120

✓ Excellent: Levels off and stays stable
  Income covers expenses, wealth is sustainable
  Plan works indefinitely
```

**Growth (Unexpected):**
```
$4.0M ┐                      ┌──────
      │                   ╱╱│
$3.0M │               ╱╱╱│
      │
$2.0M │
      │
$1.0M │
      │
    0 └──────────────────────────
      Age 60  70  80  90 100 110

✓ Great: Growing wealth over time
  Usually from inheritance, high investment returns,
  or significantly lower expenses than expected
```

## Analysis Tab: Sensitivity Analysis

The **Analysis** tab lets you test how changes affect your plan.

### Parameter Sweeps

Test multiple values of a parameter and see the impact:

**Example: Retirement Age**
```
Test ages 60-70 to find optimal retirement year

60  → 75% success rate (risky)
62  → 88% success rate (good)
64  → 95% success rate (very safe)
66  → 98% success rate (overly safe)

Answer: Retire at 64 for 95% confidence
```

### Chart Types

**1D Scatter Plot (One Parameter):**
```
Success Rate (%)
100% |     ╱╱╱
 90% |   ╱╱
 80% | ╱╱
 70%|╱
    └─────────────────
      60  62  64  66  68  70
         Retirement Age
```

How a single parameter affects outcome.

**2D Heatmap (Two Parameters):**
```
Annual    $100k  🟩🟩🟩  (80% success)
Spending  $80k   🟨🟨🟩  (60% success)
          $60k   🟥🟨🟩  (40% success)
               └────────────────
                 60  65  70
              Retirement Age
```

Color intensity shows outcome quality. Bright = good, dark = bad.

### Metrics You Can Analyze

| Metric | What It Shows |
|--------|---------------|
| **Success Rate** | % of scenarios where plan doesn't run out of money |
| **P50 Final Net Worth** | Median ending wealth |
| **P5 Final Net Worth** | Conservative ending wealth |
| **P95 Final Net Worth** | Optimistic ending wealth |
| **Lifetime Taxes** | Total taxes paid over lifetime |
| **Max Drawdown** | Worst year-to-year decline |

### Analysis Workflow

```
1. Decide what you want to optimize
   → "I want to retire as early as possible"

2. Add that parameter to analysis
   → Retirement age (60-70)

3. Run analysis (press 'r')
   → Tests all ages from 60 to 70

4. Look at the chart
   → Find the age where success rate reaches 90%

5. Answer: "I can safely retire at age 62"
```

### Finding Optimal Values

**Example 1: Spending Level**
```
Find maximum safe spending

Test: $50K to $100K annually

Results:
  $50K  → 99% success (can spend more)
  $70K  → 92% success (good)
  $80K  → 85% success (acceptable)
  $100K → 70% success (too much)

Recommendation: $80K spending for 85% confidence
```

**Example 2: Early vs Deferred Social Security**
```
Test: Claim at 62 vs 70

Claim at 62:
  Small amount per year, collected 8 years early
  Success rate: 85%

Wait until 70:
  Larger amount per year, delayed 8 years
  Success rate: 92%

Recommendation: Wait until 70 (higher success rate)
```

**Example 3: Work Longer vs Higher Spending**
```
Two parameters: Retirement Age, Annual Spending

Retire at 60, Spend $100K → 70% success
Retire at 62, Spend $90K  → 85% success
Retire at 64, Spend $80K  → 95% success
Retire at 60, Spend $70K  → 88% success (same as retire at 62 with $90K)

Options:
1. Work 2 more years (62 vs 60), reduce spending $10K → +15% success
2. Work 4 more years (64 vs 60), reduce spending $20K → +25% success
3. Keep 60 age, reduce spending $30K → +18% success

Choose based on preferences (work-life balance vs spending desires)
```

## Common Analysis Questions

### "When can I retire?"

1. Go to **Analysis** tab
2. Add your retirement age event as parameter
3. Test ages 60-70
4. Find where success rate reaches 90%

### "How much can I spend?"

1. Add annual spending as parameter
2. Test $40K to $120K
3. Find maximum spending at 90% success rate

### "Should I delay Social Security?"

1. Create two scenarios: Claim at 62 vs 70
2. Run Monte Carlo on each
3. Compare success rates
4. (Usually waiting wins, but depends on longevity)

### "What's my safe withdrawal rate?"

1. Add spending or withdrawal percentage as parameter
2. Test 2%, 3%, 4%, 5%
3. Find the rate at 90% success
4. This is your personal safe withdrawal rate

## Tips for Interpretation

### What's a Good Outcome?

- **Success rate**: 90%+ is strong
- **P5 net worth**: Should be positive (safety margin)
- **P50 net worth**: Should support your lifestyle
- **No cliff**: Chart shouldn't suddenly drop

### When to Adjust Your Plan

- **Success rate < 85%**: Plan is risky, consider changes
- **P5 outcome negative**: Plan might fail in bad luck scenario
- **Rapid decline**: Spending might be too high
- **All wealth in one account**: Lacks tax flexibility

### Changes to Consider

- **Reduce spending**: 10% cut often adds 5-10% success rate
- **Work longer**: Each extra year adds 3-5% success rate
- **Optimize withdrawal strategy**: Can save 1-2% in lifetime taxes
- **Adjust asset allocation**: More conservative = lower risk but also lower growth

---

**Next:** See [Keybindings Reference](06-keybindings.md) for complete keyboard shortcut reference.
