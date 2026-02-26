# FinPlan Scenario YAML Validator

## Overview

The FinPlan application now includes a comprehensive YAML validator that checks scenario files for configuration errors before they're used in the simulation. The validator provides detailed error messages with helpful guidance for fixing issues.

## Using the Validator

### Command-Line Usage

Validate a scenario file without running the application:

```bash
finplan --scenario path/to/scenario.yaml
```

This will:
- Parse the YAML file
- Run semantic validation checks
- Report any errors found with detailed guidance
- Exit with code 0 if valid, 1 if invalid

Example with valid scenario:
```bash
$ finplan --scenario examples/example.yaml
✓ Scenario is valid!
```

Example with invalid scenario:
```bash
$ finplan --scenario test_invalid.yaml
✗ Scenario validation failed:
Validation errors found:
  • portfolios.accounts → No accounts defined
  Help: Define at least one account (Checking, Savings, 401k, Brokerage, etc.) under portfolios.accounts
  • events[0].effects[0].to → References unknown account 'NonExistentAccount'
  Help: Account 'NonExistentAccount' is not defined in portfolios.accounts
```

### Import Validation

When importing a scenario file through the TUI application, the same validation is automatically performed. If validation fails, a clear error message is displayed explaining what needs to be fixed.

## Validation Checks

The validator performs the following semantic checks:

### Portfolio Validation
- ✓ Portfolio name is not empty
- ✓ At least one account is defined
- ✓ Account names are not empty
- ✓ Account names are unique

### Profile Validation
- ✓ Profile names are not empty
- ✓ Assets reference defined profiles
- ✓ Historical assets reference built-in profile names (when using historical mode):
  - S&P 500
  - US Small Cap
  - Intl Developed
  - Intl Emerging
  - US Bonds
  - Intl Bonds
  - Real Estate (REITs)
  - Commodities

### Event Validation
- ✓ Event names are not empty
- ✓ Event names are unique
- ✓ Event effects reference defined accounts
- ✓ Event trigger and effect account references are valid
- ✓ Events that trigger other events reference defined events

### Parameter Validation
- ✓ Birth date is in YYYY-MM-DD format
- ✓ Start date is in YYYY-MM-DD format
- ✓ Duration is a positive number

### Analysis Validation
- ✓ Monte Carlo iterations >= 10 (recommends >= 100)

## Error Messages

The validator provides context-specific error messages with:

1. **Section identifier**: Where in the YAML the error occurred (e.g., `events[0].effects[0].to`)
2. **Error message**: What is wrong
3. **Help text**: How to fix it (when applicable)

### Example: Multiple Validation Errors

```
✗ Scenario validation failed:
Validation errors found:
  • portfolios.accounts[1] → Account name is empty
  Help: Each account must have a non-empty name
  • assets → Asset 'Stock2' references unknown profile 'Nonexistent Profile'
  Help: Define profile 'Nonexistent Profile' under profiles section, or use a built-in profile name
  • events[0].effects[0].to → References unknown account 'InvalidAccount'
  Help: Account 'InvalidAccount' is not defined in portfolios.accounts
  • parameters.birth_date → Invalid date format: 'invalid-date'
  Help: Use YYYY-MM-DD format, e.g., '1965-03-15'
  • analysis.mc_iterations → Too few Monte Carlo iterations: 5
  Help: Use at least 100 iterations for reasonable accuracy. 1000 is recommended.
```

## Implementation Details

### Files Modified/Created

- **`crates/finplan/src/data/validator.rs`**: Core validation module with all semantic checks
- **`crates/finplan/src/lib.rs`**: Public `validate_scenario_file()` function
- **`crates/finplan/src/main.rs`**: Command-line argument parsing for `--scenario` and `--validate`
- **`crates/finplan/src/data/storage.rs`**: Integration of validator into import flow

### Validation Flow

1. **YAML Parsing**: serde_saphyr deserializes YAML to `SimulationData` struct
   - Catches syntax errors, missing required fields, type mismatches
2. **Semantic Validation**: `validator::validate_scenario()` checks business logic
   - Cross-references between sections
   - Valid values and formats
   - Account/event/profile existence
3. **Error Reporting**: All errors collected and reported together

### Validator Architecture

The validator is modular with separate functions for each section:

- `validate_portfolio()`: Checks accounts are defined and valid
- `validate_profiles()`: Checks profile references and definitions
- `validate_events()`: Checks account/event references, triggers, and effects
- `validate_parameters()`: Checks date formats and parameter ranges
- `validate_analysis()`: Checks iteration counts and configuration

Each function returns `Result<(), Vec<ValidationError>>` to collect all errors.

## Testing

Test files are included in the project root:

- `test_invalid.yaml`: Simple invalid scenario (empty accounts, bad references)
- `test_complex_errors.yaml`: Multiple validation errors across all sections

Validate them:
```bash
finplan --scenario test_invalid.yaml
finplan --scenario test_complex_errors.yaml
```

## Benefits

1. **Early Error Detection**: Catch configuration problems before simulation runs
2. **Clear Guidance**: Error messages explain what's wrong and how to fix it
3. **Batch Error Reporting**: See all issues at once, not one at a time
4. **Scriptable**: CLI validation enables integration with CI/CD pipelines
5. **Better UX**: Import failures in the app show helpful error messages

## Future Enhancements

Potential validator improvements:
- Warn about unused profiles or events
- Validate date ranges and logical constraints
- Check for circular event references
- Validate monetary amounts (warning for suspiciously high/low values)
- Performance profiling for large scenarios
- Suggest corrections for typos in account/event names
