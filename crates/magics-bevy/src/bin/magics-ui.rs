//! Main entry point for the Magics planner with UI
//! 
//! This binary allows running Magics simulations with the full graphical UI.

use anyhow::{Result, Context};
use magics_cli::{parse_arguments, Cli, config, args::HeadlessMode};
use bevy::prelude::*;
use magics_bevy::MagicsBevyPlugin;

fn main() -> Result<()> {
    // Parse command-line arguments and environment variables
    let cli = parse_arguments();
    
    // Print metadata if requested
    if cli.metadata {
        config::print_metadata();
    }
    
    // Handle dump requests
    if let Some(dump_default) = cli.dump_default {
        return handle_dump_default(dump_default);
    }
    
    if let Some(env_type) = cli.dump_environment {
        return handle_dump_environment(env_type);
    }
    
    if cli.list_scenarios {
        return list_scenarios();
    }
    
    // Run with UI
    run_ui(&cli)
}

/// Handle dumping default configuration
fn handle_dump_default(dump: magics_cli::args::DumpDefault) -> Result<()> {
    match dump {
        magics_cli::args::DumpDefault::Config => {
            let config = gbp_config::Config::default();
            config::print_config(&config)
        },
        magics_cli::args::DumpDefault::Environment => {
            let environment = gbp_environment::Environment::default();
            config::print_environment(&environment)
        },
        magics_cli::args::DumpDefault::Formation => {
            // This is just a placeholder - formation handling would be implemented here
            println!("Formation dump not yet implemented");
            Ok(())
        },
    }
}

/// Handle dumping specific environment type
fn handle_dump_environment(env_type: gbp_environment::EnvironmentType) -> Result<()> {
    let environment = match env_type {
        gbp_environment::EnvironmentType::Circle => gbp_environment::Environment::circle(),
        gbp_environment::EnvironmentType::Complex => gbp_environment::Environment::complex(),
        gbp_environment::EnvironmentType::Intersection => gbp_environment::Environment::intersection(),
        gbp_environment::EnvironmentType::Maze => gbp_environment::Environment::maze(),
        gbp_environment::EnvironmentType::Intermediate => gbp_environment::Environment::intermediate(),
        gbp_environment::EnvironmentType::Test => gbp_environment::Environment::test(),
    };
    
    config::print_environment(&environment)
}

/// List available simulation scenarios
fn list_scenarios() -> Result<()> {
    let scenarios_dir = std::path::Path::new("./config/scenarios");
    if !scenarios_dir.exists() {
        println!("Scenarios directory not found: {}", scenarios_dir.display());
        return Ok(());
    }
    
    let entries = std::fs::read_dir(scenarios_dir)
        .with_context(|| format!("Failed to read scenarios directory: {}", scenarios_dir.display()))?;
    
    let mut scenarios = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            scenarios.push(path.to_string_lossy().to_string());
        }
    }
    
    // Sort for consistent output
    scenarios.sort();
    
    // Find max length for alignment
    let max_basename_length = scenarios
        .iter()
        .map(|s| std::path::Path::new(s).file_name().unwrap().to_string_lossy().len())
        .max()
        .unwrap_or(0);
    
    // Print the scenarios
    for name in &scenarios {
        let basename = std::path::Path::new(name).file_name().unwrap().to_string_lossy();
        if atty::is(atty::Stream::Stdout) {
            println!(
                "{:width$} {}",
                colored::Colorize::green(&basename).bold(),
                name,
                width = max_basename_length
            );
        } else {
            println!("{:width$} {}", basename, name, width = max_basename_length);
        }
    }
    
    Ok(())
}

/// Run with UI
fn run_ui(cli: &Cli) -> Result<()> {
    println!("Starting simulation with UI...");
    
    // Create the Bevy app with our plugin
    App::new()
        .add_plugins(MagicsBevyPlugin {
            headless: false, // This binary always runs with UI
            fullscreen: cli.fullscreen,
            simulations_dir: cli.simulations_dir.clone(),
            initial_scenario: cli.initial_scenario.clone(),
            width: cli.width,
            height: cli.height,
            record: cli.record,
        })
        .run();
    
    Ok(())
}
