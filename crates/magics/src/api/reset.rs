//! Module for handling API state reset when environments change.

use bevy::prelude::*;
use super::state::{ApiState, EnvironmentState};
use crate::simulation_loader::{LoadSimulation, ReloadSimulation, SimulationManager};
use super::plugin::PreviousCollisionCounts;

/// Event sent when a reset operation is completed.
#[derive(Event)]
pub struct ResetCompleted;

/// Event sent when an environment load operation is completed.
#[derive(Event)]
pub struct LoadEnvironmentCompleted {
    /// Name of the environment that was loaded.
    pub name: String,
}

/// System that handles reset and load environment requests from the API.
pub fn handle_reset_and_load_requests(
    api_state: Res<ApiState>,
    mut simulation_manager: Option<ResMut<SimulationManager>>,
) {
    // Check if a reset has been requested
    if api_state.is_reset_requested() {
        info!("API: Processing reset request");
        
        // Check if we have a simulation manager
        if let Some(ref mut sim_manager) = simulation_manager {
            // Reload the current simulation
            sim_manager.reload();
            info!("API: Reset request processed - Reloaded current simulation");   
        } else {
            error!("API: Failed to process reset request - SimulationManager not available");
        }
        // Clear the reset request flag
        api_state.clear_reset_request();
    }
    
    // Check if an environment load has been requested
    if let Some(env_name) = api_state.get_load_environment_request() {
        info!("API: Processing load environment request for '{}'", env_name);
        
        // Check if we have a simulation manager
        if let Some(ref mut sim_manager) = simulation_manager {
            // Find the simulation ID by name
            if let Some(id) = sim_manager.id_from_name(&env_name) {
                // Load the simulation
                sim_manager.load(id);
                info!("API: Load environment request processed - Loaded simulation '{}'", env_name);
            } else {
                error!("API: Failed to process load environment request - Simulation '{}' not found", env_name);
            }
        } else {
            error!("API: Failed to process load environment request - SimulationManager not available");
        }
        // Clear the load environment request
        api_state.clear_load_environment_request();
    }
}

/// System that resets the API state when a new environment is loaded.
pub fn reset_api_state_on_simulation_change(
    mut api_state: ResMut<ApiState>,
    mut previous_collision_counts: ResMut<PreviousCollisionCounts>,
    mut evr_load_simulation: EventReader<LoadSimulation>,
    mut evr_reload_simulation: EventReader<ReloadSimulation>,
    mut ev_reset_completed: EventWriter<ResetCompleted>,
    mut ev_load_environment_completed: EventWriter<LoadEnvironmentCompleted>,
) {
    // Track if we're processing a reset or load event
    let mut is_reset = false;
    let mut loaded_environment_name = None;
    
    // Check if any load or reload events have been received
    let should_reset = !evr_load_simulation.is_empty() || !evr_reload_simulation.is_empty();
    
    // Consume all events
    for ev in evr_load_simulation.read() {
        info!("API: Detected LoadSimulation event for '{}', resetting API state", ev.name);
        loaded_environment_name = Some(ev.name.clone());
    }
    
    for _ in evr_reload_simulation.read() {
        info!("API: Detected ReloadSimulation event, resetting API state");
        is_reset = true;
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
        
        // Send appropriate completion event
        if is_reset {
            ev_reset_completed.send(ResetCompleted);
            info!("API: Sent ResetCompleted event");
        } else if let Some(name) = loaded_environment_name {
            ev_load_environment_completed.send(LoadEnvironmentCompleted { name: name.clone() });
            info!("API: Sent LoadEnvironmentCompleted event for '{}'", name);
        }
    }
}

/// System that handles completion events and sets the completion flags in ApiState.
pub fn handle_completion_events(
    api_state: Res<ApiState>,
    mut evr_reset_completed: EventReader<ResetCompleted>,
    mut evr_load_environment_completed: EventReader<LoadEnvironmentCompleted>,
) {
    // Handle reset completion events
    for _ in evr_reset_completed.read() {
        api_state.complete_reset();
        info!("API: Reset completed, marked in API state");
        

    }
    
    // Handle load environment completion events
    for ev in evr_load_environment_completed.read() {
        api_state.complete_load_environment();
        info!("API: Load of environment '{}' completed, marked in API state", ev.name);
    }
}
