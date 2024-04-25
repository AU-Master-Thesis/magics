#![deny(missing_docs)]
//! This module contains a newtype representing a sample rate.

use std::{num::NonZeroU64, time::Duration};

/// Newtype representing a sample rate
/// The newtype wraps a `std::time::Duration` to ensure the invariant that the
/// Duration is never zero time.
#[derive(Debug, Clone, Copy)]
pub struct SampleRate(Duration);

/// Error type for fallible functions in this module
#[derive(Debug)]
pub enum Error {
    /// A `SampleRate` cannot be negative
    NegativeTime(f64),
    /// A `SampleRate` cannot be instantaneous, i.e. the delay it represents
    /// takes 0 time
    Instantaneous,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NegativeTime(t) => {
                write!(f, "A SampleRate cannot be negative, provided value is {t}",)
            }
            Self::Instantaneous => write!(
                f,
                "An instantaneous SampleRate is not allowed. I.e. a Duration of 0.0 seconds is invalid"
            ),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for fallible functions in this module
pub type Result<T> = std::result::Result<T, Error>;

impl SampleRate {
    /// Create a `SampleRate` from a number of seconds.
    /// Returns `Err(SampleRateError::NegativeTime)` if the provided seconds is
    /// negative. Returns `Err(SampleRateError::Instantaneous)` if the
    /// provided seconds is 0.0 Returns `Ok` otherwise.
    ///
    /// # Errors
    ///
    /// Will return `Err` if `secs` is <= 0.0
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn from_secs(secs: f64) -> Result<Self> {
        if secs.is_sign_negative() {
            Err(Error::NegativeTime(secs))
        } else if secs == 0.0 {
            Err(Error::Instantaneous)
        } else {
            let nanos = (secs.fract() * 1e9) as u32;
            Ok(Self(Duration::new(secs as u64, nanos)))
        }
    }

    /// Create a `SampleRate` from a number of samples per second (Hz).
    #[allow(clippy::cast_precision_loss)]
    #[must_use]
    pub fn from_hz(hz: NonZeroU64) -> Self {
        let freq = 1.0 / hz.get() as f64;
        Self(Duration::from_secs_f64(freq))
    }

    /// Returns the number of samples per second.
    #[inline(always)]
    #[must_use]
    pub fn as_secs(&self) -> f64 {
        self.0.as_secs_f64()
    }

    /// Returns the inner wrapped `std::time::Duration`
    #[inline(always)]
    #[must_use]
    pub const fn as_duration(self) -> Duration {
        self.0
    }
}

impl std::ops::Deref for SampleRate {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     // use pretty_assertions::{assert_eq, assert_ne};
//
//     // #[test]
//     // fn () {
//
//     // }
// }
