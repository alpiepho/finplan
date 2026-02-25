# Managing Scenarios

Scenarios are the core of FinPlan. They contain all your financial data: accounts, events, parameters, and settings. This guide shows you how to create, import, export, and manage them.

## Scenario Basics

A **scenario** is a complete financial plan containing:
- All your accounts and assets
- All your life events (income, expenses, etc.)
- Simulation parameters (dates, duration, spending)
- Tax and inflation settings
- Return profile mappings

Think of scenarios like projects or "what-if" plans.

## Creating Scenarios

### New Blank Scenario

1. Go to **Scenario** tab (press `3`)
2. Press `n` (new)
3. Enter a name (e.g., "Retirement Plan 2025")
4. A blank scenario is created
5. You'll need to populate it:
   - Go to **Portfolio & Profiles** (`1`) to add accounts
   - Go to **Events** (`2`) to add income/expenses
   - Return to **Scenario** (`3`) to configure parameters
   - Press `r` to test your setup

### Copy Existing Scenario

A faster way to create variations:

1. Select the scenario in the list
2. Press `c` (copy)
3. Enter a new name for the copy (e.g., "Retirement - Conservative")
4. You now have an identical copy to modify

**Workflow Example:**
```
Base scenario: "Retirement - Baseline"
  ↓ copy
"Retirement - Conservative" (lower returns, higher expenses)
"Retirement - Optimistic" (higher returns, lower expenses)
```

Then modify each copy and compare results.

## Managing Your Scenario List

### Switching Scenarios

1. Go to **Scenario** tab
2. Use `j/k` or arrows to navigate the list
3. Press `Enter` to switch to that scenario

The selected scenario becomes active. Your accounts, events, and parameters switch to that scenario.

### Saving and Naming

Your current scenario auto-saves when you make changes. To explicitly save with a new name:

1. In **Scenario** tab, press `s` (save as)
2. Enter new name
3. Your current data is saved under the new name

**Note:** After you press `s`, you're now working on the newly-named scenario.

### Editing Scenario Name

To rename an existing scenario:

1. Select it in the list
2. Press `e` (edit name)
3. Enter new name
4. Confirm with `Enter`

### Deleting Scenarios

To delete a scenario:

1. Select it in the list
2. Press `d` (delete)
3. Confirm the deletion
4. Scenario is permanently removed

**Warning:** This is permanent. Consider exporting first if you might need it later.

### Reordering (Alphabetical)

Scenarios are sorted alphabetically by name. To control order:
- Rename: "01-Baseline", "02-Conservative", "03-Optimistic"
- Or use descriptive names that sort naturally

## Importing Scenarios

Import scenarios from YAML files to:
- Load examples
- Share scenarios with others
- Restore backups
- Use templates

### How to Import

1. Go to **Scenario** tab (press `3`)
2. Press `i` (import)
3. Enter the path to the YAML file:
   - **Absolute path**: `/Users/name/Desktop/plan.yaml`
   - **Relative path**: `examples/example.yaml` (from app directory)
4. Enter a name for the scenario
5. It appears in your scenario list

### File Paths

**Absolute paths** (start from your home folder):
```
/Users/john/Desktop/plan.yaml
/Users/john/Documents/retirement/baseline.yaml
```

**Relative paths** (from application directory):
```
examples/example.yaml
../scenarios/archive/2024.yaml
./backup/last_known_good.yaml
```

**On macOS/Linux:**
- Use `/` for paths
- `~` expands to your home folder

**On Windows:**
- Use `C:\Users\name\...` or `C:/Users/name/...`

### Bundled Example

FinPlan includes an example scenario:

1. Press `i` to import
2. Enter: `examples/example.yaml`
3. Name it: "Example"
4. It loads with realistic accounts and events

This is great for learning or as a template.

## Exporting Scenarios

Export scenarios to:
- Back them up
- Share with others
- Edit manually
- Use as templates

### How to Export

1. Go to **Scenario** tab
2. Select the scenario you want to export
3. Press `x` (export)
4. Choose where to save:
   - Desktop: `~/Desktop/plan.yaml`
   - Dropbox: `~/Dropbox/FinPlan/`
   - Backup folder: `~/FinPlan_Backups/`
   - Custom location: Any path you enter

### File Naming

Use clear, descriptive names:

**Good names:**
- `Retirement_2025_v1.yaml`
- `Baseline_Age62_Retire.yaml`
- `Conservative_Scenario.yaml`
- `2024-12-01_backup.yaml`

**Less clear:**
- `plan.yaml` (which plan?)
- `scenario.yaml` (which one?)

### Backup Strategy

Recommended backup workflow:

1. **Monthly Backups:**
   - Export current scenario with timestamp
   - Save to a backup folder: `~/FinPlan_Backups/`
   - Name: `main_scenario_2025-01.yaml`

2. **Before Major Changes:**
   - Export current working version
   - Name: `before_major_change_20250115.yaml`
   - Safe to experiment after

3. **Cloud Backup:**
   - Export to Dropbox/iCloud/OneDrive
   - `~/Dropbox/FinPlan/`
   - Survives computer crashes

## Sharing Scenarios

To share a scenario with someone:

1. Export it with `x`
2. Save to a file
3. Share the `.yaml` file via email, Dropbox, etc.
4. They import with `i`

**Sharing Tips:**
- Exporting removes sensitive details if you wish
- Recipient sees your exact plan (accounts, events, assumptions)
- They can copy and modify without affecting original
- Great for financial advisor collaboration

## Scenario File Format

FinPlan stores scenarios as **YAML**, a human-readable text format. You can edit them manually if needed.

### Typical Scenario Structure

```yaml
# Basic parameters
parameters:
  birth_date: 1965-03-15
  start_date: 2025-01-01
  duration_years: 40
  annual_spending: 75000

# Your accounts
portfolios:
  accounts:
    - name: Checking
      account_type: Checking
      balance: 50000
    - name: Roth IRA
      account_type: RothIRA
      balance: 150000
      assets:
        - ticker: VTSAX
          shares: 1000
          price: 150

# Tax settings
tax_config:
  brackets: preset_2024_single
  state_tax_rate: 0.06

# Life events
events:
  - name: Annual Expenses
    trigger: Repeating(yearly, age 45)
    effects:
      - type: Expense
        amount: 75000
```

### Editing Manually

You can edit YAML directly if comfortable:

1. Export scenario to a file
2. Open with text editor (VS Code, Notepad, etc.)
3. Modify values carefully
4. Save the file
5. Import it back into FinPlan

**Common edits:**
- Change dates and amounts
- Add/remove accounts
- Modify event triggers

**Caution:** YAML is sensitive to indentation. If you get import errors:
- Check indentation (use spaces, not tabs)
- Ensure all quotes and braces match
- Use a YAML validator online

## Organizing Scenarios

### Naming Convention

Use clear, systematic names:

**By retirement age:**
```
Retire_60
Retire_62
Retire_65
```

**By assumptions:**
```
Conservative_Returns_High_Spending
Baseline_Historical_Returns
Optimistic_Returns_Low_Spending
```

**By date:**
```
2025_Q1_Plan
2025_Q2_Update
2025_Annual_Review
```

**By variation:**
```
Main_Scenario
Main_Scenario_No_SocialSecurity
Main_Scenario_Inheritance_at_70
```

### Folder-Like Organization

Create prefixes to group related scenarios:

```
Base_Main_Plan
Base_Conservative
Base_Optimistic

Alt_Earlier_Retirement_55
Alt_Extended_Work_70

Test_HighInflation
Test_StockMarketCrash
Test_ExpenseIncrease
```

This helps when you have many scenarios.

## Working with Multiple Scenarios

### Comparison Workflow

1. **Create variations:**
   - Copy main scenario 3 times
   - Name: "Retire_60", "Retire_62", "Retire_65"

2. **Adjust each:**
   - Go to Scenario tab
   - Switch to "Retire_60"
   - Press `e` to edit parameters
   - Change retirement age to 60
   - Press `m` to run Monte Carlo
   - View results

3. **Compare outcomes:**
   - Note the success rates
   - Compare final net worth
   - See which age best matches your goals

### Testing Changes

Before making a major change:

1. Export current scenario (backup)
2. Make the change
3. Test with `r` or `m`
4. If bad, import the backup
5. If good, keep it

## Data Storage Location

All scenarios are stored in `~/.finplan/scenarios/`:

```
~/.finplan/
├── config.yaml                 # Current active scenario name
├── summaries.yaml              # Cached Monte Carlo summaries
├── keybindings.yaml            # Custom keybindings (optional)
└── scenarios/
    ├── Retirement.yaml
    ├── Conservative.yaml
    ├── Optimistic.yaml
    └── Archive/
        └── 2024_backup.yaml
```

**On macOS/Linux:**
- `~/.finplan/` = `/Users/yourname/.finplan/`

**On Windows:**
- `~/.finplan/` = `C:\Users\yourname\.finplan\`

You can browse and edit these folders directly using your file explorer.

### Backup Entire Directory

To back up all scenarios at once:

**macOS/Linux:**
```bash
cp -r ~/.finplan ~/FinPlan_Backup_2025-01-15
tar czf ~/FinPlan_Backup_2025-01-15.tar.gz ~/.finplan/
```

**Windows (PowerShell):**
```powershell
Copy-Item -Recurse -Path "$HOME\.finplan" -Destination "$HOME\FinPlan_Backup_2025-01-15"
```

## Common Workflows

### Scenario A/B Testing

```
1. Create baseline scenario
2. Export it as backup
3. Copy it as "Test_Version"
4. Make changes to test version
5. Run both with Monte Carlo
6. Compare success rates
7. Keep winner, delete loser
```

### Building from Example

```
1. Import examples/example.yaml
2. Copy it with name "My Plan"
3. Edit accounts to match yours
4. Edit events to match your life
5. Adjust tax settings
6. Run simulation
7. Iterate until results match expectations
```

### Archiving Old Plans

```
1. Create "Archive" folder: mkdir ~/FinPlan_Archive
2. Export old scenarios to archive folder
3. Delete from FinPlan (in app) to reduce clutter
4. Can re-import from archive later if needed
```

---

**Next:** Read [Running Simulations](04-simulations.md) to learn about single runs vs Monte Carlo.
