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
    println!("\n═══ Tool List: verify all 16 tools are registered ═══");
    let tool_list = tools::list_tools();
    let names: Vec<&str> = tool_list.iter().map(|t| t.name.as_ref()).collect();
    println!("  registered tools ({}):", names.len());
    for name in &names {
        println!("    - {name}");
    }

    for expected in &[
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
    ] {
        assert!(names.contains(expected), "Missing tool: {}", expected);
    }
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
    let params = locked.parameters.as_ref().expect("parameters should be set");
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
    tools::parameters::set_parameters(
        args(json!({"birth_date": "1980-01-01"})),
        &st,
    )
    .unwrap();

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
