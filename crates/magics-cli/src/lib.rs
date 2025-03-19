//! Command-line interface and configuration management for the Magics planner

pub mod args;
pub mod config;
pub mod scenarios;

use clap::Parser;
use std::path::Path;
use anyhow::Result;
use log::{info, warn, error, debug};
use magics_core::simulation::Simulation;

pub use args::{Cli, HeadlessMode, Verbosity, DumpDefault, OutputFormat, BevySchedule};
pub use config::{print_config, print_environment, print_metadata, load_config, load_environment};
pub use scenarios::{Scenario, discover_scenarios, load_scenario, print_scenarios, list_all_scenarios};

/// Parse command-line arguments with support for environment variables
/// 
/// This function will parse arguments from the command line and also
/// check for relevant environment variables such as MAGICS_HEADLESS.
pub fn parse_arguments() -> Cli {
    let mut cli = Cli::parse();
    
    // Check for MAGICS_HEADLESS environment variable
    if std::env::var("MAGICS_HEADLESS").map(|v| v == "1").unwrap_or(false) {
        cli.headless = HeadlessMode::Enabled;
    }
    
    cli
}

/// Set up logging based on verbosity level
pub fn setup_logging(cli: &Cli) {
    let env = env_logger::Env::default()
        .filter_or("MAGICS_LOG", match cli.verbosity() {
            Verbosity::None => "warn",
            Verbosity::Normal => "info",
            Verbosity::Very => "debug",
            Verbosity::Ultra => "trace",
        });
    
    env_logger::Builder::from_env(env)
        .format_timestamp(Some(env_logger::fmt::TimestampPrecision::Millis))
        .init();
}

/// Run in headless mode
pub fn run_headless(args: &Cli) -> Result<()> {
    info!("Running in headless mode");
    
    // Default scenarios directory
    let scenarios_dir = args.simulations_dir.as_ref()
        .map(|p| p.as_path())
        .unwrap_or_else(|| Path::new("./config/scenarios"));
    
    // Handle scenario listing
    if args.list_scenarios {
        list_all_scenarios(scenarios_dir)?;
        return Ok(());
    }
    
    // Load scenario
    let scenario_name = match &args.initial_scenario {
        Some(name) => name.clone(),
        None => {
            // Get first scenario in sorted order
            let scenarios = discover_scenarios(scenarios_dir)?;
            if scenarios.is_empty() {
                return Err(anyhow::anyhow!("No scenarios found in {}", scenarios_dir.display()));
            }
            info!("No scenario specified, using first available: {}", scenarios[0].name);
            scenarios[0].name.clone()
        }
    };
    
    let mut scenario = load_scenario(scenarios_dir, &scenario_name)?;
    info!("Loaded scenario: {}", scenario.name);
    
    // Run simulation using magics-core
    let _config = scenario.config.take().unwrap_or_default();
    let environment = scenario.environment.take().unwrap_or_default();
    let formation = scenario.formation.take();
    
    // Convert CLI arguments to simulation config
    let output_config = config::cli_to_output_config(args);
    
    // Create and run simulation
    info!("Creating simulation with environment: {:?}", environment);
    
    // Create environment and apply formation if available
    let mut env = magics_core::environment::GbpSimulationEnvironment::new(environment);
    if let Some(formation_group) = formation {
        info!("Applying formation with {} formations", formation_group.formations.len());
        for formation in formation_group.formations.iter() {
            info!("Formation has {} robots", formation.robots);
        }
        env = env.with_formation(formation_group);
    } else {
        info!("No formation data found, using default agents");
    }
    let sim_config = magics_core::simulation::SimulationConfig {
        output: magics_core::simulation::OutputSettings {
            stdout: output_config.stdout,
            file_path: output_config.file_path,
            frequency: output_config.frequency,
        },
        ..Default::default()
    };
    
    let mut simulation = magics_core::simulation::GbpSimulation::new(
        env,
        sim_config,
    );
    
    // Create planner
    let planner = magics_core::planner::GbpPlanner::default();
    simulation.set_planner(planner);
    
    // Run for specified number of steps or until finished
    let max_steps = 250; // Could be configurable
    for step in 0..max_steps {
        // Adding debug code to better verify simulation state
        if step % 100 == 0 {
            debug!("Agent states at step {}: {:#?}", step, simulation.get_state().agents);
            debug!("Time: {}", simulation.get_state().time.get());
        }
        
        if simulation.is_finished() {
            info!("Simulation finished at step {}", step);
            break;
        }
        
        if let Err(e) = simulation.step() {
            error!("Error during simulation step {}: {}", step, e);
            return Err(anyhow::anyhow!("Simulation failed: {}", e));
        }
        
        if step % 100 == 0 {
            info!("Simulation at step {}", step);
        }
    }
    
    info!("Simulation completed successfully");
    // Print final state to verify results
    info!("Final state: {:#?}", simulation.get_state());
    Ok(())
}

/// Run with UI (conditionally compiled)
#[cfg(feature = "ui")]
pub fn run_with_ui(args: &Cli) -> Result<()> {
    info!("Running with UI");
    
    // First, create the core simulation
    info!("Initializing core simulation");
    
    // Default scenarios directory
    let scenarios_dir = args.simulations_dir.as_ref()
        .map(|p| p.as_path())
        .unwrap_or_else(|| std::path::Path::new("./config/scenarios"));
    
    // Load scenario
    let scenario_name = match &args.initial_scenario {
        Some(name) => name.clone(),
        None => {
            // Get first scenario in sorted order
            let scenarios = discover_scenarios(scenarios_dir)?;
            if scenarios.is_empty() {
                return Err(anyhow::anyhow!("No scenarios found in {}", scenarios_dir.display()));
            }
            info!("No scenario specified, using first available: {}", scenarios[0].name);
            scenarios[0].name.clone()
        }
    };
    
    info!("Loading scenario: {}", scenario_name);
    let mut scenario = load_scenario(scenarios_dir, &scenario_name)?;
    
    // Extract configuration
    let _config = scenario.config.take().unwrap_or_default();
    let environment = scenario.environment.take().unwrap_or_default();
    let formation = scenario.formation.take();
    
    // Create environment and apply formation if available
    let mut env = magics_core::environment::GbpSimulationEnvironment::new(environment);
    if let Some(formation_group) = formation {
        info!("Applying formation with {} formations", formation_group.formations.len());
        env = env.with_formation(formation_group);
    } else {
        info!("No formation data found, using default agents");
    }
    
    // Create simulation
    let sim_config = magics_core::simulation::SimulationConfig::default();
    let simulation: Box<dyn magics_core::simulation::Simulation> = Box::new(
        magics_core::simulation::GbpSimulation::new(env, sim_config)
    );
    
    // Create planner
    let mut simulation_mut = simulation.as_any_mut().downcast_mut::<magics_core::simulation::GbpSimulation>().unwrap();
    let planner = magics_core::planner::GbpPlanner::default();
    simulation_mut.set_planner(planner);
    
    // Wrap simulation in Arc<Mutex<>> for thread-safe sharing
    use std::sync::{Arc, Mutex};
    let simulation = Arc::new(Mutex::new(simulation));
    
    // Now initialize UI with the simulation
    info!("Initializing UI with simulation");
    
    use magics_bevy::MagicsBevyPlugin;
    use bevy::prelude::*;
    
    App::new()
        .add_plugins(MagicsBevyPlugin {
            simulation: Some(simulation),
            fullscreen: args.fullscreen,
            simulations_dir: args.simulations_dir.clone(),
            initial_scenario: args.initial_scenario.clone(),
            width: args.width,
            height: args.height,
            record: args.record,
        })
        .run();
    
    info!("UI session completed");
    Ok(())
}

/// Implement a prelude module for easy imports
pub mod prelude {
    pub use crate::args::{Cli, HeadlessMode, OutputFormat, Verbosity};
    pub use crate::config::{load_config, load_environment, print_metadata};
    pub use crate::scenarios::{Scenario, discover_scenarios, load_scenario, list_all_scenarios};
    pub use crate::{parse_arguments, setup_logging};
    
    #[cfg(feature = "ui")]
    pub use crate::run_with_ui;
    
    pub use crate::run_headless;
}
