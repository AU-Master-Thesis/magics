use min_len_vec::OneOrMore;
use serde::{Deserialize, Serialize};
use typed_floats::StrictlyPositiveFinite;
use unit_interval::UnitInterval;

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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Shape {
    Circle {
        radius: StrictlyPositiveFinite<f32>,
        center: RelativePoint,
    },
    Polygon(OneOrMore<RelativePoint>),
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

    pub fn as_polygon(&self) -> Option<&OneOrMore<RelativePoint>> {
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
        	crate::config::geometry::RelativePoint::new($x, $y).expect("both x and y are within the interval [0.0, 1.0]")
            ),+
        ];
        Shape::Polygon(OneOrMore::new(vertices).expect("at least one vertex"))
    }}
}

/// Shorthand to construct `Shape::Line((Point {x: $x1, y: $y1}, Point {x: $x2,
/// y: $y2}))`
#[macro_export]
macro_rules! line {
    [($x1:expr, $y1:expr), ($x2:expr, $y2:expr)] => {
        // Shape::Line((Point { x: $x1, y: $y1 }, Point { x: $x2, y: $y2 }))
        // Shape::Line((Point { x: ($x1 as f64).try_from().unwrap(), y: ($y1 as f64).try_from().unwrap() }, Point { x: ($x2 as f64).try_from().unwrap(), y: f64::try_from().unwrap() }))
        Shape::Line((crate::config::geometry::RelativePoint::new($x1, $y1).expect("both x1 and y1 are within the interval [0.0, 1.0]"), crate::config::geometry::RelativePoint::new($x2, $y2).expect("both x2 and y2 are within the interval [0.0, 1.0]")))
    };
}
