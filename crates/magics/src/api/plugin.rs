//! Plugin for API integration with the simulation.
//!
//! This module provides a Bevy plugin that integrates the API functionality
//! with the simulation.

use std::sync::Arc;
use std::time::Duration;

use bevy::prelude::*;
use gbp_config::Config;

use super::{
    state::{
        AgentState, ApiState, EnvironmentState, FactorGraphState, FactorWeights, WeightUpdate,
    },
    zmq_server::{ZmqServer, DEFAULT_PORT},
};
use crate::{
    environment::ObstacleMarker,
    factorgraph::factorgraph::FactorGraph,
    pause_play::PausePlay,
    planner::robot::{RobotConnections, StateVector},
};

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
        Self { port: Some(port) }
    }
}

impl Plugin for ApiPlugin {
    fn build(&self, app: &mut App) {
        // Initialize the API state
        app.init_resource::<ApiState>();

        // Get the API state and create the ZMQ server
        let mut api_state = app.world.resource_mut::<ApiState>().clone();
        
        // Set up references to Config and Time<Fixed> resources
        let config = app.world.resource::<Config>().clone();
        let config_arc = Arc::new(RwLock::new(config));
        api_state.set_config(config_arc.clone());
        
        let time_fixed = app.world.resource::<Time<Fixed>>().clone();
        let time_fixed_arc = Arc::new(RwLock::new(time_fixed));
        api_state.set_time_fixed(time_fixed_arc.clone());
        
        // Update the ApiState resource
        app.insert_resource(api_state.clone());
        
        // Create the ZMQ server with the updated API state
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
           
           // FixedUpdate Integration
           .add_systems(PreUpdate, process_step_request.run_if(api_mode_active))
           .add_systems(FixedUpdate, monitor_fixed_update.run_if(api_mode_active).run_if(api_step_in_progress))
           .add_systems(FixedUpdate, complete_step_in_fixed_update.after(monitor_fixed_update).run_if(api_mode_active).run_if(api_step_in_progress))
           
           // Add systems for weight updates
           .add_systems(PreUpdate, apply_weight_updates)
           
           // Extract state
           .add_systems(PostUpdate, extract_state.run_if(api_mode_active));
    }
}

// Run condition for when a step is in progress
fn api_step_in_progress(api_state: Res<ApiState>) -> bool {
    api_state.is_step_requested() && 
    api_state.get_step_iterations_remaining() > 0
}

/// Clean up the ZMQ server on app exit.
fn cleanup_zmq_server(mut zmq_server: ResMut<ZmqServer>) {
    zmq_server.stop();
}

/// System that pauses the simulation when the API is active.
fn pause_on_api_active(api_state: Res<ApiState>, mut time_virtual: ResMut<Time<Virtual>>) {
    // if api_state.is_active() {
    //     // Pause the simulation when API is active by directly pausing the
    // virtual time     info!("API is active, pausing virtual time in
    // PostStartup");     let virtual_time =
    // time_virtual.bypass_change_detection();     virtual_time.pause();
    // }
}

/// Run condition that checks if the API mode is active.
fn api_mode_active(api_state: Res<ApiState>) -> bool {
    api_state.is_active()
}

// Store the start time of each step
#[derive(Default)]
struct StepTimeTracker {
    start_time: Option<f32>,
    last_time: Option<f32>,
}

/// System to process step requests at the beginning of the frame
fn process_step_request(
    api_state: Res<ApiState>,
    mut time_virtual: ResMut<Time<Virtual>>,
    config: Res<Config>,
    mut step_tracker: Local<StepTimeTracker>,
) {
    if api_state.is_step_requested() && 
       api_state.get_step_iterations_remaining() == 0 {
        // Log the current virtual time
        let before_time = time_virtual.elapsed_seconds();
        info!("API: Starting fixed step - Virtual time before: {:.6}s", before_time);
        
        // Store the start time
        step_tracker.start_time = Some(before_time);
        step_tracker.last_time = Some(before_time);
        
        // Set the number of iterations to run based on the configured iterations_per_step
        let iterations = api_state.get_iterations_per_step();
        api_state.set_step_iterations_remaining(iterations);
        
        // Unpause the simulation to allow systems to run
        let virtual_time = time_virtual.bypass_change_detection();
        virtual_time.unpause();
        
        // Calculate fixed delta based on simulation Hz
        let fixed_delta = 1.0 / config.simulation.hz;
        info!("API: Started step with {} iterations using fixed delta of {:.6}s", 
              iterations, fixed_delta);
    }
}

/// System to monitor FixedUpdate ticks
fn monitor_fixed_update(
    time_virtual: Res<Time<Virtual>>,
    api_state: Res<ApiState>,
    mut step_tracker: Local<StepTimeTracker>,
) {
    let current_time = time_virtual.elapsed_seconds();
    let delta = time_virtual.delta_seconds();
    let remaining = api_state.get_step_iterations_remaining();
    
    // Calculate time difference from last tick
    let time_diff = if let Some(last) = step_tracker.last_time {
        let diff = current_time - last;
        step_tracker.last_time = Some(current_time);
        diff
    } else {
        step_tracker.last_time = Some(current_time);
        0.0
    };
    
    // Log detailed time information for each FixedUpdate tick during a step
    info!(
        "FixedUpdate tick - Virtual time: {:.6}s, Delta: {:.6}s, Time since last tick: {:.6}s, Iterations remaining: {}",
        current_time,
        delta,
        time_diff,
        remaining
    );
}

/// System to complete step after iterations
fn complete_step_in_fixed_update(
    api_state: Res<ApiState>,
    mut time_virtual: ResMut<Time<Virtual>>,
    mut step_tracker: Local<StepTimeTracker>,
) {
    let remaining = api_state.decrement_step_iterations_remaining();
    
    if remaining <= 1 {
        // Log the current virtual time
        let after_time = time_virtual.elapsed_seconds();
        info!("API: Fixed step completed - Virtual time after: {:.6}s", after_time);
        
        // Calculate and log the total time advancement
        if let Some(start_time) = step_tracker.start_time {
            let time_advancement = after_time - start_time;
            info!("API: Step advanced virtual time by: {:.6}s", time_advancement);
            step_tracker.start_time = None;
        }
        
        // Pause the simulation again
        let virtual_time = time_virtual.bypass_change_detection();
        virtual_time.pause();
        
        // Mark the step as completed
        api_state.complete_step();
        
        info!("API: Completed step after {} iterations", 
              api_state.get_step_iterations_remaining());
    }
}

/// System that extracts the state of the simulation for the API.
fn extract_state(
    api_state: Res<ApiState>,
    robots: Query<(
        Entity,
        &Transform,
        &StateVector,
        &FactorGraph,
        &RobotConnections,
    )>,
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
                    dynamic:    config.gbp.sigma_factor_dynamics as f32,
                    obstacle:   config.gbp.sigma_factor_obstacle as f32,
                    interrobot: config.gbp.sigma_factor_interrobot as f32,
                    tracking:   config.gbp.sigma_factor_tracking as f32,
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
            }
            // Agent-specific update
            Some(agent_id) => {
                if let Ok((_, mut factor_graph)) = robots.get_mut(agent_id) {
                    // TODO: Implement per-agent weight updates
                    // This will require extending the FactorGraph
                    // implementation to support per-agent
                    // weights
                }
            }
        }
    }
}
