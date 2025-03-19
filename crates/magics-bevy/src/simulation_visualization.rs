//! Simulation visualization systems
//!
//! This module contains systems for visualizing the state of the simulation,
//! converting core simulation data into visual representations.

use std::sync::{Arc, Mutex};
use bevy::prelude::*;
use magics_core::prelude::*;
use magics_core::simulation::Simulation;

use crate::SimulationState;

/// Plugin for visualizing simulation state
pub struct SimulationVisualizationPlugin {
    /// Reference to the core simulation
    pub simulation: Arc<Mutex<Box<dyn Simulation>>>,
}

impl Plugin for SimulationVisualizationPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(SimulationState {
                simulation: self.simulation.clone(),
                paused: false,
                time: 0.0,
                agent_states: Vec::new(),
            })
            .add_systems(Update, (
                update_simulation_visualization,
                render_agents,
                render_environment,
                render_trajectories,
            ));
    }
}

/// Component for agent visualization
#[derive(Component)]
pub struct AgentVisual {
    /// ID of the agent
    pub id: u32,
}

/// Component for environment obstacle visualization
#[derive(Component)]
pub struct ObstacleVisual;

/// Component for trajectory visualization
#[derive(Component)]
pub struct TrajectoryVisual {
    /// ID of the agent
    pub agent_id: u32,
}

/// Update visualization data from simulation
pub fn update_simulation_visualization(
    mut commands: Commands,
    mut state: ResMut<SimulationState>,
    time: Res<Time>,
) {
    // Skip updates if paused
    if state.paused {
        return;
    }

    // Lock simulation to get current state
    let mut simulation = match state.simulation.lock() {
        Ok(sim) => sim,
        Err(poisoned) => {
            // If the mutex is poisoned, we can still use the data
            poisoned.into_inner()
        }
    };

    // Optional: Step simulation if running in UI mode
    // This might be controlled elsewhere via pause/play
    if !state.paused {
        let _ = simulation.step();
    }

    // Extract state for visualization
    let sim_state = simulation.get_state();
    
    // Update visualization state resources
    state.agent_states = sim_state.agents.clone();
    state.time = sim_state.time.get();

    // Update visualization entities (or queue updates)
    // Additional logic will be added here
}

/// Render agents based on simulation state
pub fn render_agents(
    mut commands: Commands,
    state: Res<SimulationState>,
    query: Query<(Entity, &AgentVisual)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // This will be implemented to render agent entities based on state
    // For now, this is a placeholder for the implementation
    // We'll create/update visual entities for each agent in the simulation
}

/// Render environment obstacles
pub fn render_environment(
    mut commands: Commands,
    state: Res<SimulationState>,
    query: Query<Entity, With<ObstacleVisual>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // This will be implemented to render environment obstacles
    // For now, this is a placeholder for the implementation
}

/// Render agent trajectories
pub fn render_trajectories(
    mut commands: Commands,
    state: Res<SimulationState>,
    query: Query<(Entity, &TrajectoryVisual)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // This will be implemented to render agent trajectories
    // For now, this is a placeholder for the implementation
}
