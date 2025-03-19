//! Serde adapters for external types that don't implement Serialize/Deserialize

use gbp_linalg::Float;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use typed_floats::StrictlyPositiveFinite;

/// Serde adapter for StrictlyPositiveFinite<Float>
pub mod strictly_positive_finite_float {
    use super::*;

    /// Serialize a StrictlyPositiveFinite<Float> as a plain Float
    pub fn serialize<S>(value: &StrictlyPositiveFinite<Float>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert to the underlying Float value
        let f = value.get();
        f.serialize(serializer)
    }

    /// Deserialize a StrictlyPositiveFinite<Float> from a plain Float
    pub fn deserialize<'de, D>(deserializer: D) -> Result<StrictlyPositiveFinite<Float>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // First deserialize as a plain Float
        let f = Float::deserialize(deserializer)?;
        
        // Then try to convert to StrictlyPositiveFinite
        StrictlyPositiveFinite::<Float>::new(f).map_err(|_| {
            serde::de::Error::custom(format!(
                "Invalid value for StrictlyPositiveFinite<Float>: {f}, must be > 0 and finite"
            ))
        })
    }
}
