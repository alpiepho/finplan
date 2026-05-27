# Married Couple Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add first-class married couple support so spouses each have their own birth date, age-triggered events, account ownership, and correct early withdrawal / RMD calculations.

**Architecture:** Add `spouse_birth_date` to `SimulationConfig` and `SimTimeline`, add `EventTrigger::SpouseAge` for spouse age-based triggers, add a `Person` enum to `Account` for ownership so early-withdrawal penalties and RMDs use the correct person's age. Extend the TUI parameters form and MCP parameter tool to expose spouse fields.

**Tech Stack:** Rust, `jiff` (date math), ratatui (TUI), serde (JSON/YAML serialization)

---

## Problem Summary

The app already supports `married_joint2024` federal tax brackets, but has no way to model a spouse as a second person. All age-based logic keys off a single `birth_date`:

| Gap | Location | Impact |
|-----|----------|--------|
| `EventTrigger::Age` uses only primary birth_date | `simulation_state.rs`, `evaluate.rs` | Can't trigger "spouse retires at 62" |
| `is_below_early_withdrawal_age()` uses primary birth_date | `SimTimeline` | Spouse's 401k gets wrong 10% penalty |
| `current_age()` / RMD uses primary birth_date | `simulation_state.rs:619` | Spouse's IRA RMDs computed at wrong age |
| `SimulationConfig::birth_date` is single field | `config/mod.rs:141` | No place to store spouse birth date |
| TUI "Edit Parameters" form has one birth date field | `screens/scenario.rs:296` | No UI to enter spouse DOB |
| MCP `set_portfolio_overview` has no spouse DOB | `tools/parameters.rs` | Can't configure via LLM |

---

## File Map

| File | Change |
|------|--------|
| `crates/finplan_core/src/config/mod.rs` | Add `spouse_birth_date: Option<Date>` to `SimulationConfig` |
| `crates/finplan_core/src/model/events.rs` | Add `EventTrigger::SpouseAge { years, months }` |
| `crates/finplan_core/src/model/accounts.rs` | Add `Person` enum; add `owner: Person` to `Account` |
| `crates/finplan_core/src/simulation_state.rs` | `SimTimeline` gets `spouse_birth_date`; new methods `spouse_age()`, `is_spouse_below_early_withdrawal_age()`; `collect_age_trigger_dates` handles `SpouseAge` |
| `crates/finplan_core/src/evaluate.rs` | Handle `EventTrigger::SpouseAge` in trigger evaluation |
| `crates/finplan_core/src/apply.rs` | Early withdrawal penalty uses account owner's birth_date |
| `crates/finplan_core/src/config/builder.rs` | `.spouse_birth_date()` method on `SimulationBuilder` |
| `crates/finplan_core/src/config/account_builder.rs` | `.owner()` method on `AccountBuilder` |
| `crates/finplan_core/src/tests/` | New `married.rs` test file |
| `crates/finplan/src/data/parameters_data.rs` | `spouse_birth_date: Option<String>` in `ParametersData` |
| `crates/finplan/src/screens/scenario.rs` | Add spouse birth date field to Edit Parameters form |
| `crates/finplan/src/data/convert.rs` | Pass `spouse_birth_date` to `SimulationConfig` |
| `crates/finplan/src/actions/event.rs` | Add "Spouse Social Security" event template |
| `crates/finplan_mcp/src/tools/parameters.rs` | Add `spouse_birth_date` param |
| `crates/finplan_mcp/src/schema_text.rs` | Document spouse fields |

---

## Phase 1: Core Data Model

### Task 1: Add `spouse_birth_date` to `SimulationConfig`

**Files:**
- Modify: `crates/finplan_core/src/config/mod.rs`

- [ ] **Step 1: Add the field**

  In `mod.rs`, after the existing `birth_date` field (line ~141), add:

  ```rust
  /// Birth date of spouse for age-based triggers and RMD calculations on spouse-owned accounts
  pub spouse_birth_date: Option<jiff::civil::Date>,
  ```

  In `Default::default()`, after `birth_date: None,` add:

  ```rust
  spouse_birth_date: None,
  ```

- [ ] **Step 2: Add `spouse_age()` helper**

  After the existing `initial_age()` method (line ~260), add:

  ```rust
  /// Calculate spouse's age at start date, if spouse_birth_date is set
  #[must_use]
  pub fn spouse_initial_age(&self) -> Option<u8> {
      let birth = self.spouse_birth_date?;
      let start = self.start_date?;
      let years = start.year() - birth.year();

      if start.month() < birth.month()
          || (start.month() == birth.month() && start.day() < birth.day())
      {
          Some((years - 1) as u8)
      } else {
          Some(years as u8)
      }
  }
  ```

- [ ] **Step 3: Build and verify**

  ```bash
  cargo build -p finplan_core 2>&1 | head -30
  ```
  Expected: clean build (no warnings about missing fields because it's `Option` with default).

- [ ] **Step 4: Commit**

  ```bash
  git add crates/finplan_core/src/config/mod.rs
  git commit -m "feat(core): add spouse_birth_date to SimulationConfig"
  ```

---

### Task 2: Add `EventTrigger::SpouseAge`

**Files:**
- Modify: `crates/finplan_core/src/model/events.rs`

- [ ] **Step 1: Add the variant**

  In `events.rs`, after the `Age` variant (line ~353):

  ```rust
  /// Trigger at a specific spouse age (requires `spouse_birth_date` in `SimulationParameters`)
  SpouseAge { years: u8, months: Option<u8> },
  ```

- [ ] **Step 2: Build to find all match arms that need updating**

  ```bash
  cargo build -p finplan_core 2>&1 | grep "error\[E0004\]\|non-exhaustive"
  ```
  Expected: errors listing every `match trigger` that is non-exhaustive. Work through each one in the tasks below.

- [ ] **Step 3: Commit**

  ```bash
  git add crates/finplan_core/src/model/events.rs
  git commit -m "feat(core): add EventTrigger::SpouseAge variant"
  ```

---

### Task 3: Update `SimTimeline` with spouse birth date

**Files:**
- Modify: `crates/finplan_core/src/simulation_state.rs`

- [ ] **Step 1: Add field to `SimTimeline`**

  In `SimTimeline` struct (line ~43), add:

  ```rust
  pub spouse_birth_date: Option<jiff::civil::Date>,
  ```

- [ ] **Step 2: Add `is_spouse_below_early_withdrawal_age()` method**

  After the existing `is_below_early_withdrawal_age()` method (line ~76), add:

  ```rust
  /// Check if the spouse is below early withdrawal age (59.5).
  /// Returns `None` if `spouse_birth_date` is not set.
  #[must_use]
  pub fn is_spouse_below_early_withdrawal_age(&self) -> Option<bool> {
      let birth = self.spouse_birth_date?;
      let mut years = self.current_date.year() - birth.year();
      let mut months = i32::from(self.current_date.month()) - i32::from(birth.month());

      if self.current_date.month() < birth.month()
          || (self.current_date.month() == birth.month()
              && self.current_date.day() < birth.day())
      {
          years -= 1;
          months += 12;
      }

      if months < 0 {
          months += 12;
      }

      Some(years < 59 || (years == 59 && months < 6))
  }
  ```

- [ ] **Step 3: Wire `spouse_birth_date` into `SimTimeline` initialisation**

  In `SimulationState::from_parameters()`, at the `SimTimeline { ... }` literal (line ~458):

  ```rust
  timeline: SimTimeline {
      current_date: start_date,
      start_date,
      end_date,
      birth_date: params.birth_date.unwrap_or(jiff::civil::date(1970, 1, 1)),
      spouse_birth_date: params.spouse_birth_date,  // NEW
  },
  ```

- [ ] **Step 4: Add `spouse_age()` to `SimulationState`**

  After the existing `current_age()` method (line ~640), add:

  ```rust
  /// Get spouse's current age in years and months.
  /// Returns `None` if `spouse_birth_date` is not configured.
  pub fn spouse_age(&self) -> Option<(u8, u8)> {
      let birth = self.timeline.spouse_birth_date?;
      let mut years = self.timeline.current_date.year() - birth.year();
      let mut months = i32::from(self.timeline.current_date.month()) - i32::from(birth.month());

      if self.timeline.current_date.month() < birth.month()
          || (self.timeline.current_date.month() == birth.month()
              && self.timeline.current_date.day() < birth.day())
      {
          years -= 1;
          months += 12;
      }

      if months < 0 {
          months += 12;
      }

      Some((years as u8, months as u8))
  }
  ```

- [ ] **Step 5: Update `collect_age_trigger_dates` to handle `SpouseAge`**

  Change the function signature to accept `spouse_birth_date`:

  ```rust
  fn collect_age_trigger_dates(
      trigger: &EventTrigger,
      birth_date: jiff::civil::Date,
      spouse_birth_date: Option<jiff::civil::Date>,
      age_dates: &mut Vec<jiff::civil::Date>,
  ) {
  ```

  Add an arm for `SpouseAge` inside the match:

  ```rust
  EventTrigger::SpouseAge { years, months } => {
      if let Some(spouse_birth) = spouse_birth_date {
          let target_months = months.unwrap_or(0);
          let total_months = i32::from(*years) * 12 + i32::from(target_months);
          let trigger_date =
              crate::model::TriggerOffset::Months(total_months).add_to_date(spouse_birth);
          age_dates.push(trigger_date);
      }
  }
  ```

  Also update all recursive calls to pass `spouse_birth_date`:

  ```rust
  EventTrigger::Repeating { start_condition, end_condition, .. } => {
      if let Some(cond) = start_condition {
          collect_age_trigger_dates(cond, birth_date, spouse_birth_date, age_dates);
      }
      if let Some(cond) = end_condition {
          collect_age_trigger_dates(cond, birth_date, spouse_birth_date, age_dates);
      }
  }
  EventTrigger::And(triggers) | EventTrigger::Or(triggers) => {
      for t in triggers {
          collect_age_trigger_dates(t, birth_date, spouse_birth_date, age_dates);
      }
  }
  ```

  Update the call site in `from_parameters()` (line ~444):

  ```rust
  collect_age_trigger_dates(
      &event.trigger,
      birth_date,
      params.spouse_birth_date,
      &mut age_dates,
  );
  ```

- [ ] **Step 6: Build and verify**

  ```bash
  cargo build -p finplan_core 2>&1 | grep "^error" | head -20
  ```
  Expected: Only errors from evaluate.rs (non-exhaustive match on `SpouseAge`) — fixed in Task 4.

- [ ] **Step 7: Commit**

  ```bash
  git add crates/finplan_core/src/simulation_state.rs
  git commit -m "feat(core): SimTimeline carries spouse_birth_date; add spouse_age() and SpouseAge date caching"
  ```

---

### Task 4: Handle `EventTrigger::SpouseAge` in `evaluate.rs`

**Files:**
- Modify: `crates/finplan_core/src/evaluate.rs`

- [ ] **Step 1: Add match arm for `SpouseAge`**

  In the `evaluate_trigger` function, after the `EventTrigger::Age { .. }` arm (line ~152), add:

  ```rust
  EventTrigger::SpouseAge { .. } => {
      // Use same pre-computed cache slot as Age triggers (SpouseAge shares the slot)
      if let Some(trigger_date) = state.event_state.age_trigger_date(*event_id) {
          if state.timeline.current_date >= trigger_date {
              Ok(TriggerEvent::Triggered)
          } else {
              Ok(TriggerEvent::NextTriggerDate(trigger_date))
          }
      } else {
          // No spouse_birth_date configured — SpouseAge events never fire
          Ok(TriggerEvent::NotTriggered)
      }
  }
  ```

  Also scan the file for any `collect_age_trigger_dates` helper calls (in the analysis evaluator) and add the `spouse_birth_date` argument:

  ```bash
  grep -n "collect_age_trigger_dates" crates/finplan_core/src/evaluate.rs
  ```

  Update each call to:

  ```rust
  collect_age_trigger_dates(&event.trigger, birth_date, spouse_birth_date, &mut age_dates);
  ```

  The `analysis/evaluator.rs` file uses `birth_year` from `base_config.birth_date`. If it calls `collect_age_trigger_dates` (check with grep), update those calls too.

- [ ] **Step 2: Build cleanly**

  ```bash
  cargo build -p finplan_core 2>&1 | grep "^error" | head -20
  ```
  Expected: clean build.

- [ ] **Step 3: Commit**

  ```bash
  git add crates/finplan_core/src/evaluate.rs
  git commit -m "feat(core): evaluate EventTrigger::SpouseAge using pre-cached trigger date"
  ```

---

### Task 5: Write integration tests for spouse age triggers

**Files:**
- Create: `crates/finplan_core/src/tests/married.rs`
- Modify: `crates/finplan_core/src/tests/mod.rs` (add `mod married;`)

- [ ] **Step 1: Write failing tests**

  Create `crates/finplan_core/src/tests/married.rs`:

  ```rust
  //! Tests for married couple / spouse age trigger support

  use crate::config::{AccountBuilder, AssetBuilder, EventBuilder, SimulationBuilder};
  use crate::model::{EventTrigger, MonteCarloSummary};
  use crate::simulation::monte_carlo_simulate_with_config;
  use crate::tests::prelude::*;

  /// Spouse age trigger fires at the correct date based on spouse_birth_date.
  #[test]
  fn test_spouse_age_trigger_fires_at_correct_date() {
      // Primary born 1975-01-01, spouse born 1980-06-01.
      // SpouseAge trigger at 62 years → spouse reaches 62 on 2042-06-01.
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
                  .monthly()
                  .trigger(EventTrigger::SpouseAge { years: 62, months: None }),
          )
          .build();
      config.collect_ledger = true;

      let result = crate::simulation::simulate(&config, 42).unwrap();

      // Income from "Spouse SS" should appear in or after 2042
      let first_ss_year = result
          .yearly_cash_flows
          .iter()
          .find(|cf| cf.income > 0.0)
          .map(|cf| cf.year);

      assert_eq!(first_ss_year, Some(2042), "Spouse SS should start in 2042");
  }

  /// Primary age trigger still works correctly when spouse_birth_date is set.
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
                  .monthly()
                  .until_age(90)
                  .trigger_age(67),
          )
          .build();
      config.collect_ledger = true;

      let result = crate::simulation::simulate(&config, 42).unwrap();

      // Primary born 1970-03-15, age 67 → March 2037
      let first_ss_year = result
          .yearly_cash_flows
          .iter()
          .find(|cf| cf.income > 0.0)
          .map(|cf| cf.year);

      assert_eq!(first_ss_year, Some(2037), "Primary SS should start in 2037");
  }

  /// SpouseAge trigger with no spouse_birth_date configured never fires (no panic).
  #[test]
  fn test_spouse_age_trigger_no_spouse_configured() {
      let (mut config, _meta) = SimulationBuilder::new()
          .start(2025, 1, 1)
          .years(10)
          .birth_date(1970, 1, 1)
          // No spouse_birth_date
          .account(AccountBuilder::bank_account("Checking").cash(10_000.0))
          .event(
              EventBuilder::income("Spouse SS")
                  .to_account("Checking")
                  .amount(1_000.0)
                  .monthly()
                  .trigger(EventTrigger::SpouseAge { years: 62, months: None }),
          )
          .build();
      config.collect_ledger = true;

      // Should not panic; SpouseAge event simply never fires
      let result = crate::simulation::simulate(&config, 42);
      assert!(result.is_ok(), "simulation with SpouseAge but no spouse DOB should not panic");

      let total_income: f64 = result
          .unwrap()
          .yearly_cash_flows
          .iter()
          .map(|cf| cf.income)
          .sum();
      assert_eq!(total_income, 0.0, "SpouseAge event should never fire without spouse_birth_date");
  }
  ```

  Add `mod married;` to `crates/finplan_core/src/tests/mod.rs`.

- [ ] **Step 2: Run tests (expect failure until `SimulationBuilder::spouse_birth_date` exists)**

  ```bash
  cargo test -p finplan_core -- married 2>&1 | tail -20
  ```
  Expected: compile error — `spouse_birth_date` not yet on `SimulationBuilder`.

- [ ] **Step 3: Add `spouse_birth_date()` to `SimulationBuilder`**

  In `crates/finplan_core/src/config/builder.rs`, after the `.birth_date()` method, add:

  ```rust
  /// Set spouse's birth date for age-based calculations on spouse-owned accounts/events.
  #[must_use]
  pub fn spouse_birth_date(mut self, year: i16, month: i8, day: i8) -> Self {
      self.config.spouse_birth_date = Some(jiff::civil::date(year, month, day));
      self
  }
  ```

- [ ] **Step 4: Add `.trigger()` escape hatch to `EventBuilder` (if not present)**

  Check if `EventBuilder` in `event_builder.rs` supports arbitrary `EventTrigger`. If not, add:

  ```rust
  /// Override with a specific trigger (for advanced use cases like SpouseAge).
  #[must_use]
  pub fn trigger(mut self, trigger: EventTrigger) -> Self {
      self.trigger = TriggerSpec::Raw(trigger);
      self
  }
  ```

  And add `Raw(EventTrigger)` to `TriggerSpec` enum in that file.

- [ ] **Step 5: Run tests and make them pass**

  ```bash
  cargo test -p finplan_core -- married --nocapture 2>&1
  ```
  Expected: all 3 tests PASS.

- [ ] **Step 6: Commit**

  ```bash
  git add crates/finplan_core/src/tests/married.rs \
          crates/finplan_core/src/tests/mod.rs \
          crates/finplan_core/src/config/builder.rs \
          crates/finplan_core/src/config/event_builder.rs
  git commit -m "test(core): married couple spouse age trigger integration tests"
  ```

---

## Phase 2: Account Ownership (Correct Per-Person Age for Penalties & RMDs)

### Task 6: Add `Person` enum and `owner` field to `Account`

**Files:**
- Modify: `crates/finplan_core/src/model/accounts.rs`

- [ ] **Step 1: Add `Person` enum before `Account` struct**

  ```rust
  /// Which person in the household owns this account.
  /// Determines whose birth_date is used for early withdrawal penalties and RMDs.
  #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
  #[serde(rename_all = "snake_case")]
  pub enum Person {
      /// The primary simulation person (default).
      #[default]
      Primary,
      /// The spouse.
      Spouse,
  }
  ```

- [ ] **Step 2: Add `owner` field to `Account`**

  ```rust
  pub struct Account {
      pub account_id: AccountId,
      pub flavor: AccountFlavor,
      /// Which person owns this account (defaults to Primary).
      #[serde(default)]
      pub owner: Person,
  }
  ```

- [ ] **Step 3: Build and fix any construction sites**

  ```bash
  cargo build -p finplan_core 2>&1 | grep "^error" | head -20
  ```
  Any `Account { account_id, flavor }` literal will need `owner: Person::Primary` added, OR rely on `..Default::default()` if you impl Default for Account.

  Add to `Account`:

  ```rust
  impl Default for Account {
      fn default() -> Self {
          Self {
              account_id: AccountId(0),
              flavor: AccountFlavor::Bank(Cash { value: 0.0, return_profile_id: ReturnProfileId(0) }),
              owner: Person::Primary,
          }
      }
  }
  ```

  All struct literals for Account in `config/account_builder.rs`, `liquidation.rs`, and tests need `owner: Person::Primary` or use the struct update pattern `..Account::default()`.

- [ ] **Step 4: Update `AccountBuilder` to expose `.owner()`**

  In `crates/finplan_core/src/config/account_builder.rs`, add:

  ```rust
  use crate::model::Person;

  pub struct AccountBuilder {
      pub(crate) name: Option<String>,
      pub(crate) description: Option<String>,
      flavor: AccountFlavorBuilder,
      owner: Person,  // NEW
  }
  ```

  In the constructors (`bank_account`, `taxable_brokerage`, etc.), set `owner: Person::Primary`.

  Add a builder method:

  ```rust
  /// Mark this account as owned by the spouse.
  #[must_use]
  pub fn owned_by_spouse(mut self) -> Self {
      self.owner = Person::Spouse;
      self
  }
  ```

  In `AccountBuilder::build()` (where it constructs `Account`), pass `owner: self.owner`.

- [ ] **Step 5: Build cleanly**

  ```bash
  cargo build -p finplan_core 2>&1 | grep "^error" | head -20
  ```
  Expected: clean.

- [ ] **Step 6: Commit**

  ```bash
  git add crates/finplan_core/src/model/accounts.rs \
          crates/finplan_core/src/config/account_builder.rs
  git commit -m "feat(core): add Person enum and Account.owner for per-person age rules"
  ```

---

### Task 7: Use account owner's birth date for early withdrawal penalty

**Files:**
- Modify: `crates/finplan_core/src/apply.rs`
- Modify: `crates/finplan_core/src/simulation_state.rs`

- [ ] **Step 1: Add a helper to `SimulationState` that checks penalty by account owner**

  In `simulation_state.rs`, add after `is_spouse_below_early_withdrawal_age()`:

  ```rust
  /// Check early withdrawal penalty for a specific account,
  /// using the owner's birth_date (Primary or Spouse).
  pub fn is_account_below_early_withdrawal_age(&self, account_id: AccountId) -> bool {
      let Some(account) = self.portfolio.accounts.get(&account_id) else {
          return true; // safe default: assume penalty applies
      };
      match account.owner {
          crate::model::Person::Primary => self.timeline.is_below_early_withdrawal_age(),
          crate::model::Person::Spouse => {
              self.timeline
                  .is_spouse_below_early_withdrawal_age()
                  .unwrap_or(true) // no spouse configured → apply penalty
          }
      }
  }
  ```

- [ ] **Step 2: Update the early withdrawal penalty check in `apply.rs`**

  In `apply.rs`, find the usage of `is_below_early_withdrawal_age` (search for it):

  ```bash
  grep -n "is_below_early_withdrawal_age" crates/finplan_core/src/apply.rs
  ```

  Replace the generic call with the account-owner-aware version. For example, if the code looks like:

  ```rust
  if state.timeline.is_below_early_withdrawal_age() {
      // apply 10% penalty
  }
  ```

  And the account being withdrawn from is identified by `account_id`, change to:

  ```rust
  if state.is_account_below_early_withdrawal_age(account_id) {
      // apply 10% penalty
  }
  ```

- [ ] **Step 3: Update `PenaltyAware` withdrawal strategy in `evaluate.rs`**

  Search for `is_below_early_withdrawal_age` in `evaluate.rs`:

  ```bash
  grep -n "is_below_early_withdrawal_age" crates/finplan_core/src/evaluate.rs
  ```

  The `PenaltyAware` strategy currently avoids TaxDeferred accounts if the primary person is below 59.5. For a married couple, the relevant person is the account owner. If the existing code uses `state.timeline.is_below_early_withdrawal_age()` without an account reference, and the withdrawal strategy pulls from all accounts, the penalty check is necessarily conservative (if *any* TaxDeferred account belongs to someone below 59.5, it should be avoided for that person).

  The correct fix: in the withdrawal order loop, when considering a TaxDeferred account, check `state.is_account_below_early_withdrawal_age(candidate_account_id)`.

- [ ] **Step 4: Write a test for spouse-owned account penalty**

  In `crates/finplan_core/src/tests/married.rs`, add:

  ```rust
  /// Spouse-owned 401k does NOT incur early withdrawal penalty if spouse >= 59.5,
  /// even if primary person is < 59.5.
  #[test]
  fn test_spouse_owned_account_no_early_withdrawal_penalty() {
      // Primary born 2000-01-01 (age 30 in 2030), spouse born 1970-01-01 (age 60 in 2030).
      // Spouse's 401k withdrawal should NOT be penalized.
      let asset = AssetBuilder::us_total_market("VTSAX").price(100.0);

      let (mut config, _meta) = SimulationBuilder::new()
          .start(2030, 1, 1)
          .years(5)
          .birth_date(2000, 1, 1)            // primary is 30
          .spouse_birth_date(1970, 1, 1)     // spouse is 60
          .asset(asset)
          .account(
              AccountBuilder::traditional_401k("Spouse 401k")
                  .cash(50_000.0)
                  .owned_by_spouse(),        // key: spouse owns this
          )
          .account(AccountBuilder::bank_account("Checking").cash(0.0))
          // Withdraw $10k from spouse's 401k yearly
          .event(
              EventBuilder::withdrawal("Withdraw from Spouse 401k")
                  .from_account("Spouse 401k")
                  .to_account("Checking")
                  .amount(10_000.0)
                  .yearly(),
          )
          .build();
      config.collect_ledger = true;

      let result = crate::simulation::simulate(&config, 42).unwrap();

      // No early withdrawal penalties should appear in any year
      let total_penalty: f64 = result
          .yearly_taxes
          .iter()
          .map(|t| t.early_withdrawal_penalties)
          .sum();

      assert_eq!(total_penalty, 0.0, "Spouse (age 60) should not incur early withdrawal penalty");
  }
  ```

- [ ] **Step 5: Run all tests**

  ```bash
  cargo test -p finplan_core -- married --nocapture 2>&1
  ```
  Expected: all pass.

- [ ] **Step 6: Commit**

  ```bash
  git add crates/finplan_core/src/simulation_state.rs \
          crates/finplan_core/src/apply.rs \
          crates/finplan_core/src/evaluate.rs \
          crates/finplan_core/src/tests/married.rs
  git commit -m "feat(core): early withdrawal penalty uses account owner's birth_date"
  ```

---

### Task 8: RMDs use account owner's birth date

**Files:**
- Modify: `crates/finplan_core/src/simulation_state.rs`

- [ ] **Step 1: Add `current_rmd_divisor_for_account()` helper**

  In `simulation_state.rs`, after `current_rmd_divisor()` (line ~724):

  ```rust
  /// Get IRS divisor for the owner of a specific account.
  pub fn current_rmd_divisor_for_account(
      &self,
      account_id: AccountId,
      rmd_table: &RmdTable,
  ) -> Option<f64> {
      let account = self.portfolio.accounts.get(&account_id)?;
      let age = match account.owner {
          crate::model::Person::Primary => {
              let (years, _) = self.current_age();
              years
          }
          crate::model::Person::Spouse => {
              let (years, _) = self.spouse_age()?;
              years
          }
      };
      rmd_table.divisor_for_age(age)
  }
  ```

- [ ] **Step 2: Use the new helper in the RMD effect**

  In `evaluate.rs` (around the `ApplyRmd` effect handling, line ~836):

  ```bash
  grep -n "current_rmd_divisor\|current_age\|rmd_table" crates/finplan_core/src/evaluate.rs | head -10
  ```

  Find where `current_rmd_divisor` is called. If it's for a specific account `account_id`, replace:

  ```rust
  let Some(rmd_divisor) = rmd_table.divisor_for_age(age) else { ... };
  ```

  With:

  ```rust
  let Some(rmd_divisor) = state.current_rmd_divisor_for_account(account_id, &rmd_table) else { ... };
  ```

  Remove the now-unused `let (age, _) = state.current_age();` line above it.

- [ ] **Step 3: Build cleanly**

  ```bash
  cargo build -p finplan_core 2>&1 | grep "^error" | head -20
  ```
  Expected: clean.

- [ ] **Step 4: Run all core tests**

  ```bash
  cargo test -p finplan_core 2>&1 | tail -20
  ```
  Expected: all pass.

- [ ] **Step 5: Commit**

  ```bash
  git add crates/finplan_core/src/simulation_state.rs \
          crates/finplan_core/src/evaluate.rs
  git commit -m "feat(core): RMD calculation uses account owner's birth_date for spouse-owned accounts"
  ```

---

## Phase 3: TUI Updates

### Task 9: Add `spouse_birth_date` to TUI parameters

**Files:**
- Modify: `crates/finplan/src/data/parameters_data.rs`
- Modify: `crates/finplan/src/data/convert.rs`
- Modify: `crates/finplan/src/screens/scenario.rs`

- [ ] **Step 1: Add field to `ParametersData`**

  In `parameters_data.rs`, after `birth_date: String`, add:

  ```rust
  /// Spouse birth date (YYYY-MM-DD format). Optional.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub spouse_birth_date: Option<String>,
  ```

- [ ] **Step 2: Update `convert_parameters()` in `convert.rs`**

  After the line `config.birth_date = Some(parse_date(&params.birth_date)?);` (line ~171), add:

  ```rust
  if let Some(ref s) = params.spouse_birth_date {
      if !s.is_empty() {
          config.spouse_birth_date = Some(parse_date(s)?);
      }
  }
  ```

- [ ] **Step 3: Update the "Edit Parameters" form in `scenario.rs`**

  In the `FormModal::new(...)` that builds the edit-parameters form (line ~291), add a new field after the existing Birth Date field:

  ```rust
  FormField::new(
      "Spouse Birth Date (YYYY-MM-DD, optional)",
      FieldType::Text,
      params.spouse_birth_date.as_deref().unwrap_or(""),
  ),
  ```

- [ ] **Step 4: Update the modal action handler that saves the form**

  Find where `ModalAction::EDIT_PARAMETERS` is handled (in `screens/scenario.rs` or `actions/scenario.rs`). It will read fields by index. After reading `birth_date` (field 1), read the new `spouse_birth_date` (field 2):

  ```rust
  let spouse_birth_raw = fields.get(2).map(|f| f.trim().to_string());
  state.data_mut().parameters.spouse_birth_date = match spouse_birth_raw {
      Some(s) if !s.is_empty() => Some(s),
      _ => None,
  };
  ```

  Note: the seed field index will shift from 2 → 3; update any hard-coded index references for subsequent fields.

- [ ] **Step 5: Update scenario summary display**

  In `scenario.rs` `render()`, find where the parameters panel shows the birth date string (line ~596). Add a line showing spouse's birth date when set:

  ```rust
  if let Some(ref spouse_dob) = params.spouse_birth_date {
      lines.push(Line::from(vec![
          Span::styled("Spouse DOB: ", label_style),
          Span::raw(spouse_dob.as_str()),
      ]));
  }
  ```

- [ ] **Step 6: Build and verify**

  ```bash
  cargo build -p finplan 2>&1 | grep "^error" | head -20
  ```
  Expected: clean.

- [ ] **Step 7: Commit**

  ```bash
  git add crates/finplan/src/data/parameters_data.rs \
          crates/finplan/src/data/convert.rs \
          crates/finplan/src/screens/scenario.rs
  git commit -m "feat(tui): add spouse_birth_date to parameters form and data model"
  ```

---

### Task 10: Add "Spouse Social Security" event template to TUI

**Files:**
- Modify: `crates/finplan/src/actions/event.rs`

- [ ] **Step 1: Add template option to the event template picker**

  In `event.rs`, find the `options` vec in the template picker (the list that includes "Social Security", "RMD", "Medicare Part B"). Add:

  ```rust
  "Spouse Social Security".to_string(),
  ```

- [ ] **Step 2: Add match arm for the new template**

  Find the match block that dispatches `create_social_security_template`, `create_rmd_template`, etc. Add:

  ```rust
  "Spouse Social Security" => create_spouse_social_security_template(state),
  ```

- [ ] **Step 3: Implement the template function**

  After `create_social_security_template`, add:

  ```rust
  fn create_spouse_social_security_template(state: &AppState) -> EventData {
      let dest = first_cash_account_name(state);
      EventData {
          name: EventTag("Spouse Social Security".to_string()),
          description: Some("Monthly Social Security benefits for spouse".to_string()),
          trigger: TriggerData::Repeating {
              interval: IntervalData::Monthly,
              start: Some(Box::new(TriggerData::SpouseAge {
                  years: 67,
                  months: None,
              })),
              end: None,
              max_occurrences: None,
          },
          effects: vec![EffectData::Income {
              to: AccountTag(dest),
              amount: AmountData::fixed(2000.0), // Placeholder — user should customize
              gross: true,
              taxable: true,
          }],
          once: false,
          enabled: true,
      }
  }
  ```

  Note: `TriggerData::SpouseAge` must be added to the `TriggerData` enum in the TUI data model. Find where `TriggerData::Age` is defined (in `crates/finplan/src/data/app_data.rs` or similar) and add the matching `SpouseAge` variant. Update `convert.rs` to convert it to `EventTrigger::SpouseAge`.

- [ ] **Step 4: Build and verify**

  ```bash
  cargo build -p finplan 2>&1 | grep "^error" | head -20
  ```
  Expected: clean.

- [ ] **Step 5: Commit**

  ```bash
  git add crates/finplan/src/actions/event.rs \
          crates/finplan/src/data/app_data.rs \
          crates/finplan/src/data/convert.rs
  git commit -m "feat(tui): add Spouse Social Security event template using SpouseAge trigger"
  ```

---

## Phase 3b: Working Example YAML

### Task 9b: Add `owner` field to `AccountData` and create `examples/example_married.yaml`

**Goal:** Make the YAML format fully express the married-couple features from Phases 1–3 (spouse birth date, SpouseAge triggers, and spouse-owned accounts). Then ship a working example demonstrating all three.

**Files:**
- Modify: `crates/finplan/src/data/portfolio_data.rs` — add `owner: Person` to `AccountData`
- Modify: `crates/finplan/src/data/convert.rs` — use `account_data.owner` instead of hardcoded `Person::Primary`
- Create: `examples/example_married.yaml`

- [ ] **Step 1: Add `owner` field to `AccountData`**

  In `portfolio_data.rs`, add the import and field:

  ```rust
  use finplan_core::model::Person;

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct AccountData {
      pub name: String,
      #[serde(skip_serializing_if = "Option::is_none")]
      pub description: Option<String>,
      /// Account owner (defaults to Primary). Use `spouse` for spouse-owned accounts.
      #[serde(default)]
      pub owner: Person,
      #[serde(flatten)]
      pub account_type: AccountType,
  }
  ```

  `Person` is `#[serde(rename_all = "snake_case")]` so YAML uses `owner: primary` / `owner: spouse`.

- [ ] **Step 2: Update `convert_accounts()` in `convert.rs`**

  Change the hardcoded `owner: Person::Primary` to use the data field:

  ```rust
  config.accounts.push(Account {
      account_id,
      flavor,
      owner: account_data.owner,
  });
  ```

  Remove the now-unused `Person::Primary` literal (keep the `use` import of `Person` which is already there).

- [ ] **Step 3: Build and verify existing tests still pass**

  ```bash
  docker run --rm -v "$(pwd)":/app -w /app rust:slim cargo build -p finplan 2>&1 | grep "^error" | head -20
  docker run --rm -v "$(pwd)":/app -w /app rust:slim cargo test -p finplan_core 2>&1 | grep "test result"
  ```

  The `owner` field has `#[serde(default)]` so all existing YAML files without it still parse as `Person::Primary`. Expected: clean build, all tests pass.

- [ ] **Step 4: Create `examples/example_married.yaml`**

  Scenario: Two-earner household, five years apart. Primary born 1975-01-01, spouse born 1978-06-01. Both retire around age 60. Using `married_joint2024` brackets.

  Demonstrate:
  - `spouse_birth_date` in parameters
  - `owner: spouse` on spouse-owned accounts (Spouse 401k, Spouse Roth IRA)
  - `SpouseAge` triggers for spouse retirement, Spouse SS, Spouse Medicare, Spouse RMD
  - `Age` triggers for primary retirement, Primary SS, Primary Medicare, Primary RMD
  - Both incomes ending at respective retirements (via `RelativeToEvent`)
  - A post-retirement Sweep event to fund spending

  ```yaml
  portfolios:
    name: Married Couple Retirement
    description: Two-earner household — demonstrates spouse_birth_date, SpouseAge triggers, and spouse-owned accounts
    accounts:
      - name: Checking
        type: Checking
        value: 50000.0
      - name: Primary 401k
        type: Traditional401k
        assets:
          - asset: FXAIX
            value: 320000.0
      - name: Spouse 401k
        type: Traditional401k
        owner: spouse
        assets:
          - asset: FXAIX
            value: 210000.0
      - name: Primary Roth IRA
        type: RothIRA
        assets:
          - asset: VTSAX
            value: 85000.0
      - name: Spouse Roth IRA
        type: RothIRA
        owner: spouse
        assets:
          - asset: VTSAX
            value: 60000.0
      - name: Brokerage
        type: Brokerage
        assets:
          - asset: VTSAX
            value: 150000.0

  historical_assets:
    FXAIX: S&P 500
    VTSAX: S&P 500

  events:
    - name: Primary Salary
      trigger:
        type: Repeating
        interval: biweekly
        end:
          type: RelativeToEvent
          event: Primary Retirement
          offset:
            unit: Months
            value: 0
      effects:
        - type: Income
          to: Checking
          amount:
            type: InflationAdjusted
            inner:
              type: Fixed
              value: 5500.0
          gross: true
          taxable: true
      once: false
      enabled: true

    - name: Spouse Salary
      trigger:
        type: Repeating
        interval: biweekly
        end:
          type: RelativeToEvent
          event: Spouse Retirement
          offset:
            unit: Months
            value: 0
      effects:
        - type: Income
          to: Checking
          amount:
            type: InflationAdjusted
            inner:
              type: Fixed
              value: 4200.0
          gross: true
          taxable: true
      once: false
      enabled: true

    - name: Living Expenses
      trigger:
        type: Repeating
        interval: monthly
      effects:
        - type: Expense
          from: Checking
          amount:
            type: InflationAdjusted
            inner:
              type: Fixed
              value: 7000.0
      once: false
      enabled: true

    - name: Primary 401k Contribution
      trigger:
        type: Repeating
        interval: yearly
        end:
          type: RelativeToEvent
          event: Primary Retirement
          offset:
            unit: Months
            value: 0
      effects:
        - type: AssetPurchase
          from: Checking
          to_account: Primary 401k
          asset: FXAIX
          amount:
            type: Fixed
            value: 23000.0
      once: false
      enabled: true

    - name: Spouse 401k Contribution
      trigger:
        type: Repeating
        interval: yearly
        end:
          type: RelativeToEvent
          event: Spouse Retirement
          offset:
            unit: Months
            value: 0
      effects:
        - type: AssetPurchase
          from: Checking
          to_account: Spouse 401k
          asset: FXAIX
          amount:
            type: Fixed
            value: 23000.0
      once: false
      enabled: true

    - name: Primary Retirement
      description: Primary person retires at 60
      trigger:
        type: Age
        years: 60
      once: true
      enabled: true

    - name: Spouse Retirement
      description: Spouse retires at 60
      trigger:
        type: SpouseAge
        years: 60
      once: true
      enabled: true

    - name: Fund Retirement Spending
      description: Sweep accounts to cover annual spending in retirement
      trigger:
        type: Repeating
        interval: yearly
        start:
          type: RelativeToEvent
          event: Primary Retirement
          offset:
            unit: Months
            value: 0
      effects:
        - type: Sweep
          to: Checking
          amount:
            type: TargetToBalance
            target: 100000.0
          strategy: penalty_aware
          gross: false
          taxable: true
          lot_method: fifo
      once: false
      enabled: true

    - name: Primary Social Security
      description: Monthly Social Security for primary (age 67)
      trigger:
        type: Repeating
        interval: monthly
        start:
          type: Age
          years: 67
      effects:
        - type: Income
          to: Checking
          amount:
            type: Fixed
            value: 2800.0
          gross: true
          taxable: true
      once: false
      enabled: true

    - name: Spouse Social Security
      description: Monthly Social Security for spouse (spouse age 67)
      trigger:
        type: Repeating
        interval: monthly
        start:
          type: SpouseAge
          years: 67
      effects:
        - type: Income
          to: Checking
          amount:
            type: Fixed
            value: 2200.0
          gross: true
          taxable: true
      once: false
      enabled: true

    - name: Primary Medicare Part B
      description: Medicare Part B premiums for primary
      trigger:
        type: Repeating
        interval: monthly
        start:
          type: Age
          years: 65
      effects:
        - type: Expense
          from: Checking
          amount:
            type: Fixed
            value: 174.7
      once: false
      enabled: true

    - name: Spouse Medicare Part B
      description: Medicare Part B premiums for spouse
      trigger:
        type: Repeating
        interval: monthly
        start:
          type: SpouseAge
          years: 65
      effects:
        - type: Expense
          from: Checking
          amount:
            type: Fixed
            value: 174.7
      once: false
      enabled: true

    - name: Primary RMD
      description: Required Minimum Distributions from primary tax-deferred accounts (age 73)
      trigger:
        type: Repeating
        interval: yearly
        start:
          type: Age
          years: 73
      effects:
        - type: ApplyRmd
          destination: Checking
          lot_method: fifo
      once: false
      enabled: true

    - name: Spouse RMD
      description: Required Minimum Distributions from spouse tax-deferred accounts (spouse age 73)
      trigger:
        type: Repeating
        interval: yearly
        start:
          type: SpouseAge
          years: 73
      effects:
        - type: ApplyRmd
          destination: Checking
          lot_method: fifo
      once: false
      enabled: true

  parameters:
    birth_date: "1975-01-01"
    spouse_birth_date: "1978-06-01"
    start_date: "2026-01-01"
    duration_years: 45
    inflation:
      type: USHistorical
      distribution: lognormal
    tax_config:
      state_rate: 0.05
      capital_gains_rate: 0.15
      federal_brackets: married_joint2024
    returns_mode: historical
    historical_block_size: 5
  ```

- [ ] **Step 5: Load the example in the TUI to verify it parses without errors**

  ```bash
  # The TUI supports file import — confirm no parse errors by running conversion:
  docker run --rm -v "$(pwd)":/app -w /app rust:slim cargo test -p finplan_core -- married --nocapture 2>&1 | tail -10
  ```

- [ ] **Step 6: Commit**

  ```bash
  cargo fmt
  git add crates/finplan/src/data/portfolio_data.rs \
          crates/finplan/src/data/convert.rs \
          examples/example_married.yaml
  git commit -m "feat(tui): add owner field to AccountData; add examples/example_married.yaml"
  ```

---

## Phase 4: MCP Server Updates

All MCP tests run via Docker (no local Rust toolchain needed):

```bash
docker run --rm -v "$(pwd)":/app -w /app rust:slim cargo test -p finplan_mcp -- --nocapture
```

### Task 11: Add `spouse_birth_date` to MCP parameters

**Files:**
- Modify: `crates/finplan_mcp/src/tools/parameters.rs`
- Modify: `crates/finplan_mcp/src/schema_text.rs`

- [ ] **Step 1: Add `spouse_birth_date` to the parameters tool JSON schema**

  In `parameters.rs`, find the `properties` object that defines `birth_date`. Add alongside it:

  ```rust
  "spouse_birth_date": {
      "type": "string",
      "description": "Spouse's date of birth in YYYY-MM-DD format. Optional. Required for SpouseAge event triggers.",
      "pattern": "^\\d{4}-\\d{2}-\\d{2}$"
  },
  ```

- [ ] **Step 2: Parse and apply `spouse_birth_date` in the handler**

  In the handler function that reads `birth_date` from args and calls `state.set_birth_date(...)` (or equivalent), add:

  ```rust
  if let Some(spouse_dob) = args.get("spouse_birth_date").and_then(|v| v.as_str()) {
      if !spouse_dob.is_empty() {
          state.spouse_birth_date = Some(spouse_dob.to_string());
      }
  }
  ```

  Ensure `spouse_birth_date` is stored in the MCP state struct and propagated to the YAML / `SimulationConfig` during export/validation.

- [ ] **Step 3: Update `schema_text.rs`**

  In the parameters schema documentation section, add:

  ```
  - `spouse_birth_date` (optional) — YYYY-MM-DD. Required if you use SpouseAge event triggers.
  ```

- [ ] **Step 4: Run MCP tests**

  ```bash
  docker run --rm -v "$(pwd)":/app -w /app rust:slim cargo test -p finplan_mcp -- --nocapture 2>&1 | tail -20
  ```
  Expected: all existing tests pass (spouse_birth_date is optional; existing tests don't use it).

- [ ] **Step 5: Commit**

  ```bash
  git add crates/finplan_mcp/src/tools/parameters.rs \
          crates/finplan_mcp/src/schema_text.rs
  git commit -m "feat(mcp): add spouse_birth_date parameter to set_portfolio_overview tool"
  ```

---

## Phase 5: Final Verification

### Task 12: Full test suite + format + clippy

- [ ] **Step 1: Format**

  ```bash
  cargo fmt
  ```

- [ ] **Step 2: Clippy**

  ```bash
  cargo clippy -p finplan_core -p finplan 2>&1 | grep "^warning\|^error" | grep -v "generated"
  ```
  Fix any clippy warnings that don't require major refactors.

- [ ] **Step 3: All core tests**

  ```bash
  cargo test -p finplan_core 2>&1 | tail -20
  ```
  Expected: all pass.

- [ ] **Step 4: MCP tests**

  ```bash
  docker run --rm -v "$(pwd)":/app -w /app rust:slim cargo test -p finplan_mcp -- --nocapture 2>&1 | tail -20
  ```
  Expected: all pass.

- [ ] **Step 5: Manual smoke test — TUI**

  ```bash
  cargo run --bin finplan
  ```

  Open a scenario → press the key to edit parameters → verify the "Spouse Birth Date" field appears. Enter a spouse DOB. Navigate to Events → Add event → verify "Spouse Social Security" template appears in the picker.

- [ ] **Step 6: Final commit**

  ```bash
  cargo fmt
  git add -u
  git commit -m "chore: fmt + clippy cleanup for married couple feature"
  ```

---

## Self-Review Checklist

| Requirement | Covered by |
|-------------|-----------|
| `spouse_birth_date` in `SimulationConfig` | Task 1 |
| `EventTrigger::SpouseAge` variant | Task 2 |
| SpouseAge trigger fires at correct date | Task 3, 4, 5 |
| SpouseAge never fires if no spouse DOB set | Task 5 (test) |
| Primary Age trigger unaffected | Task 5 (test) |
| Early withdrawal penalty uses account owner | Task 6, 7 |
| RMD uses account owner's age | Task 8 |
| TUI parameters form shows spouse DOB | Task 9 |
| TUI event template for Spouse SS | Task 10 |
| Builder DSL `.spouse_birth_date()` | Task 5 |
| Builder DSL `.owned_by_spouse()` | Task 6 |
| MCP `spouse_birth_date` parameter | Task 11 |
| All tests pass | Task 12 |

---

## Out of Scope (Future Work)

- **Spousal IRA**: contribution limits for a non-working spouse's IRA (IRS rule: can contribute based on working spouse's income)
- **Spousal Social Security benefit**: the 50% rule (spouse can claim up to 50% of primary's SS benefit)
- **Survivor benefits**: simulation after one spouse's death (death event that removes their income/SS)
- **Two-income household optimization**: optimizing *which* spouse defers/withdraws first for tax efficiency
