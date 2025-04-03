//! Plugin for API integration with the simulation.
//!
//! This module provides a Bevy plugin that integrates the API functionality
//! with the simulation.

use std::{
    collections::HashMap,
    ops::DerefMut,
    sync::{Arc, RwLock},
    time::Duration,
};

use bevy::prelude::*;
use bevy_mod_picking::prelude::{Click, On, PickableBundle, Pointer};
use bevy_rand::prelude::{ForkableRng, GlobalEntropy};
use gbp_config::{
    formation::{self, ReachedWhen, Formation as FormationConfig}, // Use alias FormationConfig
    Config,
};
use gbp_linalg::VectorNorm;
use min_len_vec::{one_or_more, two_or_more, OneOrMore, TwoOrMore}; // Import OneOrMore
use rand::seq::IteratorRandom;
use strum::IntoEnumIterator;

use super::{
    state::{
        AgentState, ApiState, CollisionInfo, EnvironmentState, FactorCounts, FactorDetails, FactorGraphState, 
        FactorWeights, MessageStats, MissionProgress, MissionState, PlanningStrategy, 
        StateVectorInfo, WeightUpdate,
    },
    zmq_server::{ZmqServer, DEFAULT_PORT},
    despawned_agents::{DespawnedAgentsTracker, track_robots_about_to_despawn, track_entities_with_despawn_timer, clear_despawned_agents_after_step},
};
use crate::{
    environment::{FollowCameraMe, ObstacleMarker},
    factorgraph::factorgraph::FactorGraph, // Keep single FactorGraph import
    movement::Velocity,
    pause_play::PausePlay,
    planner::{
        collisions::resources::{RobotEnvironmentCollisions, RobotRobotCollisions},
        robot::{
            Mission, RadioAntenna, Radius, RobotBundle, RobotConnections, RobotDespawned,
            RobotSpawned, Route, StateVector,
        },
        spawner::{RobotClickedOn, WaypointCreated},
        tracking::{PositionTracker, VelocityTracker},
        visualiser::waypoints::{AssociatedWithRobot, WaypointVisualiser}, // Added visualiser imports
    },
    simulation_loader::{LoadSimulation, Reloadable, Sdf, SimulationManager}, // Added LoadSimulation, SimulationManager
    theme::{CatppuccinTheme, ColorAssociation, ColorFromCatppuccinColourExt, DisplayColour},
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
           .add_systems(PreUpdate, process_step_request.run_if(api_mode_active))
           .add_systems(FixedUpdate, monitor_fixed_update.run_if(api_mode_active).run_if(api_step_in_progress))
           .add_systems(FixedUpdate, complete_step_in_fixed_update.after(monitor_fixed_update).run_if(api_mode_active).run_if(api_step_in_progress))
           
           // Add systems for weight updates
           .add_systems(PreUpdate, super::weights::apply_weight_updates)
           
           // Add system to reset API state when a new environment is loaded
           .add_systems(Update, super::reset::reset_api_state_on_simulation_change)
           
           // Add system to handle reset and load environment requests
            .add_systems(PreUpdate, super::reset::handle_reset_and_load_requests.run_if(api_mode_active))
            
            // Add system to handle completion events
            .add_systems(Update, super::reset::handle_completion_events)

            // Add system to handle agent removal requests
            .add_systems(Update, handle_agent_removal_requests.run_if(api_mode_active)) // Corrected typo

            // Add system to handle agent spawn requests
            .add_systems(Update, handle_agent_spawn_requests.run_if(api_mode_active))

            // Add system to update API state with current scenario name on load
            .add_systems(Update, update_api_scenario_name_on_load)
            
           // Add systems for tracking despawned agents
           .add_systems(Update, track_robots_about_to_despawn)
           .add_systems(PreUpdate, track_entities_with_despawn_timer)
           .add_systems(PostUpdate, clear_despawned_agents_after_step.after(complete_step_in_fixed_update));
           
           // Note: extract_state is now called directly from complete_step_in_fixed_update
           // when remaining <= 1, so we don't need to add it as a separate system
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
    // Add parameters needed for extract_state
    robots: Query<(
        Entity,
        &Transform,
        &FactorGraph,
        &RobotConnections,
        Option<&Mission>,
        Option<&formation::PlanningStrategy>,
        Option<&Radius>,
        Option<&RadioAntenna>,
    )>,
    obstacles: Query<&Transform, With<ObstacleMarker>>,
    config: Res<Config>,
    robot_robot_collisions: Res<RobotRobotCollisions>,
    robot_environment_collisions: Res<RobotEnvironmentCollisions>,
    mut previous_collision_counts: ResMut<PreviousCollisionCounts>,
    mut despawned_agents_tracker: ResMut<DespawnedAgentsTracker>, // Change to mutable ResMut
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
        
        // Extract state at the end of the step
        info!("API: Extracting state at end of step");
        super::extract::extract_state(
            &api_state, 
            robots, 
            obstacles, 
            config, 
            &robot_robot_collisions, 
            &robot_environment_collisions, 
            &mut previous_collision_counts,
            &mut despawned_agents_tracker // Pass mutably
        );

        // Pause the simulation again
        let virtual_time = time_virtual.bypass_change_detection();
        virtual_time.pause();
        
        // Mark the step as completed
        api_state.complete_step();
        
        info!("API: Completed step after {} iterations", 
              api_state.get_step_iterations_remaining());
    }
}


/// System to handle agent removal requests from the API
fn handle_agent_removal_requests(
    mut commands: Commands,
    api_state: Res<ApiState>,
    mut despawned_agents_tracker: ResMut<DespawnedAgentsTracker>,
    config: Res<Config>,
    robot_robot_collisions: Res<RobotRobotCollisions>,
    robot_environment_collisions: Res<RobotEnvironmentCollisions>,
    previous_collision_counts: Res<PreviousCollisionCounts>,
    mut evw_robot_despawned: EventWriter<RobotDespawned>,
    mut traces: ResMut<crate::planner::visualiser::tracer::Traces>, // Add Traces resource
    // Query to find the robot entity and its components
    robots_query: Query<(
        Entity,
        &Transform,
        &FactorGraph,
        &RobotConnections,
        Option<&Mission>,
        Option<&formation::PlanningStrategy>,
        Option<&Radius>,
        Option<&RadioAntenna>,
    )>,
    // Query for waypoint visualizers associated with robots
    waypoint_viz_query: Query<(Entity, &AssociatedWithRobot), With<WaypointVisualiser>>,
) {
    let agent_ids_to_remove = api_state.get_agent_removal_requests();

    if agent_ids_to_remove.is_empty() {
        return;
    }

    info!("API: Processing removal requests for agents: {:?}", agent_ids_to_remove);

    for agent_id_to_remove in agent_ids_to_remove {
        // Find the entity corresponding to the agent_id
        let mut found_entity: Option<Entity> = None;
        for (entity, ..) in robots_query.iter() {
            if entity.index() == agent_id_to_remove {
                found_entity = Some(entity);
                break;
            }
        }

        if let Some(entity_to_remove) = found_entity {
            // Get the components for the specific entity
            if let Ok((
                _entity, // We already have entity_to_remove
                transform,
                factor_graph,
                connections,
                mission_opt,
                planning_strategy_opt,
                radius_opt,
                antenna_opt,
            )) = robots_query.get(entity_to_remove)
            {
                info!("API: Found entity {:?} for removal request ID {}", entity_to_remove, agent_id_to_remove);

                // 1. Capture final state
                let final_state = super::state_utils::create_agent_state(
                    entity_to_remove,
                    transform,
                    factor_graph,
                    connections,
                    mission_opt,
                    planning_strategy_opt,
                    radius_opt,
                    antenna_opt,
                    &robot_robot_collisions,
                    &robot_environment_collisions,
                    &previous_collision_counts,
                    &config,
                    true, // Always extract factor details for despawned agents
                );

                // 2. Add state to tracker
                despawned_agents_tracker.despawned_agents.insert(entity_to_remove, final_state);
                info!("API: Added final state of {:?} to DespawnedAgentsTracker", entity_to_remove);

                // 3. Remove path trace data
                if traces.0.remove(&entity_to_remove).is_some() {
                    info!("API: Removed path trace data for robot {:?}", entity_to_remove);
                } else {
                    warn!("API: No path trace data found for robot {:?} during removal", entity_to_remove);
                }

                // 4. Despawn associated waypoint visualizers
                for (viz_entity, associated_robot) in waypoint_viz_query.iter() {
                    if associated_robot.0 == entity_to_remove {
                        commands.entity(viz_entity).despawn_recursive();
                        info!("API: Despawned waypoint visualizer {:?} for robot {:?}", viz_entity, entity_to_remove);
                    }
                }

                // 5. Despawn main robot entity
                commands.entity(entity_to_remove).despawn_recursive();
                info!("API: Despawned entity {:?}", entity_to_remove);

                // 6. Send event (might still be useful for other listeners)
                evw_robot_despawned.send(RobotDespawned(entity_to_remove));
                info!("API: Sent RobotDespawned event for {:?}", entity_to_remove);

            } else {
                warn!("API: Could not query components for entity {:?} (ID {}) during removal request. Maybe already despawned?", entity_to_remove, agent_id_to_remove);
            }
        } else {
            warn!("API: Agent with ID {} not found for removal request.", agent_id_to_remove);
        }
    }
}

/// System to update the current scenario name in ApiState when a LoadSimulation event occurs.
fn update_api_scenario_name_on_load(
    mut evr_load_simulation: EventReader<LoadSimulation>,
    api_state: Res<ApiState>,
) {
    for event in evr_load_simulation.read() {
        if let Ok(mut scenario_name_lock) = api_state.current_scenario_name.write() {
            *scenario_name_lock = Some(event.name.clone());
            info!("API: Updated current scenario name in ApiState to: {}", event.name);
        } else {
            error!("API: Failed to acquire write lock on current_scenario_name");
        }
    }
}


/// System to handle agent spawn requests from the API
fn handle_agent_spawn_requests(
    mut commands: Commands,
    api_state: Res<ApiState>,
    config: Res<Config>,
    env_config: Res<gbp_environment::Environment>,
    sdf: Res<Sdf>,
    mut prng: ResMut<GlobalEntropy<bevy_prng::WyRand>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    theme: Res<CatppuccinTheme>,
    time_fixed: Res<Time<Fixed>>,
    // Events needed?
    mut evw_robot_spawned: EventWriter<RobotSpawned>,
    mut evw_waypoint_created: EventWriter<WaypointCreated>,
) {
    let spawn_requests = api_state.get_agent_spawn_requests();

    if spawn_requests.is_empty() {
        return;
    }

    info!("API: Processing {} agent spawn requests", spawn_requests.len());

    for params in spawn_requests {
        // --- Prepare Agent Parameters ---
        let initial_pos = Vec2::from_array(params.initial_position);
        let goal_pos = Vec2::from_array(params.goal_position);
        let initial_vel = params.initial_velocity.map_or(Vec2::ZERO, Vec2::from_array);
        
        // Use provided radius or default from config
        let radius = params.radius.unwrap_or_else(|| {
            // Use the start of the range
            *config.robot.radius.range().start() // Keep using start()
        });

        // Use provided target speed or default from config
        let target_speed = params.target_speed.unwrap_or_else(|| config.robot.target_speed.get());

        // Determine planning strategy
        let planning_strategy = params.planning_strategy.map_or(
            formation::PlanningStrategy::OnlyLocal, // Default
            |s| match s.to_lowercase().as_str() {
                "rrtstar" => formation::PlanningStrategy::RrtStar,
                _ => formation::PlanningStrategy::OnlyLocal,
            }
        );

        // Waypoints: Create a simple route from initial pos to goal pos
        // Velocity at goal can be zero or derived? Using zero for now.
        let initial_state_vec = StateVector::new(Vec4::new(initial_pos.x, initial_pos.y, initial_vel.x, initial_vel.y));
        let goal_state_vec = StateVector::new(Vec4::new(goal_pos.x, goal_pos.y, 0.0, 0.0)); // Zero velocity at goal
        
        // Waypoints for the bundle constructor
        let waypoints = two_or_more![initial_state_vec, goal_state_vec];

        // Call the helper function to spawn the robot
        let new_entity = crate::planner::spawn_utils::spawn_robot(
            &mut commands,
            &config,
            &env_config,
            &sdf,
            &mut prng,
            &mut materials,
            &mut mesh_assets,
            &theme,
            &time_fixed,
            &mut evw_robot_spawned,
            &mut evw_waypoint_created,
            // Robot specific parameters
            initial_state_vec,
            waypoints, // Pass the TwoOrMore<StateVector> directly
            radius,
            planning_strategy,
            target_speed,
            ReachedWhen::same_as_paper(), // Default waypoint reached
            ReachedWhen::same_as_paper(), // Default finished reached
            None, // Custom weights handled below
        );

        // Add the new agent's ID to the ApiState queue for the ZMQ server
        api_state.add_spawned_agent_id(new_entity.index());

        // Handle custom weights separately by queuing an update
        if let Some(custom_weights) = params.weights {
             let update = WeightUpdate {
                 agent_id: Some(new_entity),
                 weights: custom_weights,
             };
             api_state.add_weight_update(update);
             info!("API: Queued custom weights for spawned agent {:?}", new_entity);
        }
    }
}
