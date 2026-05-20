# Create Your First Scenario

A **scenario** is a complete financial plan: your accounts, life events (income, expenses, investments), tax settings, and simulation parameters. Everything FinPlan needs to run projections lives in a scenario.

## Two Ways to Create a Scenario

**Path A — Build it in the TUI (recommended for first-timers)**

Use FinPlan's interactive terminal interface to add accounts and events through guided forms. No file editing required.

→ [Build it in the TUI](00a-tui-tutorial.md)

**Path B — Write a YAML file directly**

Write a `.yaml` file in a text editor, validate it from the command line, then import it into FinPlan. Best for bulk edits, sharing scenarios, or scripting.

→ [Write the YAML file](00b-yaml-tutorial.md)

## Which Path Should I Choose?

| Situation | Recommended Path |
|-----------|-----------------|
| First time using FinPlan | TUI (Path A) |
| Copying and tweaking an example | YAML (Path B) |
| Making small changes to an existing plan | TUI (Path A) |
| Bulk-editing many events or accounts | YAML (Path B) |
| Sharing a scenario with someone else | YAML (Path B) |

Both paths produce the same result — a scenario stored in `~/.finplan/scenarios/`. You can always export a TUI-built scenario to YAML, edit it, and re-import it.

## Before You Start

Make sure FinPlan is installed and runs:

```bash
# From source
cargo run --bin finplan --release

# Or via Docker
docker compose run --rm finplan
```

You should see the tab bar at the top of the terminal:
```
[1] Portfolio & Profiles  [2] Events  [3] Scenario  [4] Results  [5] Analysis
```

If that's working, pick your path above and follow the tutorial.
