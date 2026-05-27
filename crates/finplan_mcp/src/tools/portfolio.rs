use rmcp::{ErrorData as McpError, model::*};
use serde_json::{Map, Value, json};

use finplan::data::portfolio_data::{
    AccountData, AccountType, AssetAccount, AssetTag, AssetValue, Debt, PortfolioData, Property,
};
use finplan::data::profiles_data::ReturnProfileTag;

use super::{error_result, make_tool, text_result};
use crate::state::SharedState;

pub fn tools() -> Vec<Tool> {
    vec![
        make_tool(
            "set_portfolio",
            "Initialize the portfolio with a name. Optionally provide accounts inline. \
             After calling this, use add_account to add accounts one at a time.",
            json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Portfolio name (e.g. \"My Retirement Plan\")"
                    },
                    "description": {
                        "type": "string",
                        "description": "Optional portfolio description"
                    }
                },
                "required": ["name"]
            }),
        ),
        make_tool(
            "add_account",
            "Add an account to the portfolio. Supports investment accounts (Brokerage, Traditional401k, \
             Roth401k, TraditionalIRA, RothIRA) with assets, cash accounts (Checking, Savings, HSA, \
             Property, Collectible) with a value, and debt accounts (Mortgage, LoanDebt, StudentLoanDebt) \
             with balance and interest rate.",
            json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Account name (e.g. \"401k\", \"Checking\", \"Mortgage\")"
                    },
                    "account_type": {
                        "type": "string",
                        "enum": [
                            "Brokerage", "Traditional401k", "Roth401k",
                            "TraditionalIRA", "RothIRA",
                            "Checking", "Savings", "HSA", "Property", "Collectible",
                            "Mortgage", "LoanDebt", "StudentLoanDebt"
                        ],
                        "description": "The type of account"
                    },
                    "assets": {
                        "type": "array",
                        "description": "For investment accounts: array of {ticker, value} objects",
                        "items": {
                            "type": "object",
                            "properties": {
                                "ticker": { "type": "string" },
                                "value": { "type": "number" }
                            },
                            "required": ["ticker", "value"]
                        }
                    },
                    "value": {
                        "type": "number",
                        "description": "For cash/property accounts: dollar value"
                    },
                    "return_profile": {
                        "type": "string",
                        "description": "For cash/property accounts: optional return profile name"
                    },
                    "balance": {
                        "type": "number",
                        "description": "For debt accounts: outstanding balance (positive number)"
                    },
                    "interest_rate": {
                        "type": "number",
                        "description": "For debt accounts: annual interest rate as decimal (e.g. 0.065)"
                    },
                    "description": {
                        "type": "string",
                        "description": "Optional account description"
                    }
                },
                "required": ["name", "account_type"]
            }),
        ),
    ]
}

pub fn set_portfolio(
    args: Map<String, Value>,
    state: &SharedState,
) -> Result<CallToolResult, McpError> {
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("name is required", None))?
        .to_string();

    let description = args
        .get("description")
        .and_then(|v| v.as_str())
        .map(String::from);

    let portfolio = PortfolioData {
        name: name.clone(),
        description,
        accounts: Vec::new(),
    };

    let mut st = state.lock().unwrap();
    st.portfolio = Some(portfolio);

    text_result(format!(
        "Portfolio \"{}\" initialized with 0 accounts. Use add_account to add accounts.",
        name
    ))
}

pub fn add_account(
    args: Map<String, Value>,
    state: &SharedState,
) -> Result<CallToolResult, McpError> {
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("name is required", None))?
        .to_string();

    let acct_type_str = args
        .get("account_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("account_type is required", None))?;

    let description = args
        .get("description")
        .and_then(|v| v.as_str())
        .map(String::from);

    let account_type = match acct_type_str {
        // Investment accounts
        "Brokerage" | "Traditional401k" | "Roth401k" | "TraditionalIRA" | "RothIRA" => {
            let assets = parse_assets(&args)?;
            let acct = AssetAccount { assets };
            match acct_type_str {
                "Brokerage" => AccountType::Brokerage(acct),
                "Traditional401k" => AccountType::Traditional401k(acct),
                "Roth401k" => AccountType::Roth401k(acct),
                "TraditionalIRA" => AccountType::TraditionalIRA(acct),
                "RothIRA" => AccountType::RothIRA(acct),
                _ => unreachable!(),
            }
        }
        // Cash/property accounts
        "Checking" | "Savings" | "HSA" | "Property" | "Collectible" => {
            let value = args.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let return_profile = args
                .get("return_profile")
                .and_then(|v| v.as_str())
                .map(|s| ReturnProfileTag(s.to_string()));
            let prop = Property {
                value,
                return_profile,
            };
            match acct_type_str {
                "Checking" => AccountType::Checking(prop),
                "Savings" => AccountType::Savings(prop),
                "HSA" => AccountType::HSA(prop),
                "Property" => AccountType::Property(prop),
                "Collectible" => AccountType::Collectible(prop),
                _ => unreachable!(),
            }
        }
        // Debt accounts
        "Mortgage" | "LoanDebt" | "StudentLoanDebt" => {
            let balance = args.get("balance").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let interest_rate = args
                .get("interest_rate")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let debt = Debt {
                balance,
                interest_rate,
            };
            match acct_type_str {
                "Mortgage" => AccountType::Mortgage(debt),
                "LoanDebt" => AccountType::LoanDebt(debt),
                "StudentLoanDebt" => AccountType::StudentLoanDebt(debt),
                _ => unreachable!(),
            }
        }
        _ => {
            return error_result(format!("Unknown account type: {}", acct_type_str));
        }
    };

    let account = AccountData {
        name: name.clone(),
        description,
        owner: Default::default(),
        account_type,
    };

    let mut st = state.lock().unwrap();
    let portfolio = st
        .portfolio
        .as_mut()
        .ok_or_else(|| McpError::invalid_params("Call set_portfolio first", None))?;

    // Check for duplicate
    if portfolio.accounts.iter().any(|a| a.name == name) {
        return error_result(format!(
            "Account \"{}\" already exists. Use a different name.",
            name
        ));
    }

    portfolio.accounts.push(account);

    text_result(format!(
        "Added {} account \"{}\". Portfolio now has {} accounts.",
        acct_type_str,
        name,
        portfolio.accounts.len()
    ))
}

fn parse_assets(args: &Map<String, Value>) -> Result<Vec<AssetValue>, McpError> {
    let Some(assets_val) = args.get("assets") else {
        return Ok(Vec::new());
    };
    let arr = assets_val
        .as_array()
        .ok_or_else(|| McpError::invalid_params("assets must be an array", None))?;

    let mut assets = Vec::new();
    for item in arr {
        let ticker = item
            .get("ticker")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("each asset needs a ticker", None))?;
        let value = item
            .get("value")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| McpError::invalid_params("each asset needs a value", None))?;
        assets.push(AssetValue {
            asset: AssetTag(ticker.to_string()),
            value,
        });
    }
    Ok(assets)
}
