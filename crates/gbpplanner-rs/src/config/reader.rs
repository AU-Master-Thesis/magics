#![deny(missing_docs)]

use std::path::Path;

use directories::BaseDirs;

use super::Config;

/// Error type for [`read_config`]
#[derive(Debug, thiserror::Error)]
pub enum ConfigReaderError {
    /// IO error, i.e. could not read file
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// No config file found among the default locations or the one given as
    /// input
    #[error("No config file found")]
    NoConfigFile,
    /// Config parse error. See [`ParseError`]
    #[error("Parse error {0}")]
    Parse(#[from] super::ParseError),
}

/// Result type for [`read_config`]
pub type Result<T> = std::result::Result<T, ConfigReaderError>;

fn default_paths() -> Vec<std::path::PathBuf> {
    let mut paths = vec![];

    if let Some(base_dirs) = BaseDirs::new() {
        paths.push(
            base_dirs
                .config_dir()
                .join("gbpplanner")
                .join("config.toml")
                .to_path_buf(),
        );
    }

    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join("config/config.toml"));
    }

    paths
}

pub fn read_config<P: AsRef<Path>>(path: Option<P>) -> Result<Config> {
    if let Some(path) = path
        .map(|p| p.as_ref().to_path_buf())
        .into_iter()
        .chain(default_paths().into_iter())
        .filter(|p| p.exists())
        .nth(0)
    {
        Ok(Config::from_file(path)?)
    } else {
        Err(ConfigReaderError::NoConfigFile)
    }
}
