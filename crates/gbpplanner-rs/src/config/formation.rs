//! A module for working with robot formations declaratively.
use std::{num::NonZeroUsize, path::Path, time::Duration};

use bevy::ecs::system::Resource;
use min_len_vec::{one_or_more, OneOrMore};
use serde::{Deserialize, Serialize};

use super::geometry::Shape;
use crate::line;

/// Strategy to use for the starting point of a formation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InitialPlacementStrategy {
    /// Place robots with equal distance between them
    Equal,
    /// Place robots with a random distance between them, with the
    /// constraint that surface of area of none of the robots overlap.
    Random {
        /// How many attempts to use before returning an error saying it was not
        /// possible to place the robots Along the shape.
        /// A high number > 1000, is a good idea, since it is not very intensive
        /// to run the algorithm determining the random placements.
        attempts: NonZeroUsize,
    },
}

// impl InitialPlacementStrategy {
//     pub const RANDOM_ATTEMPTS: NonZeroUsize = 1000.try_into().unwrap();
// }

/// Strategy to use for waypoints after the initial starting position.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectionStrategy {
    /// Simply map the relative position along a line strip from the previous
    /// waypoint
    ///
    /// # Examples
    Identity,
    Cross,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlacementStrategy {
    Equal,
    Random,
    Map,
}

/// Waypoint a group of robots has to reach, from either the robots initial
/// position, or a previous waypoint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Waypoint {
    pub shape: Shape,
    // pub placement_strategy: PlacementStrategy,
    pub projection_strategy: ProjectionStrategy,
}

impl Waypoint {
    /// Crate a new `Waypoint`
    #[must_use]
    pub fn new(shape: Shape, projection_strategy: ProjectionStrategy) -> Self {
        Self {
            shape,
            projection_strategy,
        }
    }
}

/// Initial position of where a group of robots has to spawn
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct InitialPosition {
    /// The shape in which the robots should spawn
    pub shape: Shape,
    /// Strategy for how to place the robots
    pub placement_strategy: InitialPlacementStrategy,
}

// #[derive(Debug, thiserror::Error)]
// pub enum FormationError {
//     #[error("FormationGroup has no formations")]
//     NoFormations,
// }

/// A description of a formation of robots in the simulation.
/// It describes how/where the robots are to be spawned, how many will be
/// spawned, how often and where they should move to.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Formation {
    /// Optionally spawn this formation again repeatedly with the given
    /// duration.
    pub repeat_every: Option<Duration>,
    // pub repeat_every: bool,
    /// The delay from the start of the simulation after which the formation
    /// should spawn.
    pub delay: Duration,
    /// Number of robots to spawn
    pub robots: NonZeroUsize,
    /// Where to spawn the formation
    pub initial_position: InitialPosition,
    /// List of waypoints.
    pub waypoints: OneOrMore<Waypoint>,
}

impl Default for Formation {
    fn default() -> Self {
        Self {
            repeat_every: None,
            delay: Duration::from_secs(5),
            robots: NonZeroUsize::new(1).expect("1 > 0"),
            initial_position: InitialPosition {
                shape: line![(0.4, 0.0), (0.6, 0.0)],
                placement_strategy: InitialPlacementStrategy::Random {
                    attempts: 1000.try_into().expect("1000 > 0"),
                },
            },
            waypoints: one_or_more![
                Waypoint::new(line![(0.4, 0.4), (0.6, 0.6)], ProjectionStrategy::Identity),
                Waypoint::new(line![(0.0, 0.4), (0.0, 0.6)], ProjectionStrategy::Identity),
            ],
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    // #[error("TOML error: {0}")]
    // Toml(#[from] toml::de::Error),
    // #[error("Validation error: {0}")]
    // InvalidFormation(#[from] FormationError),
    // #[error("Invalid formation group: {0}")]
    // InvalidFormationGroup(#[from] FormationGroupError),
    #[error("RON error: {0}")]
    Ron(#[from] ron::Error),
}

/// A `FormationGroup` represent multiple `Formation`s
#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
#[serde(rename_all = "kebab-case")]
pub struct FormationGroup {
    pub formations: OneOrMore<Formation>,
}

// #[derive(Debug, thiserror::Error)]
// pub enum FormationGroupError {
//     NoFormations,
//     // InvalidFormations(Vec<(usize, FormationError)>),
// }

// impl std::fmt::Display for FormationGroupError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Self::NoFormations => write!(
//                 f,
//                 "Formation group is empty. A formation group contains at
// least one formation",             ),
//             Self::InvalidFormations(ref inner) => {
//                 for (index, error) in inner {
//                     write!(f, "formation[{}] is invalid with error: {}",
// index, error)?;                 }
//                 Ok(())
//             }
//         }
//     }
// }

impl FormationGroup {
    /// Attempt to parse a `FormationGroup` from a RON file
    ///
    /// # Errors
    ///
    /// Will return `Err` if:
    /// 1. `path` does not exist on the filesystem.
    /// 2. The contents of `path` is not valid RON.
    /// 3. The parsed data does not represent a valid `FormationGroup`.
    pub fn from_ron_file<P: AsRef<Path>>(path: P) -> Result<Self, ParseError> {
        std::fs::read_to_string(path)
            .map(|file_contents| Self::parse_from_ron(file_contents.as_str()))?
    }

    pub fn from_yaml_file<P: AsRef<Path>>(path: P) -> Result<Self, ParseError> {
        std::fs::read_to_string(path)
            .map(|file_contents| Self::parse_from_yaml(file_contents.as_str()))?
    }

    /// Attempt to parse a `FormationGroup` from a RON encoded string.
    ///
    /// # Errors
    ///
    /// Will return `Err`  if:
    /// 1. `contents` is not valid RON.
    /// 2. The parsed data does not represent a valid `FormationGroup`.
    pub fn parse_from_ron(contents: &str) -> Result<Self, ParseError> {
        Ok(ron::from_str::<Self>(contents).map_err(|span| span.code)?)
    }

    /// Attempt to parse a `FormationGroup` from a YAML encoded string.
    ///
    /// # Errors
    ///
    /// Will return `Err`  if:
    /// 1. `contents` is not valid YAML.
    /// 2. The parsed data does not represent a valid `FormationGroup`.
    pub fn parse_from_yaml(contents: &str) -> Result<Self, ParseError> {
        unimplemented!()
        // Ok(ron::from_str::<Self>(contents).map_err(|span| span.code)?)
    }
}

impl Default for FormationGroup {
    fn default() -> Self {
        Self {
            formations: one_or_more![
                Formation {
                    repeat_every: Some(Duration::from_secs(4)),
                    delay: Duration::from_secs(2),
                    robots: 1.try_into().expect("1 > 0"),
                    initial_position: InitialPosition {
                        shape: line![(0.45, 0.0), (0.55, 0.0)],
                        placement_strategy: InitialPlacementStrategy::Equal,
                    },

                    waypoints: one_or_more![Waypoint::new(
                        line![(0.45, 1.25), (0.55, 1.25)],
                        ProjectionStrategy::Identity
                    ),],
                },
                Formation {
                    repeat_every: Some(Duration::from_secs(4)),
                    delay: Duration::from_secs(2),
                    robots: 1.try_into().expect("1 > 0"),
                    initial_position: InitialPosition {
                        shape: line![(0.0, 0.45), (0.0, 0.55)],
                        placement_strategy: InitialPlacementStrategy::Equal,
                    },

                    waypoints: one_or_more![Waypoint::new(
                        line![(1.25, 0.45), (1.25, 0.55)],
                        ProjectionStrategy::Identity,
                    ),],
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod formation {
        use super::*;

        // #[test]
        // fn default_is_valid() {
        //     let default = Formation::default();
        //     assert!(matches!(default.validate(), Ok(Formation { .. })));
        // }

        mod polygon_macro {

            use pretty_assertions::assert_eq;

            use super::*;
            use crate::polygon;

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
                // assert!(!inner.is_empty());
                assert_eq!(inner.len(), 1);

                assert!(inner
                    .iter()
                    .map(|p| (p.x, p.y))
                    .zip(vec![(0.1, 0.1)].into_iter())
                    .all(|(p, (x, y))| float_eq(p.0, x) && float_eq(p.1, y)));
            }

            #[test]
            fn multiple_vertices() {
                let shape = polygon![(0.35, 0.35), (0.35, 0.5), (0.7, 0.8)];
                assert!(shape.is_polygon());
                let inner = shape.as_polygon().expect("know it is a Polygon");
                // assert!(!inner.is_empty());
                assert_eq!(inner.len(), 3);

                assert!(inner
                    .iter()
                    .map(|p| (p.x, p.y))
                    .zip(vec![(0.35, 0.35), (0.35, 0.5), (0.7, 0.8)].into_iter())
                    .all(|(p, (x, y))| float_eq(p.0, x) && float_eq(p.1, y)));
            }

            /// Really just a test to see if the macro compiles and matches
            #[test]
            fn trailing_comma() {
                let shape = polygon![(0., 0.1),];
                assert!(shape.is_polygon());
                let inner = shape.as_polygon().expect("know it is a Polygon");
                // assert!(!inner.is_empty());
                assert_eq!(inner.len(), 1);

                assert!(inner
                    .iter()
                    .map(|p| (p.x, p.y))
                    .zip(vec![(0., 0.1)].into_iter())
                    .all(|(p, (x, y))| float_eq(p.0, x) && float_eq(p.1, y)));
            }
        }
    }

    // mod formation_group {
    //     use super::*;
    // }
}
