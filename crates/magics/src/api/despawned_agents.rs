//! Module for tracking agents that are despawned during an API step.
//!
//! This module provides functionality to track agents that are despawned during
//! an API step, ensuring that their final state is included in the API response.

use bevy::prelude::*;
use std::collections::HashMap;

use crate::{
    factorgraph::factorgraph::FactorGraph,
    planner::robot::{Mission, RadioAntenna, Radius, RobotConnections, RobotFinishedRoute},
    planner::collisions::resources::{RobotRobotCollisions, RobotEnvironmentCollisions},
};
use gbp_config::{Config, formation};

use super::{
    state::{AgentState, MissionState},
    plugin::PreviousCollisionCounts,
    ApiState,
    state_utils,
};

/// Resource to track agents that despawned during an API step
#[derive(Resource, Default)]
pub struct DespawnedAgentsTracker {
    /// Map of entity IDs to their final state before despawning
    pub despawned_agents: HashMap<Entity, AgentState>,
}

/// System to track robots that finish their route and will be despawned
pub fn track_robots_about_to_despawn(
    mut despawned_agents: ResMut<DespawnedAgentsTracker>,
    mut evr_robot_finished_route: EventReader<RobotFinishedRoute>,
    api_state: Res<ApiState>,
    config: Res<Config>,
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
    robot_robot_collisions: Res<RobotRobotCollisions>,
    robot_environment_collisions: Res<RobotEnvironmentCollisions>,
    previous_collision_counts: Res<PreviousCollisionCounts>,
) {
    // Only track if API is active and a step is in progress
    if !api_state.is_active() || !api_state.is_step_requested() {
        return;
    }

    // Process robots that finished their route (they will be despawned soon if config says so)
    for RobotFinishedRoute(entity) in evr_robot_finished_route.read() {
        // Only track if the robot will actually be despawned
        if !config.simulation.despawn_robot_when_final_waypoint_reached {
            continue;
        }

        if let Ok((entity, transform, factor_graph, connections, mission_opt, planning_strategy_opt, radius_opt, antenna_opt)) = robots.get(*entity) {
            // Create an AgentState with the final information
            let mut agent_state = state_utils::create_agent_state(
                entity,
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
            
            // Ensure mission state is set to Completed
            agent_state.mission_state = MissionState::Completed;
            
            // Store the agent's final state
            despawned_agents.despawned_agents.insert(entity, agent_state);
            
            info!("Tracked robot {:?} that finished its route and will be despawned", entity);
        }
    }
}

/// System to track entities with despawn timers
pub fn track_entities_with_despawn_timer(
    mut despawned_agents: ResMut<DespawnedAgentsTracker>,
    api_state: Res<ApiState>,
    config: Res<Config>,
    despawn_timers: Query<&crate::despawn_entity_after::components::DespawnEntityAfter<Virtual>>,
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
    robot_robot_collisions: Res<RobotRobotCollisions>,
    robot_environment_collisions: Res<RobotEnvironmentCollisions>,
    previous_collision_counts: Res<PreviousCollisionCounts>,
) {
    // Only track if API is active and a step is in progress
    if !api_state.is_active() || !api_state.is_step_requested() {
        return;
    }

    // Find all entities that have a despawn timer attached
    for despawn_timer in &despawn_timers {
        let entity = despawn_timer.entity_to_despawn;
        
        // Skip if we already have this entity
        if despawned_agents.despawned_agents.contains_key(&entity) {
            continue;
        }

        // Check if this is a robot entity
        if let Ok((entity, transform, factor_graph, connections, mission_opt, planning_strategy_opt, radius_opt, antenna_opt)) = robots.get(entity) {
            // Create an AgentState with the final information
            let agent_state = state_utils::create_agent_state(
                entity,
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
            
            // Store the agent's final state
            despawned_agents.despawned_agents.insert(entity, agent_state);
            
            info!("Tracked robot {:?} with despawn timer", entity);
        }
    }
}

/// System to clear the despawned agents tracker after each API step
pub fn clear_despawned_agents_after_step(
    mut despawned_agents: ResMut<DespawnedAgentsTracker>,
    api_state: Res<ApiState>,
) {
    if api_state.is_step_completed() {
        if !despawned_agents.despawned_agents.is_empty() {
            info!(
                "Clearing {} despawned agents from tracker after API step completion",
                despawned_agents.despawned_agents.len()
            );
            despawned_agents.despawned_agents.clear();
        }
    }
}
