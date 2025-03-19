//! Simulation state tracking and management

/// Current state of a simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationState {
    /// Simulation is initializing
    Initializing,
    /// Simulation is running
    Running,
    /// Simulation is paused
    Paused,
    /// Simulation is complete
    Complete,
    /// Simulation has encountered an error
    Error,
}

impl Default for SimulationState {
    fn default() -> Self {
        Self::Initializing
    }
}
