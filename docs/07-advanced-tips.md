# Advanced Tips & Tricks

This guide covers power-user techniques and advanced workflows for FinPlan.

## Advanced Scenario Techniques

### Scenario Branching

Create a "family tree" of related scenarios to test variations:

```
Base_Plan
├── Base_Plan_Early_Retire_60
│   ├── Early_60_Conservative
│   ├── Early_60_Aggressive
│   └── Early_60_No_HealthCare
├── Base_Plan_Standard_Retire_65
│   ├── Standard_65_High_Spending
│   └── Standard_65_Low_Spending
└── Base_Plan_Late_Retire_70
    ├── Late_70_With_Inheritance
    └── Late_70_Extended_Work
```

**Workflow:**
1. Create base scenario with shared accounts/events
2. Copy it multiple times
3. Make small variations in each copy
4. Run Monte Carlo on all
5. Compare success rates to find optimal strategy

### Template Scenarios

Create a template scenario with your base assumptions:

```
My_Template
├── Your accounts setup
├── Your standard events
├── Your tax bracket
└── Historical return profiles
```

Then copy this template and customize for different "what-ifs":

```
My_Template → My_Plan_Retire_60
My_Template → My_Plan_Retire_65
My_Template → My_Plan_With_Inheritance
```

This ensures consistency across variations.

### Manual YAML Editing

For advanced users, manually edit scenario YAML files:

1. Export scenario with `x`
2. Open the `.yaml` file in a text editor
3. Make bulk changes:
   - Change all event amounts by 10%
   - Add events in batch
   - Copy event structures
4. Save the file
5. Re-import with `i`

**Example: Bulk Edit Expenses**

```yaml
# Before (annual spending of $75,000)
events:
  - name: Annual_Expenses
    amount: 75000
  - name: Healthcare_Annual
    amount: 5000

# After (increase all by 10% for inflation)
events:
  - name: Annual_Expenses
    amount: 82500    # 75000 * 1.1
  - name: Healthcare_Annual
    amount: 5500     # 5000 * 1.1
```

## Advanced Event Techniques

### Modeling Complex Events

**Graduated Expenses (Increasing Over Time):**

Create multiple events with the same trigger type but increasing amounts:

```
Healthcare_Age_60: $3,000
Healthcare_Age_65: $5,000
Healthcare_Age_75: $10,000
Healthcare_Age_85: $15,000
```

Each triggers at different ages with different amounts.

**Conditional Spending (If/Then):**

Model events that only occur under certain conditions:

```
Event: "Healthcare_Crisis"
Trigger: Balance Threshold (if account drops below $100k)
Effect: Withdraw $20,000 for emergency care

Result: Only triggers if you're in financial distress
```

**Seasonal Events:**

If your app supports repeating with custom intervals:

```
Event: "Holiday_Spending"
Trigger: Repeating, December 1st each year
Effect: Withdraw $5,000 for gifts

Effect: Only happens in December
```

### Modeling Roth Conversions

Strategy: Convert Traditional IRA to Roth during lower-income years

```
Event: "Roth_Conversion_Age_55"
Trigger: At age 55
Effects:
  1. Transfer $50,000 from Traditional IRA → Roth IRA
  2. Pay taxes on conversion ($10,000) from Checking

Result: Move money to tax-free account, pay taxes once
Benefits: Tax-free growth for 35+ years
```

### Modeling Stock Grants (RSU Vesting)

```
Event: "RSU_Vesting_Grant_1"
Trigger: Repeating, annual, starting age 40
Effects:
  1. Income: $50,000 (RSU vesting)
  2. Taxes: $15,000 (withholding)
  3. Contribution: $35,000 to Brokerage

Event: "RSU_Vesting_Grant_2"
Trigger: Repeating, annual, starting age 45
Effects:
  1. Income: $100,000 (larger grant later)
  2. Taxes: $30,000
  3. Contribution: $70,000 to Brokerage
```

This models career progression with increasing comp.

### Modeling Required Minimum Distributions (RMDs)

```
Event: "RMD_Starting_Age_73"
Trigger: At age 73
Effects:
  1. Withdrawal: ~4% of Traditional IRA (RMD amount)
  2. Goes to Checking for spending
  3. Taxes: Withdrawal is taxable income

Result: Forced withdrawals at age 73+, increases income/taxes
```

## Advanced Analysis Techniques

### Multi-Dimensional Sensitivity

Test 3+ parameters simultaneously:

```
Parameters:
1. Retirement Age (60-70)
2. Annual Spending ($50K-$100K)
3. Asset Allocation (50/50 - 80/20)

Results: See how combinations interact
Example: Early retirement + high spending + conservative = 65% success
Example: Deferred retirement + low spending + aggressive = 98% success
```

### Optimal Strategy Search

Use analysis to find the optimal combination:

```
Test all combinations of:
1. When to claim Social Security (62 vs 70)
2. How much to spend ($50K to $100K)
3. When to do Roth conversions (age 55-65)

Results: Find the single best combination for your situation
```

### Stress Testing

Create scenarios with extreme assumptions:

```
Best Case:
- Market returns: +12% annually (optimistic)
- Inflation: 1.5% (low)
- Longevity: Age 95
- Result: Success rate 99%, $2M ending

Baseline:
- Market returns: +8% annually (historical)
- Inflation: 2.5% (normal)
- Longevity: Age 100
- Result: Success rate 92%, $600K ending

Worst Case:
- Market returns: +3% annually (pessimistic)
- Inflation: 4.5% (high)
- Longevity: Age 105
- Result: Success rate 70%, $100K ending
```

How different does your plan look under different assumptions?

### Tax Optimization

Use events to model different tax strategies:

**Strategy 1: Maximize Tax-Deferred Contributions**
```
Create scenario with maxed 401k contributions
Run Monte Carlo
Compare to scenario without
Sees tax savings impact on retirement success
```

**Strategy 2: Tax-Loss Harvesting**
```
Event: "Tax_Loss_Harvesting"
Trigger: Repeating yearly
Effects:
  1. Sell losing positions in taxable account
  2. Realize losses to offset gains
  Result: Reduces tax liability

Compare with/without harvesting scenario
```

**Strategy 3: Roth Conversion Ladder**
```
Events:
- Age 55: Convert $20,000 IRA → Roth
- Age 56: Convert $20,000 IRA → Roth
- Age 57: Convert $20,000 IRA → Roth
...continues until Traditional IRA is empty

Result: Move entire IRA to tax-free over time
Tax cost spread over multiple years (lower tax brackets)
```

## Performance Optimization

### Simplifying Complex Scenarios

If your scenario is very slow to simulate:

**Problem Indicators:**
- Single run takes > 5 seconds
- Monte Carlo takes > 2 minutes
- App feels sluggish

**Solutions:**

1. **Combine Recurring Events:**
   ```
   Before: 40 separate "Annual_Expense" events (one per year, manually added)
   After: 1 "Annual_Expense" event set to repeat yearly
   
   Impact: 40× speed improvement
   ```

2. **Simplify Asset Allocation:**
   ```
   Before: 20 different ETFs (VFIAX, VTSAX, VTIAX, BND, VGLT, etc.)
   After: 5 asset classes (US Stocks, Intl, Bonds, Real Estate, Cash)
   
   Impact: Reduce processing by 75%
   ```

3. **Reduce Simulation Duration:**
   ```
   Before: Run 60-year simulation (age 40-100)
   After: Run 40-year simulation (age 40-80)
   
   Impact: Much faster, covers your likely lifespan
   ```

4. **Use Single Run for Development:**
   ```
   Before: Running Monte Carlo (1000 runs) for every test
   After: Use Single Run (deterministic) to test, then Monte Carlo for final
   
   Impact: 30-50× speedup during scenario building
   ```

## Advanced Visualization

### Comparing Multiple Scenarios

While FinPlan doesn't have built-in side-by-side comparison, you can:

1. Run single run on Scenario A, note results
2. Export results to a text file/screenshot
3. Switch to Scenario B, run it
4. Export those results
5. Compare manually in spreadsheet

### Tracking Results Over Time

Workflow for long-term planning:

```
Month 1: Create "Annual_Review_2025-01" scenario, run Monte Carlo
         Export results, save to archive
         Success rate: 88%, P50: $750K

Month 6: Create "Mid_Year_Check_2025-06" scenario
         Export results
         Success rate: 85%, P50: $720K
         (Markets down, adjust spending or work longer)

Month 12: Create "Annual_Review_2025-12" scenario
          Export results
          Success rate: 92%, P50: $800K
          (Markets recovered, update plan for next year)
```

This creates a history of how your plan changes with time.

## Data Management

### Backing Up Your Scenarios

**Automated Backup Workflow:**

1. Monthly: Export main scenario
   ```bash
   # At end of each month
   cp ~/.finplan/scenarios/Main_Plan.yaml ~/Dropbox/FinPlan_Backups/Main_2025-01.yaml
   cp ~/.finplan/scenarios/Main_Plan.yaml ~/FinPlan_Archive/Main_2025-01.yaml
   ```

2. Cloud Sync:
   ```bash
   # Symlink to cloud storage
   ln -s ~/Dropbox/FinPlan ~/.finplan_cloud_backup
   ```

3. Version Control (Git):
   ```bash
   cd ~/.finplan/scenarios
   git init
   git add .
   git commit -m "Initial scenarios"
   # Commit after major changes
   ```

### Archiving Old Scenarios

Keep working scenarios lean:

```bash
# Create archive
mkdir ~/.finplan/archive

# Move old scenarios
mv ~/.finplan/scenarios/2023_* ~/.finplan/archive/
mv ~/.finplan/scenarios/Old_* ~/.finplan/archive/
mv ~/.finplan/scenarios/Test_* ~/.finplan/archive/

# Can re-import from archive if needed
```

### Organizing by Year

Use naming to organize chronologically:

```
2024_Annual_Review_Baseline
2024_Q4_Update
2025_January_Replan
2025_Spring_Adjustment
2025_Summer_Analysis
```

Scenarios sort alphabetically, so years naturally group.

## Troubleshooting Tips

### Scenario Won't Import

```
Error: "Failed to parse YAML"

Solutions:
1. Check file is actually YAML format (.yaml or .yml)
2. Open in text editor, check for:
   - Proper indentation (spaces, not tabs)
   - Matching quotes and braces
   - No special characters in wrong places
3. Use online YAML validator to find syntax errors
4. Make a copy, gradually remove sections until it imports
```

### Results Don't Make Sense

```
Unexpected result? (e.g., huge taxes, negative net worth)

Debugging workflow:
1. Run single deterministic simulation (faster)
2. View ledger year by year
3. Find the problem year
4. Check what events fired that year
5. Edit event to fix issue
6. Re-run simulation
```

### Success Rate Seems Wrong

```
Got 85% success but felt like should be 95%

Check:
1. Are all accounts/assets included? (May be forgetting one)
2. Are taxes set correctly? (Check federal bracket, state tax)
3. Are all events included? (Major expense forgotten?)
4. Is spending too high? (More than you estimated?)
5. Check inflation setting (high inflation = harder to maintain spending)
6. Review asset allocations (too conservative = lower returns)
```

## Power User Workflows

### Quarterly Review Process

```
Q1 Review:
1. Create "Q1_Review_Current" scenario
2. Update all balances to actual (Jan 1 values)
3. Run Monte Carlo
4. Compare to prior quarter
5. Adjust if needed
6. Archive Q1_Review_Current

Q2 Review:
1. Same process with April 1 values
2. Compare trend over quarters
```

### Annual Planning Process

```
January 1:
1. Create "2025_Baseline" from prior year
2. Update all accounts to January 1 values
3. Update life events (income changes, major purchases planned)
4. Adjust return assumptions if needed
5. Run full Monte Carlo
6. Export results, save as baseline

Throughout year:
- Compare quarterly results to 2025_Baseline
- Adjust if actual vs expected differs significantly

December 31:
- Create "2025_YearEnd" with actual values
- Compare actual performance vs projected
- Use learnings to refine 2026 plan
```

### What-If Analysis Process

```
Starting scenario: "Baseline_Retire_65"
Success rate: 92%

Question: Can I retire at 63 instead?

Process:
1. Copy "Baseline_Retire_65" → "Test_Retire_63"
2. Adjust retirement trigger to age 63
3. Run Monte Carlo
4. Result: 83% success (too risky)

Question: What if I spend less?

Process:
1. Copy "Baseline_Retire_65" → "Test_Low_Spending"
2. Reduce annual spending $10,000
3. Run Monte Carlo
4. Result: 94% success (good!)

Question: Can I retire at 63 if I spend less?

Process:
1. Copy "Test_Retire_63" → "Test_63_Low_Spend"
2. Reduce spending $10,000
3. Run Monte Carlo
4. Result: 88% success (acceptable!)

Conclusion: Can retire at 63 with $10k less spending
```

---

**Ready to build your perfect plan?** Use these advanced techniques to model complex scenarios and find your optimal retirement strategy.
