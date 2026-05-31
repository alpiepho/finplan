use clap::Parser;
use finplan::{App, init_logging};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "finplan")]
#[command(about = "A terminal-based financial planning simulator")]
struct Args {
    /// Path to the data directory (default: ~/.finplan/)
    #[arg(short, long)]
    data_dir: Option<PathBuf>,

    /// Log level (debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Load a scenario file at startup
    #[arg(short, long)]
    scenario: Option<PathBuf>,

    /// Validate scenario and exit (use with --scenario)
    #[arg(short, long)]
    validate: bool,

    /// Render all tabs to text files in DIR and exit (no terminal required).
    /// Loads --scenario if provided and runs a single simulation first.
    /// Output: 01-portfolio-profiles.txt … 05-analysis.txt
    #[arg(long, value_name = "DIR")]
    headless_dump: Option<PathBuf>,

    /// Terminal width for --headless-dump (default: 160)
    #[arg(long, default_value = "160", requires = "headless_dump")]
    headless_width: u16,

    /// Terminal height for --headless-dump (default: 50)
    #[arg(long, default_value = "50", requires = "headless_dump")]
    headless_height: u16,

    /// Suppress informational modals at startup (useful for VHS/scripted recording)
    #[arg(long)]
    quiet: bool,
}

fn default_data_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".finplan")
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    let data_dir = args.data_dir.unwrap_or_else(default_data_dir);

    init_logging(&data_dir, &args.log_level)?;

    // Handle headless-dump mode: render all tabs to text files and exit.
    if let Some(output_dir) = args.headless_dump {
        return finplan::headless_dump(
            args.scenario,
            &output_dir,
            args.headless_width,
            args.headless_height,
        );
    }

    // Handle validation mode: -s <path> -v validates and exits
    if args.validate {
        if let Some(scenario_path) = &args.scenario {
            let result = finplan::validate_scenario_file(scenario_path);
            match result {
                Ok(_) => {
                    println!("✓ Scenario is valid!");
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("✗ Scenario validation failed:");
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            }
        } else {
            eprintln!("Error: --validate requires --scenario <PATH>");
            std::process::exit(1);
        }
    }

    let mut app = App::with_data_dir(data_dir);
    if let Some(scenario_path) = args.scenario {
        app = app.with_startup_scenario(scenario_path);
    }
    if args.quiet {
        app = app.with_quiet();
    }

    ratatui::run(|terminal| app.run(terminal))?;

    tracing::info!("Application shutting down");

    if let Err(err) = ratatui::try_restore() {
        tracing::error!("Failed to restore terminal: {err}");
    }

    Ok(())
}
