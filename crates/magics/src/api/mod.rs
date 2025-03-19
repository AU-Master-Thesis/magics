//! API module for external control of the simulation.
//!
//! This module provides functionality for controlling the simulation from external
//! sources, particularly for integration with Python via PyO3.

mod plugin;
mod state;

pub use plugin::ApiPlugin;
pub use state::{AgentState, ApiState, EnvironmentState, WeightUpdate, FactorWeights, FactorGraphState};

/// Feature flag for enabling the API functionality
#[cfg(feature = "api")]
pub const API_ENABLED: bool = true;

/// Feature flag for enabling the API functionality
#[cfg(not(feature = "api"))]
pub const API_ENABLED: bool = false;
