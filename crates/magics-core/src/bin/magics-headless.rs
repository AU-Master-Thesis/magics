//! Headless execution of the MAGICS simulation
//!
//! This binary provides a command-line interface to run the
//! MAGICS simulation without any graphical user interface.
//! It's useful for running experiments, benchmarks, and tests.

use std::env;
use std::path::PathBuf;
use clap::Parser;
use magics_core::prelude::*;
use magics_core::simulation::SimulationConfig;
use magics_core::error::SimulationError;

/// Command line arguments for the headless MAGICS simulation
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Path to the configuration file
    #[clap(short, long)]
    config: Option<PathBuf>,
    
    /// Run in headless mode
    #[clap(long)]
    headless: bool,
    
    /// Output directory for simulation results
    #[clap(short, long)]
    output_dir: Option<PathBuf>,
    
    /// Output format for simulation results
    #[clap(long)]
    output_format: Option<OutputFormat>,
    
    /// Verbosity level
    #[clap(short, long)]
    verbose: Option<VerbosityLevel>,
}

/// Verbosity level for logging
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum VerbosityLevel {
    /// Only show errors
    Quiet,
    /// Show info messages (default)
    Normal,
    /// Show debug messages
    Verbose,
    /// Show trace messages
    VeryVerbose,
}

impl std::str::FromStr for VerbosityLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "quiet" => Ok(VerbosityLevel::Quiet),
            "normal" => Ok(VerbosityLevel::Normal),
            "verbose" => Ok(VerbosityLevel::Verbose),
            "veryverbose" => Ok(VerbosityLevel::VeryVerbose),
            _ => Err(format!("Unknown verbosity level: {}", s)),
        }
    }
}

fn main() -> Result<(), SimulationError> {
    // Setup better panic handling
    if cfg!(debug_assertions) {
        better_panic::debug_install();
    } else {
        better_panic::install();
    }

    // Parse command line arguments
    let args = Args::parse();
    
    // Set up environment variables based on arguments
    if args.headless {
        env::set_var("MAGICS_HEADLESS", "1");
    }
    
    // Configure logging based on verbosity
    let verbosity = args.verbose.unwrap_or(VerbosityLevel::Normal);
    match verbosity {
        VerbosityLevel::Quiet => env::set_var("RUST_LOG", "error"),
        VerbosityLevel::Normal => env::set_var("RUST_LOG", "info"),
        VerbosityLevel::Verbose => env::set_var("RUST_LOG", "debug"),
        VerbosityLevel::VeryVerbose => env::set_var("RUST_LOG", "trace"),
    }
    
    // Initialize logging
    env_logger::init();
    
    // Load configuration
    let config = match &args.config {
        Some(path) => {
            // In a real implementation, this would load config from file
            // For now, we'll just use the default
            SimulationConfig::default()
        },
        None => SimulationConfig::default(),
    };
    
    // Set up the simulation environment
    let environment = GbpSimulationEnvironment::default();
    
    // Create and configure the planner
    let planner = GbpPlanner::default();
    
    // Set up the output settings
    let output_settings = if let Some(output_dir) = args.output_dir.clone() {
        OutputSettings {
            stdout: false,
            file_path: Some(output_dir),
            frequency: 1, // Output every step
        }
    } else {
        OutputSettings::default()
    };
    
    // Create simulation
    let mut simulation = GbpSimulation::new(
        environment,
        config,
    );
    
    // Set planner and output settings
    simulation.set_planner(planner);
    simulation.set_output_settings(output_settings);
    
    // Run simulation for 1000 steps or until completion
    for _ in 0..1000 {
        if simulation.is_finished() {
            break;
        }
        
        simulation.step()?;
    }
    
    Ok(())
}
