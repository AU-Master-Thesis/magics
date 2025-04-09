//! Replan functionality for the API.
//!
//! This module provides functionality for replanning completed agents.

use bevy::prelude::*;
use bevy_rand::prelude::GlobalEntropy;
use gbp_config::{
    formation::{self, FormationGroup, SerializedSquare},
    geometry::Shape,
    Config,
};
use gbp_linalg::VectorNorm;
use min_len_vec::{two_or_more, TwoOrMore};
use rand::{seq::IteratorRandom, Rng};

use crate::{
    planner::robot::{Mission, MissionState as RobotMissionState, Route, StateVector, FinishedPath},
    simulation_loader::{Sdf, SimulationManager},
};

use super::state::{ApiState, ReplanRequestParams};

/// System to handle replan requests from the API
pub fn handle_replan_requests(
    mut commands: Commands,
    api_state: Res<ApiState>,
    config: Res<Config>,
    env_config: Res<gbp_environment::Environment>,
    sdf: Res<Sdf>,
    time: Res<Time>,
    simulation_manager: Res<SimulationManager>,
    // Added &mut crate::planner::robot::FinishedPath to the query
    // Corrected query tuple access
    mut query: Query<(Entity, &Transform, &mut Mission, &formation::PlanningStrategy, &mut crate::factorgraph::factorgraph::FactorGraph, &mut FinishedPath)>,
    mut prng: ResMut<GlobalEntropy<bevy_prng::WyRand>>, // For random selection
) {
    if let Some(replan_params) = api_state.take_replan_request() {
        debug!("API: Processing replan request with params: {:?}", replan_params);

        // Get available squares from the simulation manager
        let available_squares = if let Some(formation_group) = simulation_manager.active_formation_group() {
            let squares = extract_available_squares(formation_group);
            // Update the available squares in the API state for ZMQ server access
            api_state.update_available_squares(squares.clone());
            debug!("API Replan: Using formation group from simulation manager with {} squares", squares.len());
            squares
        } else {
            error!("API Replan: Failed to get active formation group from simulation manager");
            Vec::new()
        };

        // Calculate world size directly from env_config, similar to formation.rs
        let tile_size = env_config.tiles.settings.tile_size as f64;
        let width = tile_size * env_config.tiles.grid.ncols() as f64;
        let height = tile_size * env_config.tiles.grid.nrows() as f64;
        let world_size = Some((width, height));
        
        debug!("API Replan: Using world size: width={}, height={}", width, height);


        // Iterate mutably and destructure the tuple correctly
        for (entity, transform, mut mission, planning_strategy, mut factorgraph, mut finished_path) in query.iter_mut() {
            if mission.state == RobotMissionState::Completed {
                debug!("API Replan: Replanning agent {:?}", entity);

                let current_pos = transform.translation.xz();
                let current_vel = Vec2::ZERO; // Assume zero velocity when starting new route? Or use last known?
                let current_state_vec = StateVector::new(Vec4::new(current_pos.x, current_pos.y, current_vel.x, current_vel.y));

                // Generate new goal position
                let new_goal_data = generate_new_position(
                    &replan_params.strategy,
                    replan_params.square_id.as_deref(),
                    replan_params.avoid_current_square,
                    mission.target_square_id.as_deref(),
                    &available_squares,
                    &sdf,
                    world_size, // Pass world_size Option
                    &mut *prng,
                );


                if let Some((new_goal_pos, selected_square_id)) = new_goal_data {
                    info!("API Replan: New goal for {:?}: {:?}, Square: {:?}", entity, new_goal_pos, selected_square_id);

                    let new_goal_state_vec = StateVector::new(Vec4::new(new_goal_pos.x, new_goal_pos.y, 0.0, 0.0)); // Zero velocity at goal

                    // Update Mission - Clear old routes and create a new one
                    mission.taskpoints.clear(); // Clear old taskpoints
                    mission.taskpoints.push(current_state_vec); // Add current position as first taskpoint
                    mission.taskpoints.push(new_goal_state_vec); // Add new goal as second taskpoint
                    
                    // Clear old routes
                    mission.routes.clear();
                    
                    // Create a new route
                    let new_route = Route::new(
                        two_or_more![current_state_vec, new_goal_state_vec],
                        time.elapsed_seconds_f64(),
                    );
                    mission.routes.push(new_route);
                    mission.active_route = 0; // Set the active route to the only route (index 0)
                    mission.state = match planning_strategy {
                        formation::PlanningStrategy::OnlyLocal => RobotMissionState::Active,
                        formation::PlanningStrategy::RrtStar => RobotMissionState::Idle { waiting_for_waypoints: false },
                    };
                    mission.finished_at = None;
                    mission.target_square_id = selected_square_id.clone(); // Update target square

                    // Reset FinishedPath flag using the component from the query tuple
                    finished_path.0 = false;


                    // Update FactorGraph (Reset variables and tracking factors)
                    // Access factorgraph directly from the query tuple
                         // Simplified reset for now: reset variables towards the new goal
                         // A more sophisticated approach might be needed, especially for RRT*
                         let num_vars = factorgraph.node_count().variables; // Call on factorgraph component
                         let start = current_state_vec.0; // Bevy Vec4
                         let goal = new_goal_state_vec.0;  // Bevy Vec4
                         let dir = (goal - start).normalize_or_zero();
                         let target_speed_val = config.robot.target_speed.get();
                         // Correct multiplication with StrictlyPositiveFinite
                         let horizon_dist = (config.robot.planning_horizon.get() * target_speed_val);
                         let segment_dist = start.distance(goal);
                         let travel_dist = horizon_dist.min(segment_dist * 0.9); // Move towards goal, but not all the way instantly
                         let horizon_point = start + dir * travel_dist;
                         let vel_at_horizon = dir * target_speed_val; // Target speed in goal direction

                         let means: Vec<_> = (0..num_vars)
                             .map(|i| i as f32 / (num_vars - 1).max(1) as f32) // Avoid division by zero if num_vars is 1
                             .map(|t| {
                                 let pos = start.lerp(horizon_point, t).xy();
                                 // Interpolate velocity? Or just use target speed? Using target speed for now.
                                 // let vel = start.zw().lerp(vel_at_horizon.zw(), t);
                                 let vel = vel_at_horizon.zw(); // Use target velocity directly
                                 // Create a fixed-size array [f64; 4] directly
                                 [pos.x as f64, pos.y as f64, vel.x as f64, vel.y as f64]
                             })
                             .collect();

                         // Use qualified path for Float::INFINITY and call method on factorgraph component
                         // Now `means` is Vec<[f64; 4]>, which can be sliced as &[[f64; 4]]
                         factorgraph.reset_variables(&means, 1e30, gbp_linalg::Float::INFINITY);
                         factorgraph.modify_tracking_factors(|tracking| { // Call on factorgraph component
                             let waypoints = vec![current_pos, new_goal_pos];
                             if let Ok(two_or_more) = TwoOrMore::try_from(waypoints) {
                                tracking.set_tracking_path(two_or_more);
                                tracking.set_tracking_index(1); // Start tracking towards the new goal
                                tracking.reset_tracking_record(); // Use the new public method
                             } else {
                                 error!("API Replan: Failed to create TwoOrMore for tracking path update");
                             }
                         });
                         info!("API Replan: Reset FactorGraph variables and tracking for {:?}", entity);


                    // Update ApiState (optional, if needed elsewhere)
                    if let Ok(mut agent_states) = api_state.agent_states.write() {
                        if let Some(agent_state) = agent_states.get_mut(&entity) {
                            agent_state.target_square_id = selected_square_id;
                            // Update other relevant fields if necessary
                        }
                    }

                } else {
                    warn!("API Replan: Failed to generate a new valid goal for agent {:?}", entity);
                    // Optionally, keep the agent completed or try again later?
                    // For now, it remains completed.
                }
            }
        }

        // Mark replan as completed
        api_state.complete_replan();
        info!("API: Replan request processing completed");
    }
}

/// Generates a new valid goal position based on the specified strategy.
pub fn generate_new_position(
    strategy: &str,
    requested_square_id: Option<&str>,
    avoid_current_square: bool,
    current_square_id: Option<&str>,
    available_squares: &[SerializedSquare],
    sdf: &Sdf,
    world_size: Option<(f64, f64)>, // Made Option
    rng: &mut impl Rng,
) -> Option<(Vec2, Option<String>)> {
    // Returns position and selected square_id
    const MAX_ATTEMPTS: usize = 100; // Max attempts to find a valid point

    info!("Generating new position with strategy: {}, requested_square_id: {:?}, avoid_current: {}, current_square_id: {:?}, available squares: {}",
          strategy, requested_square_id, avoid_current_square, current_square_id, available_squares.len());
    
    // Log available squares for debugging
    for (i, square) in available_squares.iter().enumerate() {
        info!("Available square {}: id={}, type={}, min=[{}, {}], max=[{}, {}]",
              i, square.id, square.square_type, square.min[0], square.min[1], square.max[0], square.max[1]);
    }

    match strategy {
        "CompleteRandom" => {
            let (world_width, world_height) =
                world_size.expect("World size required for CompleteRandom strategy");
            let min_x = -world_width as f32 / 2.0;
            let max_x = world_width as f32 / 2.0;
            let min_y = -world_height as f32 / 2.0;
            let max_y = world_height as f32 / 2.0;

            info!("CompleteRandom strategy: world bounds min_x={}, max_x={}, min_y={}, max_y={}", 
                  min_x, max_x, min_y, max_y);

            for attempt in 0..MAX_ATTEMPTS {
                let x = rng.gen_range(min_x..max_x);
                let y = rng.gen_range(min_y..max_y);
                let point = Vec2::new(x, y);
                
                info!("CompleteRandom attempt {}: Generated point ({}, {})", attempt, x, y);
                
                // Removed SDF check for now, relying on GBP obstacle factors
                // if sdf.value(point) >= 0.0 {
                return Some((point, None)); // No square ID for complete random
                // }
            }
            warn!(
                "CompleteRandom strategy failed to find a valid point after {} attempts",
                MAX_ATTEMPTS
            );
            return None;
        }
        "RandomSquares" => {
            if available_squares.is_empty() {
                warn!("RandomSquares strategy requested but no squares are available.");
                return None;
            }

            // Filter to only include GoalPosition squares
            let goal_squares: Vec<&SerializedSquare> = available_squares
                .iter()
                .filter(|s| s.square_type == "GoalPosition")
                .collect();
                
            if goal_squares.is_empty() {
                warn!("No GoalPosition squares available for replanning");
                return None;
            }
            
            let selected_square = if let Some(id) = requested_square_id {
                let square = goal_squares.iter().find(|s| s.id == id).copied();
                if let Some(sq) = square {
                    info!("Using requested square: id={}, min=[{}, {}], max=[{}, {}]", 
                          sq.id, sq.min[0], sq.min[1], sq.max[0], sq.max[1]);
                } else {
                    warn!("Requested square with id '{}' not found or not a GoalPosition square", id);
                }
                square
            } else {
                // Use only goal squares for selection
                let square = select_new_square(
                    current_square_id,
                    available_squares,
                    avoid_current_square,
                    rng,
                );
                if let Some(sq) = square {
                    info!("Selected new square: id={}, min=[{}, {}], max=[{}, {}]", 
                          sq.id, sq.min[0], sq.min[1], sq.max[0], sq.max[1]);
                } else {
                    warn!("Failed to select a new square");
                }
                square
            };

            if let Some(square) = selected_square {
                let min_x = square.min[0];
                let max_x = square.max[0];
                let min_y = square.min[1];
                let max_y = square.max[1];

                info!("Generating random point in square: min_x={}, max_x={}, min_y={}, max_y={}", 
                      min_x, max_x, min_y, max_y);

                // Scale the normalized coordinates to world coordinates
                // The coordinates in the SerializedSquare are normalized (0-1)
                // We need to scale them to world coordinates using the same method as in formation.rs
                let world_width = world_size.map_or(100.0, |(w, _)| w as f32);
                let world_height = world_size.map_or(100.0, |(_, h)| h as f32);
                info!("World size: width={}, height={}", world_width, world_height);
                
                // Create a WorldDimensions struct to use the same scaling method as formation.rs
                let world_dims = gbp_config::formation::WorldDimensions::new(world_width as f64, world_height as f64);
                
                // Convert points using the same method as in formation.rs
                let p1 = gbp_config::geometry::Point::new(min_x as f64, min_y as f64);
                let p2 = gbp_config::geometry::Point::new(max_x as f64, max_y as f64);
                
                let world_p1 = world_dims.point_to_world_position(p1);
                let world_p2 = world_dims.point_to_world_position(p2);
                
                // Determine min/max for the square bounds (same as in formation.rs)
                let scaled_min_x = world_p1.x.min(world_p2.x);
                let scaled_max_x = world_p1.x.max(world_p2.x);
                let scaled_min_y = world_p1.y.min(world_p2.y);
                let scaled_max_y = world_p1.y.max(world_p2.y);
                
                info!("Scaled square bounds: min_x={}, max_x={}, min_y={}, max_y={}", 
                      scaled_min_x, scaled_max_x, scaled_min_y, scaled_max_y);

                for attempt in 0..MAX_ATTEMPTS {
                    // Check if the ranges are valid (not empty)
                    if scaled_min_x >= scaled_max_x || scaled_min_y >= scaled_max_y {
                        warn!("Invalid square bounds: min_x={}, max_x={}, min_y={}, max_y={}", 
                              scaled_min_x, scaled_max_x, scaled_min_y, scaled_max_y);
                        
                        // Use the midpoint of the square as a fallback
                        let x = (scaled_min_x + scaled_max_x) / 2.0;
                        let y = (scaled_min_y + scaled_max_y) / 2.0;
                        let point = Vec2::new(x, y);
                        
                        info!("Using midpoint ({}, {}) due to invalid range", x, y);
                        return Some((point, Some(square.id.clone())));
                    }
                    
                    // Generate random position within the scaled square bounds
                    let x = rng.gen_range(scaled_min_x..scaled_max_x);
                    let y = rng.gen_range(scaled_min_y..scaled_max_y);
                    let point = Vec2::new(x, y);
                    
                    info!("RandomSquares attempt {}: Generated point ({}, {})", attempt, x, y);

                    // TODO: Add min_distance check if needed within the square?
                    // The formation placement already handles min_distance between *initial* points.
                    // Do we need it for *goals* within the same square? Assuming not for now.

                    // Removed SDF check for now, relying on GBP obstacle factors
                    // if sdf.value(point) >= 0.0 {
                    return Some((point, Some(square.id.clone())));
                    // }
                }
                warn!(
                    "RandomSquares strategy failed to find a valid point in square '{}' after {} attempts",
                    square.id, MAX_ATTEMPTS
                );
                return None; // Failed to find point in selected square
            } else {
                warn!(
                    "RandomSquares strategy failed: Could not select a suitable square (requested='{:?}', current='{:?}', avoid={})",
                    requested_square_id, current_square_id, avoid_current_square
                );
                return None; // No suitable square found
            }
        }
        _ => {
            error!("Unknown replan strategy: {}", strategy);
            return None;
        }
    }
}

/// Helper to select a square for the RandomSquares strategy.
pub fn select_new_square<'a>(
    current_square_id: Option<&str>,
    available_squares: &'a [SerializedSquare],
    avoid_current: bool,
    rng: &mut impl Rng,
) -> Option<&'a SerializedSquare> {
    if available_squares.is_empty() {
        return None;
    }

    // First, filter to only include GoalPosition squares
    let goal_squares: Vec<&'a SerializedSquare> = available_squares
        .iter()
        .filter(|s| s.square_type == "GoalPosition")
        .collect();
    
    if goal_squares.is_empty() {
        warn!("No GoalPosition squares available for replanning");
        return None;
    }
    
    // Then apply the avoid_current filter if needed
    let eligible_squares: Vec<&'a SerializedSquare> =
        if avoid_current && goal_squares.len() > 1 && current_square_id.is_some() {
            goal_squares
                .into_iter()
                .filter(|s| Some(s.id.as_str()) != current_square_id)
                .collect()
        } else {
            // If not avoiding, or only one square, or no current square, all goal squares are eligible
            goal_squares
        };

    if eligible_squares.is_empty() {
        // This can happen if avoid_current is true, there's only one square,
        // and it's the current one. Fall back to allowing the current square.
        if avoid_current
            && available_squares.len() == 1
            && Some(available_squares[0].id.as_str()) == current_square_id
        {
            warn!("Cannot avoid current square as it's the only one available. Selecting it anyway.");
            return Some(&available_squares[0]);
        }
        // Otherwise, no eligible squares found (e.g., available_squares was empty initially)
        return None;
    }

    // Choose a random square from the eligible ones using choose method from rand crate
    eligible_squares.into_iter().choose(rng)
}

/// Extracts available square definitions from the formation group configuration.
pub fn extract_available_squares(formation_group: &FormationGroup) -> Vec<SerializedSquare> {
    let mut squares = Vec::new();

    for (i, formation) in formation_group.formations.iter().enumerate() {
        // Extract initial position squares
        if let Shape::RandomSquare { p1, p2, min_distance } = &formation.initial_position.shape {
            squares.push(SerializedSquare {
                id: format!("initial_{}", i),
                square_type: "InitialPosition".to_string(),
                min: [p1.x as f32, p1.y as f32], // Cast to f32
                max: [p2.x as f32, p2.y as f32], // Cast to f32
                min_distance: Some(*min_distance),
            });
        }

        // Extract waypoint squares
        for (j, waypoint) in formation.waypoints.iter().enumerate() {
            if let Shape::RandomSquare { p1, p2, min_distance } = &waypoint.shape {
                squares.push(SerializedSquare {
                    id: format!("waypoint_{}_{}", i, j),
                    square_type: "Waypoint".to_string(),
                    min: [p1.x as f32, p1.y as f32], // Cast to f32
                    max: [p2.x as f32, p2.y as f32], // Cast to f32
                    min_distance: Some(*min_distance),
                });
            }
        }

        // Extract goal position squares
        if let Some(goal_positions) = &formation.goal_position {
            for (j, goal) in goal_positions.iter().enumerate() {
                if let Shape::RandomSquare { p1, p2, min_distance } = &goal.shape {
                    squares.push(SerializedSquare {
                        id: format!("goal_{}_{}", i, j),
                        square_type: "GoalPosition".to_string(),
                        min: [p1.x as f32, p1.y as f32], // Cast to f32
                        max: [p2.x as f32, p2.y as f32], // Cast to f32
                        min_distance: Some(*min_distance),
                    });
                }
            }
        }
    }

    squares
}
