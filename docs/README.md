# FinPlan User Manual

Welcome to FinPlan, a Monte Carlo retirement planning simulator. This manual will guide you through using the application to model your financial future.

## What is FinPlan?

FinPlan runs thousands of simulations with varying market conditions to answer critical retirement questions:

- **What's the probability my savings will last through retirement?**
- **How does retiring at 62 vs 65 affect my outcomes?**
- **What's the optimal withdrawal strategy given my account mix?**
- **How do different market conditions affect my plan?**

Unlike simple retirement calculators, FinPlan models:
- Tax-aware withdrawals with progressive tax brackets
- RSU vesting and stock compensation events
- Multiple account types (taxable, tax-deferred, tax-free)
- Realistic asset liquidation with tax-loss harvesting considerations
- Complex event triggers based on dates, ages, or account balances

## Quick Start

### Installation

**Option 1: Docker (recommended, no Rust required)**
```bash
docker compose run --rm finplan
```
The first run builds the image (~5–10 min). Subsequent starts are instant. Your data persists in `~/.finplan/` on your machine.

**Option 2: From Source**
Requires [Rust](https://rustup.rs/) (stable).
```bash
cargo install --path crates/finplan
# or
cargo run --bin finplan --release
```

### Your First Scenario

1. Launch FinPlan: `docker compose run --rm finplan` or `cargo run --bin finplan`
2. Go to **Scenario** tab (press `3`)
3. Press `i` to import `examples/example.yaml` to see a real example
4. Return to **Scenario** tab and press `r` to run a single simulation
5. Press `4` to see **Results** showing your projected net worth
6. Press `5` to visit **Analysis** for sensitivity testing

### Validating Scenario Files

Before importing a scenario, you can validate it from the command line:

```bash
# Validate a scenario file
finplan --scenario examples/example.yaml --validate

# Shows: ✓ Scenario is valid!
# Or detailed error messages if there are issues
```

See [Managing Scenarios](03-scenarios.md#validating-scenarios) for more details.

## Documentation Structure

- [**Getting Started**](01-getting-started.md) - Basic concepts and navigation
- [**Tab Reference**](02-tabs-guide.md) - Detailed guide to all 5 tabs
  - Portfolio & Profiles Tab
  - Events Tab
  - Scenario Tab
  - Results Tab
  - Analysis Tab
- [**Managing Scenarios**](03-scenarios.md) - Import, export, and manage your plans
- [**Running Simulations**](04-simulations.md) - Single runs, Monte Carlo, and convergence
- [**Understanding Results**](05-results-interpretation.md) - How to read charts and metrics
- [**Keybindings Reference**](06-keybindings.md) - Complete keyboard shortcut guide
- [**Advanced Tips**](07-advanced-tips.md) - Power user techniques
- [**Scenario YAML Reference**](08-scenario-yaml-reference.md) - Complete parameter guide for scenario files

## Key Concepts

### Account Types

FinPlan supports multiple account types with different tax treatments:

| Type | Examples | Tax Treatment |
|------|----------|---------------|
| **Taxable** | Brokerage accounts | Capital gains taxed each year |
| **Tax-Deferred** | 401k, Traditional IRA | Withdrawals taxed as income |
| **Tax-Free** | Roth IRA, Roth 401k | Qualified withdrawals are tax-free |
| **Illiquid** | Real estate, vehicles | Not used for spending |
| **Debt** | Mortgages, loans | Reduces net worth |

### Asset Classes

- **Investable**: Stocks, bonds, mutual funds, ETFs
- **Real Estate**: Properties (included in net worth but not liquidated)
- **Depreciating Assets**: Vehicles, collectibles
- **Liabilities**: Mortgages, personal loans, student loans

### Events

Events are the primary way to model life changes:

- **Income**: Salary, bonuses, RSU vesting, Social Security
- **Expenses**: Living costs, medical, gifts
- **Transactions**: Asset purchases, account transfers
- **Triggers**: Can be age-based, date-based, or balance-based
- **Recurring**: Can repeat yearly, monthly, or on specific dates

### Simulation Types

- **Single Run**: One deterministic simulation with median returns
- **Monte Carlo**: 1000+ simulations with randomized market returns
- **Convergence Test**: Runs Monte Carlo multiple times to verify result stability

## Getting Help

- Press `?` for context-sensitive help in any tab (when available)
- Check the status bar at the bottom for command hints
- Review the keybindings reference in this manual
- All data is stored locally in `~/.finplan/` — you can edit YAML files directly if needed

## Data Storage

Your scenarios are stored in YAML format at `~/.finplan/scenarios/`:

```
~/.finplan/
├── config.yaml              # Current scenario name
├── summaries.yaml           # Cached Monte Carlo summaries
├── keybindings.yaml         # Your custom keybindings (optional)
└── scenarios/
    ├── retirement.yaml
    ├── aggressive.yaml
    └── conservative.yaml
```

You can manually edit scenario files or use the TUI interface.

---

**Ready to get started?** Begin with [Getting Started](01-getting-started.md) or jump to the [Tab Reference](02-tabs-guide.md).
