//! Scenario management for Magics CLI
//!
//! This module provides utilities for discovering, loading, and managing scenarios

use std::path::{Path, PathBuf};
use std::fs;
use std::io;

use anyhow::{Result, Context, anyhow};
use colored::Colorize;
use gbp_config::{Config, FormationGroup};
use gbp_environment::Environment;

/// Scenario information
#[derive(Debug, Clone)]
pub struct Scenario {
    /// Name of the scenario
    pub name: String,
    /// Path to the scenario directory
    pub path: PathBuf,
    /// Description of the scenario (if available)
    pub description: Option<String>,
    /// Configuration (if loaded)
    pub config: Option<Config>,
    /// Environment (if loaded)
    pub environment: Option<Environment>,
    /// Formation group (if loaded)
    pub formation: Option<FormationGroup>,
}

impl Scenario {
    /// Create a new scenario from a directory path
    pub fn from_directory(dir_path: &Path) -> Result<Self> {
        let name = dir_path
            .file_name()
            .ok_or_else(|| anyhow!("Invalid scenario directory path"))?
            .to_string_lossy()
            .to_string();

        // Check for description file
        let description_path = dir_path.join("description.txt");
        let description = if description_path.exists() {
            Some(fs::read_to_string(&description_path)
                .with_context(|| format!("Failed to read description file: {}", description_path.display()))?)
        } else {
            None
        };

        Ok(Self {
            name,
            path: dir_path.to_path_buf(),
            description,
            config: None,
            environment: None,
            formation: None,
        })
    }

    /// Load the scenario's configuration and environment
    pub fn load(&mut self) -> Result<()> {
        // Load config
        let config_path = self.path.join("config.toml");
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;
            
            let config = toml::from_str(&content)
                .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;
            
            self.config = Some(config);
        }

        // Load environment
        let env_path = self.path.join("environment.yaml");
        if env_path.exists() {
            let content = fs::read_to_string(&env_path)
                .with_context(|| format!("Failed to read environment file: {}", env_path.display()))?;
            
            let environment = serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to parse environment file: {}", env_path.display()))?;
            
            self.environment = Some(environment);
        }
        
        // Load formation group
        let formation_files = ["formation.yaml", "formation.ron"];
        for filename in &formation_files {
            let formation_path = self.path.join(filename);
            if formation_path.exists() {
                self.formation = if filename.ends_with(".yaml") {
                    let content = fs::read_to_string(&formation_path)
                        .with_context(|| format!("Failed to read formation file: {}", formation_path.display()))?;
                    
                    Some(FormationGroup::parse_from_yaml(&content)
                        .with_context(|| format!("Failed to parse formation file: {}", formation_path.display()))?)
                } else {
                    // RON format
                    Some(FormationGroup::from_ron_file(&formation_path)
                        .with_context(|| format!("Failed to parse formation file: {}", formation_path.display()))?)
                };
                
                // Break after loading the first formation file found
                break;
            }
        }

        Ok(())
    }
}

/// Discover all available scenarios in a directory
pub fn discover_scenarios(dir: &Path) -> Result<Vec<Scenario>> {
    if !dir.exists() {
        return Err(anyhow!("Scenarios directory does not exist: {}", dir.display()));
    }

    if !dir.is_dir() {
        return Err(anyhow!("Scenarios path is not a directory: {}", dir.display()));
    }

    let mut scenarios = Vec::new();

    // Read all subdirectories in the scenarios directory
    let entries = fs::read_dir(dir)
        .with_context(|| format!("Failed to read scenarios directory: {}", dir.display()))?;

    for entry in entries {
        let entry = entry.with_context(|| "Failed to read directory entry")?;
        let path = entry.path();

        if path.is_dir() {
            // Check if this looks like a scenario directory
            // At minimum, it should have either a config.toml or environment.yaml file
            let has_config = path.join("config.toml").exists();
            let has_env = path.join("environment.yaml").exists();

            if has_config || has_env {
                match Scenario::from_directory(&path) {
                    Ok(scenario) => scenarios.push(scenario),
                    Err(err) => eprintln!("Warning: Failed to load scenario from {}: {}", path.display(), err),
                }
            }
        }
    }

    // Sort scenarios by name
    scenarios.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(scenarios)
}

/// Load a specific scenario by name
pub fn load_scenario(scenarios_dir: &Path, name: &str) -> Result<Scenario> {
    let scenario_dir = scenarios_dir.join(name);
    
    if !scenario_dir.exists() {
        return Err(anyhow!("Scenario directory does not exist: {}", scenario_dir.display()));
    }

    let mut scenario = Scenario::from_directory(&scenario_dir)?;
    scenario.load()?;

    Ok(scenario)
}

/// Print a list of all available scenarios
pub fn print_scenarios(scenarios: &[Scenario]) {
    if scenarios.is_empty() {
        println!("No scenarios found.");
        return;
    }

    println!("\n{}", "Available Scenarios:".green().bold());
    println!("{}", "===================".green());

    for (idx, scenario) in scenarios.iter().enumerate() {
        println!("{}. {} ({})", 
            idx + 1, 
            scenario.name.yellow().bold(),
            scenario.path.display());
        
        if let Some(desc) = &scenario.description {
            println!("   {}", desc.trim());
        }
        
        println!();
    }
}

/// List all available scenarios in the given directory
pub fn list_all_scenarios(scenarios_dir: &Path) -> Result<()> {
    let scenarios = discover_scenarios(scenarios_dir)?;
    print_scenarios(&scenarios);
    Ok(())
}
