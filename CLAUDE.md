# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# FinPlan - Monte Carlo Retirement Simulation

## Quick Commands

**IMPORTANT: No local Rust toolchain. ALL cargo commands must run via Docker. Use `$PWD`, not `$(pwd)`.**

```bash
docker run --rm -v "$PWD":/app -w /app rust:slim cargo build
docker run --rm -v "$PWD":/app -w /app rust:slim cargo test
docker run --rm -v "$PWD":/app -w /app rust:slim sh -c "rustup component add rustfmt 2>/dev/null; cargo fmt"
docker run --rm -v "$PWD":/app -w /app rust:slim sh -c "rustup component add clippy 2>/dev/null; cargo clippy"
cargo run --bin finplan  # TUI runs locally (has local binary)
```

IMPORTANT:
- When finished making changes run cargo fmt via Docker (see above)
- Run cargo clippy via Docker and fix any warnings if they will not cause major refactor work.
- `git add` changed files to track
- Suggest a commit message for the completed work

## Project Structure

```
finplan/
├── crates/
│   ├── finplan_core/   # Simulation engine library (~2500 LOC)
│   └── finplan/        # Terminal UI application (~2600 LOC)
├── spec/               # Detailed specifications
└── web/                # Next.js frontend (not actively developed)
```

## Key Entry Points

| Task | Location |
|------|----------|
| Run simulation | `crates/finplan_core/src/simulation.rs` - `simulate()` |
| Monte Carlo | `crates/finplan_core/src/simulation.rs` - `monte_carlo_simulate_with_config()` |
| TUI entry | `crates/finplan/src/main.rs` |
| App event loop | `crates/finplan/src/app.rs` - `App::run()` |

## finplan_core Navigation

### Core Simulation
- `simulation.rs` - Main loop, Monte Carlo orchestration
- `simulation_state.rs` - Runtime state (accounts, timeline, taxes)
- `apply.rs` - Execute event effects
- `evaluate.rs` - Evaluate triggers and transfer amounts
- `liquidation.rs` - Asset sale with tax handling
- `taxes.rs` - Progressive tax calculations
- `metrics.rs` - Simulation metrics computation

### Data Model (`model/`)
- `accounts.rs` - Account, TaxStatus, AssetLot, InvestmentContainer
- `events.rs` - Event, EventTrigger, EventEffect, TransferAmount
- `market.rs` - ReturnProfile, InflationProfile
- `results.rs` - SimulationResult, MonteCarloSummary
- `records.rs` - Ledger records and transaction history
- `rmd.rs` - Required Minimum Distribution modeling
- `state_event.rs` - State change event types
- `tax_config.rs` - Tax bracket configuration
- `ids.rs` - AccountId, AssetId, EventId, ReturnProfileId

### Builder DSL (`config/`)
- `builder.rs` - SimulationBuilder fluent API
- `account_builder.rs` - Preset accounts (Checking, 401k, Roth, etc.)
- `asset_builder.rs` - Asset definitions
- `event_builder.rs` - Event construction helpers

### Analysis (`analysis/`)
- Parameter sweep / sensitivity analysis across N-dimensional grids
- `SweepConfig`, `SweepParameter`, `AnalysisMetric`, `sweep_evaluate()`, `sweep_simulate()`
- Two-phase: run simulations once, compute different metrics repeatedly

### Optimization (`optimization/`)
- Find optimal parameter values (retirement age, contribution rates, withdrawal amounts)
- Algorithms: `binary_search` (1 param), `grid_search` (2-3 params), `nelder_mead` (4+ params)
- Entry point: `optimize()` in `mod.rs` — auto-selects algorithm based on param count

## finplan (TUI) Navigation

### Screens (`screens/`)
- `portfolio_profiles.rs` - Accounts and return profiles
- `scenario.rs` - Simulation parameters, tax config
- `events.rs` - Life event management
- `results.rs` - Monte Carlo results display
- `analysis.rs` - Parameter sweep / sensitivity analysis screen

### State (`state/`)
- `app_state.rs` - Root application state
- `screen_state.rs` - Per-screen state
- `tabs.rs` - Tab management
- `panels.rs` - Panel focus tracking
- `cache.rs` - Cached simulation results
- `errors.rs` - Error state

### Modals (`modals/`)
- `state.rs` - ModalState enum (Form, Picker, TextInput, Confirm, etc.)
- `action.rs` - ModalAction dispatch enum
- `handler.rs` - Key event routing to active modal
- `form.rs`, `confirm.rs`, `picker.rs`, `text_input.rs`, `message.rs` - Modal UI renderers
- `amount_builder.rs` - Multi-step amount editor
- `context.rs` - Modal context passed to action handlers

### Actions (`actions/`)
- `scenario.rs` - New, Load, Save, Duplicate, Delete
- `account.rs` - Account CRUD
- `event.rs` - Event configuration
- `effect.rs` - Event effect management
- `analysis.rs` - Analysis configuration

### Components (`components/`)
- `charts/` - Distribution and sweep result charts
- `lists/` - Selectable list widget
- `panels/` - Accounts, events, ledger, profiles panels
- `portfolio_overview.rs`, `status_bar.rs`, `tab_bar.rs`

### Data (`data/`)
- `storage.rs` - File persistence (`~/.finplan/scenarios/`)
- `app_data.rs` - In-memory data structures
- `convert.rs` - Core↔TUI data conversion
- `keybindings_data.rs` - Keybinding loading/parsing

### Other
- `worker.rs` - Background thread for running simulations without blocking the UI
- `keybindings.rs` - Custom keybinding support (`~/.finplan/keybindings.yaml`)

## Common Tasks

### Adding a new EventEffect
1. Add variant to `crates/finplan_core/src/model/events.rs` - `EventEffect` enum
2. Implement in `crates/finplan_core/src/apply.rs` - `apply_effect()`
3. Add evaluation in `crates/finplan_core/src/evaluate.rs` if needed
4. Add TUI support in `crates/finplan/src/actions/effect.rs`

### Adding a new EventTrigger
1. Add variant to `crates/finplan_core/src/model/events.rs` - `EventTrigger` enum
2. Implement evaluation in `crates/finplan_core/src/evaluate.rs` - `evaluate_trigger()`
3. Add TUI support in `crates/finplan/src/actions/event.rs`

### Adding a new Account type
1. Modify `crates/finplan_core/src/model/accounts.rs` - `AccountFlavor` enum
2. Update `Account::total_value()` and `Account::snapshot()`
3. Add builder preset in `crates/finplan_core/src/config/account_builder.rs`
4. Add TUI support in `crates/finplan/src/actions/account.rs`

## Testing

```bash
cargo test -p finplan_core                      # Core library tests
cargo test -p finplan_core -- basic             # Run tests matching "basic"
cargo test -p finplan_core -- --nocapture       # Show println output
```

Key test files in `crates/finplan_core/src/tests/`:
- `basic.rs` - Basic simulation tests
- `builder_dsl.rs` - Builder API tests
- `accounts.rs` - Account operation tests
- `contribution_limits.rs` - 401k/IRA contribution limit tests
- `returns.rs` - Return profile distribution tests
- `rsu.rs` - RSU vesting tests
- `simulation_result.rs` - Result structure tests

### MCP Server Tests

The MCP crate has no local Rust toolchain — all cargo commands run via Docker:

```bash
# Run all MCP tests (quiet)
docker run --rm -v "$(pwd)":/app -w /app rust:slim cargo test -p finplan_mcp

# Run with verbose step-by-step output (shows tool responses + merged YAML)
docker run --rm -v "$(pwd)":/app -w /app rust:slim cargo test -p finplan_mcp -- --nocapture

# Run a single test by name
docker run --rm -v "$(pwd)":/app -w /app rust:slim cargo test -p finplan_mcp -- test_full_scenario_build --nocapture
```

Test file: `crates/finplan_mcp/tests/integration_test.rs`
- `test_full_scenario_build` — full 15-step workflow: portfolio → events → tickers → validate → YAML
- `test_tool_list_contains_expected_tools` — all 16 tools registered
- `test_all_resources_readable_and_non_empty` — all 11 schema resources return content
- `test_state_summary_empty / test_reset_clears_state` — state management
- `test_merge_fails_without_portfolio / test_duplicate_account_rejected / test_income_event_requires_name_field` — error handling

## Specifications

Detailed documentation in `spec/`:
- `00_project_overview.md` - Architecture overview
- `01_core_architecture.md` - Engine module organization
- `02_data_model.md` - Core data structures
- `03_simulation_engine.md` - Simulation mechanics
- `04_tui_application.md` - TUI architecture
- `05_future_roadmap.md` - Planned features (optimization, what-ifs, estate planning)

## Code Style

- Run `cargo fmt` before commits (REQUIRED)
- Use type-safe IDs (AccountId, EventId, etc.)
- Record all state changes to the immutable ledger
- Prefer builder pattern for complex configurations
