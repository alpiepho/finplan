# MCP Server — AI-Assisted Scenario Building

The FinPlan MCP server (`finplan-mcp`) lets an AI agent build a complete scenario YAML file through a series of tool calls. Instead of filling out the TUI manually, you describe your financial situation in conversation and the AI constructs the file for you.

## What It Does

The server exposes FinPlan's scenario construction as MCP tools, resources, and prompts:

- **Resources** — schema documentation the AI reads to understand the YAML format
- **Tools** — structured calls to add accounts, events, and parameters step by step
- **Prompts** — guided interview templates (`build_scenario`, `quick_retirement`)

A typical session: you describe your accounts, income, and retirement goals; the AI calls the tools in the right order; you get back a validated YAML file ready to load into FinPlan.

---

## Connecting to Claude Desktop

Add `finplan-mcp` to your `claude_desktop_config.json`. On macOS this file is at
`~/Library/Application Support/Claude/claude_desktop_config.json`.

**Option 1 — Docker (no Rust required, recommended):**

```json
{
  "mcpServers": {
    "finplan": {
      "command": "docker",
      "args": ["compose", "run", "--rm", "finplan-mcp"],
      "cwd": "/path/to/finplan"
    }
  }
}
```

**Option 2 — from source (requires Rust):**

```json
{
  "mcpServers": {
    "finplan": {
      "command": "cargo",
      "args": ["run", "--bin", "finplan-mcp"],
      "cwd": "/path/to/finplan"
    }
  }
}
```

Replace `/path/to/finplan` with the absolute path to this repository. Restart Claude Desktop after editing the config — the server starts automatically on the first message that uses it.

---

## How to Invoke the Server

There are three styles of prompts depending on how much you want to type upfront.

### 1. Guided Interview (recommended for first-timers)

Ask Claude to walk you through it step by step:

```
I want to build a FinPlan retirement scenario. Use the finplan MCP server
and its build_scenario prompt to interview me and create the file.
```

Claude will ask questions one section at a time — accounts, income, expenses,
retirement age — and call the tools as you answer. You don't need to know
the YAML format at all.

### 2. Quick Start (minimal input)

If you just want a reasonable starting point fast:

```
Use the finplan MCP server's quick_retirement prompt to build a basic
scenario for me. My birth date is 1985-06-15, I earn $130,000/year gross,
and my monthly expenses are around $7,000.
```

Claude fills in sensible defaults for everything you don't specify.

### 3. Upfront Detail (fastest, most accurate)

If you have your numbers ready, give Claude everything at once:

```
Use the finplan MCP server to build a retirement scenario with these details:

Name: Sarah's Retirement Plan
Birth date: 1985-06-15
State: Texas (no income tax)
Filing status: single

Accounts:
- Chase Checking: $15,000 cash
- Fidelity Brokerage: $230,000 (VTI $150k, VXUS $50k, BND $30k)
- Company 401k: $200,000 (FXAIX)
- Roth IRA: $75,000 (VOO)

Income: $5,769.23 biweekly salary, ends at retirement
Expenses: $6,000/month living expenses

Retirement: age 60, $80,000/year spending target
Social Security: $2,800/month starting age 67
401k contribution: $23,500/year into FXAIX, ends at retirement

Save the result as sarah_plan.yaml
```

---

## Example Session

Here is a realistic example of the guided interview style from start to finish.

---

**You type:**

```
I want to build a FinPlan retirement scenario. Use the finplan MCP server
and the build_scenario prompt to interview me and create the file.
```

---

**Claude responds:**

> I'll use the FinPlan MCP server to build your retirement scenario. Let me start
> by reading the schema so I understand the format.
>
> *(reads `schema://full`, `schema://accounts`, `schema://patterns`)*
>
> Great, I'm ready. Let's go section by section.
>
> **Step 1 — Basic Info**
> What's your date of birth? (YYYY-MM-DD)
> What state do you live in?
> Are you filing taxes as single or married filing jointly?

---

**You type:**

```
Born 1985-06-15, I live in Texas, filing single.
```

---

**Claude responds:**

> *(calls `set_parameters` with birth_date: "1985-06-15", state_tax_rate: 0.0, federal_brackets: "single2024")*
>
> Parameters set. Texas has no state income tax, so I've set the state rate to 0%.
> I'll default to 40 years of simulation (to age 100) and historical returns.
>
> **Step 2 — Accounts**
> Tell me about your accounts. For each one I need:
> - Account name and type (checking, savings, 401k, Roth IRA, brokerage, etc.)
> - Current balance or holdings
>
> List as many as you have.

---

**You type:**

```
Chase Checking: about $15,000
Fidelity Brokerage: VTI $150k, VXUS $50k, BND $30k
Company 401k: FXAIX $200,000
Roth IRA: VOO $75,000
```

---

**Claude responds:**

> *(calls `set_portfolio` with name: "My Retirement Plan")*
> *(calls `add_account` × 4 — Checking, Brokerage, 401k, Roth IRA)*
>
> Added 4 accounts. Found tickers: VTI, VXUS, BND, FXAIX, VOO.
> I'll map those to return profiles later.
>
> **Step 3 — Income**
> What are your income sources?
> - How much do you earn per paycheck, and how often are you paid?
> - Any other income (freelance, rental, etc.)?

---

**You type:**

```
$5,769.23 biweekly salary. That's my only income right now.
```

---

**Claude responds:**

> *(calls `add_income_event` with name: "Salary", amount: 5769.23, interval: "biweekly", end_event: "Retirement")*
>
> Salary added — biweekly, inflation-adjusted, ends at retirement.
>
> **Step 4 — Expenses**
> What are your monthly living expenses? Include rent/mortgage, groceries,
> utilities, subscriptions — everything.

---

**You type:**

```
About $6,000/month all in.
```

---

**Claude responds:**

> *(calls `add_expense_event` with name: "Living Expenses", amount: 6000.0, interval: "monthly")*
>
> Got it. I've made it inflation-adjusted so it grows with CPI over time.
>
> **Step 5 — Retirement**
> When do you want to retire, and how much do you expect to spend per year
> in retirement?

---

**You type:**

```
I'm hoping to retire at 60. I think I'd need about $80,000/year.
```

---

**Claude responds:**

> *(calls `add_retirement_event` with age: 60)*
> *(calls `add_sweep_event` — yearly withdrawal targeting $80k in Checking, starting at Retirement)*
>
> Retirement milestone set at age 60. I've added a yearly portfolio sweep that
> tops up your checking account to $80,000 each year, using a penalty-aware
> strategy to minimize early withdrawal taxes.
>
> **Step 6 — Contributions**
> Are you contributing to your 401k or IRA? If so, how much per year?

---

**You type:**

```
Maxing out the 401k, $23,500/year into FXAIX. No IRA contributions right now.
```

---

**Claude responds:**

> *(calls `add_contribution_event` with name: "401k Contribution", amount: 23500, asset: "FXAIX", end_event: "Retirement")*
>
> 401k contribution added — $23,500/year, stops at retirement.
>
> **Step 7 — Social Security**
> Do you know roughly what your Social Security benefit will be, and when
> you plan to claim it?

---

**You type:**

```
I'll probably claim at 67. SSA.gov says about $2,800/month.
```

---

**Claude responds:**

> *(calls `add_social_security_event` with start_age: 67, monthly_amount: 2800)*
>
> Social Security added — $2,800/month starting at 67.
> I'll also add RMDs automatically since you have a Traditional 401k.
>
> *(calls `add_rmd_event` with start_age: 73)*
>
> **Step 8 — Tickers**
> *(calls `map_tickers`)*
>
> All 5 tickers mapped to historical presets:
> - VTI, VXUS, BND, FXAIX, VOO → S&P 500 / Intl Developed / US Agg Bonds
>
> **Step 9 — Validate**
> *(calls `validate_scenario`)*
>
> ✅ Scenario is valid — no errors.
>
> **Step 10 — Final YAML**
> *(calls `merge_scenario`)*

---

**Claude delivers:**

> Here's your completed scenario. Save this as `~/.finplan/scenarios/sarah_plan.yaml`
> and load it in FinPlan with `3` → `i`.

```yaml
portfolios:
  name: My Retirement Plan
  accounts:
    - name: Chase Checking
      type: Checking
      value: 15000.0
    - name: Fidelity Brokerage
      type: Brokerage
      assets:
        - asset: VTI
          value: 150000.0
        - asset: VXUS
          value: 50000.0
        - asset: BND
          value: 30000.0
    - name: Company 401k
      type: Traditional401k
      assets:
        - asset: FXAIX
          value: 200000.0
    - name: Roth IRA
      type: RothIRA
      assets:
        - asset: VOO
          value: 75000.0

historical_assets:
  VTI: S&P 500
  VXUS: Intl Developed
  BND: US Agg Bonds
  FXAIX: S&P 500
  VOO: S&P 500

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
        to: Chase Checking
        amount:
          type: InflationAdjusted
          inner: { type: Fixed, value: 5769.23 }
        gross: true
        taxable: true

  - name: Living Expenses
    trigger:
      type: Repeating
      interval: monthly
    effects:
      - type: Expense
        from: Chase Checking
        amount:
          type: InflationAdjusted
          inner: { type: Fixed, value: 6000.0 }

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
        from: Chase Checking
        to_account: Company 401k
        asset: FXAIX
        amount: { type: Fixed, value: 23500.0 }

  - name: Retirement
    trigger: { type: Age, years: 60 }
    once: true

  - name: Yearly Spend Post-Retirement
    trigger:
      type: Repeating
      interval: yearly
      start:
        type: RelativeToEvent
        event: Retirement
        offset: { unit: Months, value: 0 }
    effects:
      - type: Sweep
        to: Chase Checking
        amount: { type: TargetToBalance, target: 80000.0 }
        strategy: penalty_aware
        lot_method: fifo

  - name: Social Security
    trigger:
      type: Repeating
      interval: monthly
      start: { type: Age, years: 67 }
    effects:
      - type: Income
        to: Chase Checking
        amount: { type: Fixed, value: 2800.0 }
        gross: true
        taxable: true

  - name: RMD
    trigger:
      type: Repeating
      interval: yearly
      start: { type: Age, years: 73 }
    effects:
      - type: ApplyRmd
        destination: Chase Checking
        lot_method: fifo

parameters:
  birth_date: "1985-06-15"
  start_date: "2026-01-01"
  duration_years: 40
  inflation:
    type: USHistorical
    distribution: lognormal
  tax_config:
    state_rate: 0.0
    capital_gains_rate: 0.15
    federal_brackets: single2024
  returns_mode: historical
  historical_block_size: 5

analysis:
  mc_iterations: 500
  default_steps: 6
```

---

## Refining a Scenario

Once you have a base scenario you can ask Claude to modify it without
starting over. The server's `reset_state` tool clears the accumulated state,
but you can also just rebuild with different inputs:

```
Update my scenario: change retirement age to 62 and increase
post-retirement spending to $90,000/year.
```

```
Add a home purchase event: buy a $650,000 house in June 2029,
$130,000 down, 5.5% mortgage, $3,200/month payment.
```

```
I forgot to include my Roth 401k. Add it: $45,000 in FXAIX.
```

```
What if I added a second income — my spouse earns $85,000/year?
Add that and rebuild the scenario.
```

---

## Tips for Good Results

**Have your numbers ready before starting.** The AI can work with estimates,
but the more specific you are, the more accurate the scenario:

- Account names and current balances or holdings (ticker + value)
- Gross income per paycheck and pay frequency
- Monthly spending (a rough total is fine)
- Target retirement age and annual spending goal
- Social Security estimate from [ssa.gov/myaccount](https://www.ssa.gov/myaccount/)

**Name your accounts clearly.** The scenario YAML references accounts by name,
so "Company 401k" is better than "401k" if you have multiple retirement accounts.

**Unknown tickers are reported.** If you hold something like `PLTR` or a
custom fund, the AI will tell you it couldn't map it and ask you to specify
a profile (e.g., "treat it like S&P 500").

**Load and run immediately.** Once you have the YAML:
1. Save it to `~/.finplan/scenarios/yourname.yaml`
2. Open FinPlan: `docker compose run --rm finplan`
3. Go to tab `3` (Scenario), press `i`, enter the file path
4. Press `r` to run Monte Carlo
5. Press `4` to see results

---

## Behind the Scenes: Tool Call Sequence

For reference, here is the sequence of MCP tool calls Claude makes for a
typical full scenario:

| Step | Tool | What it does |
|------|------|--------------|
| 1 | `set_portfolio` | Names the portfolio |
| 2 | `add_account` ×N | Adds each account |
| 3 | `set_parameters` | Birth date, duration, tax config, returns mode |
| 4 | `add_income_event` | Salary and other recurring income |
| 5 | `add_expense_event` | Living expenses |
| 6 | `add_contribution_event` | 401k/IRA contributions |
| 7 | `add_retirement_event` | Retirement milestone age |
| 8 | `add_sweep_event` | Post-retirement portfolio withdrawal |
| 9 | `add_social_security_event` | SS income starting at claim age |
| 10 | `add_rmd_event` | Required Minimum Distributions at 73 |
| 11 | `map_tickers` | Maps holdings to historical return profiles |
| 12 | `validate_scenario` | Checks for errors |
| 13 | `merge_scenario` | Returns the final YAML |

---

## Tool Reference

### Portfolio

| Tool | Purpose |
|------|---------|
| `set_portfolio` | Initialize portfolio with a name |
| `add_account` | Add an account (see [Account Types](08-scenario-yaml-reference.md#portfolios)) |

### Parameters

| Tool | Purpose |
|------|---------|
| `set_parameters` | Birth date, duration, inflation, taxes, returns mode |

### Events

| Tool | Purpose |
|------|---------|
| `add_income_event` | Recurring income (salary, freelance) |
| `add_expense_event` | Recurring expenses (living costs, rent) |
| `add_contribution_event` | Investment contributions (401k, IRA) |
| `add_retirement_event` | Retirement milestone — other events reference this |
| `add_social_security_event` | Social Security income starting at a given age |
| `add_rmd_event` | Required Minimum Distributions (default: age 73) |
| `add_sweep_event` | Post-retirement portfolio withdrawal to fund spending |
| `add_custom_event` | Any event not covered by the templates above |

### Assembly

| Tool | Purpose |
|------|---------|
| `map_tickers` | Auto-map portfolio tickers to historical or parametric return profiles |
| `validate_scenario` | Check the accumulated state for errors |
| `merge_scenario` | Assemble everything into a final YAML string |

### Utilities

| Tool | Purpose |
|------|---------|
| `get_state_summary` | See what's been accumulated so far |
| `reset_state` | Clear everything and start over |

---

## Schema Resources

The AI reads these before building your scenario:

| URI | Contents |
|-----|---------|
| `schema://full` | Complete YAML reference |
| `schema://accounts` | All 13 account types with fields |
| `schema://events` | Event structure (triggers, effects, once, enabled) |
| `schema://triggers` | All trigger types with examples |
| `schema://effects` | All effect types with examples |
| `schema://amounts` | Amount types (Fixed, InflationAdjusted, Scale, etc.) |
| `schema://parameters` | Parameter fields (dates, inflation, taxes, returns) |
| `schema://profiles` | Return profile types and presets |
| `schema://analysis` | Monte Carlo and sweep configuration |
| `schema://example` | The bundled `examples/example.yaml` |
| `schema://patterns` | Common event pattern templates |

---

## Supported Tickers

`map_tickers` recognizes ~80 common ETFs and mutual funds:

| Category | Tickers |
|----------|---------|
| S&P 500 | VOO, SPY, IVV, VFIAX, FXAIX, SWPPX |
| US Total Market | VTI, VTSAX, ITOT, SCHB, FSKAX |
| US Small Cap | VB, IJR, SCHA, IWM, VSMAX |
| US Agg Bond | BND, AGG, VBTLX, SCHZ, FXNAX |
| Intl Developed | VXUS, VEA, EFA, IEFA, SWISX |
| Emerging Markets | VWO, IEMG, EEM, VEMAX |
| REITs | VNQ, IYR, SCHH, VGSLX |
| Gold | GLD, IAU, SGOL, GLDM |
| Money Market | VMFXX, SPAXX, FDRXX, SGOV |
| Long-Term Treasury | TLT, VGLT, EDV |
| TIPS | TIP, VTIP, SCHP |

Unknown tickers are reported back — use the `overrides` parameter to map them
manually, e.g. `"PLTR": "S&P 500"`.

---

## Running the Test Suite

The integration tests exercise the full tool-call sequence and are the fastest
way to verify the server is working correctly after changes:

```bash
# quiet
docker run --rm -v "$(pwd)":/app -w /app rust:slim cargo test -p finplan_mcp

# verbose — shows each step, tool responses, and the merged YAML output
docker run --rm -v "$(pwd)":/app -w /app rust:slim cargo test -p finplan_mcp -- --nocapture
```

Test file: `crates/finplan_mcp/tests/integration_test.rs`

---

**Next:** [Scenario YAML Reference](08-scenario-yaml-reference.md) — the format the MCP server produces.
