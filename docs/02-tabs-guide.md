# Tab Reference Guide

FinPlan has 5 tabs, each with a specific purpose. This guide details what you can do in each tab.

---

## Tab 1: Portfolio & Profiles

**Access with:** `1` or `Shift+P`

This tab is where you define your financial accounts and configure asset mappings.

### Overview

The **Portfolio & Profiles** tab has three main sections:

1. **Portfolio Overview** (Top) - Visual summary of your net worth
2. **Accounts & Profiles** (Middle) - Your accounts and asset mappings
3. **Asset Mappings & Tax Config** (Bottom) - Return profiles and tax settings

### Portfolio Overview

Shows:
- **Total Net Worth**: Sum of all accounts minus debt
- **Account Breakdown**: Bar chart of each account's value
- **Color Codes**:
  - Green: Cash accounts (Checking, Savings, HSA)
  - Blue: Tax-Deferred (401k, Traditional IRA)
  - Purple: Tax-Free (Roth IRA, Roth 401k)
  - Yellow: Taxable (Brokerage)
  - Red: Debt (Mortgages, loans)

### Accounts & Profiles Panel

**Keybindings:**
| Key | Action |
|-----|--------|
| `a` | Add new account |
| `e` | Edit selected account |
| `d` | Delete account |
| `j/k` | Navigate up/down |
| `m` | Map return profile to account |
| `y` | Toggle asset history display |

**Account Types Available:**

- **Checking**: Daily spending account
- **Savings**: High-yield savings, emergency fund
- **Taxable**: Brokerage account (taxable on gains)
- **Traditional 401k**: Employer retirement plan (tax-deferred)
- **Roth 401k**: Employer Roth plan (tax-free growth)
- **Traditional IRA**: Self-directed retirement account (tax-deferred)
- **Roth IRA**: Self-directed Roth account (tax-free)
- **HSA**: Health Savings Account (triple tax-free)
- **Property**: Real estate (illiquid, doesn't generate returns)
- **Collectible**: Art, jewelry, etc. (illiquid)
- **Mortgage**: Home loan (liability)
- **Loan Debt**: Personal loans (liability)
- **Student Loan Debt**: Education loans (liability)

**Adding an Account:**

1. Press `a` to open the account creation form
2. Enter account name (e.g., "Roth IRA")
3. Choose account type
4. Enter current balance
5. Press `Enter` to create
6. Now add holdings to the account

**Adding Holdings (Assets):**

1. Select the account you created
2. Press `e` to edit and add assets
3. Choose asset type (Stock, Bond, Fund, etc.)
4. Enter ticker symbol (e.g., VFIAX, VTI, BND)
5. Enter shares and current price
6. Assign a return profile (next section)

### Asset Mappings Panel

Links your tickers to market indices and return profiles. This tells FinPlan which historical returns to use for each asset.

**Keybindings:**
| Key | Action |
|-----|--------|
| `m` | Map selected asset to profile |
| `Shift+A` | Suggest all assets to profiles |
| `a` | Suggest mapping for selected asset |
| `b` | Change asset block size for display |

**Return Profiles:**

Pre-configured historical return profiles you can assign:

| Profile | Historical Returns | Use Case |
|---------|-------------------|----------|
| S&P 500 | 1926-2024 actual returns | US large-cap stocks (VTI, VOO, SPY) |
| Total Market | 1926-2024 actual returns | Total US market (VTI, VTSAX) |
| Bonds | Historical bond returns | Fixed income (BND, AGG) |
| Small Cap | Historical small-cap returns | Russell 2000 |
| International | EAFE historical returns | Foreign stocks (VXUS, VEA) |

**How to Map:**

1. Select an asset in the mappings panel
2. Press `m` (map)
3. Choose a return profile from the list
4. That asset will now use that profile's historical returns

**Suggestion Mode:**

- Press `a` to auto-suggest a profile based on ticker name
- Press `Shift+A` to suggest all unmapped assets

### Tax & Inflation Config Panel

Configure tax settings and inflation assumptions.

**Keybindings:**
| Key | Action |
|-----|--------|
| `e` | Edit setting |
| `Tab` / `Shift+Tab` | Navigate fields |

**Tax Settings:**

- **Federal Tax Brackets**: Preset options include 2024 brackets
  - Choose your filing status (Single, Married Filing Jointly, etc.)
  - Brackets determine how much income tax you pay

- **State Income Tax**: 
  - Enter your state or fixed percentage rate
  - Applied to earned income (not investment income in most states)

- **Capital Gains Tax**: 
  - 0% / 15% / 20% based on income level
  - Automatically calculated from federal brackets

- **Long-term vs Short-term**:
  - Holdings > 1 year get favorable long-term rates
  - Holdings < 1 year taxed as ordinary income

**Inflation Settings:**

- **Inflation Rate**: 
  - Fixed percentage (e.g., 2.5% annually)
  - Or choose historical inflation profile
  - Affects purchasing power in "real dollars" view

**Display Modes:**

- **Nominal Dollars**: Raw numbers (unadjusted for inflation)
- **Real Dollars**: Adjusted for inflation (2024 dollars)

Flip between them with `$` in the Results or Scenario tabs.

---

## Tab 2: Events

**Access with:** `2` or `Shift+E`

Events are how you model life changes. They trigger income, expenses, asset purchases, account transfers, and more.

### Overview

The **Events** tab has two main panels:

1. **Event List** (Left) - All your events
2. **Event Details & Timeline** (Right) - Details of selected event + timeline view

### Event List

**Keybindings:**
| Key | Action |
|-----|--------|
| `a` | Add new event |
| `e` | Edit selected event |
| `d` | Delete event |
| `c` | Copy event (duplicate) |
| `t` | Toggle enabled/disabled |
| `f` | Configure effects (what the event does) |
| `j/k` or arrows | Navigate list |

**Event Status Indicators:**

- **✓ Enabled**: Event will run in simulations
- **✗ Disabled**: Event will be skipped (good for testing)
- **Once**: Event fires once, then never again
- **Repeating**: Event fires multiple times

### Creating Events

Press `a` to create a new event. You'll configure:

**Basic Info:**
- **Name**: E.g., "Annual Expenses" or "Retirement at 60"
- **Description** (optional): Notes about the event
- **Enabled**: Whether to include in simulations
- **Once Only**: If true, triggers once and stops

**Trigger (When):**

Events fire based on:

- **Age**: "At age 62" — fires when you turn 62
- **Date**: "On 2025-01-01" — fires on specific date
- **Balance Threshold**: "When account balance > $500,000" — fires when condition met
- **Repeating**: Fires on a schedule (yearly, monthly, specific days)

**Effects (What Happens):**

Each event can have multiple effects:

- **Income**: Salary, bonuses, gifts received
- **Expense**: Bills, living expenses
- **Transfer**: Move money between accounts
- **Purchase Asset**: Buy stock or other asset
- **Sell Asset**: Liquidate holdings
- **Contribution**: Add to retirement account
- **Distribution**: Withdraw from account

### Event Details

The right panel shows detailed information about your selected event:

- Event name and description
- Trigger type and timing
- All effects listed
- Whether it's enabled

**Example Event: "Annual Expenses"**
```
Trigger: Repeating every year starting at age 45
Effects:
  1. Transfer $50,000 from Checking to Expenses (repeat yearly)
```

**Example Event: "Roth Conversion at 55"**
```
Trigger: At age 55 (once only)
Effects:
  1. Transfer $20,000 from Traditional IRA to Roth IRA
  2. Pay taxes of $5,000 from Checking (for conversion income)
```

### Timeline View

Shows all events chronologically:

- **Event Name**: What happens
- **Trigger Year**: When it fires (calculated from your birth date)
- **Repeat Status**: (R) for repeating, single year for one-time
- Helpful to see the order of events in your life plan

Press `e` to edit the timeline section or `y` to toggle its visibility.

### Event Effects Panel

Press `f` to edit what an event actually does.

**Effect Configuration:**

For each effect, you specify:

- **Source Account**: Where money comes from
- **Destination Account**: Where money goes
- **Amount**: Fixed amount or formula based on other parameters
- **Frequency**: Once, yearly, or custom interval
- **Inflation Adjusted**: Whether amount grows with inflation

**Amount Types:**

- **Fixed**: $50,000 every year
- **Percentage**: 5% of account balance
- **Calculated**: Based on age, income, or other factors

---

## Tab 3: Scenario

**Access with:** `3` or `Shift+S`

The **Scenario** tab is where you manage scenarios and configure simulation parameters.

### Overview

Two main panels:

1. **Scenario List** (Left) - All your saved scenarios
2. **Simulation Parameters** (Right) - Configuration settings

### Scenario List

**Keybindings:**
| Key | Action |
|-----|--------|
| `n` | Create new scenario |
| `c` | Copy selected scenario |
| `s` | Save current as new name |
| `l` | Load scenario |
| `i` | Import scenario from YAML file |
| `x` | Export scenario to YAML file |
| `e` | Edit scenario name |
| `d` | Delete scenario |
| `Enter` | Switch to selected scenario |
| `j/k` | Navigate list |

**Creating a New Scenario:**

1. Press `n`
2. Enter a scenario name (e.g., "Retirement at 62")
3. You'll be switched to that scenario
4. Configure it using the parameters on the right

**Saving/Loading:**

- Current scenario is auto-saved
- Use `s` to save as a different name
- Use `l` to load a different scenario
- Use `c` to duplicate before making changes

**Importing Scenarios:**

1. Press `i` to import
2. Enter path to YAML file
   - Absolute path: `/Users/name/Desktop/plan.yaml`
   - Relative path: `examples/example.yaml` (relative to app directory)
3. Give it a name when prompted
4. It will appear in your scenario list

**Exporting Scenarios:**

1. Select scenario from list
2. Press `x` to export
3. Choose where to save it
4. Share with others or backup

### Simulation Parameters

Configure when and how the simulation runs.

**Keybindings:**
| Key | Action |
|-----|--------|
| `e` | Edit parameters |
| `r` | Run single simulation |
| `m` | Run Monte Carlo (1000 runs) |
| `Shift+M` | Run convergence test (verify stability) |
| `Shift+R` | Run all scenarios |
| `p` | Preview simulation (show setup check) |
| `$` | Toggle real/nominal dollars |

**Key Parameters to Set:**

**Life Dates:**
- **Birth Date**: Your birth date (e.g., 1965-03-15)
- **Start Date**: When simulation begins (e.g., 2025-01-01)
- **Duration (Years)**: How many years to simulate (e.g., 40 for age 60 to 100)

**Spending:**
- **Annual Spending**: Your estimated yearly expenses
  - Used if no events defined
  - Can be overridden by events

**Withdrawals:**
- **Withdrawal Strategy**: How to withdraw from accounts
  - FIFO: Take from accounts in order
  - Tax-aware: Minimize taxes
  - Specific account selection

**Other:**
- **Inflation Rate**: Annual inflation (e.g., 2.5%)
- **Tax Settings**: Federal bracket, state taxes (set in Portfolio tab)

### Running Simulations

**Single Run (Deterministic):**
1. Press `r` from the Scenario tab
2. Uses median/middle-of-the-road market returns
3. Quick (~1 second)
4. Good for testing setup and exploring quickly

**Monte Carlo (Probabilistic):**
1. Press `m` from the Scenario tab
2. Runs 1000+ simulations with randomized returns
3. Takes 30-60 seconds
4. Shows probability of success, ranges, and percentiles
5. Most realistic view of retirement outcomes

**Convergence Test:**
1. Press `Shift+M`
2. Runs Monte Carlo multiple times
3. Verifies results are stable (same probability when run again)
4. Takes longer but ensures reliable results

### Errors and Validation

If simulation fails, you'll see error messages. Common issues:

- **No accounts defined**: Add accounts in Portfolio tab
- **No events defined and no spending**: Add annual spending or events
- **Invalid dates**: Check birth date and start date format (YYYY-MM-DD)
- **Negative net worth**: Liabilities exceed assets

Fix the issue and try again. Errors are logged in a panel you can scroll.

---

## Tab 4: Results

**Access with:** `4` or `Shift+R`

Results show your projected wealth, account balances, and transaction details.

### Overview

Multiple panels:

1. **Net Worth Chart** (Top-Left) - Your wealth projection over time
2. **Account Breakdown** (Top-Right) - Breakdown by account type
3. **Ledger** (Bottom) - Year-by-year account activity
4. **Status Indicators** - Success rate, percentiles, display mode

### Net Worth Chart

**Keybindings:**
| Key | Action |
|-----|--------|
| `h/←` | Previous year |
| `l/→` | Next year |
| `Home` | First year |
| `End` | Last year |
| `v` | Cycle percentile view (P5/P50/P95/Mean) |
| `$` | Toggle real/nominal dollars |
| `g` | Toggle granularity (annual/monthly) |
| `r` | Re-run simulation |
| `m` | Run Monte Carlo |
| `f` | Filter results by account |

**Understanding the Chart:**

- **Bar Height** = Total net worth that year
- **Stacked Colors** = Breakdown by account type:
  - Green: Cash (Checking, Savings)
  - Blue: Tax-Deferred (401k, Traditional IRA)
  - Purple: Tax-Free (Roth)
  - Yellow: Taxable (Brokerage)
  - Red: Debt (negative)

**Navigation:**

- Use arrow keys to move between years
- Chart scrolls to keep selected year visible
- Current year is highlighted

**Display Modes:**

- **Nominal $**: Actual dollars (unadjusted)
- **Real $**: Adjusted for inflation (2024 dollars)

Press `$` to toggle between them.

**Percentile View (Monte Carlo Only):**

When in Monte Carlo mode, press `v` to cycle between:

- **P5**: 5th percentile (worst 5% of scenarios)
- **P50**: 50th percentile (median, middle outcome)
- **P95**: 95th percentile (best 5% of scenarios)
- **Mean**: Average across all scenarios

**Granularity:**

Press `g` to toggle between:

- **Annual**: One bar per year (default)
- **Monthly**: One bar per month (fine detail)

### Account Breakdown

Shows your money distributed across account types:

| Type | Color | Description |
|------|-------|-------------|
| Cash | Green | Checking, Savings, HSA |
| Tax-Deferred | Blue | 401k, Traditional IRA |
| Tax-Free | Purple | Roth IRA, Roth 401k |
| Taxable | Yellow | Brokerage accounts |
| Debt | Red | Mortgages, loans (negative) |

### Ledger Panel

Year-by-year breakdown of:
- **Starting Balance**: Net worth at start of year
- **Income**: Money in (salary, transfers)
- **Expenses**: Money out (living costs, taxes)
- **Taxes Paid**: Federal and state income taxes
- **Investment Returns**: Gains/losses from market
- **Ending Balance**: Net worth at end of year

Press `↓/↑` to scroll through years. In Monte Carlo mode, shows values for selected percentile.

### Viewing Modes

**Single Run vs Monte Carlo:**

- **Single Run** (after pressing `r`):
  - Shows one deterministic path
  - Uses median market returns
  - Helpful for first look

- **Monte Carlo** (after pressing `m`):
  - Shows probability distribution
  - Runs 1000+ simulations
  - Multiple percentile views

Press `v` to cycle percentiles in Monte Carlo mode.

### Key Metrics

**Displayed in status area:**

- **Success Rate**: % of Monte Carlo runs that didn't run out of money
- **Final Net Worth (P5/P50/P95)**:
  - P5: 5th percentile (conservative)
  - P50: Median (middle)
  - P95: 95th percentile (optimistic)
- **Max Drawdown**: Largest year-to-year decline

### Tips for Reading Results

1. **Look at P5 (Worst Case)**: Can you survive if markets crash?
2. **Review Ledger**: See where money goes (taxes, expenses, returns)
3. **Test Multiple Scenarios**: Try retire at 60, 62, 65 — compare
4. **Check Year of Depletion**: Does money run out? When?
5. **Understand Tax Impact**: How much goes to taxes vs spending?

---

## Tab 5: Analysis

**Access with:** `5` or `Shift+A`

Run sensitivity analysis to see how changes in parameters affect your outcomes.

### Overview

Two main panels:

1. **Parameters Panel** (Left) - Select which parameters to sweep
2. **Results Panel** (Right) - Charts showing sensitivity to parameter changes

### Parameters Panel

**Keybindings:**
| Key | Action |
|-----|--------|
| `a` | Add parameter to sweep |
| `d` | Delete parameter |
| `e` or `Enter` | Edit parameter settings |
| `r` | Run analysis |
| `s` | Analysis settings |
| `t` | Toggle metric |
| `j/k` | Navigate parameters |

**Adding Parameters:**

Press `a` to add a parameter to analyze. Options include:

- **Event trigger ages**: Test retiring at different ages
- **Effect amounts**: What if I spend 10% less? More?
- **Repeating event start/end**: When do recurring expenses begin/end?

For each parameter, configure:
- **Parameter Name**: What you're testing
- **Minimum Value**: Lowest value to test
- **Maximum Value**: Highest value to test
- **Step Size**: How many increments between min and max

**Example:**
```
Parameter: Retirement Age
Min: 60
Max: 70
Steps: 5 increments
Result: Test ages 60, 62, 64, 66, 68, 70
```

### Results Panel

Once you run analysis with `r`, the results panel shows charts.

**Keybindings (in Results panel):**
| Key | Action |
|-----|--------|
`h/l` | Navigate between charts |
| `c` or `Enter` | Configure selected chart |
| `+` | Add new chart |
| `-` | Delete chart |
| `t` | Toggle metric displayed |

**Chart Types:**

**1D Scatter Plot:**
- X-axis: One parameter (e.g., retirement age from 60-70)
- Y-axis: Outcome metric (e.g., success rate from 0-100%)
- Shows how one parameter affects outcome

**2D Heatmap:**
- X-axis: First parameter
- Y-axis: Second parameter  
- Color intensity: Outcome metric
- Shows interaction between two parameters

**Example 1D Chart:**
```
How does retirement age affect success rate?

Success Rate (%)
100% |     ╱╱╱╱
 75% |   ╱╱
 50% | ╱╱
 25%|╱
  0%└─────────────────
    60   65   70
      Retirement Age
```

**Example 2D Heatmap:**
```
Success Rate by Retirement Age × Annual Spending

Annual Spending ($)
  100k |  🟩🟩🟩 (80%)
   80k |  🟨🟨🟩 (60%)
   60k |  🟥🟨🟩 (40%)
   40k |  🟥🟥🟨 (20%)
       └─────────────
         60 65 70
      Retirement Age
```

**Available Metrics:**

| Metric | Meaning |
|--------|---------|
| Success Rate | % of runs that don't run out of money |
| P50 Final Net Worth | Median ending wealth |
| P5 Final Net Worth | Conservative ending wealth |
| P95 Final Net Worth | Optimistic ending wealth |
| Lifetime Taxes | Total taxes paid over simulation |
| Max Drawdown | Largest year-to-year decline |

### Configuring Charts

Press `c` or `Enter` on a chart to configure it:

1. **Chart Type**: 1D Scatter or 2D Heatmap
2. **X Parameter**: What goes on X-axis
3. **Y Parameter**: (2D only) What goes on Y-axis
4. **Metric**: What to display (success rate, wealth, etc.)
5. **Fixed Values**: For parameters not shown

For example, if you have 3 parameters but want a 2D chart of only 2:
- You select which 2 appear on axes
- You set fixed values for the third (usually the midpoint)

### Workflow: Finding Optimal Retirement Age

1. Go to **Events** tab, find your retirement event
2. Go to **Analysis** tab
3. Press `a` to add the event's trigger age as a parameter
4. Set Min: 60, Max: 70
5. Press `r` to run
6. View the chart showing success rate vs age
7. Find the age where success rate reaches your comfort level (e.g., 90%)

---

## Tab Navigation Summary

| Tab | Key | Purpose | Main Actions |
|-----|-----|---------|--------------|
| 1. Portfolio & Profiles | `1` | Define accounts, assets, tax settings | `a` add, `e` edit, `m` map returns |
| 2. Events | `2` | Create income, expenses, transactions | `a` add, `e` edit, `f` effects |
| 3. Scenario | `3` | Manage plans, run simulations | `r` run, `m` monte carlo, `l` load |
| 4. Results | `4` | View projections, ledger, outcomes | `h/l` navigate years, `v` percentile |
| 5. Analysis | `5` | Test sensitivity, find optimal values | `a` add param, `r` run, `c` config |

---

**Next:** Read [Managing Scenarios](03-scenarios.md) to learn import/export and backups.
