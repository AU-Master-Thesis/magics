//! Serde adapters for external types that don't implement Serialize/Deserialize

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use typed_floats::StrictlyPositiveFinite;

/// Serde adapter for StrictlyPositiveFinite<f32>
pub mod strictly_positive_finite_f32 {
    use super::*;

    /// Serialize a StrictlyPositiveFinite<f32> as a plain f32
    pub fn serialize<S>(value: &StrictlyPositiveFinite<f32>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert to the underlying f32 value
        let f = value.get();
        f.serialize(serializer)
    }

    /// Deserialize a StrictlyPositiveFinite<f32> from a plain f32
    pub fn deserialize<'de, D>(deserializer: D) -> Result<StrictlyPositiveFinite<f32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // First deserialize as a plain f32
        let f = f32::deserialize(deserializer)?;
        
        // Then try to convert to StrictlyPositiveFinite
        StrictlyPositiveFinite::<f32>::new(f).map_err(|_| {
            serde::de::Error::custom(format!(
                "Invalid value for StrictlyPositiveFinite<f32>: {f}, must be > 0 and finite"
            ))
        })
    }
}

/// Serde adapter for StrictlyPositiveFinite<f64>
pub mod strictly_positive_finite_f64 {
    use super::*;

    /// Serialize a StrictlyPositiveFinite<f64> as a plain f64
    pub fn serialize<S>(value: &StrictlyPositiveFinite<f64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert to the underlying f64 value
        let f = value.get();
        f.serialize(serializer)
    }

    /// Deserialize a StrictlyPositiveFinite<f64> from a plain f64
    pub fn deserialize<'de, D>(deserializer: D) -> Result<StrictlyPositiveFinite<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // First deserialize as a plain f64
        let f = f64::deserialize(deserializer)?;
        
        // Then try to convert to StrictlyPositiveFinite
        StrictlyPositiveFinite::<f64>::new(f).map_err(|_| {
            serde::de::Error::custom(format!(
                "Invalid value for StrictlyPositiveFinite<f64>: {f}, must be > 0 and finite"
            ))
        })
    }
}
