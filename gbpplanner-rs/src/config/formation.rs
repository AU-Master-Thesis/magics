// use super::ParseError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlacementStrategy {
    Equal,
    Random,
}

impl PlacementStrategy {
    /// Returns `true` if the placement strategy is [`Equal`].
    ///
    /// [`Equal`]: PlacementStrategy::Equal
    #[must_use]
    pub fn is_equal(&self) -> bool {
        matches!(self, Self::Equal)
    }

    /// Returns `true` if the placement strategy is [`Random`].
    ///
    /// [`Random`]: PlacementStrategy::Random
    #[must_use]
    pub fn is_random(&self) -> bool {
        matches!(self, Self::Random)
    }
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

impl From<(f32, f32)> for Point {
    fn from(value: (f32, f32)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ShapeError {
    #[error("radius cannot be negative")]
    NegativeRadius,
    #[error("A polygon with 0 vertices is not allowed")]
    PolygonWithZeroVertices,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Shape {
    Circle { radius: f32, center: Point },
    Polygon(Vec<Point>),
}

impl Shape {
    /// Returns `true` if the shape is [`Polygon`].
    ///
    /// [`Polygon`]: Shape::Polygon
    #[must_use]
    pub fn is_polygon(&self) -> bool {
        matches!(self, Self::Polygon(..))
    }

    /// Returns `true` if the shape is [`Circle`].
    ///
    /// [`Circle`]: Shape::Circle
    #[must_use]
    pub fn is_circle(&self) -> bool {
        matches!(self, Self::Circle { .. })
    }

    pub fn as_polygon(&self) -> Option<&Vec<Point>> {
        if let Self::Polygon(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

/// Shorthand to construct `Shape::Polygon(vec![Point {x: $x, y: $y}, ... ])`
#[macro_export]
macro_rules! polygon {
    [$(($x:expr, $y:expr)),+ $(,)?] => {{
        let vertices = vec![
            $(
                Point {x: $x, y: $y}
            ),+
        ];
        Shape::Polygon(vertices)
    }}
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
            // shape: Shape::Circle {
            //     radius: 5.0,
            //     center: Point { x: 0.0, y: 0.0 },
            // },
            shape: polygon![(0., 0.), (2., 3.), (5., 0.5)],
            placement_strategy: PlacementStrategy::Equal,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("time cannot be negative")]
    NegativeTime,
    #[error("Shape error: {0}")]
    ShapeError(#[from] ShapeError),
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("Validation error: {0}")]
    Invalid(#[from] ValidationError),
}

impl Formation {
    /// Attempt to parse a `Formation` from a TOML file at `path`
    /// Returns `Err(ParseError)`  if:
    /// 1. `path` does not exist on the filesystem.
    /// 2. The contents of `path` is not valid TOML.
    /// 3. The parsed data does not represent a valid `Formation`.
    pub fn from_file(path: &std::path::PathBuf) -> Result<Self, ParseError> {
        let file_contents = std::fs::read_to_string(path)?;
        let formation = Self::parse(file_contents.as_str())?;
        Ok(formation)
    }

    /// Attempt to parse a `Formation` from a TOML encoded string.
    /// Returns `Err(ParseError)`  if:
    /// 1. `contents` is not valid TOML.
    /// 2. The parsed data does not represent a valid `Formation`.
    pub fn parse(contents: &str) -> Result<Self, ParseError> {
        let formation: Formation = toml::from_str(contents)?;
        let formation = formation.validate()?;
        Ok(formation)
    }

    /// Ensure that the Formation is in a valid state
    /// Invalid states are:
    /// 1. self.time <= 0.0
    /// 2. if self.shape is a circle and the radius is <= 0.0
    /// 3. if self.shape is a polygon and polygon.is_empty()
    pub fn validate(self) -> Result<Self, ValidationError> {
        if self.time < 0.0 {
            Err(ValidationError::NegativeTime)
        } else {
            match self.shape {
                Shape::Polygon(vertices) if vertices.is_empty() => {
                    Err(ShapeError::PolygonWithZeroVertices.into())
                }
                Shape::Circle { radius, center: _ } if radius <= 0.0 => {
                    Err(ShapeError::NegativeRadius.into())
                }
                _ => Ok(self),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_valid() {
        let default = Formation::default();
        assert!(default.validate().is_ok());
    }

    #[test]
    fn negative_time_is_invalid() {
        let invalid = Formation {
            time: -1.0,
            ..Default::default()
        };

        assert!(matches!(
            invalid.validate(),
            Err(ValidationError::NegativeTime)
        ));
    }

    #[test]
    fn zero_vertices_is_invalid() {
        let invalid = Formation {
            shape: Shape::Polygon(vec![]),
            ..Default::default()
        };

        assert!(matches!(
            invalid.validate(),
            Err(ValidationError::ShapeError(
                ShapeError::PolygonWithZeroVertices
            ))
        ))
    }

    #[test]
    fn negative_radius_is_invalid() {
        let invalid = Formation {
            shape: Shape::Circle {
                radius: -1.0,
                center: Point { x: 0.0, y: 0.0 },
            },
            ..Default::default()
        };

        assert!(matches!(
            invalid.validate(),
            Err(ValidationError::ShapeError(ShapeError::NegativeRadius))
        ))
    }

    #[test]
    fn zero_radius_is_invalid() {
        let invalid = Formation {
            shape: Shape::Circle {
                radius: 0.0,
                center: Point { x: 0.0, y: 0.0 },
            },
            ..Default::default()
        };

        assert!(matches!(
            invalid.validate(),
            Err(ValidationError::ShapeError(ShapeError::NegativeRadius))
        ))
    }

    #[test]
    fn zero_time_is_okay() {
        let zero_is_okay = Formation {
            time: 0.0,
            ..Default::default()
        };

        assert!(zero_is_okay.validate().is_ok());
    }

    mod polygon_macro {

        use super::*;
        use pretty_assertions::assert_eq;

        fn float_eq(lhs: f32, rhs: f32) -> bool {
            f32::abs(lhs - rhs) <= f32::EPSILON
        }

        #[test]
        fn single_vertex() {
            let shape = polygon![(1e2, 42.0)];
            assert!(shape.is_polygon());
            let inner = shape.as_polygon().expect("know it is a Polygon");
            assert!(!inner.is_empty());
            assert_eq!(inner.len(), 1);

            assert!(inner
                .iter()
                .zip(vec![(1e2, 42.0)].into_iter())
                .all(|(p, (x, y))| float_eq(p.x, x) && float_eq(p.y, y)));
        }

        #[test]
        fn multiple_vertices() {
            let shape = polygon![(0., 0.), (1., 1.), (5., -8.)];
            assert!(shape.is_polygon());
            let inner = shape.as_polygon().expect("know it is a Polygon");
            assert!(!inner.is_empty());
            assert_eq!(inner.len(), 3);

            assert!(inner
                .iter()
                .zip(vec![(0., 0.), (1., 1.), (5., -8.)].into_iter())
                .all(|(p, (x, y))| float_eq(p.x, x) && float_eq(p.y, y)));
        }

        /// Really just a test to see if the macro compiles and matches
        #[test]
        fn trailing_comma() {
            let shape = polygon![(0., 0.),];
            assert!(shape.is_polygon());
            let inner = shape.as_polygon().expect("know it is a Polygon");
            assert!(!inner.is_empty());
            assert_eq!(inner.len(), 1);

            assert!(inner
                .iter()
                .zip(vec![(0., 0.)].into_iter())
                .all(|(p, (x, y))| float_eq(p.x, x) && float_eq(p.y, y)));
        }
    }
}
