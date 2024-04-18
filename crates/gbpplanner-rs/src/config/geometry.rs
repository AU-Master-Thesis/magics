use min_len_vec::OneOrMore;
use serde::{Deserialize, Serialize};
use typed_floats::StrictlyPositiveFinite;
use unit_interval::UnitInterval;

// /// A relative point within the boudaries of the map.
// /// ...
// #[derive(Debug, Serialize, Deserialize, Clone, Copy)]
// pub struct RelativePoint {
//     pub x: UnitInterval,
//     pub y: UnitInterval,
// }

// impl RelativePoint {
//     /// Create a new `RelativePoint` from a pair of values.
//     /// Returns an error if either `x` or `y` is not in the interval [0.0,
// 1.0].     pub fn new(x: f64, y: f64) -> Result<Self,
// unit_interval::UnitIntervalError> {         Ok(Self {
//             x: UnitInterval::new(x)?,
//             y: UnitInterval::new(y)?,
//         })
//     }

//     /// Returns the x and y values as a tuple
//     pub fn get(&self) -> (f64, f64) {
//         (self.x.get(), self.y.get())
//     }
// }

// impl TryFrom<(f64, f64)> for RelativePoint {
//     type Error = unit_interval::UnitIntervalError;

//     fn try_from(value: (f64, f64)) -> Result<Self, Self::Error> {
//         Ok(Self {
//             x: UnitInterval::new(value.0)?,
//             y: UnitInterval::new(value.1)?,
//         })
//     }
// }

// #[derive(Debug, Serialize, Deserialize)]
// #[serde(rename_all = "kebab-case")]
// pub enum Shape {
//     Circle {
//         radius: StrictlyPositiveFinite<f32>,
//         center: RelativePoint,
//     },
//     Polygon(OneOrMore<RelativePoint>),
//     Line((RelativePoint, RelativePoint)),
// }

// impl Shape {
//     /// Returns `true` if the shape is [`Polygon`].
//     ///
//     /// [`Polygon`]: Shape::Polygon
//     #[must_use]
//     pub fn is_polygon(&self) -> bool {
//         matches!(self, Self::Polygon(..))
//     }

//     /// Returns `true` if the shape is [`Circle`].
//     ///
//     /// [`Circle`]: Shape::Circle
//     #[must_use]
//     pub fn is_circle(&self) -> bool {
//         matches!(self, Self::Circle { .. })
//     }

//     pub fn as_polygon(&self) -> Option<&OneOrMore<RelativePoint>> {
//         if let Self::Polygon(v) = self {
//             Some(v)
//         } else {
//             None
//         }
//     }

//     // /// General triangle constructor
//     // /// Takes a baseline, a height
//     // /// Returns a `Shape::Polygon` with the vertices of the triangle
//     // pub fn triangle(base_width: f64, height: f64, mid_point: UnitInterval)
// ->     // Shape {     let half_base = base_width / 2.0;
//     //     let half_height = height / 2.0;
//     //     let top = RelativePoint::new(mid_point.get(), mid_point.get() +
//     // half_height).unwrap();     let left =
//     //         RelativePoint::new(mid_point.get() - half_base,
// mid_point.get() -     // half_height).unwrap();     let right =
//     //         RelativePoint::new(mid_point.get() + half_base,
// mid_point.get() -     // half_height).unwrap();     polygon![
//     //         (top.x.get(), top.y.get()),
//     //         (left.x.get(), left.y.get()),
//     //         (right.x.get(), right.y.get())
//     //     ]
//     // }

//     // /// General rectangle constructor
//     // /// Takes a width and a height
//     // /// Returns a `Shape::Polygon` with the vertices of the rectangle
//     // pub fn rectangle(width: f64, height: f64, mid_point: UnitInterval) ->
// Shape {     //     let half_width = width / 2.0;
//     //     let half_height = height / 2.0;
//     //     let top_left =
//     //         RelativePoint::new(mid_point.get() - half_width,
// mid_point.get() +     // half_height)             .unwrap();
//     //     let top_right =
//     //         RelativePoint::new(mid_point.get() + half_width,
// mid_point.get() +     // half_height)             .unwrap();
//     //     let bottom_left =
//     //         RelativePoint::new(mid_point.get() - half_width,
// mid_point.get() -     // half_height)             .unwrap();
//     //     let bottom_right =
//     //         RelativePoint::new(mid_point.get() + half_width,
// mid_point.get() -     // half_height)             .unwrap();
//     //     polygon![
//     //         (top_left.x.get(), top_left.y.get()),
//     //         (top_right.x.get(), top_right.y.get()),
//     //         (bottom_right.x.get(), bottom_right.y.get()),
//     //         (bottom_left.x.get(), bottom_left.y.get())
//     //     ]
//     // }

//     // /// General square constructor
//     // /// Takes a side length
//     // /// Returns a `Shape::Polygon` with the vertices of the square
//     // pub fn square(side_length: f64) -> Shape {
//     //     let half_side = side_length / 2.0;
//     //     let top_left = RelativePoint::new(0.5 - half_side, 0.5 +
//     // half_side).unwrap();     let top_right = RelativePoint::new(0.5 +
//     // half_side, 0.5 + half_side).unwrap();     let bottom_right =
//     // RelativePoint::new(0.5 + half_side, 0.5 - half_side).unwrap();
//     //     let bottom_left = RelativePoint::new(0.5 - half_side, 0.5 -
//     // half_side).unwrap();     polygon![
//     //         (top_left.x.get(), top_left.y.get()),
//     //         (top_right.x.get(), top_right.y.get()),
//     //         (bottom_right.x.get(), bottom_right.y.get()),
//     //         (bottom_left.x.get(), bottom_left.y.get())
//     //     ]
//     // }

//     // /// General circle constructor
//     // /// Takes a radius
//     // /// Returns a `Shape::Circle` with the center at origin
//     // pub fn circle(radius: f32) -> Shape {
//     //     Shape::Circle {
//     //         radius: StrictlyPositiveFinite::<f32>::new(radius).unwrap(),
//     //         center: RelativePoint::new(0.5, 0.5).unwrap(),
//     //     }
//     // }
// }

// /// Shorthand to construct `Shape::Polygon(vec![Point {x: $x, y: $y}, ... ])`
// #[macro_export]
// macro_rules! polygon {
//     [$(($x:expr, $y:expr)),+ $(,)?] => {{
//         let vertices = vec![
//             $(
//         	crate::config::geometry::RelativePoint::new($x, $y).expect("both x
// and y are within the interval [0.0, 1.0]")             ),+
//         ];
//         Shape::Polygon(OneOrMore::new(vertices).expect("at least one
// vertex"))     }}
// }

// /// Shorthand to construct `Shape::Line((Point {x: $x1, y: $y1}, Point {x:
// $x2, /// y: $y2}))`
// #[macro_export]
// macro_rules! line {
//     [($x1:expr, $y1:expr), ($x2:expr, $y2:expr)] => {
//         // Shape::Line((Point { x: $x1, y: $y1 }, Point { x: $x2, y: $y2 }))
//         // Shape::Line((Point { x: ($x1 as f64).try_from().unwrap(), y: ($y1
// as f64).try_from().unwrap() }, Point { x: ($x2 as f64).try_from().unwrap(),
// y: f64::try_from().unwrap() }))
//         Shape::Line((crate::config::geometry::RelativePoint::new($x1,
// $y1).expect("both x1 and y1 are within the interval [0.0, 1.0]"),
// crate::config::geometry::RelativePoint::new($x2, $y2).expect("both x2 and y2
// are within the interval [0.0, 1.0]")))     };
// }

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
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
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
    pub fn new(x: f64, y: f64) -> Result<Self, unit_interval::UnitIntervalError> {
        Ok(Self {
            x: UnitInterval::new(x)?,
            y: UnitInterval::new(y)?,
        })
    }

    // /// Returns the x and y values as a tuple
    // #[inline]
    // pub const fn get(&self) -> (f64, f64) {
    //     (self.x.get(), self.y.get())
    // }
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

#[derive(Debug, Clone, Serialize, Deserialize, derive_more::IsVariant)]
#[serde(rename_all = "kebab-case")]
pub enum Shape {
    Circle {
        radius: StrictlyPositiveFinite<f32>,
        center: RelativePoint,
    },
    Polygon(OneOrMore<Point>),
    Line((Point, Point)),
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
                $crate::config::geometry::Point::new($x, $y)
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
        Shape::Line(($crate::config::geometry::Point::new($x1, $y1), $crate::config::geometry::Point::new($x2, $y2)))
    };
}
