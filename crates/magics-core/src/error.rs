//! Error types for the magics-core crate

use std::io;
use thiserror::Error;

/// Error type for simulation errors
#[derive(Error, Debug)]
pub enum SimulationError {
    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
    
    /// Error initializing the simulation environment
    #[error("Environment initialization error: {0}")]
    EnvironmentInitError(String),
    
    /// Error with the environment
    #[error("Environment error: {0}")]
    Environment(String),
    
    /// Error initializing the planner
    #[error("Planner initialization error: {0}")]
    PlannerInitError(String),
    
    /// Error running the simulation
    #[error("Simulation error: {0}")]
    SimulationError(String),
    
    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Error loading configuration
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
    
    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl From<serde_json::Error> for SimulationError {
    fn from(err: serde_json::Error) -> Self {
        SimulationError::SerializationError(err.to_string())
    }
}

impl From<serde_yaml::Error> for SimulationError {
    fn from(err: serde_yaml::Error) -> Self {
        SimulationError::SerializationError(err.to_string())
    }
}
