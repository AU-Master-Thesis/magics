//! Configuration handling for Magics CLI
//!
//! This module provides utilities for loading and saving configuration files.

use std::path::{Path, PathBuf};
use std::fs;
use std::io;

use anyhow::{Result, Context};
use gbp_config::Config;
use gbp_environment::Environment;
use serde::{Serialize, Deserialize};
use colored::Colorize;

use crate::args::{Cli, OutputFormat};

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Whether to output to stdout
    pub stdout: bool,
    /// Output file path
    pub file_path: Option<PathBuf>,
    /// Output format
    pub format: String,
    /// Output frequency (every N steps)
    pub frequency: usize,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            stdout: true,
            file_path: None,
            format: "json".to_string(),
            frequency: 1,
        }
    }
}

/// Convert CLI arguments to output configuration
pub fn cli_to_output_config(cli: &Cli) -> OutputConfig {
    OutputConfig {
        stdout: true, // Always output to stdout unless explicitly disabled
        file_path: cli.output_file.clone(),
        format: match cli.output_format {
            OutputFormat::Json => "json".to_string(),
            OutputFormat::Csv => "csv".to_string(),
            OutputFormat::Yaml => "yaml".to_string(),
        },
        frequency: 1, // Default to every step
    }
}

/// Load a GBP configuration from a file
pub fn load_config(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;
    
    let config = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
    
    Ok(config)
}

/// Save a GBP configuration to a file
pub fn save_config(config: &Config, path: &Path) -> Result<()> {
    let content = toml::to_string_pretty(config)
        .with_context(|| "Failed to serialize config")?;
    
    fs::write(path, content)
        .with_context(|| format!("Failed to write config to file: {}", path.display()))?;
    
    Ok(())
}

/// Load an environment from a file
pub fn load_environment(path: &Path) -> Result<Environment> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read environment file: {}", path.display()))?;
    
    let environment = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse environment file: {}", path.display()))?;
    
    Ok(environment)
}

/// Save an environment to a file
pub fn save_environment(environment: &Environment, path: &Path) -> Result<()> {
    let content = serde_yaml::to_string(environment)
        .with_context(|| "Failed to serialize environment")?;
    
    fs::write(path, content)
        .with_context(|| format!("Failed to write environment to file: {}", path.display()))?;
    
    Ok(())
}

/// Print the configuration to stdout
pub fn print_config(config: &Config) -> Result<()> {
    let is_terminal = atty::is(atty::Stream::Stdout);
    let toml_str = toml::to_string_pretty(config)
        .with_context(|| "Failed to serialize config")?;
    
    if is_terminal {
        println!("{}", toml_str);
    } else {
        println!("{}", toml_str);
    }
    
    Ok(())
}

/// Print the environment to stdout
pub fn print_environment(environment: &Environment) -> Result<()> {
    let is_terminal = atty::is(atty::Stream::Stdout);
    let yaml_str = serde_yaml::to_string(environment)
        .with_context(|| "Failed to serialize environment")?;
    
    if is_terminal {
        println!("{}", yaml_str);
    } else {
        println!("{}", yaml_str);
    }
    
    Ok(())
}

/// Print metadata about the application
pub fn print_metadata() {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let authors = env!("CARGO_PKG_AUTHORS").split(':').collect::<Vec<_>>();
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    
    eprintln!("{}:   {}", "target arch".green().bold(), std::env::consts::ARCH);
    eprintln!("{}:     {}", "target os".green().bold(), std::env::consts::OS);
    eprintln!("{}: {}", "target family".green().bold(), std::env::consts::FAMILY);
    
    eprintln!("{}:          {}", "name".green().bold(), name);
    eprintln!("{}:", "authors".green().bold());
    for &author in &authors {
        eprintln!(" - {}", author);
    }
    eprintln!("{}:       {}", "version".green().bold(), version);
    eprintln!("{}:  {}", "manifest_dir".green().bold(), manifest_dir);
}
