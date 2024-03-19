use bevy::ecs::system::Resource;
use serde::{Deserialize, Serialize};

/// **Bevy** [`Resource`]
/// The en
#[derive(Debug, Serialize, Deserialize, Resource)]
#[serde(rename_all = "kebab-case")]
pub struct Environment {
    pub matrix_representation: Vec<String>,
}

impl Default for Environment {
    fn default() -> Self {
        Environment::simple()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("RON error: {0}")]
    Ron(#[from] ron::Error),
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("Validation error: {0}")]
    InvalidEnvironment(#[from] EnvironmentError),
}

#[derive(Debug, thiserror::Error)]
pub enum EnvironmentError {
    EmptyMatrixRepresentation,
    DifferentLengthRows,
}

impl std::fmt::Display for EnvironmentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnvironmentError::EmptyMatrixRepresentation => {
                write!(f, "Environment matrix representation is empty")
            }
            EnvironmentError::DifferentLengthRows => {
                write!(
                    f,
                    "Environment matrix representation has rows of different lengths"
                )
            }
        }
    }
}

impl Environment {
    /// Attempt to parse an [`Environment`] from a RON file at `path`
    /// Returns `Err(ParseError)` if:
    /// 1. `path` does not exist on the filesystem
    /// 2. The contents of `path` are not valid RON
    /// 3. The parsed data does not represent a valid [`Environment`]
    pub fn from_file(path: &std::path::Path) -> Result<Self, ParseError> {
        std::fs::read_to_string(path)
            .map_err(Into::into)
            .and_then(|contents| Self::parse(contents.as_str()))
    }

    pub fn parse(contents: &str) -> Result<Self, ParseError> {
        let config = toml::from_str(contents)?;
        Ok(config)
    }

    /// Ensure that the [`Environment`] is valid
    /// 1. The matrix representation is not empty
    /// 2. All rows in the matrix representation are the same length
    pub fn validate(self) -> Result<Self, EnvironmentError> {
        if self.matrix_representation.is_empty() {
            Err(EnvironmentError::EmptyMatrixRepresentation)
        } else if self
            .matrix_representation
            .iter()
            .any(|row| row.len() != self.matrix_representation[0].len())
        {
            Err(EnvironmentError::DifferentLengthRows)
        } else {
            Ok(self)
        }
    }

    pub fn new(matrix_representation: Vec<String>) -> Self {
        Environment {
            matrix_representation,
        }
    }

    pub fn simple() -> Self {
        Environment {
            matrix_representation: vec!["┼".to_string()],
        }
    }

    #[rustfmt::skip]
    pub fn intermediate() -> Self {
        Environment {
            matrix_representation: vec![
                "┌┬┐ ".to_string(),
                "┘└┼┬".to_string(),
                "  └┘".to_string()
            ],
        }
    }

    #[rustfmt::skip]
    pub fn complex() -> Self {
        Environment {
            matrix_representation: vec![
                "┌─┼─┬─┐┌".to_string(),
                "┼─┘┌┼┬┼┘".to_string(),
                "┴┬─┴┼┘│ ".to_string(),
                "┌┴┐┌┼─┴┬".to_string(),
                "├─┴┘└──┘".to_string(),
            ],
        }
    }
}
