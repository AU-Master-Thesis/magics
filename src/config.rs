use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Meter(f64);

#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationSection {
    pub communication_radius: Meter,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GbpSection {
    pub iterations_per_timestep: usize,
    pub damping: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub simulation: SimulationSection,
    pub gbp: GbpSection,
}

impl Config {
    /// Parse a config file
    /// Returns a `ParseError` if the file cannot be parsed
    pub fn parse(file_path: std::path::PathBuf) -> Result<Self, ParseError> {
        let config = toml::from_str(std::fs::read_to_string(file_path)?.as_str())?;
        Ok(config)
    }

    /// Generate a default config
    /// Used when no config file is provided
    pub fn generate_default() -> Self {
        Self {
            simulation: SimulationSection {
                communication_radius: Meter(1.0),
            },
            gbp: GbpSection {
                iterations_per_timestep: 1,
                damping: 0.0,
            },
        }
    }
}


