use super::ParseError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlacementStrategy {
    Equal,
    Random,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl From<Point> for bevy::math::Vec2 {
    fn from(value: Point) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Shape {
    Circle { radius: f32, center: Point },
    Polygon(Vec<Point>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Formation {
    pub repeat: bool,
    pub time: f32,
    pub robots: usize,
    pub shape: Shape,
    pub placement_strategy: PlacementStrategy,
}

impl Default for Formation {
    fn default() -> Self {
        Self {
            repeat: false,
            time: 5.0,
            robots: 8,
            shape: Shape::Circle {
                radius: 5.0,
                center: Point { x: 0.0, y: 0.0 },
            },
            placement_strategy: PlacementStrategy::Equal,
        }
    }
}

impl Formation {
    /// Parse a config file
    /// Returns a `ParseError` if the file cannot be parsed
    pub fn parse(file_path: &std::path::PathBuf) -> Result<Self, ParseError> {
        let file_contents = std::fs::read_to_string(file_path)?;
        let formation = toml::from_str(file_contents.as_str())?;
        Ok(formation)
    }
}
