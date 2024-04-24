#![warn(missing_docs)]
//! A percentage value represented as a floating point number between 0.0 and
//! 100.0.

use serde::{Deserialize, Deserializer, Serialize};

/// A percentage value represented as a floating point number between 0.0 and
/// 100.0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Percentage(f64);

/// An error that can occur when creating a `Percentage`.
#[derive(Debug)]
pub enum PercentageError {
    /// The value provided was not between 0.0 and 100.0.
    InvalidPercentage(f64),
}

impl std::fmt::Display for PercentageError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::InvalidPercentage(p) => {
                write!(f, "invalid percentage: {p} valid interval is [0.0, 100.0]")
            }
        }
    }
}

impl std::error::Error for PercentageError {}

impl Percentage {
    /// Create a new `Percentage` from a floating point number between 0.0 and
    /// 100.0.
    ///
    /// # Errors
    ///
    /// Will return `Err` if `p` is not in the interval [0.0, 100.0]
    pub fn new(p: f64) -> Result<Self, PercentageError> {
        if !(0.0..=100.0).contains(&p) {
            return Err(PercentageError::InvalidPercentage(p));
        }
        Ok(Self(p / 100.0))
    }

    /// Create a new `Percentage` from a floating point number between 0.0 and
    /// 100.0. # Safety
    /// This function is unsafe because it does not check if the value is within
    /// the valid range.
    ///
    /// # Safety
    /// It is up to the caller to ensure the invariant that the input is between
    /// [0.0, 100.0]
    #[must_use]
    pub unsafe fn new_unchecked(p: f64) -> Self {
        Self(p / 100.0)
    }

    /// Get the value of the `Percentage` as a floating point number
    #[allow(clippy::must_use_candidate)]
    pub fn get(&self) -> f64 {
        self.0 * 100.0
    }

    /// Get the value of the `Percentage` as a floating point number between 0.0
    /// and 1.0
    #[allow(clippy::must_use_candidate)]
    pub const fn as_fraction(&self) -> f64 {
        self.0
    }
}

impl std::fmt::Display for Percentage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}%", self.0)
    }
}

impl std::ops::Mul<Percentage> for f64 {
    type Output = Self;

    fn mul(self, rhs: Percentage) -> Self {
        self * rhs.as_fraction()
    }
}

impl std::ops::Mul<f64> for Percentage {
    type Output = f64;

    fn mul(self, rhs: f64) -> f64 {
        self.as_fraction() * rhs
    }
}

impl std::ops::Mul for Percentage {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self(self.as_fraction() * rhs)
    }
}

impl TryFrom<f64> for Percentage {
    type Error = PercentageError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<usize> for Percentage {
    type Error = PercentageError;

    #[allow(clippy::cast_precision_loss)]
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Self::new(value as f64)
    }
}

impl<'de> Deserialize<'de> for Percentage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let p = f64::deserialize(deserializer)?;
        Self::new(p).map_err(serde::de::Error::custom)
    }
}

impl Serialize for Percentage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        (self.0 * 100.0).serialize(serializer)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    #[allow(clippy::undocumented_unsafe_blocks)]
    fn creating_percentages() {
        assert!(matches!(Percentage::new(0.0), Ok(Percentage(0.0))));
        assert!(matches!(Percentage::new(100.0), Ok(Percentage(1.0))));
        assert!(matches!(Percentage::new(50.0), Ok(Percentage(0.5))));
        assert!(matches!(
            Percentage::new(-1.0),
            Err(PercentageError::InvalidPercentage(-1.0))
        ));
        assert!(matches!(
            Percentage::new(101.0),
            Err(PercentageError::InvalidPercentage(101.0))
        ));

        assert_relative_eq!(Percentage::new(0.0).unwrap().get(), 0.0f64);
        assert_relative_eq!(Percentage::new(100.0).unwrap().get(), 100.0f64);
        assert_relative_eq!(Percentage::new(50.0).unwrap().get(), 50.0f64);

        assert_relative_eq!(Percentage::new(0.0).unwrap().as_fraction(), 0.0f64);
        assert_relative_eq!(Percentage::new(100.0).unwrap().as_fraction(), 1.0f64);
        assert_relative_eq!(Percentage::new(50.0).unwrap().as_fraction(), 0.5f64);
        assert_relative_eq!(unsafe { Percentage::new_unchecked(0.0) }.as_fraction(), 0.0f64);
    }

    #[test]
    fn multiplying_percentages() {
        let a = 10.;
        let b = 20.;
        let p0 = Percentage::new(a).unwrap();
        let p1 = Percentage::new(b).unwrap();

        assert_relative_eq!((p0 * p1).get(), Percentage::new(2.).unwrap().get());
        assert_relative_eq!((p1 * p0).get(), Percentage::new(2.).unwrap().get());
    }

    #[test]
    fn multiplying_percentages_with_f64() {
        let ten_percent = Percentage::new(10.).unwrap();

        assert_relative_eq!(ten_percent * 80., 8.);
        assert_relative_eq!(80. * ten_percent, 8.);

        let sixty_percent = Percentage::new(60.).unwrap();
        assert_relative_eq!(sixty_percent * 80., 48.);
        assert_relative_eq!(80. * sixty_percent, 48.);

        let zero_percent = Percentage::new(0.).unwrap();
        assert_relative_eq!(zero_percent * 80., 0.);
        assert_relative_eq!(80. * zero_percent, 0.);

        let hundred_percent = Percentage::new(100.).unwrap();
        let x = 9.5;
        assert_relative_eq!(hundred_percent * x, x);
        assert_relative_eq!(x * hundred_percent, x);
    }

    // #[test]
    // fn deserializing_percentages() {
    //     let p: Percentage = toml::from_str("0.0").unwrap();
    //     assert_eq!(p, Percentage::new(0.0).unwrap());
    //
    //     let p: Percentage = toml::from_str("100.0").unwrap();
    //     assert_eq!(p, Percentage::new(100.0).unwrap());
    //
    //     let p: Percentage = toml::from_str("50.0").unwrap();
    //     assert_eq!(p, Percentage::new(50.0).unwrap());
    //
    //     let p: Percentage = toml::from_str("0").unwrap();
    //     assert_eq!(p, Percentage::new(0.0).unwrap());
    //
    //     let p: Percentage = toml::from_str("100").unwrap();
    //     assert_eq!(p, Percentage::new(100.0).unwrap());
    //
    //     let p: Percentage = toml::from_str("50").unwrap();
    //     assert_eq!(p, Percentage::new(50.0).unwrap());
    //
    //     assert!(toml::from_str::<Percentage>("101").is_err());
    //     assert!(toml::from_str::<Percentage>("-1").is_err());
    //     assert!(toml::from_str::<Percentage>("100.1").is_err());
    //     assert!(toml::from_str::<Percentage>("-0.1").is_err());
    // }

    #[test]
    fn test_try_from_f64() {
        assert!(matches!(Percentage::try_from(0.0), Ok(Percentage(0.0))));
        assert!(matches!(Percentage::try_from(100.0), Ok(Percentage(1.0))));
        assert!(matches!(Percentage::try_from(50.0), Ok(Percentage(0.5))));
        assert!(matches!(
            Percentage::try_from(-1.0),
            Err(PercentageError::InvalidPercentage(-1.0))
        ));
        assert!(matches!(
            Percentage::try_from(101.0),
            Err(PercentageError::InvalidPercentage(101.0))
        ));
    }

    #[test]
    fn test_try_from_usize() {
        assert!(matches!(Percentage::try_from(0), Ok(Percentage(0.0))));
        assert!(matches!(Percentage::try_from(100), Ok(Percentage(1.0))));
        assert!(matches!(Percentage::try_from(50), Ok(Percentage(0.5))));
        assert!(matches!(
            Percentage::try_from(101),
            Err(PercentageError::InvalidPercentage(101.0))
        ));
    }
}
