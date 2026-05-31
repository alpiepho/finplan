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
use std::path::{Path, PathBuf};

/// Render every tab to a plain-text file inside `output_dir` using a
/// headless [`ratatui::backend::TestBackend`].
///
/// If `scenario` is supplied the scenario is loaded before rendering.
/// A single deterministic simulation is run automatically so the Results
/// tab shows real data rather than an empty state.
///
/// Output files are named `01-portfolio-profiles.txt` … `05-analysis.txt`.
/// The directory is created if it does not already exist.
pub fn headless_dump(
    scenario: Option<PathBuf>,
    output_dir: &Path,
    width: u16,
    height: u16,
) -> color_eyre::Result<()> {
    use std::fs;

    fs::create_dir_all(output_dir).map_err(|e| {
        color_eyre::eyre::eyre!(
            "Cannot create output directory '{}': {}",
            output_dir.display(),
            e
        )
    })?;

    let data_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".finplan");

    let mut app = App::with_data_dir(data_dir);
    if let Some(path) = scenario {
        app = app.with_startup_scenario(path);
    }

    let screens = app.dump_screens(width, height)?;

    for (filename, content) in &screens {
        let dest = output_dir.join(filename);
        fs::write(&dest, content)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to write '{}': {}", dest.display(), e))?;
        println!("  wrote {}", dest.display());
    }

    println!(
        "headless-dump: {} files written to {}",
        screens.len(),
        output_dir.display()
    );
    Ok(())
}

/// Validate a scenario file and return detailed error information
pub fn validate_scenario_file(path: &Path) -> Result<(), String> {
    use std::fs;

    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

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
