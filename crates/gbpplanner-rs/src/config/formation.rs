//! A module for working with robot formations declaratively.

use std::{
    f32::consts::{PI, TAU},
    num::NonZeroUsize,
    path::Path,
    time::Duration,
};

use bevy::{ecs::system::Resource, math::Vec2};
use min_len_vec::{one_or_more, OneOrMore};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use typed_floats::StrictlyPositiveFinite;

use super::geometry::{Point, Shape};
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
        /// TODO: use
        attempts: NonZeroUsize,
    },
}

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
    pub const fn new(shape: Shape, projection_strategy: ProjectionStrategy) -> Self {
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

/// Enum representing the number of times a formation should repeat.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RepeatTimes {
    #[default]
    Infinite,
    Finite(usize),
}

impl RepeatTimes {
    pub const ONCE: Self = Self::Finite(1);

    /// Construct a new `RepeatTimes::Finite` variant
    pub fn finite(times: NonZeroUsize) -> Self {
        Self::Finite(times.into())
    }

    /// Returns true if there are one or more times left repeating
    pub const fn exhausted(&self) -> bool {
        match self {
            Self::Infinite => false,
            Self::Finite(remaining) => *remaining == 0,
        }
    }

    /// Decrement the number of repeats left in self.
    /// If `Self::Infinite`, do nothing
    /// If `Self::Finite(remaining)` decrement remaining if > 0
    pub fn decrement(&mut self) {
        match self {
            Self::Finite(ref mut remaining) if *remaining > 0 => *remaining -= 1,
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Repeat {
    pub every: Duration,
    pub times: RepeatTimes,
}

impl Repeat {
    /// Construct a new `Repeat` struct
    pub const fn new(every: Duration, times: RepeatTimes) -> Self {
        Self { every, times }
    }
}

/// How to evaluate if a robot has reached a waypoint
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum WaypointReachedWhenIntersects {
    /// When the current variable i.e. the robots current position intersects
    /// the waypoint
    Current,
    /// When the horizon state aka. the last variable intersects the waypoint
    /// This is the default, as this is what the gbpplanner paper does
    #[default]
    Horizon,
    /// When the nth variable from the current variable intersects the waypoint
    ///
    /// # Invariant
    /// Variable(n) -> n in [1, Horizon)
    ///
    /// # Panics
    ///
    /// A `panic!()` is issued if the invariant is not satisfied
    Variable(NonZeroUsize),
}

/// A description of a formation of robots in the simulation.
/// It describes how/where the robots are to be spawned, how many will be
/// spawned, how often and where they should move to.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Formation {
    /// Optionally spawn this formation again repeatedly with the given
    /// duration.
    pub repeat: Option<Repeat>,
    // pub repeat_every: bool,
    /// The delay from the start of the simulation after which the formation
    /// should spawn.
    pub delay: Duration,
    /// Number of robots to spawn every iteration
    pub robots: NonZeroUsize,
    /// Where to spawn the formation
    pub initial_position: InitialPosition,
    /// List of waypoints.
    pub waypoints: OneOrMore<Waypoint>,
    pub waypoint_reached_when_intersects: WaypointReachedWhenIntersects,
}

impl Default for Formation {
    fn default() -> Self {
        Self::circle_from_paper()
        // Self {
        //     repeat_every: None,
        //     delay: Duration::from_secs(5),
        //     robots: NonZeroUsize::new(1).expect("1 > 0"),
        //     initial_position: InitialPosition {
        //         shape: line![(0.4, 0.0), (0.6, 0.0)],
        //         placement_strategy: InitialPlacementStrategy::Random {
        //             attempts: 1000.try_into().expect("1000 > 0"),
        //         },
        //     },
        //     waypoints: one_or_more![
        //         Waypoint::new(line![(0.4, 0.4), (0.6, 0.6)],
        // ProjectionStrategy::Identity),         Waypoint::new(line!
        // [(0.0, 0.4), (0.0, 0.6)], ProjectionStrategy::Identity),
        //     ],
        // }
    }
}

impl Formation {
    pub fn robots_to_spawn(&self) -> usize {
        // FIXME(kpbaks): handle of 1 by in repeat count
        self.robots.get()
            * (0 + self.repeat.map_or(0, |repeat| match repeat.times {
                RepeatTimes::Infinite => unreachable!("ehh ..."),
                RepeatTimes::Finite(times) => times,
            }))
    }

    /// Return a new `Formation` matching the used in the **gbpplanner** paper
    /// for the circle formation scenario
    #[allow(clippy::missing_panics_doc)]
    pub fn circle_from_paper() -> Self {
        let circle = Shape::Circle {
            radius: 25.0.try_into().expect("positive and finite"),
            center: Point::new(0.5, 0.5),
        };
        Self {
            // repeat: None,
            repeat: Some(Repeat::new(Duration::from_secs(10), RepeatTimes::Finite(1))),
            delay: Duration::from_secs(1),
            robots: 3.try_into().expect("3 > 0"),
            initial_position: InitialPosition {
                shape: circle.clone(),
                placement_strategy: InitialPlacementStrategy::Equal,
            },
            waypoints: one_or_more![Waypoint::new(circle, ProjectionStrategy::Cross)],
            waypoint_reached_when_intersects: WaypointReachedWhenIntersects::Horizon,
        }
    }

    /// Convert a `Formation` description into the waypoints the robot has to
    /// follow
    #[allow(
        clippy::missing_panics_doc,
        clippy::too_many_lines,
        clippy::cast_precision_loss
    )]
    pub fn as_positions(
        &self,
        world_dims: WorldDimensions,
        robot_radius: StrictlyPositiveFinite<f32>,
        rng: &mut impl Rng,
    ) -> Option<(Vec<Vec2>, Vec<Vec<Vec2>>)> {
        match self.initial_position.shape {
            Shape::LineSegment((ls_start, ls_end)) => {
                let ls_start = world_dims.point_to_world_position(ls_start);
                let ls_end = world_dims.point_to_world_position(ls_end);

                let lerp_amounts = match &self.initial_position.placement_strategy {
                    InitialPlacementStrategy::Random { attempts } => {
                        randomly_place_nonoverlapping_circles_along_line_segment(
                            ls_start,
                            ls_end,
                            self.robots,
                            robot_radius,
                            *attempts,
                            // max_placement_attempts,
                            rng,
                        )
                    }
                    InitialPlacementStrategy::Equal => {
                        let d = ls_start.distance(ls_end);
                        let n_robots: f32 = self.robots.get() as f32;
                        let robot_diameter = robot_radius.get() * 2.0;
                        if d < robot_diameter * n_robots {
                            None
                        } else {
                            Some(
                                (0..self.robots.get())
                                    .map(|x| x as f32 / n_robots)
                                    .collect(),
                            )
                        }
                    }
                }?;

                // dbg!(&lerp_amounts);

                assert_eq!(lerp_amounts.len(), self.robots.get());

                let initial_positions: Vec<_> = lerp_amounts
                    .iter()
                    .map(|by| ls_start.lerp(ls_end, *by))
                    .collect();

                let waypoints_of_each_robot: Vec<Vec<Vec2>> = self
                    .waypoints
                    .iter()
                    .map(|wp| {
                        let Shape::LineSegment((ls_start, ls_end)) = wp.shape else {
                            unimplemented!("no time for the other combinations sadly :(");
                        };
                        let ls_start = world_dims.point_to_world_position(ls_start);
                        let ls_end = world_dims.point_to_world_position(ls_end);
                        let positions: Vec<Vec2> = match wp.projection_strategy {
                            ProjectionStrategy::Identity => lerp_amounts
                                .iter()
                                .map(|by| ls_start.lerp(ls_end, *by))
                                .collect(),
                            ProjectionStrategy::Cross => lerp_amounts
                                .iter()
                                .rev()
                                .map(|by| ls_start.lerp(ls_end, *by))
                                .collect(),
                        };
                        // }
                        //
                        // let positions: Vec<Vec2> = lerp_amounts
                        //     .iter()
                        //     .map(|by| ls_start.lerp(ls_end, *by))
                        //     .collect();
                        positions
                    })
                    .collect();

                assert!(waypoints_of_each_robot
                    .iter()
                    .map(std::vec::Vec::len)
                    .all(|l| l == initial_positions.len()));

                Some((initial_positions, waypoints_of_each_robot))
            }
            Shape::Circle { radius, center } => {
                let perimeter_radius: f32 = radius.get();
                let center = world_dims.point_to_world_position(center);
                // dbg!(&center);
                let angles: Vec<f32> = match self.initial_position.placement_strategy {
                    InitialPlacementStrategy::Equal => {
                        let angle = 2.0 * PI / self.robots.get() as f32;
                        let angles = (0..self.robots.get()).map(|i| i as f32 * angle).collect();
                        Some(angles)
                    }
                    InitialPlacementStrategy::Random { attempts } => {
                        // TODO: check if it even makes sense to iterate
                        // if B * n > 2 * math.pi * A:
                        let mut rng = thread_rng();

                        randomly_place_nonoverlapping_circles_along_circle_perimeter(
                            self.robots,
                            robot_radius,
                            radius,
                            attempts,
                            &mut rng,
                        )
                    }
                }?;
                assert_eq!(angles.len(), self.robots.get());

                let initial_positions: Vec<Vec2> = angles
                    .iter()
                    .map(|angle| polar(*angle, perimeter_radius))
                    .map(|polar| center + polar)
                    .collect();

                let waypoints_of_each_robots: Vec<Vec<Vec2>> = self
                    .waypoints
                    .iter()
                    .map(|wp| {
                        let Shape::Circle { radius, center } = wp.shape else {
                            unimplemented!("no time for the other combinations sadly :(");
                        };
                        match wp.projection_strategy {
                            ProjectionStrategy::Identity => {
                                panic!("does not make sense for a circle")
                            }
                            ProjectionStrategy::Cross => {
                                let center = world_dims.point_to_world_position(center);
                                // have each robots move across the circle to the opposite side
                                angles
                                    .iter()
                                    .map(|angle| angle + std::f32::consts::PI)
                                    .map(|angle| polar(angle, radius.get()))
                                    .map(|polar| center + polar)
                                    .collect()
                            }
                        }
                    })
                    .collect();

                Some((initial_positions, waypoints_of_each_robots))
            }
            Shape::Polygon(_) => todo!(),
        }
    }
}

/// Create a vector from polar coordinates
#[must_use]
#[inline]
fn polar(angle: f32, magnitude: f32) -> Vec2 {
    Vec2::new(angle.cos() * magnitude, angle.sin() * magnitude)
}

trait Vec2Ext {
    fn from_polar(angle: f32, magnitude: f32) -> Self;
}

impl Vec2Ext for bevy::math::Vec2 {
    /// Create a vector from polar coordinates
    #[must_use]
    #[inline]
    fn from_polar(angle: f32, magnitude: f32) -> Self {
        Self::new(angle.cos() * magnitude, angle.sin() * magnitude)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WorldDimensions {
    width:  StrictlyPositiveFinite<f64>,
    height: StrictlyPositiveFinite<f64>,
}

impl WorldDimensions {
    /// Create a new `WorldDimensions` structure
    ///
    /// # Panics
    ///
    /// Panics if `width` or `height` is not positive and finite
    #[must_use]
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width:  width.try_into().expect("width is not zero"),
            height: height.try_into().expect("height is not zero"),
        }
    }

    /// Get the width of the world.
    pub const fn width(&self) -> f64 {
        self.width.get()
    }

    /// Get the height of the world.
    pub const fn height(&self) -> f64 {
        self.height.get()
    }

    pub fn point_to_world_position(&self, p: Point) -> Vec2 {
        #[allow(clippy::cast_possible_truncation)]
        Vec2::new(
            ((p.x - 0.5) * self.width()) as f32,
            ((p.y - 0.5) * self.height()) as f32,
        )
    }
}

fn randomly_place_nonoverlapping_circles_along_circle_perimeter(
    num_circles: NonZeroUsize,
    radius: StrictlyPositiveFinite<f32>,
    perimeter_radius: StrictlyPositiveFinite<f32>,
    max_attempts: NonZeroUsize,
    rng: &mut impl Rng,
) -> Option<Vec<f32>> {
    let num_circles = num_circles.get();
    let perimeter_radius = perimeter_radius.get();
    let mut placed_angles = Vec::with_capacity(num_circles);
    let mut placed_positions: Vec<Vec2> = Vec::with_capacity(num_circles);
    // TODO: use rng argument passed
    // let mut rng = thread_rng();

    for _ in 0..max_attempts.get() {
        let theta = rng.gen_range(0.0..TAU);
        let pos = Vec2::from_polar(theta, perimeter_radius);
        let not_overlapping_with_others = placed_positions
            .iter()
            .any(|other| other.distance(pos) <= radius);

        if not_overlapping_with_others {
            placed_angles.push(theta);
            placed_positions.push(pos);
            if placed_angles.len() == num_circles {
                return Some(placed_angles);
            }
        }
    }

    None
}

fn randomly_place_nonoverlapping_circles_along_line_segment(
    from: Vec2,
    to: Vec2,
    num_circles: NonZeroUsize,
    radius: StrictlyPositiveFinite<f32>,
    max_attempts: NonZeroUsize,
    rng: &mut impl Rng,
) -> Option<Vec<f32>> {
    let num_circles = num_circles.get();
    let max_attempts = max_attempts.get();
    let mut lerp_amounts: Vec<f32> = Vec::with_capacity(num_circles);
    let mut placed: Vec<Vec2> = Vec::with_capacity(num_circles);

    let diameter = radius.get() * 2.0;

    for _ in 0..max_attempts {
        placed.clear();
        lerp_amounts.clear();

        for _ in 0..num_circles {
            let lerp_amount = rng.gen_range(0.0..1.0);
            let new_position = from.lerp(to, lerp_amount);

            let valid = placed.iter().all(|&p| new_position.distance(p) >= diameter);

            if valid {
                lerp_amounts.push(lerp_amount);
                placed.push(new_position);
                if placed.len() == num_circles {
                    return Some(lerp_amounts);
                }
            }
        }
    }

    None
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

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

/// A `FormationGroup` represent multiple `Formation`s
#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
#[serde(rename_all = "kebab-case")]
pub struct FormationGroup {
    pub formations: OneOrMore<Formation>,
}

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

    #[allow(clippy::missing_errors_doc)]
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
        Ok(serde_yaml::from_str(contents)?)
        // unimplemented!()
        // Ok(ron::from_str::<Self>(contents).map_err(|span| span.code)?)
    }

    /// Returns how many robots all formations in the group together will spawn
    pub fn robots_to_spawn(&self) -> usize {
        self.formations
            .iter()
            .map(Formation::robots_to_spawn)
            .sum::<usize>()
    }

    pub fn circle_from_paper() -> Self {
        Self {
            formations: one_or_more![Formation::circle_from_paper()],
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn intersection_from_paper() -> Self {
        Self {
            formations: one_or_more![
                Formation {
                    // repeat: Some(Duration::from_secs(4)),
                    repeat: Some(Repeat {
                        every: Duration::from_secs(4),
                        times: RepeatTimes::Finite(2),
                    }),
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

                    waypoint_reached_when_intersects: WaypointReachedWhenIntersects::Horizon,
                },
                Formation {
                    // repeat: Some(Duration::from_secs(4)),
                    repeat: Some(Repeat {
                        every: Duration::from_secs(4),
                        times: RepeatTimes::Finite(2),
                    }),
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
                    waypoint_reached_when_intersects: WaypointReachedWhenIntersects::Horizon,
                },
            ],
        }
    }
}

impl Default for FormationGroup {
    fn default() -> Self {
        // Self::intersection_from_paper()
        Self::circle_from_paper()
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
}
