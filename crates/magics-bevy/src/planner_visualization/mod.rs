//! Visualization components for the Magics planner
//!
//! This module provides visualization for planner elements including
//! agents, trajectories, collisions, and communication.

use bevy::prelude::*;
use std::sync::{Arc, Mutex};

use crate::asset_loader::AssetLibrary;
use crate::simulation_visualization::SimulationState;

/// Plugin for planner visualization
pub struct PlannerVisualizationPlugin {
    /// Simulation state to visualize
    pub simulation_state: Option<Arc<Mutex<SimulationState>>>,
}

impl Default for PlannerVisualizationPlugin {
    fn default() -> Self {
        Self {
            simulation_state: None,
        }
    }
}

impl Plugin for PlannerVisualizationPlugin {
    fn build(&self, app: &mut App) {
        if let Some(simulation_state) = &self.simulation_state {
            app.insert_resource(SimulationStateResource(simulation_state.clone()));
        }

        app.add_systems(Startup, setup_visualization)
            .add_systems(
                Update,
                (
                    update_agent_positions,
                    update_trajectories,
                    update_collisions,
                ),
            );
    }
}

/// Resource wrapper for simulation state
#[derive(Resource)]
struct SimulationStateResource(Arc<Mutex<SimulationState>>);

/// Component marker for agent visualization
#[derive(Component)]
pub struct AgentMarker {
    /// ID of the agent
    pub id: usize,
}

/// Component marker for trajectory visualization
#[derive(Component)]
pub struct TrajectoryMarker {
    /// ID of the agent this trajectory belongs to
    pub agent_id: usize,
}

/// Component marker for collision visualization
#[derive(Component)]
pub struct CollisionMarker;

/// Component marker for communication visualization
#[derive(Component)]
pub struct CommunicationMarker {
    /// ID of the source agent
    pub source_id: usize,
    /// ID of the target agent
    pub target_id: usize,
}

/// Setup system for visualization
fn setup_visualization(
    mut commands: Commands,
    asset_library: Res<AssetLibrary>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create a default agent mesh if needed
    if asset_library.get_mesh("agent").is_none() {
        let agent_mesh = meshes.add(
            shape::Capsule {
                radius: 0.5,
                depth: 1.0,
                ..default()
            }
            .into()
        );
        
        let agent_material = materials.add(StandardMaterial {
            base_color: Color::rgb(0.2, 0.6, 0.9),
            ..default()
        });
        
        // Create visualization entities for debugging purposes
        commands.spawn((
            PbrBundle {
                mesh: agent_mesh,
                material: agent_material,
                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                ..default()
            },
            AgentMarker { id: 0 },
            Name::new("Debug Agent"),
        ));
    }
}

/// System to update agent positions from simulation state
fn update_agent_positions(
    simulation_state: Option<Res<SimulationStateResource>>,
    mut agent_query: Query<(&mut Transform, &AgentMarker)>,
) {
    let Some(simulation_state) = simulation_state else {
        return;
    };
    
    // Try to get a lock on the simulation state
    let Ok(state) = simulation_state.0.try_lock() else {
        return;
    };
    
    // Update positions of all agent entities
    for (mut transform, marker) in agent_query.iter_mut() {
        if let Some(agent) = state.agents.get(&marker.id) {
            // Update the position from the agent state
            transform.translation.x = agent.position.x;
            transform.translation.z = agent.position.y; // y in 2D becomes z in 3D
            
            // Update rotation if heading is available
            if let Some(heading) = agent.heading {
                transform.rotation = Quat::from_rotation_y(-heading);
            }
        }
    }
}

/// System to update trajectory visualizations
fn update_trajectories(
    simulation_state: Option<Res<SimulationStateResource>>,
    mut trajectory_query: Query<(&mut Visibility, &TrajectoryMarker)>,
) {
    let Some(simulation_state) = simulation_state else {
        return;
    };
    
    // Try to get a lock on the simulation state
    let Ok(state) = simulation_state.0.try_lock() else {
        return;
    };
    
    // Update all trajectory visualizations
    for (mut visibility, marker) in trajectory_query.iter_mut() {
        // Show trajectory only if agent exists and has a trajectory
        if let Some(agent) = state.agents.get(&marker.agent_id) {
            *visibility = if agent.trajectory.is_empty() {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

/// System to update collision visualizations
fn update_collisions(
    simulation_state: Option<Res<SimulationStateResource>>,
    mut collision_query: Query<(&mut Visibility, &mut Transform, &CollisionMarker)>,
) {
    let Some(simulation_state) = simulation_state else {
        return;
    };
    
    // Try to get a lock on the simulation state
    let Ok(state) = simulation_state.0.try_lock() else {
        return;
    };
    
    // Update all collision visualizations
    for (mut visibility, mut transform, _) in collision_query.iter_mut() {
        // Show collision markers at collision locations
        if !state.collisions.is_empty() {
            *visibility = Visibility::Visible;
            
            // For simplicity, just move to the first collision location
            // In a real implementation, you would spawn/despawn collision markers
            if let Some(collision) = state.collisions.first() {
                transform.translation.x = collision.position.x;
                transform.translation.z = collision.position.y;
            }
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}
