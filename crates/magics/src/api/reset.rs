//! Module for handling API state reset when environments change.

use bevy::prelude::*;
use super::state::{ApiState, EnvironmentState};
use crate::simulation_loader::{LoadSimulation, ReloadSimulation};
use super::plugin::PreviousCollisionCounts;

/// System that resets the API state when a new environment is loaded.
pub fn reset_api_state_on_simulation_change(
    mut api_state: ResMut<ApiState>,
    mut previous_collision_counts: ResMut<PreviousCollisionCounts>,
    mut evr_load_simulation: EventReader<LoadSimulation>,
    mut evr_reload_simulation: EventReader<ReloadSimulation>,
) {
    // Check if any load or reload events have been received
    let should_reset = !evr_load_simulation.is_empty() || !evr_reload_simulation.is_empty();
    
    // Consume all events
    for _ in evr_load_simulation.read() {
        info!("API: Detected LoadSimulation event, resetting API state");
    }
    
    for _ in evr_reload_simulation.read() {
        info!("API: Detected ReloadSimulation event, resetting API state");
    }
    
    if should_reset {
        // Reset the API state
        info!("API: Resetting API state due to environment change");
        
        // Clear agent states
        if let Ok(mut agent_states) = api_state.agent_states.write() {
            agent_states.clear();
            info!("API: Cleared agent states");
        } else {
            error!("API: Failed to acquire write lock on agent_states");
        }
        
        // Reset environment state
        if let Ok(mut env_state) = api_state.environment_state.write() {
            *env_state = EnvironmentState::default();
            info!("API: Reset environment state");
        } else {
            error!("API: Failed to acquire write lock on environment_state");
        }
        
        // Clear previous collision counts
        previous_collision_counts.robot_collisions.clear();
        previous_collision_counts.environment_collisions.clear();
        info!("API: Cleared previous collision counts");
        
        // Reset any step in progress
        if api_state.is_step_requested() {
            api_state.complete_step();
            info!("API: Completed any in-progress step");
        }
        
        // Reset step iterations remaining
        api_state.set_step_iterations_remaining(0);
        
        info!("API: API state reset complete");
    }
}
