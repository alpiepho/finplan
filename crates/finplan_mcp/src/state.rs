use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use finplan::data::{
    analysis_data::{AnalysisConfigData, SweepParameterData},
    app_data::SimulationData,
    events_data::EventData,
    parameters_data::ParametersData,
    portfolio_data::{AssetTag, PortfolioData},
    profiles_data::{ProfileData, ReturnProfileTag},
};
use finplan_core::analysis::SweepResults;
use finplan_core::model::{MonteCarloSummary, SimulationResult};

/// Accumulated scenario state built up across tool calls.
#[derive(Debug, Clone)]
pub struct ScenarioState {
    /// Portfolio section (from set_portfolio)
    pub portfolio: Option<PortfolioData>,

    /// Parameters section (from set_parameters)
    pub parameters: Option<ParametersData>,

    /// Accumulated events (from add_* tools)
    pub events: Vec<EventData>,

    /// Return profiles (from map_tickers, parametric mode)
    pub profiles: Vec<ProfileData>,

    /// Asset-to-profile mappings (parametric)
    pub assets: HashMap<AssetTag, ReturnProfileTag>,

    /// Asset-to-historical-preset mappings
    pub historical_assets: HashMap<AssetTag, ReturnProfileTag>,

    /// Explicit asset prices
    pub asset_prices: HashMap<AssetTag, f64>,

    /// Tracking errors
    pub asset_tracking_errors: HashMap<AssetTag, f64>,

    /// Analysis config
    pub analysis: AnalysisConfigData,

    /// Last single-run or P50 MC result (used by get_account_snapshot and get_ledger)
    pub last_simulation_result: Option<SimulationResult>,

    /// SimulationData used to produce last_simulation_result (for name lookups)
    pub last_sim_data: Option<SimulationData>,

    /// Full Monte Carlo summary (used by export_planner_summary in Plan 4)
    pub last_mc_summary: Option<MonteCarloSummary>,

    /// Sweep parameter axes added via add_sweep_parameter.
    pub sweep_parameters: Vec<SweepParameterData>,

    /// MC iterations per sweep grid point (default: 200).
    pub sweep_mc_iterations: usize,

    /// Default step count for new sweep parameters (default: 6).
    pub sweep_default_steps: usize,

    /// Results from the last run_sweep call.
    pub last_sweep_results: Option<SweepResults>,
}

impl Default for ScenarioState {
    fn default() -> Self {
        Self {
            portfolio: None,
            parameters: None,
            events: Vec::new(),
            profiles: Vec::new(),
            assets: HashMap::new(),
            historical_assets: HashMap::new(),
            asset_prices: HashMap::new(),
            asset_tracking_errors: HashMap::new(),
            analysis: AnalysisConfigData {
                mc_iterations: 500,
                default_steps: 6,
                ..Default::default()
            },
            last_simulation_result: None,
            last_sim_data: None,
            last_mc_summary: None,
            sweep_parameters: Vec::new(),
            sweep_mc_iterations: 200,
            sweep_default_steps: 6,
            last_sweep_results: None,
        }
    }
}

impl ScenarioState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear cached simulation results. Call whenever scenario state changes.
    pub fn invalidate_simulation_cache(&mut self) {
        self.last_simulation_result = None;
        self.last_sim_data = None;
        self.last_mc_summary = None;
        self.last_sweep_results = None;
    }

    /// Clear only the sweep results cache. Call when sweep grid changes.
    pub fn invalidate_sweep_cache(&mut self) {
        self.last_sweep_results = None;
    }

    /// Merge all accumulated sections into a complete SimulationData
    pub fn merge(&self) -> Result<SimulationData, Vec<String>> {
        let portfolio = self
            .portfolio
            .clone()
            .ok_or_else(|| vec!["Portfolio not set. Call set_portfolio first.".into()])?;

        let parameters = self.parameters.clone().unwrap_or_default();

        Ok(SimulationData {
            portfolios: portfolio,
            profiles: self.profiles.clone(),
            assets: self.assets.clone(),
            historical_assets: self.historical_assets.clone(),
            asset_prices: self.asset_prices.clone(),
            asset_tracking_errors: self.asset_tracking_errors.clone(),
            events: self.events.clone(),
            parameters,
            analysis: self.analysis.clone(),
        })
    }

    /// Collect all tickers referenced in the portfolio
    pub fn portfolio_tickers(&self) -> Vec<String> {
        let Some(portfolio) = &self.portfolio else {
            return Vec::new();
        };
        let mut tickers = Vec::new();
        for account in &portfolio.accounts {
            if let Some(inv) = account.account_type.as_investment() {
                for asset in &inv.assets {
                    tickers.push(asset.asset.0.clone());
                }
            }
        }
        tickers.sort();
        tickers.dedup();
        tickers
    }
}

/// Thread-safe wrapper around ScenarioState
pub type SharedState = Arc<Mutex<ScenarioState>>;

pub fn new_shared_state() -> SharedState {
    Arc::new(Mutex::new(ScenarioState::new()))
}
