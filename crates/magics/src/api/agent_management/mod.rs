//! Agent management functionality for the API.
//!
//! This module provides functionality for spawning and removing agents.

use bevy::prelude::*;
use bevy_rand::prelude::GlobalEntropy;
use gbp_config::{
    formation::{self, ReachedWhen},
    Config,
};
use gbp_environment::Environment;
use min_len_vec::{two_or_more, TwoOrMore};

use crate::{
    planner::{
        robot::{
            Mission, RadioAntenna, Radius, RobotConnections, RobotDespawned, RobotSpawned, StateVector,
        },
        spawner::WaypointCreated,
        visualiser::waypoints::{AssociatedWithRobot, WaypointVisualiser},
        visualiser::tracer::Traces,
    },
    simulation_loader::Sdf,
    theme::CatppuccinTheme,
    factorgraph::factorgraph::FactorGraph,
    api::plugin::PreviousCollisionCounts,
};

use super::state::{AgentState, ApiState, WeightUpdate};
use super::state_utils;

/// System to handle agent removal requests from the API
pub fn handle_agent_removal_requests(
    mut commands: Commands,
    api_state: Res<ApiState>,
    mut despawned_agents_tracker: ResMut<super::despawned_agents::DespawnedAgentsTracker>,
    config: Res<Config>,
    robot_robot_collisions: Res<crate::planner::collisions::resources::RobotRobotCollisions>,
    robot_environment_collisions: Res<crate::planner::collisions::resources::RobotEnvironmentCollisions>,
    previous_collision_counts: Res<PreviousCollisionCounts>,
    mut evw_robot_despawned: EventWriter<RobotDespawned>,
    mut traces: ResMut<Traces>, // Add Traces resource
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
                let final_state = state_utils::create_agent_state(
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

/// System to handle agent spawn requests from the API
pub fn handle_agent_spawn_requests(
    mut commands: Commands,
    api_state: Res<ApiState>,
    config: Res<Config>,
    env_config: Res<Environment>,
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
        let waypoints = min_len_vec::two_or_more![initial_state_vec, goal_state_vec];

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
                agent_id: Some(new_entity.index()),
                weights: custom_weights,
            };
            api_state.add_weight_update(update);
            info!("API: Queued custom weights for spawned agent {:?}", new_entity);
        }
    }
}
