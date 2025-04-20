use bevy::{log::warn, math::Vec2}; // Added warn import
use min_len_vec::OneOrMore;
use rand::Rng;
use serde::{Deserialize, Serialize};
use typed_floats::StrictlyPositiveFinite;
use unit_interval::UnitInterval;

use super::formation::WorldDimensions; // Added for world_dims usage

// A regular point in 2D space.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Create a new `Point` from a pair of values.
    /// Returns an error if either `x` or `y` is not in the interval [0.0, 1.0].
    #[inline]
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl From<Point> for bevy::math::Vec2 {
    #[allow(clippy::cast_possible_truncation)]
    fn from(value: Point) -> Self {
        Self::new(value.x as f32, value.y as f32)
    }
}

/// A relative point within the boundaries of the map.
/// ...
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RelativePoint {
    pub x: UnitInterval,
    pub y: UnitInterval,
}

impl RelativePoint {
    /// Create a new `RelativePoint` from a pair of values.
    /// Returns an error if either `x` or `y` is not in the interval [0.0, 1.0].
    ///
    /// # Errors
    ///
    /// Will return 'Err' if x or y not in [0.0, 1.0]
    pub fn new(x: f64, y: f64) -> Result<Self, unit_interval::UnitIntervalError> {
        Ok(Self {
            x: UnitInterval::new(x)?,
            y: UnitInterval::new(y)?,
        })
    }

    /// Create a new `RelativePoint` at (0.0, 0.0)
    #[allow(clippy::missing_panics_doc)] // invariant always satisfied
    pub fn min() -> Self {
        Self {
            x: UnitInterval::new(0.0).expect("0.0 in [0.0, 1.0]"),
            y: UnitInterval::new(0.0).expect("0.0 in [0.0, 1.0]"),
        }
    }

    /// Create a new `RelativePoint` at (1.0, 1.0)
    #[allow(clippy::missing_panics_doc)] // invariant always satisfied
    pub fn max() -> Self {
        Self {
            x: UnitInterval::new(1.0).expect("1.0 in [0.0, 1.0]"),
            y: UnitInterval::new(1.0).expect("1.0 in [0.0, 1.0]"),
        }
    }

    /// Create a new `RelativePoint` at (0.5, 0.5)
    #[allow(clippy::missing_panics_doc)] // invariant always satisfied
    pub fn center() -> Self {
        Self {
            x: UnitInterval::new(0.5).expect("0.5 in [0.0, 1.0]"),
            y: UnitInterval::new(0.5).expect("0.5 in [0.0, 1.0]"),
        }
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

impl From<RelativePoint> for bevy::math::Vec2 {
    fn from(value: RelativePoint) -> Self {
        Self::new(value.x.into(), value.y.into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, derive_more::IsVariant)]
#[serde(rename_all = "kebab-case")]
pub enum Shape { // Added pub here
    Circle {
        radius: StrictlyPositiveFinite<f32>,
        center: Point,
    },
    Polygon(OneOrMore<Point>),
    LineSegment((Point, Point)),
    /// A square defined by two opposite corner points (p1, p2) in normalized coordinates (0.0 to 1.0).
    /// Robots are spawned randomly within this area, ensuring a minimum distance between them.
    RandomSquare {
        p1: Point,
        p2: Point,
        min_distance: f32,
    },
}

impl Shape {
    pub const fn as_polygon(&self) -> Option<&OneOrMore<Point>> {
        if let Self::Polygon(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Generates a random point within the shape in world coordinates.
    /// Returns `None` if the shape type is not supported for random point generation (e.g., Polygon).
    pub fn get_random_point<R: Rng + ?Sized>(
        &self,
        world_dims: &WorldDimensions,
        rng: &mut R,
    ) -> Option<Vec2> {
        match self {
            Shape::Circle { radius, center } => {
                let center_world = world_dims.point_to_world_position(*center);
                let radius_world = radius.get();
                // Generate point within the circle using polar coordinates
                let angle = rng.gen::<f32>() * std::f32::consts::TAU;
                // Generate radius uniformly within the circle's area (sqrt for uniform distribution)
                let r = radius_world * rng.gen::<f32>().sqrt();
                Some(center_world + Vec2::new(angle.cos() * r, angle.sin() * r))
            }
            Shape::LineSegment((p1, p2)) => {
                let p1_world = world_dims.point_to_world_position(*p1);
                let p2_world = world_dims.point_to_world_position(*p2);
                let lerp_factor = rng.gen::<f32>();
                Some(p1_world.lerp(p2_world, lerp_factor))
            }
            Shape::RandomSquare { p1, p2, .. } => {
                // Use world coordinates directly as calculated elsewhere
                let world_p1 = world_dims.point_to_world_position(*p1);
                let world_p2 = world_dims.point_to_world_position(*p2);
                let min_x = world_p1.x.min(world_p2.x);
                let max_x = world_p1.x.max(world_p2.x);
                let min_y = world_p1.y.min(world_p2.y);
                let max_y = world_p1.y.max(world_p2.y);

                if min_x >= max_x || min_y >= max_y {
                    // Return center if bounds are invalid
                     warn!("Invalid square bounds for random point generation: min_x={}, max_x={}, min_y={}, max_y={}", min_x, max_x, min_y, max_y);
                    return Some(Vec2::new((min_x + max_x) / 2.0, (min_y + max_y) / 2.0));
                }

                let x = rng.gen_range(min_x..max_x);
                let y = rng.gen_range(min_y..max_y);
                Some(Vec2::new(x, y))
            }
            Shape::Polygon(_) => {
                // Polygon random point generation is complex, return None for now
                // TODO: Implement polygon random point generation if needed
                warn!("Random point generation within Polygon shape is not implemented.");
                None
            }
        }
    }
}

/// Shorthand to construct `Shape::Polygon(vec![Point {x: $x, y: $y}, ... ])`
#[macro_export]
macro_rules! polygon {
    [$(($x:expr, $y:expr)),+ $(,)?] => {{
        let vertices = vec![
            $(
                $crate::geometry::Point::new($x, $y)
            ),+
        ];
        Shape::Polygon(::min_len_vec::OneOrMore::new(vertices).expect("at least one vertex"))

    }}
}

/// Shorthand to construct `Shape::Line((Point {x: $x1, y: $y1}, Point {x: $x2,
/// y: $y2}))`
#[macro_export]
macro_rules! line {
    [($x1:expr, $y1:expr), ($x2:expr, $y2:expr)] => {
        // Shape::Line((Point { x: $x1, y: $y1 }, Point { x: $x2, y: $y2 }))
        // Shape::Line((Point { x: ($x1 as f64).try_from().unwrap(), y: ($y1 as f64).try_from().unwrap() }, Point { x: ($x2 as f64).try_from().unwrap(), y: f64::try_from().unwrap() }))
        $crate::geometry::Shape::LineSegment(($crate::geometry::Point::new($x1, $y1), $crate::geometry::Point::new($x2, $y2)))
    };
}
