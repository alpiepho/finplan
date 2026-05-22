use serde_json::{Map, Value};

use rmcp::{ErrorData as McpError, model::*};

/// List all available prompts.
pub fn list_prompts() -> Vec<Prompt> {
    vec![
        Prompt::new(
            "build_scenario",
            Some(
                "Guided workflow to build a complete FinPlan scenario from scratch. \
                  Walks through parameters, portfolio, events, and validation.",
            ),
            Some(vec![PromptArgument {
                name: "user_context".into(),
                title: None,
                description: Some(
                    "Brief description of the user's financial situation \
                         (age, income, goals, accounts, etc.)"
                        .into(),
                ),
                required: Some(true),
            }]),
        ),
        Prompt::new(
            "quick_retirement",
            Some(
                "Quick retirement planning scenario. Provide basic info and get a \
                  complete scenario with salary, expenses, 401k, retirement, SS, and RMDs.",
            ),
            Some(vec![
                PromptArgument {
                    name: "age".into(),
                    title: None,
                    description: Some("Current age".into()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "retirement_age".into(),
                    title: None,
                    description: Some("Target retirement age (default: 65)".into()),
                    required: Some(false),
                },
                PromptArgument {
                    name: "annual_income".into(),
                    title: None,
                    description: Some("Current annual gross income".into()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "monthly_expenses".into(),
                    title: None,
                    description: Some("Monthly living expenses".into()),
                    required: Some(true),
                },
                PromptArgument {
                    name: "state".into(),
                    title: None,
                    description: Some("US state abbreviation (e.g. CA, TX)".into()),
                    required: Some(false),
                },
            ]),
        ),
    ]
}

/// Get a prompt by name.
pub fn get_prompt(name: &str, args: Map<String, Value>) -> Result<GetPromptResult, McpError> {
    match name {
        "build_scenario" => build_scenario_prompt(args),
        "quick_retirement" => quick_retirement_prompt(args),
        _ => Err(McpError::invalid_params(
            format!("Unknown prompt: {}", name),
            None,
        )),
    }
}

fn build_scenario_prompt(args: Map<String, Value>) -> Result<GetPromptResult, McpError> {
    let user_context = args
        .get("user_context")
        .and_then(|v| v.as_str())
        .unwrap_or("No context provided")
        .to_string();

    let instructions = format!(
        r#"You are helping a user build a FinPlan financial planning scenario YAML file.

## User Context
{user_context}

## Workflow
Follow these steps in order, using the available MCP tools:

### 1. Read the Schema
First, read the schema resources to understand the YAML format:
- Read `schema://full` for the complete reference
- Read `schema://patterns` for common event patterns

### 2. Set Parameters
Call `set_parameters` with:
- birth_date (calculate from user's age)
- start_date (use current year)
- duration_years (enough to cover retirement)
- state_abbreviation (if provided)
- returns_mode (historical is recommended)

### 3. Build Portfolio
Call `set_portfolio` to create the portfolio, then use `add_account` for each account:
- Investment accounts (401k, IRA, Brokerage) with ticker/value pairs
- Cash accounts (Checking, Savings)
- Debt accounts if any (Mortgage, loans)

### 4. Map Tickers
Call `map_tickers` to auto-detect return profiles for portfolio tickers.

### 5. Add Events
Add events in this recommended order:
1. `add_income_event` — salary/income with end at retirement
2. `add_expense_event` — living expenses
3. `add_contribution_event` — 401k/IRA contributions
4. `add_retirement_event` — retirement milestone
5. `add_sweep_event` — post-retirement withdrawals
6. `add_social_security_event` — Social Security
7. `add_rmd_event` — Required Minimum Distributions

### 6. Validate
Call `validate_scenario` to check for errors.

### 7. Generate YAML
Call `merge_scenario` to produce the final YAML.

## Important Notes
- Account names in events must match portfolio account names exactly
- Use inflation_adjusted amounts for recurring events
- Set gross=true for employment income (taxes withheld)
- Retirement event should be once=true
- Post-retirement sweep should start after retirement event
"#
    );

    Ok(GetPromptResult {
        description: Some("Guided scenario construction workflow".into()),
        messages: vec![PromptMessage::new_text(
            PromptMessageRole::User,
            instructions,
        )],
    })
}

fn quick_retirement_prompt(args: Map<String, Value>) -> Result<GetPromptResult, McpError> {
    let age = args
        .get("age")
        .and_then(|v| {
            v.as_str()
                .and_then(|s| s.parse::<u32>().ok())
                .or_else(|| v.as_u64().map(|n| n as u32))
        })
        .unwrap_or(30);
    let retirement_age = args
        .get("retirement_age")
        .and_then(|v| {
            v.as_str()
                .and_then(|s| s.parse::<u32>().ok())
                .or_else(|| v.as_u64().map(|n| n as u32))
        })
        .unwrap_or(65);
    let annual_income = args
        .get("annual_income")
        .and_then(|v| {
            v.as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .or_else(|| v.as_f64())
        })
        .unwrap_or(100_000.0);
    let monthly_expenses = args
        .get("monthly_expenses")
        .and_then(|v| {
            v.as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .or_else(|| v.as_f64())
        })
        .unwrap_or(5_000.0);
    let state = args
        .get("state")
        .and_then(|v| v.as_str())
        .unwrap_or("CA")
        .to_string();

    let birth_year = 2025 - age;
    let biweekly_pay = annual_income / 26.0;
    let annual_expenses = monthly_expenses * 12.0;
    let post_retirement_annual = annual_expenses * 0.85; // assume 85% of pre-retirement
    let ss_estimate = crate::defaults::estimate_social_security(annual_income);
    let contribution_401k = (annual_income * 0.15).min(23500.0);

    let instructions = format!(
        r#"Build a quick retirement scenario with these tools. Execute each step:

## Step 1: Set Parameters
Call `set_parameters`:
- birth_date: "{birth_year}-06-15"
- start_date: "2026-01-01"
- duration_years: {duration}
- state_abbreviation: "{state}"
- returns_mode: "historical"

## Step 2: Set Portfolio
Call `set_portfolio` with name "Retirement Plan"
Then call `add_account` for each:
1. Checking — type: Checking, value: 25000
2. 401k — type: Traditional401k, assets: [{{ticker: "FXAIX", value: 50000}}]
3. Brokerage — type: Brokerage, assets: [{{ticker: "VTI", value: 30000}}]

## Step 3: Map Tickers
Call `map_tickers` (no overrides needed for common tickers)

## Step 4: Add Events
Call these tools in order:

1. `add_income_event`: name="Salary", to_account="Checking", amount={biweekly_pay:.0}, interval="biweekly", end_event="Retirement"
2. `add_expense_event`: name="Living Expenses", from_account="Checking", amount={monthly_expenses:.0}, interval="monthly"
3. `add_contribution_event`: name="401k Contribution", from_account="Checking", to_account="401k", asset="FXAIX", amount={contribution_401k:.0}, end_event="Retirement"
4. `add_retirement_event`: age={retirement_age}
5. `add_sweep_event`: name="Post-Retirement Spending", to_account="Checking", target_balance={post_retirement_annual:.0}, start_event="Retirement"
6. `add_social_security_event`: monthly_amount={ss_estimate:.0}, start_age=67
7. `add_rmd_event` (use defaults)

## Step 5: Validate & Generate
Call `validate_scenario`, then `merge_scenario`.
"#,
        duration = (retirement_age - age) + 30, // simulate 30 years past retirement
    );

    Ok(GetPromptResult {
        description: Some(format!(
            "Quick retirement plan: age {}, retire at {}, ${:.0}k/yr income",
            age,
            retirement_age,
            annual_income / 1000.0
        )),
        messages: vec![PromptMessage::new_text(
            PromptMessageRole::User,
            instructions,
        )],
    })
}
