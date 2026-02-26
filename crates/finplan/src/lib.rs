pub mod actions;
pub mod app;
pub mod components;
pub mod data;
pub mod keybindings;
pub mod logging;
pub mod modals;
pub mod screens;
pub mod state;
pub mod util;
pub mod worker;

pub use app::App;
pub use logging::init_logging;
pub use state::AppState;
use std::path::Path;

/// Validate a scenario file and return detailed error information
pub fn validate_scenario_file(path: &Path) -> Result<(), String> {
    use std::fs;
    
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let data = data::app_data::SimulationData::from_yaml(&content)
        .map_err(|e| format!("Failed to parse YAML:\n{}", e))?;

    // Run semantic validation
    if let Err(validation_errors) = data::validator::validate_scenario(&data) {
        let error_messages = validation_errors
            .iter()
            .map(|e| format!("  • {}", e))
            .collect::<Vec<_>>()
            .join("\n");
        
        return Err(format!("Validation errors found:\n{}", error_messages));
    }

    Ok(())
}