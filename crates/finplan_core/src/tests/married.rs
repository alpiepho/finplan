//! Tests for married couple / spouse age trigger support

use crate::config::{AccountBuilder, EventBuilder, SimulationBuilder};
use crate::simulation::simulate;

/// Spouse age trigger fires at the correct date based on spouse_birth_date.
/// Primary born 1975-01-01, spouse born 1980-06-01.
/// SpouseAge trigger at 62 years → spouse reaches 62 on 2042-06-01.
#[test]
fn test_spouse_age_trigger_fires_at_correct_date() {
    let (mut config, _meta) = SimulationBuilder::new()
        .start(2025, 1, 1)
        .years(25)
        .birth_date(1975, 1, 1)
        .spouse_birth_date(1980, 6, 1)
        .account(AccountBuilder::bank_account("Checking").cash(100_000.0))
        .event(
            EventBuilder::income("Spouse SS")
                .to_account("Checking")
                .amount(2_000.0)
                .at_spouse_age(62)
                .monthly(),
        )
        .build();
    config.collect_ledger = true;

    let result = simulate(&config, 42).unwrap();

    // Spouse reaches 62 on 2042-06-01, so income should first appear in 2042
    let first_income_year = result
        .yearly_cash_flows
        .iter()
        .find(|cf| cf.income > 0.0)
        .map(|cf| cf.year);

    assert_eq!(
        first_income_year,
        Some(2042),
        "Spouse SS should start in 2042"
    );
}

/// Primary age trigger still fires at the correct date when spouse_birth_date is also set.
/// Primary born 1970-03-15 → age 67 reached in March 2037.
#[test]
fn test_primary_age_trigger_unaffected_by_spouse() {
    let (mut config, _meta) = SimulationBuilder::new()
        .start(2025, 1, 1)
        .years(20)
        .birth_date(1970, 3, 15)
        .spouse_birth_date(1972, 11, 1)
        .account(AccountBuilder::bank_account("Checking").cash(50_000.0))
        .event(
            EventBuilder::income("My SS")
                .to_account("Checking")
                .amount(3_000.0)
                .at_age(67)
                .monthly(),
        )
        .build();
    config.collect_ledger = true;

    let result = simulate(&config, 42).unwrap();

    // Primary reaches 67 in March 2037
    let first_income_year = result
        .yearly_cash_flows
        .iter()
        .find(|cf| cf.income > 0.0)
        .map(|cf| cf.year);

    assert_eq!(
        first_income_year,
        Some(2037),
        "Primary SS should start in 2037"
    );
}

/// SpouseAge trigger with no spouse_birth_date configured never fires (no panic).
#[test]
fn test_spouse_age_trigger_no_spouse_configured() {
    let (mut config, _meta) = SimulationBuilder::new()
        .start(2025, 1, 1)
        .years(10)
        .birth_date(1970, 1, 1)
        // No spouse_birth_date set
        .account(AccountBuilder::bank_account("Checking").cash(10_000.0))
        .event(
            EventBuilder::income("Spouse SS")
                .to_account("Checking")
                .amount(1_000.0)
                .at_spouse_age(62)
                .monthly(),
        )
        .build();
    config.collect_ledger = true;

    // Should not panic; SpouseAge event simply never fires
    let result = simulate(&config, 42);
    assert!(
        result.is_ok(),
        "simulation with SpouseAge but no spouse DOB should not panic"
    );

    let total_income: f64 = result
        .unwrap()
        .yearly_cash_flows
        .iter()
        .map(|cf| cf.income)
        .sum();
    assert_eq!(
        total_income, 0.0,
        "SpouseAge event should never fire without spouse_birth_date"
    );
}

/// Both primary and spouse SS events fire independently at their respective ages.
#[test]
fn test_both_primary_and_spouse_ss_fire_independently() {
    // Primary born 1970-01-01 → SS at 67 in 2037
    // Spouse born 1975-06-01 → SS at 67 in 2042
    let (mut config, _meta) = SimulationBuilder::new()
        .start(2025, 1, 1)
        .years(25)
        .birth_date(1970, 1, 1)
        .spouse_birth_date(1975, 6, 1)
        .account(AccountBuilder::bank_account("Checking").cash(100_000.0))
        .event(
            EventBuilder::income("My SS")
                .to_account("Checking")
                .amount(3_000.0)
                .at_age(67)
                .monthly(),
        )
        .event(
            EventBuilder::income("Spouse SS")
                .to_account("Checking")
                .amount(2_000.0)
                .at_spouse_age(67)
                .monthly(),
        )
        .build();
    config.collect_ledger = true;

    let result = simulate(&config, 42).unwrap();

    // In 2037, only primary SS fires ($3k/month = $36k/year from Jan)
    let income_2037 = result
        .yearly_cash_flows
        .iter()
        .find(|cf| cf.year == 2037)
        .map(|cf| cf.income)
        .unwrap_or(0.0);
    assert!(
        income_2037 > 0.0,
        "Primary SS should produce income in 2037"
    );

    // In 2042, both are running — income should be higher than 2037
    let income_2042 = result
        .yearly_cash_flows
        .iter()
        .find(|cf| cf.year == 2042)
        .map(|cf| cf.income)
        .unwrap_or(0.0);
    assert!(
        income_2042 > income_2037,
        "Combined SS income in 2042 should exceed primary-only income in 2037"
    );
}
