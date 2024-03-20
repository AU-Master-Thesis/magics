#![warn(missing_docs)]
//! A module for working with robot formations flexibly.
use std::{num::NonZeroUsize, time::Duration};

use bevy::ecs::system::Resource;
use serde::{Deserialize, Serialize};
use typed_floats::StrictlyPositiveFinite;
use unit_interval::UnitInterval;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlacementStrategy {
    Equal,
    Random,
    Map,
}

/// A relative point within the boudaries of the map.
/// ...
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RelativePoint {
    pub x: UnitInterval,
    pub y: UnitInterval,
}

impl RelativePoint {
    /// Create a new `RelativePoint` from a pair of values.
    /// Returns an error if either `x` or `y` is not in the interval [0.0, 1.0].
    pub fn new(x: f64, y: f64) -> Result<Self, unit_interval::UnitIntervalError> {
        Ok(Self {
            x: UnitInterval::new(x)?,
            y: UnitInterval::new(y)?,
        })
    }

    /// Returns the x and y values as a tuple
    pub fn get(&self) -> (f64, f64) {
        (self.x.get(), self.y.get())
    }
}

impl TryFrom<(f64, f64)> for RelativePoint {
    type Error = unit_interval::UnitIntervalError;

    fn try_from(value: (f64, f64)) -> Result<Self, Self::Error> {
        Ok(Self {
            x: UnitInterval::new(value.0)?,
            y: UnitInterval::new(value.1)?,
        })
    }
}

// #[derive(Debug, thiserror::Error)]
// pub enum PointError {
//     #[error("x is out of bounds: {0}")]
//     XOutOfBounds(#[from] unit_interval::UnitIntervalError),
//     #[error("y is out of bounds: {0}")]
//     YOutOfBounds(#[from] unit_interval::UnitIntervalError),
//     #[error("both x and y are out of bounds: x: {0}, y: {1}")]
//     BothOutOfBounds(f64, f64),
// }
//
// impl From<Point> for bevy::math::Vec2 {
//     fn from(value: Point) -> Self {
//         Self {
//             x: value.x,
//             y: value.y,
//         }
//     }
// }
//
// impl From<&Point> for bevy::math::Vec2 {
//     fn from(value: &Point) -> Self {
//         Self {
//             x: value.x,
//             y: value.y,
//         }
//     }
// }
//
// impl From<(f32, f32)> for Point {
//     fn from(value: (f32, f32)) -> Self {
//         Self {
//             x: value.0,
//             y: value.1,
//         }
//     }
// }

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
    Circle {
        radius: StrictlyPositiveFinite<f32>,
        center: RelativePoint,
    },
    Polygon(Vec<RelativePoint>),
    Line((RelativePoint, RelativePoint)),
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

    pub fn as_polygon(&self) -> Option<&Vec<RelativePoint>> {
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
        	RelativePoint::new($x, $y).expect("both x and y are within the interval [0.0, 1.0]")
            ),+
        ];
        Shape::Polygon(vertices)
    }}
}

/// Shorthand to construct `Shape::Line((Point {x: $x1, y: $y1}, Point {x: $x2,
/// y: $y2}))`
#[macro_export]
macro_rules! line {
    [($x1:expr, $y1:expr), ($x2:expr, $y2:expr)] => {
        // Shape::Line((Point { x: $x1, y: $y1 }, Point { x: $x2, y: $y2 }))
        // Shape::Line((Point { x: ($x1 as f64).try_from().unwrap(), y: ($y1 as f64).try_from().unwrap() }, Point { x: ($x2 as f64).try_from().unwrap(), y: f64::try_from().unwrap() }))
        Shape::Line((RelativePoint::new($x1, $y1).expect("both x1 and y1 are within the interval [0.0, 1.0]"), RelativePoint::new($x2, $y2).expect("both x2 and y2 are within the interval [0.0, 1.0]")))
    };
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Waypoint {
    pub shape:              Shape,
    pub placement_strategy: PlacementStrategy,
}

#[derive(Debug, thiserror::Error)]
pub enum FormationError {
    #[error("time cannot be negative")]
    NegativeTime,
    #[error("Shape error: {0}")]
    ShapeError(#[from] ShapeError),
    #[error(
        "At least two waypoints needs to be given, as the first and last waypoint represent the \
         start and end for the formation"
    )]
    LessThanTwoWaypoints,
    #[error("FormationGroup has no formations")]
    NoFormations,
    #[error("A formation has to contain at least one robot")]
    NoRobots,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Formation {
    /// If true then then a new formation instance will be spawned every
    /// [`Self::delay`]. If false, a single formation will be spawned after
    /// [`Self::delay`]
    pub repeat:    bool,
    /// The delay from the start of the simulation after which the formation
    /// should spawn. If [`Self::repeat`] is true, then `delay` is
    /// interpreted as a timeout between every formation spawn.
    // pub delay:     f32,
    pub delay: Duration,
    /// Number of robots to spawn
    pub robots:    NonZeroUsize,
    /// A list of waypoints.
    /// `NOTE` The first waypoint is assumed to be the spawning point.
    /// < 2 waypoints is an invalid state
    pub waypoints: Vec<Waypoint>,
}

impl Default for Formation {
    fn default() -> Self {
        Self {
            repeat:    false,
            delay:     Duration::from_secs(5),
            robots:    unsafe { NonZeroUsize::new_unchecked(1) },
            waypoints: vec![
                Waypoint {
                    placement_strategy: PlacementStrategy::Random,
                    shape:              line![(0.4, 0.0), (0.6, 0.0)],
                },
                Waypoint {
                    placement_strategy: PlacementStrategy::Map,
                    shape:              line![(0.4, 0.4), (0.6, 0.6)],
                },
                Waypoint {
                    placement_strategy: PlacementStrategy::Map,
                    shape:              line![(0.0, 0.4), (0.0, 0.6)],
                },
            ],
        }
    }
}

impl Formation {
    // /// Attempt to parse a `Formation` from a TOML file at `path`
    // /// Returns `Err(ParseError)`  if:
    // /// 1. `path` does not exist on the filesystem.
    // /// 2. The contents of `path` is not valid TOML.
    // /// 3. The parsed data does not represent a valid `Formation`.
    // pub fn from_file(path: &std::path::PathBuf) -> Result<Self, ParseError> {
    //     let file_contents = std::fs::read_to_string(path)?;
    //     let formation = Self::parse(file_contents.as_str())?;
    //     Ok(formation)
    // }

    // /// Attempt to parse a `Formation` from a TOML encoded string.
    // /// Returns `Err(ParseError)`  if:
    // /// 1. `contents` is not valid TOML.
    // /// 2. The parsed data does not represent a valid `Formation`.
    // pub fn parse(contents: &str) -> Result<Self, ParseError> {
    //     let formation: Formation = toml::from_str(contents)?;
    //     let formation = formation.validate()?;
    //     Ok(formation)
    // }

    fn valid(&self) -> Result<(), FormationError> {
        Ok(())
    }

    /// Ensure that the Formation is in a valid state
    /// Invalid states are:
    /// 1. self.time <= 0.0
    /// 2. if self.shape is a circle and the radius is <= 0.0
    /// 3. if self.shape is a polygon and polygon.is_empty()
    pub fn validate(self) -> Result<Self, FormationError> {
        // if self.delay < 0.0 {
        //     Err(FormationError::NegativeTime)
        if self.waypoints.len() < 2 {
            Err(FormationError::LessThanTwoWaypoints)
        } else {
            // TODO: finish
            for (index, waypoint) in self.waypoints.iter().enumerate() {
                match &waypoint.shape {
                    Shape::Polygon(vertices) if vertices.is_empty() => {
                        return Err(ShapeError::PolygonWithZeroVertices.into())
                    }
                    // Shape::Circle { radius, center: _ } if *radius <= 0.0 => {
                    //     return Err(ShapeError::NegativeRadius.into())
                    // }
                    _ => continue,
                }
            }
            Ok(self)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    // #[error("TOML error: {0}")]
    // Toml(#[from] toml::de::Error),
    #[error("Validation error: {0}")]
    InvalidFormation(#[from] FormationError),
    #[error("Invalid formation group: {0}")]
    InvalidFormationGroup(#[from] FormationGroupError),
    #[error("RON error: {0}")]
    Ron(#[from] ron::Error),
}

/// A `FormationGroup` represent multiple `Formation`s
#[derive(Debug, Serialize, Deserialize, Resource)]
#[serde(rename_all = "kebab-case")]
pub struct FormationGroup {
    pub formations: Vec<Formation>,
}

#[derive(Debug, thiserror::Error)]
pub enum FormationGroupError {
    NoFormations,
    InvalidFormations(Vec<(usize, FormationError)>),
}

impl std::fmt::Display for FormationGroupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormationGroupError::NoFormations => write!(
                f,
                "Formation group is empty. A formation group contains at least one formation",
            ),
            FormationGroupError::InvalidFormations(ref inner) => {
                for (index, error) in inner.iter() {
                    write!(f, "formation[{}] is invalid with error: {}", index, error)?;
                }
                Ok(())
            }
        }
    }
}

impl FormationGroup {
    /// Attempt to parse a `FormationGroup` from a TOML file at `path`
    /// Returns `Err(ParseError)`  if:
    /// 1. `path` does not exist on the filesystem.
    /// 2. The contents of `path` is not valid TOML.
    /// 3. The parsed data does not represent a valid `FormationGroup`.
    pub fn from_file(path: &std::path::Path) -> Result<Self, ParseError> {
        std::fs::read_to_string(path)
            .map_err(Into::into)
            .and_then(|file_contents| Self::parse(file_contents.as_str()))
    }

    /// Attempt to parse a `FormationGroup` from a RON encoded string.
    /// Returns `Err(ParseError)`  if:
    /// 1. `contents` is not valid RON.
    /// 2. The parsed data does not represent a valid `FormationGroup`.
    pub fn parse(contents: &str) -> Result<Self, ParseError> {
        ron::from_str::<FormationGroup>(contents)
            .map_err(|span| span.code)?
            .validate()
            .map_err(Into::into)
    }

    /// Ensure that the `FormationGroup` is in a valid state
    /// 1. At least one `Formation` is required
    /// 2. Validate each `Formation`
    pub fn validate(self) -> Result<Self, FormationGroupError> {
        if self.formations.is_empty() {
            Err(FormationGroupError::NoFormations)
        } else {
            let invalid_formations = self
                .formations
                .iter()
                .enumerate()
                .filter_map(|(index, formation)| formation.valid().err().map(|err| (index, err)))
                .collect::<Vec<_>>();

            if !invalid_formations.is_empty() {
                Err(FormationGroupError::InvalidFormations(invalid_formations))
            } else {
                Ok(self)
            }
        }
    }
}

impl Default for FormationGroup {
    fn default() -> Self {
        Self {
            formations: vec![Formation::default()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod formation {
        use super::*;

        #[test]
        fn default_is_valid() {
            let default = Formation::default();
            assert!(matches!(default.validate(), Ok(Formation { .. })));
        }

        // #[test]
        // fn negative_time_is_invalid() {
        //     let invalid = Formation {
        //         delay: -1.0,
        //         ..Default::default()
        //     };
        //
        //     assert!(matches!(
        //         invalid.validate(),
        //         Err(FormationError::NegativeTime)
        //     ));
        // }

        #[test]
        fn zero_waypoints_are_invalid() {
            let invalid = Formation {
                waypoints: vec![],
                ..Default::default()
            };

            assert!(matches!(
                invalid.validate(),
                Err(FormationError::LessThanTwoWaypoints)
            ))
        }

        #[test]
        fn one_waypoint_is_invalid() {
            let invalid = Formation {
                waypoints: vec![Waypoint {
                    shape:              Shape::Line((
                        RelativePoint::new(0.0, 0.0).unwrap(),
                        RelativePoint::new(0.4, 0.4).unwrap(),
                    )),
                    placement_strategy: PlacementStrategy::Random,
                }],
                ..Default::default()
            };

            assert!(matches!(
                invalid.validate(),
                Err(FormationError::LessThanTwoWaypoints)
            ))
        }

        // TODO: rewrite tests for these cases
        // #[test]
        // fn negative_radius_is_invalid() {
        //     let invalid = Formation {
        //         shape: Shape::Circle {
        //             radius: -1.0,
        //             center: Point { x: 0.0, y: 0.0 },
        //         },
        //         ..Default::default()
        //     };

        //     assert!(matches!(
        //         invalid.validate(),
        //         Err(ValidationError::ShapeError(ShapeError::NegativeRadius))
        //     ))
        // }

        // #[test]
        // fn zero_radius_is_invalid() {
        //     let invalid = Formation {
        //         shape: Shape::Circle {
        //             radius: 0.0,
        //             center: Point { x: 0.0, y: 0.0 },
        //         },
        //         ..Default::default()
        //     };

        //     assert!(matches!(
        //         invalid.validate(),
        //         Err(ValidationError::ShapeError(ShapeError::NegativeRadius))
        //     ))
        // }

        // #[test]
        // fn zero_time_is_okay() {
        //     let zero_is_okay = Formation {
        //         delay: 0.0,
        //         ..Default::default()
        //     };
        //
        //     assert!(matches!(zero_is_okay.validate(), Ok(Formation { .. })));
        // }

        mod polygon_macro {

            use pretty_assertions::assert_eq;

            use super::*;

            // fn float_eq(lhs: f32, rhs: f32) -> bool {
            //     f32::abs(lhs - rhs) <= f32::EPSILON
            // }

            fn float_eq(lhs: f64, rhs: f64) -> bool {
                f64::abs(lhs - rhs) <= f64::EPSILON
            }

            #[test]
            fn single_vertex() {
                let shape = polygon![(0.1, 0.1)];
                assert!(shape.is_polygon());
                let inner = shape.as_polygon().expect("know it is a Polygon");
                assert!(!inner.is_empty());
                assert_eq!(inner.len(), 1);

                assert!(inner
                    .iter()
                    .map(|p| (p.x.get(), p.y.get()))
                    .zip(vec![(0.1, 0.1)].into_iter())
                    .all(|(p, (x, y))| float_eq(p.0, x) && float_eq(p.1, y)));
            }

            #[test]
            fn multiple_vertices() {
                let shape = polygon![(0.35, 0.35), (0.35, 0.5), (0.7, 0.8)];
                assert!(shape.is_polygon());
                let inner = shape.as_polygon().expect("know it is a Polygon");
                assert!(!inner.is_empty());
                assert_eq!(inner.len(), 3);

                assert!(inner
                    .iter()
                    .map(|p| (p.x.get(), p.y.get()))
                    .zip(vec![(0., 0.), (1., 1.), (5., -8.)].into_iter())
                    .all(|(p, (x, y))| float_eq(p.0, x) && float_eq(p.1, y)));
            }

            /// Really just a test to see if the macro compiles and matches
            #[test]
            fn trailing_comma() {
                let shape = polygon![(0., 0.1),];
                assert!(shape.is_polygon());
                let inner = shape.as_polygon().expect("know it is a Polygon");
                assert!(!inner.is_empty());
                assert_eq!(inner.len(), 1);

                assert!(inner
                    .iter()
                    .map(|p| (p.x.get(), p.y.get()))
                    .zip(vec![(0., 0.1)].into_iter())
                    .all(|(p, (x, y))| float_eq(p.0, x) && float_eq(p.1, y)));
            }
        }
    }

    // mod formation_group {
    //     use super::*;
    // }
}
