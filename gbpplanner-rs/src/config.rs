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
    pub num_robots: usize,
}

impl Default for SimulationSection {
    fn default() -> Self {
        Self {
            communication_radius: Meter(1.0),
            num_robots: 1,
        }
    }
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

impl Default for GbpSection {
    fn default() -> Self {
        Self {
            iterations_per_timestep: 1,
            damping: 0.0,
        }
    }
}

impl Default for Config {
    /// Generate a default config
    /// Used when no config file is provided
    fn default() -> Self {
        Self {
            simulation: SimulationSection::default(),
            gbp: GbpSection::default(),
        }
    }
}

impl Config {
    /// Parse a config file
    /// Returns a `ParseError` if the file cannot be parsed
    pub fn parse(file_path: &std::path::PathBuf) -> Result<Self, ParseError> {
        let config = toml::from_str(std::fs::read_to_string(file_path)?.as_str())?;
        Ok(config)
    }
}
