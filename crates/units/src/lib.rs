#![deny(missing_docs)]
//! Simple crate that contains newtypes for various physical units
//! It contains the following modules:
//! - `sample_rate`

pub mod sample_rate;
pub use sample_rate::SampleRate;

/// Prelude module bringing entire public api of this crate into scope
pub mod prelude {
    pub use super::sample_rate;
}
