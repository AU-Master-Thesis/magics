use bevy::ecs::system::Resource;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Grid(Vec<String>);

impl Grid {
    pub fn iter(&self) -> std::slice::Iter<String> {
        self.0.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn rows(&self) -> usize {
        self.0.len()
    }

    pub fn cols(&self) -> usize {
        self.0[0].chars().count()
    }

    // override the index operator to allow for easy access to the grid
    pub fn get(&self, row: usize, col: usize) -> Option<char> {
        self.0.get(row).and_then(|r| r.chars().nth(col))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Cell {
    pub row: usize,
    pub col: usize,
}

// #[derive(Debug, Serialize, Deserialize)]
// #[serde(rename_all = "kebab-case")]
// pub struct shapes(Vec<(Cell, Shape)>);

/// **Bevy** [`Resource`]
/// The en
#[derive(Debug, Serialize, Deserialize, Resource)]
#[serde(rename_all = "kebab-case")]
pub struct Environment {
    pub grid: Grid,
    path_width: f32,
    obstacle_height: f32,
    tile_size: f32,
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
    EmptyGrid,
    DifferentLengthRows,
}

impl std::fmt::Display for EnvironmentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnvironmentError::EmptyGrid => {
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
        let config: Environment = toml::from_str::<Environment>(contents)
            .map_err(ParseError::from)?
            .validate()
            .map_err(EnvironmentError::from)?;
        Ok(config)
    }

    /// Ensure that the [`Environment`] is valid
    /// 1. The matrix representation is not empty
    /// 2. All rows in the matrix representation are the same length
    pub fn validate(self) -> Result<Self, EnvironmentError> {
        if self.grid.is_empty() {
            Err(EnvironmentError::EmptyGrid)
        } else if self
            .grid
            .iter()
            .any(|row| row.chars().count() != self.grid.cols())
        {
            Err(EnvironmentError::DifferentLengthRows)
        } else {
            Ok(self)
        }
    }

    pub fn new(
        matrix_representation: Vec<String>,
        path_width: f32,
        obstacle_height: f32,
        tile_size: f32,
    ) -> Self {
        Environment {
            grid: Grid(matrix_representation),
            path_width,
            obstacle_height,
            tile_size,
        }
    }

    pub fn simple() -> Self {
        Environment {
            grid: Grid(vec!["┼".to_string()]),
            path_width: 0.1325,
            obstacle_height: 1.0,
            tile_size: 100.0,
        }
    }

    #[rustfmt::skip]
    pub fn intermediate() -> Self {
        Environment {
            grid: Grid(vec![
                "┌┬┐ ".to_string(),
                "┘└┼┬".to_string(),
                "  └┘".to_string()
            ]),
            path_width: 0.1325,
            obstacle_height: 1.0,
            tile_size: 50.0,
        }
    }

    #[rustfmt::skip]
    pub fn complex() -> Self {
        Environment {
            grid: Grid(vec![
                "┌─┼─┬─┐┌".to_string(),
                "┼─┘┌┼┬┼┘".to_string(),
                "┴┬─┴┼┘│ ".to_string(),
                "┌┴┐┌┼─┴┬".to_string(),
                "├─┴┘└──┘".to_string(),
            ]),
            path_width: 0.4,
            obstacle_height: 1.0,
            tile_size: 25.0,
        }
    }

    pub fn path_width(&self) -> f32 {
        self.path_width
    }

    pub fn obstacle_height(&self) -> f32 {
        self.obstacle_height
    }

    pub fn tile_size(&self) -> f32 {
        self.tile_size
    }
}
