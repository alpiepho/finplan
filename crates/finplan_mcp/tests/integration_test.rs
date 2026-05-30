use finplan_mcp::{resources, state, tools};
use rmcp::model::{CallToolResult, ResourceContents};
use serde_json::{Map, Value, json};

fn args(v: Value) -> Map<String, Value> {
    match v {
        Value::Object(m) => m,
        _ => Map::new(),
    }
}

fn is_ok(result: &CallToolResult) -> bool {
    result.is_error.unwrap_or(false) == false
}

fn text_of(result: &CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|c| c.as_text())
        .map(|t| t.text.as_str())
        .collect::<Vec<_>>()
        .join("")
}

fn step(n: u32, label: &str) {
    println!("\n[step {n}] {label}");
}

fn ok(result: &CallToolResult) {
    println!("  → {}", text_of(result));
}

// ── Full Scenario Build ──────────────────────────────────────────────────────

#[test]
fn test_full_scenario_build() {
    println!("\n═══ Full Scenario Build: Sarah's Retirement Plan ═══");
    let st = state::new_shared_state();

    step(1, "set_portfolio — initialize portfolio");
    let r = tools::portfolio::set_portfolio(args(json!({"name": "Sarah's Plan"})), &st).unwrap();
    assert!(is_ok(&r), "set_portfolio failed: {}", text_of(&r));
    ok(&r);

    step(2, "add_account — Checking ($15,000)");
    let r = tools::portfolio::add_account(
        args(json!({"name": "Checking", "account_type": "Checking", "value": 15000.0})),
        &st,
    )
    .unwrap();
    assert!(is_ok(&r), "add checking failed: {}", text_of(&r));
    ok(&r);

    step(3, "add_account — Company 401k (FXAIX $200,000)");
    let r = tools::portfolio::add_account(
        args(json!({
            "name": "Company 401k",
            "account_type": "Traditional401k",
            "assets": [{"ticker": "FXAIX", "value": 200000.0}]
        })),
        &st,
    )
    .unwrap();
    assert!(is_ok(&r), "add 401k failed: {}", text_of(&r));
    ok(&r);

    step(4, "add_account — Roth IRA (VOO $75,000)");
    let r = tools::portfolio::add_account(
        args(json!({
            "name": "Roth IRA",
            "account_type": "RothIRA",
            "assets": [{"ticker": "VOO", "value": 75000.0}]
        })),
        &st,
    )
    .unwrap();
    assert!(is_ok(&r), "add roth ira failed: {}", text_of(&r));
    ok(&r);

    step(5, "set_parameters — birth 1985-06-15, historical returns");
    let r =
        tools::parameters::set_parameters(args(json!({"birth_date": "1985-06-15"})), &st).unwrap();
    assert!(is_ok(&r), "set_parameters failed: {}", text_of(&r));
    ok(&r);

    step(
        6,
        "add_income_event — Salary $5,769/biweekly → Checking (ends at Retirement)",
    );
    let r = tools::events::add_income_event(
        args(json!({
            "name": "Salary",
            "to_account": "Checking",
            "amount": 5769.23,
            "interval": "biweekly",
            "end_event": "Retirement"
        })),
        &st,
    )
    .unwrap();
    assert!(is_ok(&r), "add_income_event failed: {}", text_of(&r));
    ok(&r);

    step(
        7,
        "add_expense_event — Living Expenses $6,000/month from Checking",
    );
    let r = tools::events::add_expense_event(
        args(json!({
            "name": "Living Expenses",
            "from_account": "Checking",
            "amount": 6000.0
        })),
        &st,
    )
    .unwrap();
    assert!(is_ok(&r), "add_expense_event failed: {}", text_of(&r));
    ok(&r);

    step(
        8,
        "add_contribution_event — 401k $23,500/yr FXAIX (ends at Retirement)",
    );
    let r = tools::events::add_contribution_event(
        args(json!({
            "name": "401k Contribution",
            "from_account": "Checking",
            "to_account": "Company 401k",
            "asset": "FXAIX",
            "amount": 23500.0,
            "end_event": "Retirement"
        })),
        &st,
    )
    .unwrap();
    assert!(is_ok(&r), "add_contribution_event failed: {}", text_of(&r));
    ok(&r);

    step(9, "add_retirement_event — Retirement at age 65");
    let r = tools::events::add_retirement_event(args(json!({"age": 65})), &st).unwrap();
    assert!(is_ok(&r), "add_retirement_event failed: {}", text_of(&r));
    ok(&r);

    step(
        10,
        "add_social_security_event — $2,800/month starting age 67",
    );
    let r = tools::events::add_social_security_event(
        args(json!({
            "start_age": 67,
            "monthly_amount": 2800.0,
            "to_account": "Checking"
        })),
        &st,
    )
    .unwrap();
    assert!(
        is_ok(&r),
        "add_social_security_event failed: {}",
        text_of(&r)
    );
    ok(&r);

    step(11, "add_rmd_event — RMD starting age 73 → Checking");
    let r = tools::events::add_rmd_event(args(json!({"destination": "Checking"})), &st).unwrap();
    assert!(is_ok(&r), "add_rmd_event failed: {}", text_of(&r));
    ok(&r);

    step(
        12,
        "map_tickers — auto-map FXAIX and VOO to historical presets",
    );
    let r = tools::ticker::map_tickers(args(json!({})), &st).unwrap();
    assert!(is_ok(&r), "map_tickers failed: {}", text_of(&r));
    ok(&r);
    let ticker_text = text_of(&r);
    assert!(
        ticker_text.contains("FXAIX"),
        "Expected FXAIX in ticker mapping output"
    );
    assert!(
        ticker_text.contains("VOO"),
        "Expected VOO in ticker mapping output"
    );

    step(13, "validate_scenario — check for errors");
    let r = tools::validate::validate_scenario(&st).unwrap();
    assert!(is_ok(&r), "validate_scenario failed:\n{}", text_of(&r));
    ok(&r);

    step(14, "merge_scenario — assemble final YAML");
    let r = tools::merge::merge_scenario(&st).unwrap();
    assert!(is_ok(&r), "merge_scenario failed: {}", text_of(&r));
    let yaml = text_of(&r);
    println!(
        "\n--- merged YAML ({} bytes) ---\n{}\n---",
        yaml.len(),
        yaml
    );

    assert!(yaml.contains("Sarah's Plan"), "YAML missing portfolio name");
    assert!(yaml.contains("Checking"), "YAML missing Checking account");
    assert!(yaml.contains("FXAIX"), "YAML missing FXAIX ticker");
    assert!(yaml.contains("Salary"), "YAML missing Salary event");
    assert!(yaml.contains("Retirement"), "YAML missing Retirement event");

    step(15, "verify state counts");
    let st_locked = st.lock().unwrap();
    let portfolio = st_locked
        .portfolio
        .as_ref()
        .expect("portfolio should be set");
    println!("  accounts : {}", portfolio.accounts.len());
    println!("  events   : {}", st_locked.events.len());
    println!("  hist maps: {}", st_locked.historical_assets.len());
    assert_eq!(portfolio.accounts.len(), 3, "Expected 3 accounts");
    assert_eq!(st_locked.events.len(), 6, "Expected 6 events");
    assert!(
        !st_locked.historical_assets.is_empty(),
        "Expected historical asset mappings"
    );

    println!("\n✓ Full scenario build passed");
}

// ── State Management ──────────────────────────────────────────────────────────

#[test]
fn test_state_summary_empty() {
    println!("\n═══ State Summary: fresh state should show nothing set ═══");
    let st = state::new_shared_state();
    let r = tools::get_state_summary(&st).unwrap();
    ok(&r);
    assert!(is_ok(&r));
    let text = text_of(&r);
    assert!(
        text.contains("Portfolio: not set"),
        "Missing 'Portfolio: not set'"
    );
    assert!(
        text.contains("Parameters: not set"),
        "Missing 'Parameters: not set'"
    );
    assert!(
        text.contains("Events: 0 defined"),
        "Missing 'Events: 0 defined'"
    );
}

#[test]
fn test_reset_clears_state() {
    println!("\n═══ Reset State: populate then clear ═══");
    let st = state::new_shared_state();

    tools::portfolio::set_portfolio(args(json!({"name": "Temp"})), &st).unwrap();
    tools::parameters::set_parameters(args(json!({"birth_date": "1980-01-01"})), &st).unwrap();
    tools::events::add_income_event(
        args(json!({"name": "Job", "to_account": "Checking", "amount": 1000.0})),
        &st,
    )
    .unwrap();

    let before_r = tools::get_state_summary(&st).unwrap();
    let before = text_of(&before_r);
    println!("  before reset: {}", before.replace('\n', " | "));
    assert!(
        before.contains("Events: 1 defined"),
        "Expected 1 event before reset"
    );

    tools::reset_state(&st).unwrap();
    println!("  reset_state called");

    let after_r = tools::get_state_summary(&st).unwrap();
    let after = text_of(&after_r);
    println!("  after reset:  {}", after.replace('\n', " | "));
    assert!(
        after.contains("Portfolio: not set"),
        "Expected portfolio reset"
    );
    assert!(
        after.contains("Events: 0 defined"),
        "Expected 0 events after reset"
    );
}

// ── Tool List ─────────────────────────────────────────────────────────────────

#[test]
fn test_tool_list_contains_expected_tools() {
    println!("\n═══ Tool List: verify all 20 tools are registered ═══");
    let tool_list = tools::list_tools();
    let names: Vec<&str> = tool_list.iter().map(|t| t.name.as_ref()).collect();
    println!("  registered tools ({}):", names.len());
    for name in &names {
        println!("    - {name}");
    }

    for expected in &[
        // Plan 1 — scenario building
        "set_parameters",
        "set_portfolio",
        "add_account",
        "map_tickers",
        "add_income_event",
        "add_expense_event",
        "add_retirement_event",
        "add_contribution_event",
        "add_social_security_event",
        "add_rmd_event",
        "add_custom_event",
        "add_sweep_event",
        "merge_scenario",
        "validate_scenario",
        "get_state_summary",
        "reset_state",
        // Plan 2 — simulation & results
        "run_simulation",
        "run_monte_carlo",
        "get_account_snapshot",
        "get_ledger",
    ] {
        assert!(names.contains(expected), "Missing tool: {}", expected);
    }
    assert_eq!(names.len(), 20, "Expected 20 tools, got {}", names.len());
}

// ── Error Handling ────────────────────────────────────────────────────────────

#[test]
fn test_merge_fails_without_portfolio() {
    let st = state::new_shared_state();
    let r = tools::merge::merge_scenario(&st).unwrap();
    assert_eq!(
        r.is_error,
        Some(true),
        "Expected error result without portfolio"
    );
}

#[test]
fn test_add_account_requires_portfolio_first() {
    let st = state::new_shared_state();
    // set_portfolio was not called — add_account should fail
    let result = tools::portfolio::add_account(
        args(json!({"name": "Checking", "account_type": "Checking", "value": 1000.0})),
        &st,
    );
    let is_failure = match &result {
        Err(_) => true,
        Ok(r) => r.is_error == Some(true),
    };
    assert!(
        is_failure,
        "Expected failure when adding account without portfolio"
    );
}

#[test]
fn test_income_event_requires_name_field() {
    let st = state::new_shared_state();
    let result = tools::events::add_income_event(
        args(json!({"to_account": "Checking", "amount": 1000.0})),
        &st,
    );
    assert!(
        result.is_err(),
        "Expected Err when required 'name' field is missing"
    );
}

#[test]
fn test_duplicate_account_rejected() {
    let st = state::new_shared_state();
    tools::portfolio::set_portfolio(args(json!({"name": "Test"})), &st).unwrap();

    tools::portfolio::add_account(
        args(json!({"name": "Checking", "account_type": "Checking", "value": 1000.0})),
        &st,
    )
    .unwrap();

    let r = tools::portfolio::add_account(
        args(json!({"name": "Checking", "account_type": "Checking", "value": 2000.0})),
        &st,
    )
    .unwrap();
    assert_eq!(
        r.is_error,
        Some(true),
        "Expected error for duplicate account name"
    );
}

// ── Resources ─────────────────────────────────────────────────────────────────

#[test]
fn test_eleven_resources_listed() {
    println!("\n═══ Resources: verify 11 schema resources are listed ═══");
    let list = resources::list_resources();
    assert_eq!(
        list.len(),
        11,
        "Expected 11 schema resources, got {}",
        list.len()
    );
}

#[test]
fn test_all_resources_readable_and_non_empty() {
    println!("\n═══ Resources: verify each schema URI returns non-empty text ═══");
    let uris = [
        "schema://full",
        "schema://accounts",
        "schema://events",
        "schema://triggers",
        "schema://effects",
        "schema://amounts",
        "schema://parameters",
        "schema://profiles",
        "schema://analysis",
        "schema://example",
        "schema://patterns",
    ];
    for uri in &uris {
        let content = resources::read_resource(uri);
        assert!(content.is_some(), "Resource {} returned None", uri);

        let text = match content.unwrap() {
            ResourceContents::TextResourceContents { text, .. } => text,
            ResourceContents::BlobResourceContents { .. } => {
                panic!("Expected text content for {}", uri)
            }
        };
        assert!(!text.is_empty(), "Resource {} content is empty", uri);
        assert!(
            text.len() > 50,
            "Resource {} content suspiciously short ({} bytes)",
            uri,
            text.len()
        );
        println!("  {uri:30} {} bytes ✓", text.len());
    }
}

#[test]
fn test_unknown_resource_returns_none() {
    assert!(resources::read_resource("schema://nonexistent").is_none());
    assert!(resources::read_resource("").is_none());
    assert!(resources::read_resource("http://other.com").is_none());
}

// ── Spouse / Married Couple ───────────────────────────────────────────────────

#[test]
fn test_set_parameters_stores_spouse_birth_date() {
    let st = state::new_shared_state();
    let r = tools::parameters::set_parameters(
        args(json!({
            "birth_date": "1975-01-01",
            "spouse_birth_date": "1978-06-01"
        })),
        &st,
    )
    .unwrap();
    assert!(is_ok(&r), "set_parameters failed: {}", text_of(&r));

    let locked = st.lock().unwrap();
    let params = locked
        .parameters
        .as_ref()
        .expect("parameters should be set");
    assert_eq!(
        params.spouse_birth_date.as_deref(),
        Some("1978-06-01"),
        "spouse_birth_date not stored correctly"
    );
}

#[test]
fn test_spouse_birth_date_appears_in_merged_yaml() {
    let st = state::new_shared_state();

    tools::portfolio::set_portfolio(args(json!({"name": "Married Plan"})), &st).unwrap();
    tools::portfolio::add_account(
        args(json!({"name": "Checking", "account_type": "Checking", "value": 50000.0})),
        &st,
    )
    .unwrap();
    tools::parameters::set_parameters(
        args(json!({
            "birth_date": "1975-01-01",
            "spouse_birth_date": "1978-06-01"
        })),
        &st,
    )
    .unwrap();

    let r = tools::merge::merge_scenario(&st).unwrap();
    assert!(is_ok(&r), "merge_scenario failed: {}", text_of(&r));
    let yaml = text_of(&r);
    assert!(
        yaml.contains("spouse_birth_date"),
        "YAML missing spouse_birth_date field"
    );
    assert!(
        yaml.contains("1978-06-01"),
        "YAML missing spouse birth date value"
    );
}

#[test]
fn test_spouse_birth_date_omitted_when_not_set() {
    let st = state::new_shared_state();

    tools::portfolio::set_portfolio(args(json!({"name": "Single Plan"})), &st).unwrap();
    tools::portfolio::add_account(
        args(json!({"name": "Checking", "account_type": "Checking", "value": 10000.0})),
        &st,
    )
    .unwrap();
    tools::parameters::set_parameters(args(json!({"birth_date": "1980-01-01"})), &st).unwrap();

    let r = tools::merge::merge_scenario(&st).unwrap();
    assert!(is_ok(&r), "merge_scenario failed: {}", text_of(&r));
    let yaml = text_of(&r);
    assert!(
        !yaml.contains("spouse_birth_date"),
        "YAML should not contain spouse_birth_date when not set"
    );
}

#[test]
fn test_triggers_schema_documents_spouse_age() {
    let content = resources::read_resource("schema://triggers").expect("triggers schema missing");
    let text = match content {
        ResourceContents::TextResourceContents { text, .. } => text,
        _ => panic!("Expected text content"),
    };
    assert!(
        text.contains("SpouseAge"),
        "triggers schema missing SpouseAge documentation"
    );
}

#[test]
fn test_parameters_schema_documents_spouse_birth_date() {
    let content =
        resources::read_resource("schema://parameters").expect("parameters schema missing");
    let text = match content {
        ResourceContents::TextResourceContents { text, .. } => text,
        _ => panic!("Expected text content"),
    };
    assert!(
        text.contains("spouse_birth_date"),
        "parameters schema missing spouse_birth_date documentation"
    );
}

// ── Plan 2: Simulation & Results ──────────────────────────────────────────────

/// Build a minimal but fully runnable scenario: one Checking account, income,
/// and expenses. Cash-only so no ticker mapping is needed. 10-year duration
/// keeps tests fast.
fn setup_runnable_scenario() -> finplan_mcp::state::SharedState {
    let st = state::new_shared_state();

    tools::portfolio::set_portfolio(args(json!({"name": "Test Plan"})), &st).unwrap();
    tools::portfolio::add_account(
        args(json!({"name": "Checking", "account_type": "Checking", "value": 50000.0})),
        &st,
    )
    .unwrap();
    tools::parameters::set_parameters(
        args(json!({
            "birth_date": "1975-01-01",
            "start_date": "2025-01-01",
            "duration_years": 10
        })),
        &st,
    )
    .unwrap();
    tools::events::add_income_event(
        args(json!({
            "name": "Salary",
            "to_account": "Checking",
            "amount": 3000.0
        })),
        &st,
    )
    .unwrap();
    tools::events::add_expense_event(
        args(json!({
            "name": "Living Expenses",
            "from_account": "Checking",
            "amount": 2500.0
        })),
        &st,
    )
    .unwrap();

    st
}

// ── run_simulation ────────────────────────────────────────────────────────────

#[test]
fn test_run_simulation_basic() {
    println!("\n═══ run_simulation: basic output structure ═══");
    let st = setup_runnable_scenario();

    let r = tools::simulation::run_simulation(args(json!({})), &st).unwrap();
    println!("  → {}", &text_of(&r)[..200.min(text_of(&r).len())]);
    assert!(is_ok(&r), "run_simulation failed: {}", text_of(&r));

    let json: Value = serde_json::from_str(&text_of(&r)).expect("output must be valid JSON");
    assert!(
        json["summary"]["final_net_worth"].is_number(),
        "summary.final_net_worth should be a number"
    );
    assert!(
        json["summary"]["duration_years"].as_u64().unwrap_or(0) > 0,
        "duration_years should be > 0"
    );
    assert!(
        json["years"]
            .as_array()
            .map(|a| !a.is_empty())
            .unwrap_or(false),
        "years array should be non-empty"
    );
}

#[test]
fn test_run_simulation_year_count_matches_duration() {
    println!("\n═══ run_simulation: year count == duration_years ═══");
    let st = setup_runnable_scenario();

    let r = tools::simulation::run_simulation(args(json!({})), &st).unwrap();
    assert!(is_ok(&r));

    let json: Value = serde_json::from_str(&text_of(&r)).unwrap();
    let years = json["years"].as_array().unwrap();
    // duration_years=10 starting 2025 produces years 2025–2035 inclusive = 11 entries
    assert_eq!(
        years.len(),
        11,
        "Expected 11 year entries for duration_years=10 (2025–2035 inclusive)"
    );
}

#[test]
fn test_run_simulation_year_fields_present() {
    println!("\n═══ run_simulation: each year row has required fields ═══");
    let st = setup_runnable_scenario();

    let r = tools::simulation::run_simulation(args(json!({})), &st).unwrap();
    assert!(is_ok(&r));

    let json: Value = serde_json::from_str(&text_of(&r)).unwrap();
    let first_year = &json["years"][0];

    // Check all required fields exist
    for field in &[
        "year",
        "age",
        "spouse_age",
        "net_worth",
        "real_net_worth",
        "income",
        "expenses",
        "withdrawals",
        "contributions",
        "taxes",
    ] {
        assert!(
            !first_year[field].is_null() || *field == "spouse_age",
            "year row missing field: {field}"
        );
    }

    // First year should be 2025, age should be 50 (born 1975)
    assert_eq!(first_year["year"].as_i64(), Some(2025));
    assert_eq!(first_year["age"].as_i64(), Some(50));
}

#[test]
fn test_run_simulation_spouse_age_null_without_spouse() {
    println!("\n═══ run_simulation: spouse_age is null when no spouse set ═══");
    let st = setup_runnable_scenario();

    let r = tools::simulation::run_simulation(args(json!({})), &st).unwrap();
    assert!(is_ok(&r));

    let json: Value = serde_json::from_str(&text_of(&r)).unwrap();
    let first_year = &json["years"][0];
    assert!(
        first_year["spouse_age"].is_null(),
        "spouse_age should be null when no spouse_birth_date is set"
    );
}

#[test]
fn test_run_simulation_spouse_age_appears_in_years() {
    println!("\n═══ run_simulation: spouse_age present when spouse_birth_date is set ═══");
    let st = setup_runnable_scenario();

    // Override parameters to add a spouse (born 1978)
    tools::parameters::set_parameters(
        args(json!({
            "birth_date": "1975-01-01",
            "spouse_birth_date": "1978-01-01",
            "start_date": "2025-01-01",
            "duration_years": 10
        })),
        &st,
    )
    .unwrap();

    let r = tools::simulation::run_simulation(args(json!({})), &st).unwrap();
    assert!(is_ok(&r), "run_simulation failed: {}", text_of(&r));

    let json: Value = serde_json::from_str(&text_of(&r)).unwrap();
    let first_year = &json["years"][0];
    // 2025 - 1978 = 47
    assert_eq!(
        first_year["spouse_age"].as_i64(),
        Some(47),
        "Expected spouse_age 47 (born 1978, year 2025)"
    );
}

#[test]
fn test_run_simulation_caches_result_in_state() {
    println!("\n═══ run_simulation: result stored in state ═══");
    let st = setup_runnable_scenario();
    assert!(
        st.lock().unwrap().last_simulation_result.is_none(),
        "cache should start empty"
    );

    tools::simulation::run_simulation(args(json!({})), &st).unwrap();

    let locked = st.lock().unwrap();
    assert!(
        locked.last_simulation_result.is_some(),
        "last_simulation_result should be set after run"
    );
    assert!(
        locked.last_sim_data.is_some(),
        "last_sim_data should be set after run"
    );
}

#[test]
fn test_simulation_cache_cleared_after_scenario_change() {
    println!("\n═══ simulation cache: cleared when scenario is modified ═══");
    let st = setup_runnable_scenario();

    // Run simulation to populate the cache
    tools::simulation::run_simulation(args(json!({})), &st).unwrap();
    assert!(
        st.lock().unwrap().last_simulation_result.is_some(),
        "cache should be populated"
    );

    // Modifying scenario should clear the cache
    tools::parameters::set_parameters(
        args(json!({"birth_date": "1976-01-01", "duration_years": 10})),
        &st,
    )
    .unwrap();

    assert!(
        st.lock().unwrap().last_simulation_result.is_none(),
        "cache should be cleared after scenario change"
    );
}

// ── run_monte_carlo ───────────────────────────────────────────────────────────

#[test]
fn test_run_monte_carlo_success_rate_valid() {
    println!("\n═══ run_monte_carlo: success_rate in [0, 1] ═══");
    let st = setup_runnable_scenario();

    let r = tools::simulation::run_monte_carlo(args(json!({"iterations": 20})), &st).unwrap();
    assert!(is_ok(&r), "run_monte_carlo failed: {}", text_of(&r));

    let text = text_of(&r);
    // Output starts with "Completed N iterations.\n{json}"
    let json_start = text.find('{').expect("JSON not found in output");
    let json: Value = serde_json::from_str(&text[json_start..]).expect("invalid JSON");

    let sr = json["stats"]["success_rate"]
        .as_f64()
        .expect("success_rate missing");
    assert!((0.0..=1.0).contains(&sr), "success_rate {sr} out of [0, 1]");
    println!("  success_rate = {sr:.2}");
}

#[test]
fn test_run_monte_carlo_percentile_runs_present() {
    println!("\n═══ run_monte_carlo: default percentiles p5/p50/p95 present ═══");
    let st = setup_runnable_scenario();

    let r = tools::simulation::run_monte_carlo(args(json!({"iterations": 20})), &st).unwrap();
    assert!(is_ok(&r));

    let text = text_of(&r);
    let json_start = text.find('{').unwrap();
    let json: Value = serde_json::from_str(&text[json_start..]).unwrap();

    let runs = json["percentile_runs"]
        .as_object()
        .expect("percentile_runs missing");
    assert!(runs.contains_key("p5"), "p5 missing from percentile_runs");
    assert!(runs.contains_key("p50"), "p50 missing from percentile_runs");
    assert!(runs.contains_key("p95"), "p95 missing from percentile_runs");

    // Each percentile run should have summary and years
    let p50 = &runs["p50"];
    assert!(
        p50["summary"]["final_net_worth"].is_number(),
        "p50 summary.final_net_worth missing"
    );
    assert!(
        p50["years"]
            .as_array()
            .map(|a| !a.is_empty())
            .unwrap_or(false),
        "p50 years empty"
    );
}

#[test]
fn test_run_monte_carlo_stores_mc_summary_in_state() {
    println!("\n═══ run_monte_carlo: last_mc_summary stored in state ═══");
    let st = setup_runnable_scenario();

    tools::simulation::run_monte_carlo(args(json!({"iterations": 20})), &st).unwrap();

    let locked = st.lock().unwrap();
    assert!(
        locked.last_mc_summary.is_some(),
        "last_mc_summary should be set after MC run"
    );
    assert!(
        locked.last_simulation_result.is_some(),
        "last_simulation_result (P50) should be set"
    );
    assert!(
        locked.last_sim_data.is_some(),
        "last_sim_data should be set"
    );
}

// ── get_account_snapshot ──────────────────────────────────────────────────────

#[test]
fn test_get_account_snapshot_requires_prior_run() {
    println!("\n═══ get_account_snapshot: error without prior simulation ═══");
    let st = setup_runnable_scenario();

    let r = tools::simulation::get_account_snapshot(args(json!({"year": 2030})), &st).unwrap();
    assert_eq!(
        r.is_error,
        Some(true),
        "Expected error without prior simulation run"
    );
    println!("  → {}", text_of(&r));
}

#[test]
fn test_get_account_snapshot_returns_accounts_with_owner() {
    println!("\n═══ get_account_snapshot: returns accounts with owner field ═══");
    let st = setup_runnable_scenario();
    tools::simulation::run_simulation(args(json!({})), &st).unwrap();

    let r = tools::simulation::get_account_snapshot(args(json!({"year": 2030})), &st).unwrap();
    assert!(is_ok(&r), "get_account_snapshot failed: {}", text_of(&r));

    let json: Value = serde_json::from_str(&text_of(&r)).unwrap();
    assert_eq!(json["year"].as_i64(), Some(2030));
    assert!(
        json["net_worth"].is_number(),
        "net_worth should be a number"
    );

    let accounts = json["accounts"].as_array().expect("accounts array missing");
    assert!(!accounts.is_empty(), "accounts should be non-empty");

    let checking = accounts
        .iter()
        .find(|a| a["name"] == "Checking")
        .expect("Checking account not found");
    assert_eq!(
        checking["owner"].as_str(),
        Some("primary"),
        "Checking should be owner=primary"
    );
    assert!(
        checking["value"].is_number(),
        "account value should be a number"
    );
    println!(
        "  net_worth={}, Checking value={}",
        json["net_worth"], checking["value"]
    );
}

#[test]
fn test_get_account_snapshot_real_values_differ_from_nominal() {
    println!("\n═══ get_account_snapshot: real values differ from nominal ═══");
    let st = setup_runnable_scenario();
    tools::simulation::run_simulation(args(json!({})), &st).unwrap();

    // Nominal snapshot
    let r_nominal =
        tools::simulation::get_account_snapshot(args(json!({"year": 2034, "real": false})), &st)
            .unwrap();
    // Real snapshot
    let r_real =
        tools::simulation::get_account_snapshot(args(json!({"year": 2034, "real": true})), &st)
            .unwrap();

    assert!(is_ok(&r_nominal));
    assert!(is_ok(&r_real));

    let nominal_nw = serde_json::from_str::<Value>(&text_of(&r_nominal)).unwrap()["net_worth"]
        .as_f64()
        .unwrap();
    let real_nw = serde_json::from_str::<Value>(&text_of(&r_real)).unwrap()["net_worth"]
        .as_f64()
        .unwrap();

    // With positive inflation, real < nominal (unless net worth is 0)
    if nominal_nw.abs() > 1.0 {
        assert_ne!(
            nominal_nw, real_nw,
            "real and nominal net worth should differ when inflation > 0"
        );
        println!("  nominal={nominal_nw:.0}, real={real_nw:.0}");
    }
}

// ── get_ledger ────────────────────────────────────────────────────────────────

#[test]
fn test_get_ledger_requires_prior_run() {
    println!("\n═══ get_ledger: error without prior simulation ═══");
    let st = setup_runnable_scenario();

    let r = tools::simulation::get_ledger(args(json!({})), &st).unwrap();
    assert_eq!(
        r.is_error,
        Some(true),
        "Expected error without prior simulation run"
    );
    println!("  → {}", text_of(&r));
}

#[test]
fn test_get_ledger_income_filter_returns_only_income() {
    println!("\n═══ get_ledger: income filter returns only income entries ═══");
    let st = setup_runnable_scenario();
    tools::simulation::run_simulation(args(json!({})), &st).unwrap();

    let r =
        tools::simulation::get_ledger(args(json!({"filter": "income", "limit": 50})), &st).unwrap();
    assert!(is_ok(&r), "get_ledger failed: {}", text_of(&r));

    let json: Value = serde_json::from_str(&text_of(&r)).unwrap();
    assert_eq!(json["filter"].as_str(), Some("income"));

    let entries = json["entries"].as_array().expect("entries missing");
    assert!(
        !entries.is_empty(),
        "Expected income entries from monthly Salary event"
    );

    for entry in entries {
        assert_eq!(
            entry["category"].as_str(),
            Some("income"),
            "Non-income entry slipped through filter: {:?}",
            entry
        );
    }
    println!(
        "  {} income entries, total_matching={}",
        entries.len(),
        json["total_matching"]
    );
}

#[test]
fn test_get_ledger_expense_filter_returns_only_expenses() {
    println!("\n═══ get_ledger: expense filter returns only expense entries ═══");
    let st = setup_runnable_scenario();
    tools::simulation::run_simulation(args(json!({})), &st).unwrap();

    let r = tools::simulation::get_ledger(args(json!({"filter": "expense", "limit": 50})), &st)
        .unwrap();
    assert!(is_ok(&r));

    let json: Value = serde_json::from_str(&text_of(&r)).unwrap();
    let entries = json["entries"].as_array().unwrap();
    assert!(
        !entries.is_empty(),
        "Expected expense entries from monthly Living Expenses event"
    );

    for entry in entries {
        assert_eq!(entry["category"].as_str(), Some("expense"));
    }
}

#[test]
fn test_get_ledger_year_filter() {
    println!("\n═══ get_ledger: year filter restricts entries to one year ═══");
    let st = setup_runnable_scenario();
    tools::simulation::run_simulation(args(json!({})), &st).unwrap();

    let r = tools::simulation::get_ledger(args(json!({"year": 2027, "filter": "income"})), &st)
        .unwrap();
    assert!(is_ok(&r));

    let json: Value = serde_json::from_str(&text_of(&r)).unwrap();
    let entries = json["entries"].as_array().unwrap();
    for entry in entries {
        assert!(
            entry["date"].as_str().unwrap_or("").starts_with("2027"),
            "Entry date should be in 2027: {:?}",
            entry["date"]
        );
    }
    println!("  {} entries in 2027", entries.len());
}

#[test]
fn test_get_ledger_pagination() {
    println!("\n═══ get_ledger: limit and offset work correctly ═══");
    let st = setup_runnable_scenario();
    tools::simulation::run_simulation(args(json!({})), &st).unwrap();

    // Get first page
    let r1 = tools::simulation::get_ledger(
        args(json!({"filter": "income", "limit": 3, "offset": 0})),
        &st,
    )
    .unwrap();
    // Get second page
    let r2 = tools::simulation::get_ledger(
        args(json!({"filter": "income", "limit": 3, "offset": 3})),
        &st,
    )
    .unwrap();

    assert!(is_ok(&r1));
    assert!(is_ok(&r2));

    let j1: Value = serde_json::from_str(&text_of(&r1)).unwrap();
    let j2: Value = serde_json::from_str(&text_of(&r2)).unwrap();

    // total_matching is the same on both pages
    assert_eq!(
        j1["total_matching"], j2["total_matching"],
        "total_matching should be identical across pages"
    );

    // Each page returns at most 3 entries
    assert!(j1["returned"].as_u64().unwrap_or(99) <= 3);
    assert!(j2["returned"].as_u64().unwrap_or(99) <= 3);

    // Pages return different entries (first entry date differs)
    let first_date_p1 = &j1["entries"][0]["date"];
    let first_date_p2 = &j2["entries"][0]["date"];
    assert_ne!(
        first_date_p1, first_date_p2,
        "Page 1 and page 2 should have different entries"
    );
    println!(
        "  total_matching={}, p1[0]={}, p2[0]={}",
        j1["total_matching"], first_date_p1, first_date_p2
    );
}
