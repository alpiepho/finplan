use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use finplan::data::{
    analysis_data::AnalysisConfigData,
    app_data::SimulationData,
    events_data::EventData,
    parameters_data::ParametersData,
    portfolio_data::{AssetTag, PortfolioData},
    profiles_data::{ProfileData, ReturnProfileTag},
};

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
            analysis: AnalysisConfigData::default(),
        }
    }
}

impl ScenarioState {
    pub fn new() -> Self {
        Self::default()
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
