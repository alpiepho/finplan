# Path A: Build Your First Scenario in the TUI

This tutorial walks you from a blank app to a running Monte Carlo simulation. It takes about 15 minutes.

You'll build a simple retirement scenario: a person with a checking account, a 401k, and a Roth IRA who is working, saving, and plans to retire.

## Navigation Basics

Before you start, the key you'll use most:

| Key | Action |
|-----|--------|
| `1`–`5` | Switch tabs |
| `j` / `k` or `↓` / `↑` | Move up/down in a list |
| `Tab` / `Shift+Tab` | Move between panels within a tab |
| `a` | Add a new item |
| `e` | Edit selected item |
| `d` | Delete selected item |
| `Enter` | Confirm / open |
| `Esc` | Cancel / close |
| `Ctrl+S` | Save |

The **status bar** at the bottom always shows available commands for the current panel.

---

## Step 1 — Create a New Scenario

1. Press `3` to go to the **Scenario** tab.
2. Press `n` (new).
3. Type a name — e.g., `My Retirement Plan` — and press `Enter`.

You now have a blank scenario. The right panel shows empty simulation parameters.

---

## Step 2 — Add Accounts

Press `1` to go to the **Portfolio & Profiles** tab. The left panel shows your accounts list (currently empty).

Add a **Checking** account:
1. Press `a` (add).
2. Choose account type: `Checking`.
3. Name it `Checking`.
4. Enter a starting balance — e.g., `25000`.
5. Press `Enter` or `Ctrl+S` to save.

Add a **401k** account:
1. Press `a` again.
2. Choose type: `Traditional401k`.
3. Name it `401k`.
4. Leave balance at `0` (assets will hold the value).
5. Save.

Add a **Roth IRA** account:
1. Press `a`.
2. Choose type: `RothIRA`.
3. Name it `Roth IRA`.
4. Leave balance at `0`.
5. Save.

Your accounts panel now shows three accounts.

---

## Step 3 — Add Assets to Investment Accounts

Investment accounts (401k, Roth IRA, Brokerage) hold assets rather than a raw cash balance.

Select the `401k` account and press `e` to edit it. Inside the account editor:
1. Add an asset — e.g., ticker `FXAIX`.
2. Enter the current value — e.g., `85000`.
3. Save.

Select the `Roth IRA` account and press `e`:
1. Add asset `FXAIX`, value `45000`.
2. Save.

---

## Step 4 — Map Assets to Return Profiles

FinPlan needs to know how each asset is expected to perform. This is done through **asset mappings**.

Press `Tab` until the **Asset Mappings** panel is active (middle-right area).

For each unmapped asset (marked with `?`):
1. Select it and press `m` (map).
2. Choose a return profile from the list — e.g., `S&P 500` for a broad index fund.
3. Press `Enter` to confirm.

To auto-suggest mappings for all unmapped assets at once, press `Shift+A`.

---

## Step 5 — Configure Tax Settings

Press `Tab` to reach the **Tax & Inflation Config** panel (bottom area).

Press `e` to edit:
- **Federal brackets**: Choose your filing status — e.g., `single2024`.
- **State rate**: Enter your state income tax rate as a decimal — e.g., `0.05` for 5%.
- **Capital gains rate**: Enter `0.15` for the standard long-term rate.
- **Inflation**: Leave as `USHistorical` for realistic inflation modeling.

Save with `Ctrl+S`.

---

## Step 6 — Add Life Events

Press `2` to go to the **Events** tab. This is where you model things that happen over time: salary, expenses, retirement, Social Security.

### Add a Salary Event

1. Press `a` to add a new event.
2. Name it `Salary`.
3. Set the trigger:
   - Type: `Repeating`
   - Interval: `biweekly`
   - End: relative to a `Retirement` event (you'll add that next), offset 0 months.
4. Save the trigger.
5. Press `f` to configure effects.
6. Add an **Income** effect:
   - To account: `Checking`
   - Amount type: `InflationAdjusted`, base value `4500` (per paycheck)
   - `gross: true`, `taxable: true`
7. Save.

### Add a Retirement Trigger Event

1. Press `a`.
2. Name it `Retirement`.
3. Trigger type: `Age`, years: `62`.
4. Enable `once: true` — this event fires once and marks retirement.
5. No effects needed — it's a marker event other events reference.
6. Save.

### Add Living Expenses

1. Press `a`.
2. Name it `Living Expenses`.
3. Trigger: `Repeating`, interval `monthly` (no end date — expenses continue forever).
4. Press `f` for effects.
5. Add an **Expense** effect:
   - From: `Checking`
   - Amount: `InflationAdjusted`, base value `5500` per month
6. Save.

### Add a Retirement Sweep

After retirement, you need income from your investments. Add a sweep that tops up your checking account each year.

1. Press `a`.
2. Name it `Retirement Withdrawal`.
3. Trigger: `Repeating`, interval `yearly`, start: relative to `Retirement` event, offset 0 months.
4. Press `f` for effects.
5. Add a **Sweep** effect:
   - To: `Checking`
   - Amount: `TargetToBalance`, target `100000`
   - Strategy: `penalty_aware`
   - `gross: false`, `taxable: true`, lot method: `fifo`
6. Save.

### Add Social Security

1. Press `a`.
2. Name it `Social Security`.
3. Trigger: `Repeating`, interval `monthly`, start: `Age` 67.
4. Press `f` for effects.
5. Add an **Income** effect:
   - To: `Checking`
   - Amount: `Fixed`, value `2200`
   - `gross: true`, `taxable: true`
6. Save.

### Add RMD (Required Minimum Distributions)

Required by law starting at age 73 for tax-deferred accounts.

1. Press `a`.
2. Name it `RMD`.
3. Trigger: `Repeating`, interval `yearly`, start: `Age` 73.
4. Press `f` for effects.
5. Add an **ApplyRmd** effect:
   - Destination: `Checking`
   - Lot method: `fifo`
6. Save.

---

## Step 7 — Set Simulation Parameters

Press `3` to go to the **Scenario** tab. Press `e` to edit parameters:

- **Birth date**: Your birth date in `YYYY-MM-DD` format — e.g., `1975-06-15`.
- **Start date**: When the simulation begins — e.g., `2025-01-01`.
- **Duration**: How many years to simulate — e.g., `40`.
- **Returns mode**: `historical` (uses actual historical market data) or `profile` (uses your custom return profiles).

Save with `Ctrl+S`.

---

## Step 8 — Run a Single Simulation

Still in the **Scenario** tab, press `r` to run a single deterministic simulation.

This takes about 1 second. It uses median market returns — no randomness, just a quick sanity check.

Press `4` to go to the **Results** tab. You'll see:
- A bar chart of your projected net worth over time
- An account breakdown panel
- A year-by-year ledger at the bottom

Navigate years with `h`/`l` or arrow keys. If the chart shows your money running out, review your expenses and events.

---

## Step 9 — Run Monte Carlo

Once the single run looks reasonable, press `3` to go back to **Scenario** and press `m` to run Monte Carlo (1000+ simulations with randomized market returns).

This takes 30–60 seconds. When done, press `4` to see:
- **Success rate**: percentage of simulations where you didn't run out of money
- **P5/P50/P95**: worst-case, median, and best-case outcomes
- Press `v` to cycle between percentile views

A success rate above 90% is a good target for most retirement plans.

---

## What's Next

- See [Understanding Results](05-results-interpretation.md) to learn what the charts mean
- See [Running Simulations](04-simulations.md) for details on Monte Carlo and convergence testing
- See [Managing Scenarios](03-scenarios.md) to export, copy, and organize your scenarios
- See [Scenario YAML Reference](08-scenario-yaml-reference.md) for the complete list of account types, event triggers, and effects
