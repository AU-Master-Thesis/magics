//! Scenario functionality for the API.
//!
//! This module provides functionality for managing scenarios.

use bevy::prelude::*;
use gbp_config::formation::FormationGroup;

use crate::simulation_loader::{LoadSimulation, SimulationManager};

use super::state::ApiState;
use super::replan::extract_available_squares;

/// System to update the current scenario name and available squares in ApiState when a LoadSimulation event occurs.
pub fn update_api_scenario_name_on_load(
    mut evr_load_simulation: EventReader<LoadSimulation>,
    api_state: Res<ApiState>,
    simulation_manager: Res<SimulationManager>,
) {
    for event in evr_load_simulation.read() {
        // Update scenario name
        if let Ok(mut scenario_name_lock) = api_state.current_scenario_name.write() {
            *scenario_name_lock = Some(event.name.clone());
            info!("API: Updated current scenario name in ApiState to: {}", event.name);
        } else {
            error!("API: Failed to acquire write lock on current_scenario_name");
        }
        
        // Update available squares using the formation group from the simulation manager
        if let Some(formation_group) = simulation_manager.active_formation_group() {
            let squares = extract_available_squares(formation_group);
            api_state.update_available_squares(squares);
            info!("API: Updated available squares in ApiState for scenario: {}", event.name);
        } else {
            error!("API: Failed to get active formation group for scenario: {}", event.name);
        }
    }
}
