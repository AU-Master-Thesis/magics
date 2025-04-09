//! Utility functions for state extraction and manipulation.
//!
//! This module provides shared functionality for creating and manipulating
//! state structures used by the API.

use bevy::prelude::*;
use crate::factorgraph::factorgraph::FactorGraph;
use crate::planner::robot::{Mission, RadioAntenna, Radius, RobotConnections};
use crate::planner::collisions::resources::{RobotRobotCollisions, RobotEnvironmentCollisions};
use gbp_config::{Config, formation};

use super::state::{
    AgentState, CollisionInfo, FactorCounts, FactorDetails, FactorGraphState, 
    FactorWeights, MessageStats, MissionProgress, MissionState, PlanningStrategy, 
    StateVectorInfo,
};
use super::plugin::PreviousCollisionCounts;
use super::factor_details;
use bevy::log::debug;
/// Create an agent state from components.
///
/// This function extracts all the necessary information from the provided components
/// to create a complete agent state for API consumption.
pub fn create_agent_state(
    entity: Entity,
    transform: &Transform,
    factor_graph: &FactorGraph,
    connections: &RobotConnections,
    mission_opt: Option<&Mission>,
    planning_strategy_opt: Option<&formation::PlanningStrategy>,
    radius_opt: Option<&Radius>,
    antenna_opt: Option<&RadioAntenna>,
    robot_robot_collisions: &RobotRobotCollisions,
    robot_environment_collisions: &RobotEnvironmentCollisions,
    previous_collision_counts: &PreviousCollisionCounts,
    config: &Config,
    extract_factor_details: bool,
) -> AgentState {
    // Extract position and velocity from factor graph
    let (_, current_variable) = factor_graph
        .first_variable()
        .expect("factorgraph should have >= 2 variables");
    let [px, py] = current_variable.estimated_position();
    let [vx, vy] = current_variable.estimated_velocity();
    
    let position = Vec2::new(px as f32, py as f32);
    let velocity = Vec2::new(vx as f32, vy as f32);

    // Extract factor graph state
    let mut factor_graph_state = FactorGraphState::default();
    
    // BUG FIX: Use the actual weights from the factor graph instead of the config
    factor_graph_state.weights = *factor_graph.factor_weights();
    
    // Log the weights for debugging
    debug!("create_agent_state: Agent state extraction for entity {:?}: Using weights from factor graph: {:?}", 
          entity, factor_graph.factor_weights());
    debug!("create_agent_state: Config weights for comparison: dynamic={}, obstacle={}, interrobot={}, tracking={}",
          config.gbp.sigma_factor_dynamics, config.gbp.sigma_factor_obstacle,
          config.gbp.sigma_factor_interrobot, config.gbp.sigma_factor_tracking);
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
            
            // Construct the mission progress
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

    // Convert planning strategy
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
    
    // Extract factor details if requested
    let factor_details = if extract_factor_details {
        factor_details::extract_factor_details(factor_graph)
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
    
    // Create collision info
    let collision_info = CollisionInfo {
        robot_collisions_total,
        robot_collisions_delta,
        environment_collisions_total,
        environment_collisions_delta,
    };
    
    // Create complete agent state
    AgentState {
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
        target_square_id: mission_opt.and_then(|m| m.target_square_id.clone()), // Added this line
    }
}

/// Update previous collision counts for an entity.
///
/// This function updates the previous collision counts for an entity
/// to be used for calculating deltas in the next state extraction.
pub fn update_previous_collision_counts(
    entity: Entity,
    robot_robot_collisions: &RobotRobotCollisions,
    robot_environment_collisions: &RobotEnvironmentCollisions,
    previous_collision_counts: &mut PreviousCollisionCounts,
) {
    let robot_collisions_total = robot_robot_collisions.get(entity).unwrap_or(0);
    let environment_collisions_total = robot_environment_collisions.get(entity).unwrap_or(0);
    
    previous_collision_counts.robot_collisions.insert(entity, robot_collisions_total);
    previous_collision_counts.environment_collisions.insert(entity, environment_collisions_total);
}
