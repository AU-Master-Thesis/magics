use min_len_vec::OneOrMore;
use serde::{Deserialize, Serialize};
use typed_floats::StrictlyPositiveFinite;
use unit_interval::UnitInterval;

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
pub enum Shape {
    Circle {
        radius: StrictlyPositiveFinite<f32>,
        center: Point,
    },
    Polygon(OneOrMore<Point>),
    LineSegment((Point, Point)),
}

impl Shape {
    pub const fn as_polygon(&self) -> Option<&OneOrMore<Point>> {
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
