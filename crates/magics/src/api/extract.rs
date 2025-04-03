//! Module for extracting state information from the simulation.

use bevy::prelude::*;
use crate::factorgraph::factorgraph::FactorGraph;
use crate::planner::robot::{Mission, RadioAntenna, Radius, RobotConnections};
use crate::environment::ObstacleMarker;
use crate::planner::collisions::resources::{RobotRobotCollisions, RobotEnvironmentCollisions};
use gbp_config::{Config, formation};
use super::state::{
    AgentState, ApiState, EnvironmentState,
};
use super::plugin::PreviousCollisionCounts;
use super::despawned_agents::DespawnedAgentsTracker;
use super::state_utils;

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
    despawned_agents: &mut DespawnedAgentsTracker, // Make mutable
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
            // Create agent state using the shared function
            let agent_state = state_utils::create_agent_state(
                entity,
                transform,
                factor_graph,
                connections,
                mission_opt,
                planning_strategy_opt,
                radius_opt,
                antenna_opt,
                robot_robot_collisions,
                robot_environment_collisions,
                previous_collision_counts,
                &config,
                api_state.is_active(), // Extract factor details if API is active
            );
            
            // Update previous counts for next time
            state_utils::update_previous_collision_counts(
                entity,
                robot_robot_collisions,
                robot_environment_collisions,
                previous_collision_counts,
            );
            
            agent_states.insert(entity, agent_state);
        }
        
        // Add states for agents that despawned during this step
        for (entity, agent_state) in &despawned_agents.despawned_agents {
            agent_states.insert(*entity, agent_state.clone());
        }
        
        if !despawned_agents.despawned_agents.is_empty() {
            info!(
                "Extracted state for {} active agents and {} despawned agents",
                agent_states.len() - despawned_agents.despawned_agents.len(),
                despawned_agents.despawned_agents.len()
            );
        }

        // Clear the despawned agents tracker after extraction
        if !despawned_agents.despawned_agents.is_empty() {
            info!(
                "Clearing {} despawned agents from tracker after API step completion",
                despawned_agents.despawned_agents.len()
            );
            despawned_agents.despawned_agents.clear();
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
