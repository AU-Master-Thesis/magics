//! Module for extracting state information from the simulation.

use bevy::prelude::*;
use crate::factorgraph::factorgraph::FactorGraph;
use crate::planner::robot::{Mission, RadioAntenna, Radius, RobotConnections};
use crate::environment::ObstacleMarker;
use crate::planner::collisions::resources::{RobotRobotCollisions, RobotEnvironmentCollisions};
use gbp_config::{Config, formation};
use super::state::{
    AgentState, ApiState, CollisionInfo, EnvironmentState, FactorCounts, FactorDetails, FactorGraphState, 
    FactorWeights, MessageStats, MissionProgress, MissionState, PlanningStrategy, 
    StateVectorInfo,
};
use super::plugin::PreviousCollisionCounts;

/// System that extracts the state of the simulation for the API.
pub fn extract_state(
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
                super::factor_details::extract_factor_details(factor_graph)
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
