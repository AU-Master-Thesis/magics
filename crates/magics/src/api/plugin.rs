//! Plugin for API integration with the simulation.
//!
//! This module provides a Bevy plugin that integrates the API functionality
//! with the simulation.

use std::sync::Arc;
use bevy::prelude::*;
use crate::pause_play::PausePlay;
use super::state::{AgentState, ApiState, EnvironmentState, FactorGraphState, FactorWeights, WeightUpdate};
use super::zmq_server::{ZmqServer, DEFAULT_PORT};
use crate::factorgraph::factorgraph::FactorGraph;
use crate::planner::robot::{RobotConnections, StateVector};
use crate::environment::ObstacleMarker;
use gbp_config::Config;

/// Plugin for API integration.
pub struct ApiPlugin {
    /// Port for the ZMQ server
    pub port: Option<u16>,
}

impl Default for ApiPlugin {
    fn default() -> Self {
        Self {
            port: Some(DEFAULT_PORT),
        }
    }
}

impl ApiPlugin {
    /// Create a new API plugin with a specific port.
    pub fn with_port(port: u16) -> Self {
        Self {
            port: Some(port),
        }
    }
}

impl Plugin for ApiPlugin {
    fn build(&self, app: &mut App) {
        // Initialize the API state
        app.init_resource::<ApiState>();
        
        // Get the API state and create the ZMQ server
        let api_state = app.world.resource::<ApiState>().clone();
        let mut zmq_server = ZmqServer::new(Arc::new(api_state), self.port);
        
        // Start the ZMQ server if the API feature is enabled
        #[cfg(feature = "api")]
        if let Err(err) = zmq_server.start() {
            error!("Failed to start ZMQ server: {:?}", err);
        }
        
        // Register the ZMQ server as a resource
        app.insert_resource(zmq_server);
        
        // Add systems
        app
           // Add system to pause the simulation when API is active
           // Run in PostStartup to ensure all resources are properly initialized
           .add_systems(PostStartup, pause_on_api_active)
           // Add cleanup system for ZMQ server
           .add_systems(Last, cleanup_zmq_server)
           // Add systems for weight updates and state extraction
           .add_systems(PreUpdate, apply_weight_updates)
           // Add a system to ensure the simulation stays paused when API is active
           .add_systems(Update, ensure_paused_when_api_active.run_if(api_mode_active))
           .add_systems(PostUpdate, extract_state)
           .add_systems(PostUpdate, wait_for_step_command.run_if(api_mode_active));
    }
}

/// Clean up the ZMQ server on app exit.
fn cleanup_zmq_server(mut zmq_server: ResMut<ZmqServer>) {
    zmq_server.stop();
}

/// System that pauses the simulation when the API is active.
fn pause_on_api_active(
    api_state: Res<ApiState>,
    mut pause_play: EventWriter<PausePlay>,
) {
    if api_state.is_active() {
        // Pause the simulation when API is active
        info!("API is active, sending pause event in PostStartup");
        pause_play.send(PausePlay::Pause);
    }
}

/// Run condition that checks if the API mode is active.
fn api_mode_active(api_state: Res<ApiState>) -> bool {
    api_state.is_active()
}

/// System that ensures the simulation stays paused when API is active.
/// This is a backup system that runs every frame to make sure the simulation
/// doesn't accidentally get unpaused.
fn ensure_paused_when_api_active(
    time: Res<Time<Virtual>>,
    mut pause_play: EventWriter<PausePlay>,
) {
    // If the virtual time is not paused, send a pause event
    if !time.is_paused() {
        info!("Virtual time is not paused when API is active, sending pause event");
        pause_play.send(PausePlay::Pause);
    }
}

/// System that waits for a step command from the API.
fn wait_for_step_command(
    api_state: Res<ApiState>,
    mut pause_play: EventWriter<PausePlay>,
) {
    if api_state.is_step_requested() {
        // Allow one frame to execute
        info!("Step requested, sending play event");
        pause_play.send(PausePlay::Play);
        
        // Signal completion after frame
        api_state.complete_step();
        
        // Pause again after this frame
        info!("Step completed, sending pause event");
        pause_play.send(PausePlay::Pause);
    }
}

/// System that extracts the state of the simulation for the API.
fn extract_state(
    api_state: Res<ApiState>,
    robots: Query<(Entity, &Transform, &StateVector, &FactorGraph, &RobotConnections)>,
    obstacles: Query<&Transform, With<ObstacleMarker>>,
    config: Res<Config>,
) {
    // Only extract state if API is active
    if !api_state.is_active() {
        return;
    }

    // Extract agent states
    if let Ok(mut agent_states) = api_state.agent_states.write() {
        agent_states.clear();
        
        for (entity, transform, state_vector, factor_graph, connections) in robots.iter() {
            let position = Vec2::new(transform.translation.x, transform.translation.z);
            let velocity = state_vector.velocity();
            
            let factor_graph_state = FactorGraphState {
                weights: FactorWeights {
                    dynamic: config.gbp.sigma_factor_dynamics as f32,
                    obstacle: config.gbp.sigma_factor_obstacle as f32,
                    interrobot: config.gbp.sigma_factor_interrobot as f32,
                    tracking: config.gbp.sigma_factor_tracking as f32,
                },
                variable_count: factor_graph.node_count().variables,
                factor_count: factor_graph.node_count().factors,
            };
            
            let connected_neighbors = connections.robots_connected_with.iter().copied().collect();
            
            agent_states.insert(entity, AgentState {
                position,
                velocity,
                factor_graph_state,
                connected_neighbors,
            });
        }
    }
    
    // Extract environment state
    if let Ok(mut env_state) = api_state.environment_state.write() {
        let mut obstacle_positions = Vec::new();
        
        for transform in obstacles.iter() {
            obstacle_positions.push(Vec2::new(transform.translation.x, transform.translation.z));
        }
        
        // TODO: Extract actual environment boundaries from the config
        let boundaries = (Vec2::new(-100.0, -100.0), Vec2::new(100.0, 100.0));
        
        *env_state = EnvironmentState {
            obstacles: obstacle_positions,
            boundaries,
        };
    }
}

/// System that applies weight updates from the API.
fn apply_weight_updates(
    api_state: Res<ApiState>,
    mut robots: Query<(Entity, &mut FactorGraph)>,
    mut config: ResMut<Config>,
) {
    // Only apply updates if API is active
    if !api_state.is_active() {
        return;
    }
    
    let mut updates = Vec::new();
    
    // Get all pending weight updates
    if let Ok(mut weight_requests) = api_state.weight_requests.write() {
        updates.append(&mut weight_requests);
    }
    
    // Apply each update
    for update in updates {
        match update.agent_id {
            // System-wide update
            None => {
                // Update config
                config.gbp.sigma_factor_dynamics = update.weights.dynamic;
                config.gbp.sigma_factor_obstacle = update.weights.obstacle;
                config.gbp.sigma_factor_interrobot = update.weights.interrobot;
                config.gbp.sigma_factor_tracking = update.weights.tracking;
                
                // Update all factor graphs
                for (_, mut factor_graph) in robots.iter_mut() {
                    let mut settings = config.gbp.factors_enabled;
                    factor_graph.change_factor_enabled(settings);
                }
            },
            // Agent-specific update
            Some(agent_id) => {
                if let Ok((_, mut factor_graph)) = robots.get_mut(agent_id) {
                    // TODO: Implement per-agent weight updates
                    // This will require extending the FactorGraph implementation
                    // to support per-agent weights
                }
            }
        }
    }
}
