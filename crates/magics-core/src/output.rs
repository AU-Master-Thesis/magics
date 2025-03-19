//! Output management for simulation results

use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

use serde::{Serialize, Deserialize};
use crate::error::SimulationError;
use crate::simulation::OutputSettings;
use crate::types::EnvironmentState;

/// Output format for simulation results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
pub enum OutputFormat {
    /// JSON format
    Json,
    /// CSV format 
    Csv,
    /// YAML format
    Yaml,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "csv" => Ok(OutputFormat::Csv),
            "yaml" => Ok(OutputFormat::Yaml),
            _ => Err(format!("Unknown format: {}", s)),
        }
    }
}

/// Manages the output of simulation results
pub struct OutputManager {
    /// Output settings
    settings: OutputSettings,
    /// Optional file handle
    file: Option<File>,
}

impl OutputManager {
    /// Create a new output manager with the given settings
    pub fn new(settings: &OutputSettings) -> Result<Self, SimulationError> {
        let file = if let Some(path) = &settings.file_path {
            let file_path = Path::new(path);
            
            // Create parent directories if they don't exist
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    SimulationError::IoError(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Failed to create directories: {}", e),
                    ))
                })?;
            }
            
            // Open file for writing
            Some(File::create(file_path).map_err(|e| {
                SimulationError::IoError(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to open output file: {}", e),
                ))
            })?)
        } else {
            None
        };
        
        Ok(Self {
            settings: settings.clone(),
            file,
        })
    }
    
    /// Write the environment state to the configured outputs
    pub fn write(&mut self, state: &EnvironmentState) -> Result<(), SimulationError> {
        // Format the state as JSON (the only supported format for now)
        let formatted = serde_json::to_string_pretty(state).map_err(|e| {
            SimulationError::IoError(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to serialize to JSON: {}", e),
            ))
        })?;
        
        // Write to stdout if configured
        if self.settings.stdout {
            println!("{}", formatted);
        }
        
        // Write to file if configured
        if let Some(file) = &mut self.file {
            writeln!(file, "{}", formatted).map_err(|e| {
                SimulationError::IoError(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to write to file: {}", e),
                ))
            })?;
        }
        
        Ok(())
    }
    
    /// Format the environment state as CSV
    fn format_as_csv(&self, state: &EnvironmentState) -> Result<String, SimulationError> {
        let mut output = String::new();
        
        // Write header if file is empty
        if self.file.as_ref().map_or(true, |f| f.metadata().map_or(true, |m| m.len() == 0)) {
            output.push_str("time,agent_id,position_x,position_y,velocity_x,velocity_y,target_x,target_y\n");
        }
        
        // Write agent data
        for agent in &state.agents {
            output.push_str(&format!(
                "{},{},{},{},{},{},{},{}\n",
                state.time.get(),
                agent.id.0,
                agent.position.x,
                agent.position.y,
                agent.velocity.x,
                agent.velocity.y,
                agent.target.x,
                agent.target.y
            ));
        }
        
        Ok(output)
    }
}
