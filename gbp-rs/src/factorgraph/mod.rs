pub mod factor;
pub mod factorgraph;
mod measurement_model;
pub mod message;
pub mod variable;

use nutype::nutype;

/// Represents a closed interval [0,1]
#[nutype(
    validate(greater_or_equal = 0.0, less_or_equal = 1.0),
    derive(Debug, Clone, Copy)
)]

pub struct UnitInterval(f64);

#[nutype(validate(greater_or_equal = 0.0), derive(Debug, Clone, Copy))]
pub struct LearningRate(f64);

pub struct Dropout(bool);
#[derive(Debug, Copy, Clone)]
pub struct Include(bool);

// Dropout(false)
