/// Scenario YAML validation module
/// 
/// Validates scenario files for common configuration errors and provides
/// helpful error messages to guide users in fixing their YAML.

use super::app_data::SimulationData;
use super::events_data::EffectData;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub section: String,
    pub message: String,
    pub help: Option<String>,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} → {}", self.section, self.message)?;
        if let Some(help) = &self.help {
            write!(f, "\n  Help: {}", help)?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationError {}

/// Validate a scenario's structure and content
pub fn validate_scenario(data: &SimulationData) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // Validate portfolio
    if let Err(mut portfolio_errors) = validate_portfolio(data) {
        errors.append(&mut portfolio_errors);
    }

    // Validate profiles and assets
    if let Err(mut profile_errors) = validate_profiles(data) {
        errors.append(&mut profile_errors);
    }

    // Validate events
    if let Err(mut event_errors) = validate_events(data) {
        errors.append(&mut event_errors);
    }

    // Validate parameters
    if let Err(mut param_errors) = validate_parameters(data) {
        errors.append(&mut param_errors);
    }

    // Validate analysis
    if let Err(mut analysis_errors) = validate_analysis(data) {
        errors.append(&mut analysis_errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_portfolio(data: &SimulationData) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    let portfolio = &data.portfolios;

    // Check portfolio name
    if portfolio.name.trim().is_empty() {
        errors.push(ValidationError {
            section: "portfolios".to_string(),
            message: "Portfolio name is empty".to_string(),
            help: Some("Set portfolios.name to a descriptive string, e.g., 'My Retirement Plan'".to_string()),
        });
    }

    // Check that at least one account exists
    if portfolio.accounts.is_empty() {
        errors.push(ValidationError {
            section: "portfolios.accounts".to_string(),
            message: "No accounts defined".to_string(),
            help: Some("Define at least one account (Checking, Savings, 401k, Brokerage, etc.) under portfolios.accounts".to_string()),
        });
    }

    // Collect all account names for reference validation
    let _account_names: HashSet<_> = portfolio.accounts.iter().map(|a| a.name.as_str()).collect();

    // Validate individual accounts
    for (idx, account) in portfolio.accounts.iter().enumerate() {
        if account.name.trim().is_empty() {
            errors.push(ValidationError {
                section: format!("portfolios.accounts[{}]", idx),
                message: "Account name is empty".to_string(),
                help: Some("Each account must have a non-empty name".to_string()),
            });
        }

        // Check for duplicate account names
        let count = portfolio.accounts.iter().filter(|a| a.name == account.name).count();
        if count > 1 {
            errors.push(ValidationError {
                section: format!("portfolios.accounts[{}]", idx),
                message: format!("Duplicate account name: '{}'", account.name),
                help: Some("Account names must be unique".to_string()),
            });
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_profiles(data: &SimulationData) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // Collect all profile names
    let profile_names: HashSet<_> = data.profiles.iter().map(|p| p.name.0.as_str()).collect();

    // Built-in historical profiles for historical mode
    let builtin_historical_profiles = HashSet::from([
        "S&P 500",
        "US Small Cap",
        "Intl Developed",
        "Intl Emerging",
        "US Bonds",
        "Intl Bonds",
        "Real Estate (REITs)",
        "Commodities",
    ]);

    // Check regular assets point to defined profiles
    for (asset, profile) in &data.assets {
        if !profile_names.contains(profile.0.as_str()) {
            errors.push(ValidationError {
                section: "assets".to_string(),
                message: format!("Asset '{}' references unknown profile '{}'", asset.0, profile.0),
                help: Some(format!(
                    "Define profile '{}' under profiles section, or use a built-in profile name",
                    profile.0
                )),
            });
        }
    }

    // Check historical assets point to valid built-in profiles (when in historical mode)
    for (asset, profile) in &data.historical_assets {
        if !builtin_historical_profiles.contains(profile.0.as_str()) {
            errors.push(ValidationError {
                section: "historical_assets".to_string(),
                message: format!("Asset '{}' references unknown historical profile '{}'", asset.0, profile.0),
                help: Some(format!(
                    "When using returns_mode: historical, asset profiles must match built-in names. Available: {}",
                    builtin_historical_profiles.iter()
                        .map(|s| format!("'{}'", s))
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
            });
        }
    }

    // Validate profile definitions
    for (idx, profile) in data.profiles.iter().enumerate() {
        if profile.name.0.trim().is_empty() {
            errors.push(ValidationError {
                section: format!("profiles[{}]", idx),
                message: "Profile name is empty".to_string(),
                help: Some("Each profile must have a non-empty name".to_string()),
            });
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_events(data: &SimulationData) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    let account_names: HashSet<_> = data.portfolios.accounts.iter().map(|a| a.name.as_str()).collect();
    let event_names: HashSet<_> = data.events.iter().map(|e| e.name.0.as_str()).collect();

    for (idx, event) in data.events.iter().enumerate() {
        if event.name.0.trim().is_empty() {
            errors.push(ValidationError {
                section: format!("events[{}]", idx),
                message: "Event name is empty".to_string(),
                help: Some("Each event must have a non-empty name".to_string()),
            });
        }

        // Check for duplicate event names
        let count = data.events.iter().filter(|e| e.name == event.name).count();
        if count > 1 {
            errors.push(ValidationError {
                section: format!("events[{}]", idx),
                message: format!("Duplicate event name: '{}'", event.name.0),
                help: Some("Event names must be unique".to_string()),
            });
        }

        // Note: Events may have no effects if they serve as markers for other events to reference

        // Validate effects reference valid accounts and events
        for (effect_idx, effect) in event.effects.iter().enumerate() {
            validate_effect(
                effect,
                effect_idx,
                idx,
                &account_names,
                &event_names,
                &mut errors,
            );
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_effect(
    effect: &EffectData,
    effect_idx: usize,
    event_idx: usize,
    account_names: &HashSet<&str>,
    event_names: &HashSet<&str>,
    errors: &mut Vec<ValidationError>,
) {
    match effect {
        EffectData::Income { to, .. } => {
            if !account_names.contains(to.0.as_str()) {
                errors.push(ValidationError {
                    section: format!("events[{}].effects[{}].to", event_idx, effect_idx),
                    message: format!("References unknown account '{}'", to.0),
                    help: Some(format!(
                        "Account '{}' is not defined in portfolios.accounts",
                        to.0
                    )),
                });
            }
        }
        EffectData::Expense { from, .. } => {
            if !account_names.contains(from.0.as_str()) {
                errors.push(ValidationError {
                    section: format!("events[{}].effects[{}].from", event_idx, effect_idx),
                    message: format!("References unknown account '{}'", from.0),
                    help: Some(format!(
                        "Account '{}' is not defined in portfolios.accounts",
                        from.0
                    )),
                });
            }
        }
        EffectData::AssetPurchase {
            from,
            to_account,
            ..
        } => {
            if !account_names.contains(from.0.as_str()) {
                errors.push(ValidationError {
                    section: format!("events[{}].effects[{}].from", event_idx, effect_idx),
                    message: format!("References unknown account '{}'", from.0),
                    help: None,
                });
            }
            if !account_names.contains(to_account.0.as_str()) {
                errors.push(ValidationError {
                    section: format!("events[{}].effects[{}].to_account", event_idx, effect_idx),
                    message: format!("References unknown account '{}'", to_account.0),
                    help: None,
                });
            }
        }
        EffectData::AssetSale { from, .. } => {
            if !account_names.contains(from.0.as_str()) {
                errors.push(ValidationError {
                    section: format!("events[{}].effects[{}].from", event_idx, effect_idx),
                    message: format!("References unknown account '{}'", from.0),
                    help: None,
                });
            }
        }
        EffectData::Sweep { to, .. } => {
            if !account_names.contains(to.0.as_str()) {
                errors.push(ValidationError {
                    section: format!("events[{}].effects[{}].to", event_idx, effect_idx),
                    message: format!("References unknown account '{}'", to.0),
                    help: None,
                });
            }
        }
        EffectData::TriggerEvent { event } => {
            if !event_names.contains(event.0.as_str()) {
                errors.push(ValidationError {
                    section: format!("events[{}].effects[{}].event", event_idx, effect_idx),
                    message: format!("References unknown event '{}'", event.0),
                    help: Some(format!(
                        "Event '{}' is not defined in events section",
                        event.0
                    )),
                });
            }
        }
        EffectData::PauseEvent { event } => {
            if !event_names.contains(event.0.as_str()) {
                errors.push(ValidationError {
                    section: format!("events[{}].effects[{}].event", event_idx, effect_idx),
                    message: format!("References unknown event '{}'", event.0),
                    help: None,
                });
            }
        }
        EffectData::ResumeEvent { event } => {
            if !event_names.contains(event.0.as_str()) {
                errors.push(ValidationError {
                    section: format!("events[{}].effects[{}].event", event_idx, effect_idx),
                    message: format!("References unknown event '{}'", event.0),
                    help: None,
                });
            }
        }
        EffectData::TerminateEvent { event } => {
            if !event_names.contains(event.0.as_str()) {
                errors.push(ValidationError {
                    section: format!("events[{}].effects[{}].event", event_idx, effect_idx),
                    message: format!("References unknown event '{}'", event.0),
                    help: None,
                });
            }
        }
        EffectData::ApplyRmd { destination, .. } => {
            if !account_names.contains(destination.0.as_str()) {
                errors.push(ValidationError {
                    section: format!("events[{}].effects[{}].destination", event_idx, effect_idx),
                    message: format!("References unknown account '{}'", destination.0),
                    help: None,
                });
            }
        }
        EffectData::AdjustBalance { account, .. } => {
            if !account_names.contains(account.0.as_str()) {
                errors.push(ValidationError {
                    section: format!("events[{}].effects[{}].account", event_idx, effect_idx),
                    message: format!("References unknown account '{}'", account.0),
                    help: None,
                });
            }
        }
        EffectData::CashTransfer { from, to, .. } => {
            if !account_names.contains(from.0.as_str()) {
                errors.push(ValidationError {
                    section: format!("events[{}].effects[{}].from", event_idx, effect_idx),
                    message: format!("References unknown account '{}'", from.0),
                    help: None,
                });
            }
            if !account_names.contains(to.0.as_str()) {
                errors.push(ValidationError {
                    section: format!("events[{}].effects[{}].to", event_idx, effect_idx),
                    message: format!("References unknown account '{}'", to.0),
                    help: None,
                });
            }
        }
        EffectData::Random {
            on_true,
            on_false,
            ..
        } => {
            if !event_names.contains(on_true.0.as_str()) {
                errors.push(ValidationError {
                    section: format!("events[{}].effects[{}].on_true", event_idx, effect_idx),
                    message: format!("References unknown event '{}'", on_true.0),
                    help: None,
                });
            }
            if let Some(event) = on_false {
                if !event_names.contains(event.0.as_str()) {
                    errors.push(ValidationError {
                        section: format!("events[{}].effects[{}].on_false", event_idx, effect_idx),
                        message: format!("References unknown event '{}'", event.0),
                        help: None,
                    });
                }
            }
        }
        EffectData::RsuVesting { to, .. } => {
            if !account_names.contains(to.0.as_str()) {
                errors.push(ValidationError {
                    section: format!("events[{}].effects[{}].to", event_idx, effect_idx),
                    message: format!("References unknown account '{}'", to.0),
                    help: None,
                });
            }
        }
    }
}

fn validate_parameters(data: &SimulationData) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    let params = &data.parameters;

    // Validate birth date
    if params.birth_date.is_empty() {
        errors.push(ValidationError {
            section: "parameters.birth_date".to_string(),
            message: "Birth date is empty".to_string(),
            help: Some("Set to YYYY-MM-DD format, e.g., '1965-03-15'".to_string()),
        });
    } else if !params.birth_date.contains('-') || params.birth_date.split('-').count() != 3 {
        errors.push(ValidationError {
            section: "parameters.birth_date".to_string(),
            message: format!("Invalid date format: '{}'", params.birth_date),
            help: Some("Use YYYY-MM-DD format, e.g., '1965-03-15'".to_string()),
        });
    }

    // Validate start date
    if params.start_date.is_empty() {
        errors.push(ValidationError {
            section: "parameters.start_date".to_string(),
            message: "Start date is empty".to_string(),
            help: Some("Set to YYYY-MM-DD format, e.g., '2025-01-01'".to_string()),
        });
    } else if !params.start_date.contains('-') || params.start_date.split('-').count() != 3 {
        errors.push(ValidationError {
            section: "parameters.start_date".to_string(),
            message: format!("Invalid date format: '{}'", params.start_date),
            help: Some("Use YYYY-MM-DD format, e.g., '2025-01-01'".to_string()),
        });
    }

    // Validate duration
    if params.duration_years == 0 {
        errors.push(ValidationError {
            section: "parameters.duration_years".to_string(),
            message: "Duration must be positive".to_string(),
            help: Some("Set duration_years to a positive number, e.g., 40".to_string()),
        });
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_analysis(data: &SimulationData) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    let analysis = &data.analysis;

    // Validate MC iterations
    if analysis.mc_iterations < 10 {
        errors.push(ValidationError {
            section: "analysis.mc_iterations".to_string(),
            message: format!("Too few Monte Carlo iterations: {}", analysis.mc_iterations),
            help: Some("Use at least 100 iterations for reasonable accuracy. 1000 is recommended.".to_string()),
        });
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_accounts() {
        let mut data = SimulationData::default();
        data.portfolios.accounts.clear();

        let result = validate_scenario(&data);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.message.contains("No accounts defined")));
    }
}
