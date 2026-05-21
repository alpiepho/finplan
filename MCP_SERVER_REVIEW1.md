# MCP Server Review — `feature/mcp` Branch

> **Reviewed**: 2026-05-20
> **Branch**: `feature/mcp`
> **Base branch**: `feature/validator`
> **Tool used**: Copilot (GitHub Copilot)

---

## 1. Summary

This branch adds a new `crates/finplan_mcp` crate — a Model Context Protocol (MCP) server that exposes FinPlan scenario construction as a set of AI-callable tools, resources, and prompts. The goal is to allow an external AI agent (e.g., Claude) to build a complete scenario YAML file interactively, potentially combining data parsed from bank/brokerage statements with user interview answers.

The work was done in two Copilot-assisted commits:

| Commit | Date | Description |
|--------|------|-------------|
| `40bb4ae` | Apr 25, 2026 | Initial plan document (`MCP_SERVER_PLAN1.md`) |
| `2fadd3d` | Apr 25, 2026 | Partial MCP implementation (new crate, all core source files) |
| `82c5fc2` | (merge) | Merged `feature/validator` into `feature/mcp` |

---

## 2. Files Added

### New Crate: `crates/finplan_mcp/`

| File | Lines | Status |
|------|-------|--------|
| `Cargo.toml` | 37 | Complete |
| `src/main.rs` | 35 | Complete |
| `src/server.rs` | 132 | Complete |
| `src/state.rs` | 111 | Complete |
| `src/defaults.rs` | 63 | Complete |
| `src/prompts.rs` | 230 | Complete |
| `src/resources.rs` | 54 | Complete |
| `src/schema_text.rs` | 676 | Complete |
| `src/tools/mod.rs` | 145 | Complete |
| `src/tools/parameters.rs` | 189 | Complete |
| `src/tools/portfolio.rs` | 256 | Complete |
| `src/tools/ticker.rs` | 142 | Complete |
| `src/tools/merge.rs` | 38 | Complete |
| `src/tools/validate.rs` | 53 | Complete |
| `src/tools/events.rs` | 642 | Complete |
| **Total** | **2,766** | |

### Other Files Modified

| File | Change |
|------|--------|
| `Cargo.toml` (root) | Added `crates/finplan_mcp` to workspace members |
| `Dockerfile` | Minor update for MCP binary |
| `docker-compose.yml` | Added `finplan-mcp` service |
| `scripts/test_mcp.sh` | New manual integration test via stdio JSON-RPC |
| `MCP_SERVER_PLAN1.md` | Full implementation plan document (1,607 lines) |

---

## 3. Plan vs. Implementation

The `MCP_SERVER_PLAN1.md` is a detailed 1,607-line blueprint. The following table maps the plan's Phase checklist to actual implementation status.

### Phase 1: Scaffolding ✅ Complete

All scaffolding items from the plan are implemented and match the design.

- `Cargo.toml` created with correct dependencies (`rmcp v0.17`, `tokio`, `serde`, `serde_saphyr`, `schemars`, `tracing`)
- `main.rs`: tokio main with stdio transport
- `server.rs`: `FinplanMcpServer` struct with `ServerHandler` implementation
- `state.rs`: `ScenarioState` with `Arc<Mutex<>>` wrapper (`SharedState` type alias)

### Phase 2: Resources ✅ Complete (code), ⚠️ Missing tests

- `resources.rs` and `schema_text.rs` implemented — static schema documentation for all 11 planned resource URIs:
  - `schema://full`, `schema://accounts`, `schema://events`, `schema://triggers`, `schema://effects`, `schema://amounts`, `schema://parameters`, `schema://profiles`, `schema://analysis`, `schema://example`, `schema://patterns`
- Wired into `ServerHandler::list_resources` and `read_resource`
- **Missing**: `tests/resource_tests.rs`

### Phase 3: Core Tools ✅ Complete (code), ⚠️ Missing tests

All planned core tools are implemented:

| Tool | File | Status |
|------|------|--------|
| `set_parameters` | `tools/parameters.rs` | ✅ |
| `set_portfolio` | `tools/portfolio.rs` | ✅ |
| `add_account` | `tools/portfolio.rs` | ✅ (bonus — not in original plan) |
| `map_tickers` | `tools/ticker.rs` | ✅ |
| `merge_scenario` | `tools/merge.rs` | ✅ |
| `validate_scenario` | `tools/validate.rs` | ✅ |
| `get_state_summary` | `tools/mod.rs` | ✅ (bonus — not in original plan) |
| `reset_state` | `tools/mod.rs` | ✅ (bonus — not in original plan) |

- **Missing**: `tests/tool_tests.rs`

### Phase 4: Event Tools ✅ Mostly complete, ⚠️ One tool missing

All event tools were consolidated into a single `src/tools/events.rs` (vs. separate files per the plan). The 8 implemented tools:

| Tool | Status |
|------|--------|
| `add_income_event` | ✅ |
| `add_expense_event` | ✅ |
| `add_retirement_event` | ✅ |
| `add_social_security_event` | ✅ |
| `add_rmd_event` | ✅ |
| `add_contribution_event` | ✅ |
| `add_sweep_event` | ✅ (bonus — not in original plan) |
| `add_custom_event` | ✅ |
| `add_home_purchase` | ❌ Not implemented |
| `src/interview.rs` | ❌ Not implemented |
| Event tool tests | ❌ Not implemented |

### Phase 5: Prompts ✅ Complete

Both MCP prompts implemented in `src/prompts.rs`:
- `build_scenario` — guided multi-step interview prompt
- `quick_retirement` — minimal-input template prompt
- Wired into `ServerHandler::list_prompts` and `get_prompt`

### Phase 6: Integration Testing ⚠️ Partial

| Item | Status |
|------|--------|
| `scripts/test_mcp.sh` (stdio JSON-RPC manual test) | ✅ |
| `tests/integration_test.rs` (Rust async test) | ❌ |
| `tests/validation_tests.rs` | ❌ |
| `example.yaml` round-trip test | ❌ |
| Claude Desktop / MCP Inspector testing | ❌ (unknown) |

### Phase 7: Polish ⚠️ Partial

| Item | Status |
|------|--------|
| `tracing` logging to stderr | ✅ |
| Docker / `docker-compose.yml` | ✅ |
| `--help` with `clap` | ❌ |
| `README.md` MCP section | ❌ |
| `claude_desktop_config.json` example | ❌ |

---

## 4. Notable Implementation Differences from the Plan

The plan's **Implementation Notes** section (§11) already documents these, but they are worth highlighting:

1. **File organization**: Event tools consolidated into a single `events.rs` rather than separate files. This is a pragmatic choice at the current scale — maintainability is fine.

2. **`get_ticker_info` tool**: Not implemented as a standalone tool. Ticker info is surfaced via `map_tickers` response. A minor ergonomics gap if an agent wants to query a single ticker.

3. **`add_home_purchase`**: Not implemented. The plan acknowledged this upfront — it can be approximated via `add_custom_event`.

4. **`validate_scenario`**: Validates the accumulated server state (in-memory) rather than accepting a raw YAML string as the plan specified. This is actually more ergonomic for the tool-calling workflow.

5. **`merge_scenario`**: No `validate` parameter; validation is a separate tool call. Consistent with the implementation of `validate_scenario` above.

6. **Two bonus tools**: `get_state_summary` and `reset_state` were added beyond the plan — these are genuinely useful for workflow management.

7. **`add_sweep_event`**: An additional event tool not in the original plan that covers post-retirement portfolio withdrawal events.

8. **`add_account`**: Added as a separate tool for incremental portfolio construction (in addition to `set_portfolio`).

9. **Dependency versions**: `rmcp v0.17.0` (plan said to use v0.17), `serde_saphyr v0.0.15` (plan said 0.0.8), `schemars v0.8` (plan said v1). These are fine — the plan was drafted before versions were pinned.

10. **`rmcp` API style**: Uses manual `ServerHandler` impl instead of the `#[tool]` macro. This is a workaround for API instability in the early rmcp releases.

---

## 5. Architecture Assessment

The implementation closely follows the planned architecture:

```
AI Agent
   │
   ▼ (MCP stdio transport)
finplan-mcp binary
   ├── ServerHandler (server.rs)
   │     ├── Resources: static schema docs (resources.rs + schema_text.rs)
   │     ├── Tools: stateful scenario builder (tools/*.rs)
   │     └── Prompts: guided workflows (prompts.rs)
   │
   └── ScenarioState (state.rs)
         └── Arc<Mutex<ScenarioState>>
               ├── portfolio, parameters
               ├── events (accumulated)
               ├── historical_assets / profiles
               └── merge() → SimulationData → YAML
```

The `state.rs` `merge()` function produces a `SimulationData` struct (from the `finplan` crate) and serializes it with `serde_saphyr`, ensuring the output YAML is byte-compatible with what the TUI app reads. This is the correct approach.

The `validate_scenario` tool delegates to `finplan::data::validator::validate_scenario()` — it reuses the existing validator from the `feature/validator` branch work rather than reimplementing it.

---

## 6. What's Working vs. What's Missing

### Working (code exists, should be functional)
- Complete MCP server binary with stdio transport
- 16 tools covering the full scenario-building workflow
- 11 schema resources providing AI-readable documentation
- 2 guided prompts
- Server state accumulation and YAML merge/export
- Ticker-to-profile mapping (reuses `finplan` crate's `ticker_profiles.rs`)
- Validation via existing `validator.rs`
- Docker-based deployment (`docker-compose run finplan-mcp`)
- Manual integration test script (`scripts/test_mcp.sh`)

### Missing / Not Implemented
- `add_home_purchase` tool
- `get_ticker_info` tool (standalone)
- `src/interview.rs` (interview engine with question metadata)
- All Rust automated tests (`tests/` directory is empty)
- Claude Desktop configuration example
- Binary `--help` flag
- README documentation update

### Build Status
- Not verified in this review (no `cargo` in PATH at review time). The code structure looks complete and should compile, but build status is unknown.

---

## 7. Recommended Next Steps

Priority order for completing the MCP server:

1. **Verify it builds and runs**: `cargo build -p finplan_mcp` and run `scripts/test_mcp.sh` against the Docker container to confirm the basic workflow works end-to-end.

2. **Write automated tests**: Start with `tests/integration_test.rs` covering the full tool call sequence (set_portfolio → set_parameters → add events → map_tickers → merge → validate). This catches regressions when the `finplan` crate evolves.

3. **Add `claude_desktop_config.json`**: A ready-to-use config snippet for connecting to Claude Desktop would make the server immediately usable.

4. **Update README.md**: Add an MCP server section explaining how to run it and what it does.

5. **Implement `add_home_purchase`**: This is the only planned tool not yet built. The plan has a clear spec (§4.8).

6. **Test with a real AI agent**: Run the guided `build_scenario` prompt in Claude Desktop or with the MCP Inspector to validate the user-facing workflow.

7. **Lower priority**: `--help` flag, `get_ticker_info` as standalone tool, `interview.rs` engine.

---

## 8. Overall Assessment

The Copilot-generated implementation is **well-structured and substantially complete**. The plan is faithfully followed, the departures from the plan are pragmatic (consolidation rather than feature removal), and the two bonus tools (`get_state_summary`, `reset_state`, `add_sweep_event`, `add_account`) are genuine improvements.

The main gaps are on the testing and documentation side — all the code is there, but there are no automated Rust tests, and the server has not yet been validated against a real AI agent workflow.

**Completion estimate**: ~70% of the plan implemented. The remaining 30% is primarily tests, docs, and one tool (`add_home_purchase`).
