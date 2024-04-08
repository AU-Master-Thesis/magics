pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
use std::{error::Error, fmt::Display};

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, PartialEq)]
pub enum AngleError {
    OutOfRangeRadians(f64),
    OutOfRangeDegrees(f64),
}

impl Display for AngleError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AngleError::OutOfRangeRadians(value) => {
                write!(f, "Angle value {} is not inside [0,2π]", value)
            }
            AngleError::OutOfRangeDegrees(value) => {
                write!(f, "Angle value {} is not inside [0,360]", value)
            }
        }
    }
}

impl Error for AngleError {}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Angle(f64);

pub type Result<T> = std::result::Result<T, AngleError>;

impl Angle {
    pub fn new(value: f64) -> Result<Self> {
        if value < 0.0 || value > 2.0 * std::f64::consts::PI {
            return Err(AngleError::OutOfRangeRadians(value));
        }
        Ok(Self(value))
    }

    pub fn from_degrees(value: f64) -> Result<Self> {
        if value < 0.0 || value > 360.0 {
            return Err(AngleError::OutOfRangeDegrees(value));
        }
        Ok(Self(value.to_radians()))
    }

    pub fn as_radians(&self) -> f64 {
        self.0
    }

    pub fn as_degrees(&self) -> f64 {
        self.0.to_degrees()
    }

    /// Adds two angles together
    /// wraps the result to the interval [0, 2π]
    pub fn add(&self, other: Angle) -> Self {
        let sum = self.0 + other.0;
        let wrapped = sum % (2.0 * std::f64::consts::PI);
        Self(wrapped)
    }

    /// Subtracts two angles
    /// wraps the result to the interval [0, 2π]
    pub fn sub(&self, other: Angle) -> Self {
        let diff = self.0 - other.0;
        let wrapped = (diff + 2.0 * std::f64::consts::PI) % (2.0 * std::f64::consts::PI);
        Self(wrapped)
    }

    /// Adds and assigns two angles
    /// wraps the result to the interval [0, 2π]
    pub fn add_assign(&mut self, other: Angle) {
        *self = self.add(other);
    }

    /// Subtracts and assigns two angles
    /// wraps the result to the interval [0, 2π]
    pub fn sub_assign(&mut self, other: Angle) {
        *self = self.sub(other);
    }
}

impl std::ops::Add for Angle {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Angle::add(&self, rhs)
    }
}

impl std::ops::Sub for Angle {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Angle::sub(&self, rhs)
    }
}

impl std::ops::AddAssign for Angle {
    fn add_assign(&mut self, rhs: Self) {
        Angle::add_assign(self, rhs);
    }
}

impl std::ops::SubAssign for Angle {
    fn sub_assign(&mut self, rhs: Self) {
        Angle::sub_assign(self, rhs);
    }
}

/// Convert a floating point number to an [`Angle`].
/// Returns an error if the value is not in the interval [0, 2π].
/// Explicity call `Angle.from_degrees` if you want to input degrees.
impl TryFrom<f64> for Angle {
    type Error = AngleError;

    fn try_from(value: f64) -> Result<Self> {
        Angle::new(value)
    }
}

impl<'de> Deserialize<'de> for Angle {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Angle, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        Angle::try_from(value).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;

    use super::*;

    #[test]
    fn test_new() {
        assert!(Angle::new(-1.0).is_err());
        assert!(Angle::new(0.0).is_ok());
        assert!(Angle::new(2.0 * std::f64::consts::PI).is_ok());
        assert!(Angle::new(2.0 * std::f64::consts::PI + 1.0).is_err());
    }

    #[test]
    fn test_from_degrees() {
        assert!(Angle::from_degrees(-1.0).is_err());
        assert!(Angle::from_degrees(0.0).is_ok());
        assert!(Angle::from_degrees(360.0).is_ok());
        assert!(Angle::from_degrees(361.0).is_err());
    }

    #[test]
    fn test_as_radians() {
        let angle = Angle::from_degrees(180.0).unwrap();
        assert_abs_diff_eq!(angle.as_radians(), std::f64::consts::PI, epsilon = 1e-6);
    }

    #[test]
    fn test_as_degrees() {
        let angle = Angle::new(std::f64::consts::PI).unwrap();
        assert_abs_diff_eq!(angle.as_degrees(), 180.0, epsilon = 1e-6);
    }

    #[test]
    fn test_add() {
        let angle1 = Angle::from_degrees(180.0).unwrap();
        let angle2 = Angle::from_degrees(180.0).unwrap();
        let sum = angle1.add(angle2);
        assert_abs_diff_eq!(sum.as_degrees(), 0.0, epsilon = 1e-6);
    }

    #[test]
    fn test_sub() {
        let angle1 = Angle::from_degrees(180.0).unwrap();
        let angle2 = Angle::from_degrees(90.0).unwrap();
        let diff = angle1.sub(angle2);
        assert_abs_diff_eq!(diff.as_degrees(), 90.0, epsilon = 1e-6);
    }

    #[test]
    fn test_add_assign() {
        let mut angle1 = Angle::from_degrees(180.0).unwrap();
        let angle2 = Angle::from_degrees(100.0).unwrap();
        angle1.add_assign(angle2);
        assert_abs_diff_eq!(angle1.as_degrees(), 280.0, epsilon = 1e-6);

        angle1.add_assign(angle2);
        assert_abs_diff_eq!(angle1.as_degrees(), 20.0, epsilon = 1e-6);
    }

    #[test]
    fn test_sub_assign() {
        let mut angle1 = Angle::from_degrees(180.0).unwrap();
        let angle2 = Angle::from_degrees(100.0).unwrap();
        angle1.sub_assign(angle2);
        assert_abs_diff_eq!(angle1.as_degrees(), 80.0, epsilon = 1e-6);

        angle1.sub_assign(angle2);
        assert_abs_diff_eq!(angle1.as_degrees(), 340.0, epsilon = 1e-6);
    }

    #[test]
    fn test_add_operator() {
        let angle1 = Angle::from_degrees(180.0).unwrap();
        let angle2 = Angle::from_degrees(100.0).unwrap();
        let sum = angle1 + angle2;
        assert_abs_diff_eq!(sum.as_degrees(), 280.0, epsilon = 1e-6);

        let sum = angle1 + angle2 + angle2;
        assert_abs_diff_eq!(sum.as_degrees(), 20.0, epsilon = 1e-6);
    }

    #[test]
    fn test_sub_operator() {
        let angle1 = Angle::from_degrees(180.0).unwrap();
        let angle2 = Angle::from_degrees(100.0).unwrap();
        let diff = angle1 - angle2;
        assert_abs_diff_eq!(diff.as_degrees(), 80.0, epsilon = 1e-6);

        let diff = angle1 - angle2 - angle2;
        assert_abs_diff_eq!(diff.as_degrees(), 340.0, epsilon = 1e-6);
    }

    #[test]
    fn test_add_assign_operator() {
        let mut angle1 = Angle::from_degrees(180.0).unwrap();
        let angle2 = Angle::from_degrees(100.0).unwrap();
        angle1 += angle2;
        assert_abs_diff_eq!(angle1.as_degrees(), 280.0, epsilon = 1e-6);

        angle1 += angle2;
        assert_abs_diff_eq!(angle1.as_degrees(), 20.0, epsilon = 1e-6);
    }

    #[test]
    fn test_sub_assign_operator() {
        let mut angle1 = Angle::from_degrees(180.0).unwrap();
        let angle2 = Angle::from_degrees(100.0).unwrap();
        angle1 -= angle2;
        assert_abs_diff_eq!(angle1.as_degrees(), 80.0, epsilon = 1e-6);

        angle1 -= angle2;
        assert_abs_diff_eq!(angle1.as_degrees(), 340.0, epsilon = 1e-6);
    }

    #[test]
    fn test_try_from_f64() {
        assert!(matches!(Angle::try_from(0.0), Ok(Angle(0.0))));

        let angle = Angle::try_from(2.0 * std::f64::consts::PI);
        assert!(angle.is_ok());

        assert!(matches!(
            Angle::try_from(-1.0),
            Err(AngleError::OutOfRangeRadians(-1.0))
        ));
        assert!(matches!(
            Angle::try_from(7.0),
            Err(AngleError::OutOfRangeRadians(7.0))
        ));
    }
}
