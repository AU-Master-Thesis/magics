//! Plugin for API integration with the simulation.
//!
//! This module provides a Bevy plugin that integrates the API functionality
//! with the simulation.

use std::sync::Arc;
use std::time::Duration;

use bevy::prelude::*;
use std::sync::RwLock;
use gbp_config::Config;
use gbp_linalg::VectorNorm;
use super::{
    state::{
        AgentState, ApiState, CollisionInfo, EnvironmentState, FactorCounts, FactorDetails, FactorGraphState, 
        FactorWeights, MessageStats, MissionProgress, MissionState, PlanningStrategy, 
        StateVectorInfo, WeightUpdate,
    },
    zmq_server::{ZmqServer, DEFAULT_PORT},
};
use crate::{
    environment::ObstacleMarker,
    factorgraph::{factor::{self, Factor}, factorgraph::FactorGraph},
    movement::Velocity,
    pause_play::PausePlay,
    planner::robot::{Mission, RadioAntenna, Radius, RobotConnections, StateVector},
    planner::collisions::resources::{RobotRobotCollisions, RobotEnvironmentCollisions},
};
use gbp_config::formation;
use std::collections::HashMap;

/// Resource to track previous collision counts for calculating deltas
#[derive(Resource, Default)]
struct PreviousCollisionCounts {
    /// Previous robot-robot collision counts for each agent
    robot_collisions: HashMap<Entity, usize>,
    /// Previous robot-environment collision counts for each agent
    environment_collisions: HashMap<Entity, usize>,
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
        // Initialize the API state and collision tracking
        app.init_resource::<ApiState>()
           .init_resource::<PreviousCollisionCounts>();

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
           .add_systems(PreUpdate, apply_weight_updates);
           
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
        extract_state(
            &api_state, 
            robots, 
            obstacles, 
            config, 
            &robot_robot_collisions, 
            &robot_environment_collisions, 
            &mut previous_collision_counts
        );
        info!("IM HERE DONE EXTRACTING STATE");
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
    api_state: &Res<ApiState>,
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
    robot_robot_collisions: &RobotRobotCollisions,
    robot_environment_collisions: &RobotEnvironmentCollisions,
    previous_collision_counts: &mut PreviousCollisionCounts,
) {
    // Only extract state if API is active
    if !api_state.is_active() {
        info!("API: extract_state skipped because API is not active");
        return;
    }

    // Extract agent states
    if let Ok(mut agent_states) = api_state.agent_states.write() {
        let old_count = agent_states.len();
        agent_states.clear();

        for (entity, transform, factor_graph, connections, mission_opt, planning_strategy_opt, radius_opt, antenna_opt) in robots.iter() {
            let (_, current_variable) = factor_graph
            .first_variable()
            .expect("factorgraph should have >= 2 variables");
            let [px, py] = current_variable.estimated_position();
            let [vx, vy] = current_variable.estimated_velocity();
            
            
            let position = Vec2::new(px as f32, py as f32);
            let velocity = Vec2::new(vx as f32, vy as f32);

            // Extract factor graph state with detailed message statistics
            let mut factor_graph_state = FactorGraphState::default();
            factor_graph_state.weights = FactorWeights {
                dynamic:    config.gbp.sigma_factor_dynamics as f32,
                obstacle:   config.gbp.sigma_factor_obstacle as f32,
                interrobot: config.gbp.sigma_factor_interrobot as f32,
                tracking:   config.gbp.sigma_factor_tracking as f32,
            };
            factor_graph_state.variable_count = factor_graph.node_count().variables;
            factor_graph_state.factor_count = factor_graph.node_count().factors;
            
            // Add message statistics
            factor_graph_state.messages_sent = MessageStats {
                internal: factor_graph.messages_sent().internal,
                external: factor_graph.messages_sent().external,
            };
            factor_graph_state.messages_received = MessageStats {
                internal: factor_graph.messages_received().internal,
                external: factor_graph.messages_received().external,
            };
            
            // Add factor counts
            let factor_counts = factor_graph.factor_count();
            factor_graph_state.factor_counts = FactorCounts {
                obstacle: factor_counts.obstacle,
                interrobot: factor_counts.interrobot,
                dynamic: factor_counts.dynamic,
                tracking: factor_counts.tracking,
            };

            // Use default values for optional components
            // Extract mission and waypoint data if available
            let (mission_state, next_waypoint, goal_point, mission_progress, current_waypoint_index) = 
                if let Some(mission) = mission_opt {
                    // Convert robot mission state to API mission state
                    let api_mission_state = match mission.state {
                        crate::planner::robot::MissionState::Idle { waiting_for_waypoints } => 
                            MissionState::Idle { waiting_for_waypoints },
                        crate::planner::robot::MissionState::Active => MissionState::Active,
                        crate::planner::robot::MissionState::Completed => MissionState::Completed,
                    };
                    
                    let next_wp = mission.next_waypoint().map(|wp| StateVectorInfo {
                        position: wp.position(),
                        velocity: wp.velocity(),
                    });
                    
                    let goal = mission.taskpoints.last().map(|wp| wp.position());
                    
                    // Calculate mission progress statistics
                    let total_waypoints = mission.waypoints().count();
                    
                    // Calculate remaining waypoints based on current state
                    let remaining_waypoints = if mission.is_completed() {
                        0
                    } else {
                        // Count remaining waypoints from the current position
                        let current_route = mission.active_route().unwrap_or_else(|| mission.routes.first().unwrap());
                        let remaining_in_current = if current_route.is_completed() { 
                            0 
                        } else {
                            current_route.len().saturating_sub(
                                current_route.current_waypoint_index().unwrap_or(0)
                            )
                        };
                        
                        // Add remaining waypoints from future routes
                        let future_routes_waypoints: usize = mission.routes.iter()
                            .skip(mission.routes.iter().position(|r| std::ptr::eq(r, current_route)).unwrap_or(0) + 1)
                            .map(|route| route.len())
                            .sum();
                            
                        remaining_in_current + future_routes_waypoints
                    };
                    
                    // Construct the mission progress using available methods
                    let progress = MissionProgress {
                        started_at: mission.started_at(),
                        finished_at: mission.finished_at(),
                        active_route: mission.routes.iter().position(|r| 
                            mission.active_route().map_or(false, |ar| std::ptr::eq(r, ar))
                        ).unwrap_or(0),
                        total_routes: mission.taskpoints.len().saturating_sub(1),
                        total_waypoints,
                        remaining_waypoints,
                    };
                    
                    (api_mission_state, next_wp, goal, progress, mission.current_waypoint_index())
                } else {
                    // Default values if mission is not available
                    (MissionState::default(), None, None, MissionProgress::default(), None)
                };

            // Convert robot planning strategy to API planning strategy
            let planning_strategy = planning_strategy_opt.map_or(PlanningStrategy::default(), |p| {
                match p {
                    formation::PlanningStrategy::OnlyLocal => PlanningStrategy::OnlyLocal,
                    formation::PlanningStrategy::RrtStar => PlanningStrategy::RrtStar,
                }
            });
            
            // Get radius if available, otherwise use default
            let radius_value = radius_opt.map_or(0.5, |r| r.0);
            
            // Get antenna properties if available, otherwise use defaults
            let (communication_active, communication_radius) = antenna_opt.map_or(
                (true, 5.0), 
                |a| (a.active, a.radius)
            );
            
            // Extract factor details if API is active
            let factor_details = if api_state.is_active() {
                extract_factor_details(factor_graph)
            } else {
                FactorDetails::default()
            };
            
            // Extract collision information
            let robot_collisions_total = robot_robot_collisions.get(entity).unwrap_or(0);
            let environment_collisions_total = robot_environment_collisions.get(entity).unwrap_or(0);
            
            // Calculate deltas from previous counts
            let robot_collisions_delta = robot_collisions_total.saturating_sub(
                *previous_collision_counts.robot_collisions.get(&entity).unwrap_or(&0)
            );
            let environment_collisions_delta = environment_collisions_total.saturating_sub(
                *previous_collision_counts.environment_collisions.get(&entity).unwrap_or(&0)
            );
            
            // Update previous counts for next time
            previous_collision_counts.robot_collisions.insert(entity, robot_collisions_total);
            previous_collision_counts.environment_collisions.insert(entity, environment_collisions_total);
            
            // Create collision info
            let collision_info = CollisionInfo {
                robot_collisions_total,
                robot_collisions_delta,
                environment_collisions_total,
                environment_collisions_delta,
            };
            
            // Create complete agent state with all fields populated
            let agent_state = AgentState {
                agent_id: entity.index(),
                factorgraph_id: factor_graph.id().index() as u32,
                position,
                velocity,
                factor_graph_state,
                connected_neighbors: connections.robots_connected_with.iter().copied().collect(),
                mission_state,
                planning_strategy,
                radius: radius_value,
                communication_active,
                communication_radius,
                target_speed: config.robot.target_speed.get(),
                current_waypoint_index,
                next_waypoint,
                goal_point,
                mission_progress,
                factor_details,
                collision_info,
            };
            
            agent_states.insert(entity, agent_state);
        }
        
        // info!("API: Updated agent_states from {} to {} agents", old_count, agent_states.len());
    }

    // Extract environment state
    if let Ok(mut env_state) = api_state.environment_state.write() {
        let mut obstacle_positions = Vec::new();

        for transform in obstacles.iter() {
            obstacle_positions.push(Vec2::new(transform.translation.x, transform.translation.z));
        }

        // TODO: Extract actual environment boundaries from the config
        let boundaries = (Vec2::new(-100.0, -100.0), Vec2::new(100.0, 100.0));

        // Create a default environment state and update with known values
        let mut new_env_state = EnvironmentState::default();
        new_env_state.obstacles = obstacle_positions;
        new_env_state.boundaries = boundaries;
        // Count the number of agents from the api_state's agent_states
        if let Ok(agent_states_guard) = api_state.agent_states.read() {
            new_env_state.total_agents = agent_states_guard.len();
        }
        
        // TODO: Extract additional environment information
        // - Agent density map would require analyzing agent positions
        // - SDF resolution from environment configuration
        // - World size from environment configuration
        
        *env_state = new_env_state;
    }
}

/// Function to extract detailed information about factor graph components.
fn extract_factor_details(factor_graph: &FactorGraph) -> FactorDetails {
    use crate::api::state::{
        VariableInfo, ObstacleFactorInfo, InterRobotFactorInfo, 
        TrackingFactorInfo, DynamicFactorInfo
    };
    use crate::factorgraph::factorgraph::FactorIndex;
    
    let mut factor_details = FactorDetails::default();
    
    // Create a map of variable node indices to our variable indices
    let mut variable_index_map = std::collections::HashMap::new();
    
    // Extract variable information
    for (i, (var_index, variable)) in factor_graph.variables().enumerate() {
        // Store the mapping from node index to our variable index
        variable_index_map.insert(var_index.0.index(), i);
        
        // Properly convert Vector<Float> to [f64; 4]
        let mean: [f64; 4] = variable.belief.mean.as_slice().expect("Array is not contiguous")
        .try_into().expect("Slice length mismatch");
        
        // Convert covariance matrix to array format [f64; 16]
        let covariance: [f64;16] = variable.belief.covariance_matrix.as_slice()
        .expect("Array2 is not contiguous")
        .try_into()
        .expect("Matrix does not have exactly 16 elements");

        let node_index = variable.node_index().index();

        let estimated_position = variable.estimated_position();
        let estimated_velocity = variable.estimated_velocity();
        
        // Get the factorgraph_id
        let factorgraph_id = factor_graph.id().index() as u32;
        
        factor_details.variables.push(VariableInfo {
            index: node_index,
            factorgraph_id,
            mean,
            covariance,
            estimated_position,
            estimated_velocity,
        });
    }
    
    // Extract obstacle factor information
    for (factor_index, factor) in factor_graph.factors() {
        if let Some(obstacle_factor) = factor.kind.try_as_obstacle_ref() {
            // Find the variable this factor is connected to
            if let Some(mut neighbors) = factor_graph.factor_neighbours(FactorIndex(factor_index)) {
                // Get the first (and only) variable connected to this factor
                if let Some(variable) = neighbors.next() {
                    // Find the index of this variable in our variables list
                    let variable_index = variable_index_map.get(&variable.node_index().index()).copied().unwrap_or(0);
                    
                    let last_measurement = obstacle_factor.last_measurement();
                    
                    factor_details.obstacle_factors.push(ObstacleFactorInfo {
                        variable_index,
                        sdf_value: last_measurement.value,
                        position: [last_measurement.pos.x, last_measurement.pos.y],
                    });
                }
            }
        }
    }
    
    // Extract inter-robot factor information
    info!("IM HERE START EXTRACTING INTERROBOT FACTORS");
    let mut counter = 0;
    for (factor_index, factor) in factor_graph.factors() {
        if let Some(interrobot_factor) = factor.kind.try_as_inter_robot_ref() {
            info!("IM HERE IN INTERROBOT FACTORS {}", counter);
            counter += 1;
            let factor_node = factor_graph.get_factor(FactorIndex(factor_index)).unwrap();
            
            let skip = interrobot_factor.skip(&factor_node.state);
            let dist_xy = interrobot_factor.diff_between_estimated_positions(&factor_node.state.linearisation_point);
            let dist = dist_xy.euclidean_norm();
            // Find the variable this factor is connected to
            if let Some(mut neighbors) = factor_graph.factor_neighbours(FactorIndex(factor_index)) {
                // Get the first (and only) variable connected to this factor
                if let Some(variable) = neighbors.next() {
                    // Find the index of this variable in our variables list
                    let variable_index = variable_index_map.get(&variable.node_index().index()).copied().unwrap_or(0);
                    
                    // Get the safety distance
                    let safety_distance = interrobot_factor.safety_distance() as f32;

                    // Get the external factor graph ID and convert to u32
                    let external_id: u32 = interrobot_factor.external_variable.factorgraph_id.index();
                    
                    // interrobot_factor.skip is a bool, but we are not sure when this is set, but for now we are using it.
                    factor_details.interrobot_factors.push(InterRobotFactorInfo {
                        variable_index, // Index of the variable in our variables list
                        external_robot_id: external_id as u32, // the same as robot id
                        external_factorgraph_id: external_id as u32, // the same as robot id
                        external_variable_index: interrobot_factor.external_variable.variable_index.into(),
                        safety_distance, // Safety distance for factor to be active
                        distance_between_variables: dist, // Distance between the two variables
                        active: !skip,  // Factor is active if skip is false
                    });
                }
            }
        }
    }
    info!("IM HERE DONE EXTRACTING INTERROBOT FACTORS");
    
    // Extract tracking factor information
    for (factor_index, factor) in factor_graph.factors() {
        if let Some(tracking_factor) = factor.kind.try_as_tracking_ref() {
            // Find the variable this factor is connected to
            if let Some(mut neighbors) = factor_graph.factor_neighbours(FactorIndex(factor_index)) {
                // Get the first (and only) variable connected to this factor
                if let Some(variable) = neighbors.next() {
                    // Find the index of this variable in our variables list
                    let variable_index = variable_index_map.get(&variable.node_index().index()).copied().unwrap_or(0);
                    
                    let tracking = tracking_factor.tracking();
                    let last_measurement = tracking_factor.last_measurement();
                    
                    // Convert tracking path to the expected format using the helper method
                    let tracking_path = tracking.get_path()
                        .map(|path| path.iter().map(|v| [v.x, v.y]).collect())
                        .unwrap_or_default();
                    
                    // In the measure method of TrackingFactor, x_to_projection_distance is calculated
                    // but not stored in LastMeasurement. We need to recalculate it here.
                    let x_pos = variable.estimated_position();
                    let projected_pos = [last_measurement.pos.x as f64, last_measurement.pos.y as f64];
                    let dx = x_pos[0] - projected_pos[0];
                    let dy = x_pos[1] - projected_pos[1];
                    let distance_to_path = (dx * dx + dy * dy).sqrt();
                    
                    factor_details.tracking_factors.push(TrackingFactorInfo {
                        variable_index,
                        tracking_path,
                        tracking_index: tracking.get_index(),
                        projected_position: [last_measurement.pos.x, last_measurement.pos.y],
                        path_deviation: last_measurement.value as f32,
                        distance_to_path,
                    });
                }
            }
        }
    }
    
    // Extract dynamic factor information
    // Dynamic factors connect two variables, so we need to find both
    for (factor_index, factor) in factor_graph.factors() {
        if let Some(_dynamic_factor) = factor.kind.try_as_dynamic_ref() {
            // Find the variables this factor is connected to
            if let Some(neighbors) = factor_graph.factor_neighbours(FactorIndex(factor_index)) {
                let variables: Vec<_> = neighbors.collect();
                
                if variables.len() == 2 {
                    // Find the indices of these variables in our variables list
                    let from_variable_index = variable_index_map.get(&variables[0].node_index().index()).copied().unwrap_or(0);
                    let to_variable_index = variable_index_map.get(&variables[1].node_index().index()).copied().unwrap_or(0);
                    
                    // Use a default delta_t value
                    let delta_t = _dynamic_factor.delta_t as f32;
                    
                    factor_details.dynamic_factors.push(DynamicFactorInfo {
                        from_variable_index,
                        to_variable_index,
                        delta_t,
                    });
                }
            }
        }
    }
    
    factor_details
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
