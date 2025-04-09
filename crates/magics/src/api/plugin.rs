//! Plugin for API integration with the simulation.
//!
//! This module provides a Bevy plugin that integrates the API functionality
//! with the simulation.

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use bevy::prelude::*;
use gbp_config::Config;

use super::{
    state::ApiState,
    zmq_server::{ZmqServer, DEFAULT_PORT},
    despawned_agents::{DespawnedAgentsTracker, track_robots_about_to_despawn, track_entities_with_despawn_timer, clear_despawned_agents_after_step},
    agent_management, // New module for agent management
    replan, // New module for replan functionality
    scenario, // New module for scenario functionality
    step, // New module for step functionality
};

/// Resource to track previous collision counts for calculating deltas
#[derive(Resource, Default)]
pub struct PreviousCollisionCounts {
    /// Previous robot-robot collision counts for each agent
    pub robot_collisions: HashMap<Entity, usize>,
    /// Previous robot-environment collision counts for each agent
    pub environment_collisions: HashMap<Entity, usize>,
}

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
        // Initialize the API state, collision tracking, and despawned agents tracker
        app.init_resource::<ApiState>()
           .init_resource::<PreviousCollisionCounts>()
           .init_resource::<DespawnedAgentsTracker>();

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

        // Register event types
        app.add_event::<super::reset::ResetCompleted>()
           .add_event::<super::reset::LoadEnvironmentCompleted>();

        // Add systems
        app
           // FixedUpdate Integration
           .add_systems(PreUpdate, step::process_step_request.run_if(api_mode_active))
           .add_systems(FixedUpdate, step::monitor_fixed_update.run_if(api_mode_active).run_if(step::api_step_in_progress))
           .add_systems(FixedUpdate, step::complete_step_in_fixed_update.after(step::monitor_fixed_update).run_if(api_mode_active).run_if(step::api_step_in_progress))

           // Add systems for weight updates
           .add_systems(PreUpdate, super::weights::apply_weight_updates)

           // Add system to reset API state when a new environment is loaded
           .add_systems(Update, super::reset::reset_api_state_on_simulation_change)

           // Add system to handle reset and load environment requests
            .add_systems(PreUpdate, super::reset::handle_reset_and_load_requests.run_if(api_mode_active))

            // Add system to handle completion events
            .add_systems(Update, super::reset::handle_completion_events)

            // Add system to handle agent removal requests
            .add_systems(Update, agent_management::handle_agent_removal_requests.run_if(api_mode_active))

            // Add system to handle agent spawn requests
            .add_systems(Update, agent_management::handle_agent_spawn_requests.run_if(api_mode_active))

            // Add system to update API state with current scenario name on load
            .add_systems(Update, scenario::update_api_scenario_name_on_load)

           // Add systems for tracking despawned agents
           .add_systems(Update, track_robots_about_to_despawn)
           .add_systems(PreUpdate, track_entities_with_despawn_timer)
            .add_systems(PostUpdate, clear_despawned_agents_after_step.after(step::complete_step_in_fixed_update))

            // Add system to handle replan requests
            .add_systems(Update, replan::handle_replan_requests.run_if(api_mode_active));
    }
}

/// Clean up the ZMQ server on app exit.
fn cleanup_zmq_server(mut zmq_server: ResMut<ZmqServer>) {
    zmq_server.stop();
}

/// Run condition that checks if the API mode is active.
fn api_mode_active(api_state: Res<ApiState>) -> bool {
    api_state.is_active()
}
