use bevy::{
    ecs::{entity::Entity, system::Commands},
    math::Vec2,
    tasks::AsyncComputeTaskPool,
};
use bevy_prng::WyRand;
use gbp_config::RRTSection;
use rand::{RngCore, SeedableRng};

use crate::{Colliders, CollisionProblem, Path, PathfindingError, PathfindingTask};

// Add a constant for goal bias percentage
const GOAL_BIAS_PERCENTAGE: f64 = 0.1; // 10% chance to sample the goal directly

/// Standalone function to spawn an async task for pathfinding
/// - Used to run path-finding tasks that may take longer than a single frame to
///   complete
pub fn spawn_pathfinding_task(
    commands: &mut Commands,
    start: Vec2,
    end: Vec2,
    // smooth: bool,
    rrt_params: RRTSection,
    colliders: Colliders,
    task_target: Entity,
    rng_source: Option<Box<dyn RngCore + Send>>,
) {
    let mut rng_source: Box<dyn RngCore + Send> = match rng_source {
        Some(rng) => rng,
        None => Box::new(WyRand::from_entropy()),
    };

    let collision_solver =
        CollisionProblem::new(colliders).with_collision_radius(rrt_params.collision_radius.get());

    let task_pool = AsyncComputeTaskPool::get();

    let task = task_pool.spawn(async move {
        let start = [start.x as f64, start.y as f64];
        let end = [end.x as f64, end.y as f64];

        rrt::rrtstar::rrtstar(
            &start,
            &end,
            |x: &[f64]| collision_solver.is_feasible(x),
            // || collision_solver.random_sample(&mut *rng_source.lock().unwrap()),
            || collision_solver.random_sample(&mut *rng_source),
            rrt_params.step_size.get() as f64,
            rrt_params.max_iterations.get(),
            rrt_params.neighbourhood_radius.get() as f64,
            true,
        )
        .map(|res| {
            if let Some(goal_index) = res.goal_index {
                let resulting_path = {
                    let mut resulting_path = std::iter::once(vec![end[0], end[1]])
                        .chain(res.get_until_root(goal_index).into_iter())
                        .collect::<Vec<_>>();
                    if rrt_params.smoothing.enabled {
                        rrt::rrtstar::smooth_path(
                            &mut resulting_path,
                            |x| collision_solver.is_feasible(x),
                            rrt_params.step_size.get() as f64,
                            rrt_params.smoothing.max_iterations.get(),
                            &mut *rng_source,
                        );
                    }
                    resulting_path
                };

                Path(
                    resulting_path
                        .into_iter()
                        .rev()
                        .map(|v| Vec2::new(v[0] as f32, v[1] as f32))
                        .collect::<Vec<_>>(),
                )
            } else {
                Path(vec![])
            }
        })
        .map_err(|_| PathfindingError::ReachedMaxIterations)
    });

    commands.entity(task_target).insert(PathfindingTask(task));
}

/// Standalone function to spawn an async task for pathfinding with direct goal checking
/// - This version attempts to connect directly to the goal from each new node
/// - It also uses goal biasing to sample the goal directly with a certain probability
pub fn spawn_pathfinding_task_with_direct_goal_check(
    commands: &mut Commands,
    start: Vec2,
    end: Vec2,
    rrt_params: RRTSection,
    colliders: Colliders,
    task_target: Entity,
    rng_source: Option<Box<dyn RngCore + Send>>,
) {
    let mut rng_source: Box<dyn RngCore + Send> = match rng_source {
        Some(rng) => rng,
        None => Box::new(WyRand::from_entropy()),
    };

    let collision_solver =
        CollisionProblem::new(colliders).with_collision_radius(rrt_params.collision_radius.get());

    let task_pool = AsyncComputeTaskPool::get();

    let task = task_pool.spawn(async move {
        let start = [start.x as f64, start.y as f64];
        let end = [end.x as f64, end.y as f64];

        // Function to check if a direct path to goal is possible
        let can_reach_goal_directly = |point: &[f64]| -> bool {
            let is_feasible = |x: &[f64]| collision_solver.is_feasible(x);
            
            // Check if we can connect directly to the goal
            // We do this by checking a series of points along the line from point to goal
            let dx = end[0] - point[0];
            let dy = end[1] - point[1];
            let dist = (dx * dx + dy * dy).sqrt();
            
            // If we're already very close to the goal, we can reach it directly
            if dist < rrt_params.step_size.get() as f64 {
                return true;
            }
            
            // Check points along the line to the goal
            let steps = (dist / (rrt_params.step_size.get() as f64)).ceil() as usize;
            for i in 1..=steps {
                let t = i as f64 / steps as f64;
                let check_point = [
                    point[0] + dx * t,
                    point[1] + dy * t,
                ];
                
                if !is_feasible(&check_point) {
                    return false; // Path to goal is blocked
                }
            }
            
            // If we get here, we can reach the goal directly
            true
        };
        
        // First, check if we can reach the goal directly from the start
        if can_reach_goal_directly(&start) {
            // If we can reach the goal directly, return a path with just the start and end points
            return Ok(Path(vec![
                Vec2::new(start[0] as f32, start[1] as f32),
                Vec2::new(end[0] as f32, end[1] as f32),
            ]));
        }
        
        // Create a custom sampling function that occasionally samples the goal directly
        let goal_biased_sampler = || {
            // With GOAL_BIAS_PERCENTAGE probability, return the goal
            if rand::random::<f64>() < GOAL_BIAS_PERCENTAGE {
                return vec![end[0], end[1]];
            }
            // Otherwise, return a random sample
            collision_solver.random_sample(&mut *rng_source)
        };

        // Use the standard RRT* algorithm with goal biasing
        rrt::rrtstar::rrtstar(
            &start,
            &end,
            |x: &[f64]| collision_solver.is_feasible(x),
            goal_biased_sampler,
            rrt_params.step_size.get() as f64,
            rrt_params.max_iterations.get(),
            rrt_params.neighbourhood_radius.get() as f64,
            true,
        )
        .map(|res| {
            if let Some(goal_index) = res.goal_index {
                let resulting_path = {
                    let mut resulting_path = std::iter::once(vec![end[0], end[1]])
                        .chain(res.get_until_root(goal_index).into_iter())
                        .collect::<Vec<_>>();
                    
                    // Check if we can reach the goal directly from any point in the path
                    // Start from the second point (after the goal) and work backwards
                    for i in 1..resulting_path.len() {
                        if can_reach_goal_directly(&resulting_path[i]) {
                            // If we can reach the goal directly from this point, truncate the path
                            resulting_path.truncate(i + 1);
                            break;
                        }
                    }
                    
                    if rrt_params.smoothing.enabled {
                        rrt::rrtstar::smooth_path(
                            &mut resulting_path,
                            |x| collision_solver.is_feasible(x),
                            rrt_params.step_size.get() as f64,
                            rrt_params.smoothing.max_iterations.get(),
                            &mut *rng_source,
                        );
                    }
                    resulting_path
                };

                Path(
                    resulting_path
                        .into_iter()
                        .rev()
                        .map(|v| Vec2::new(v[0] as f32, v[1] as f32))
                        .collect::<Vec<_>>(),
                )
            } else {
                Path(vec![])
            }
        })
        .map_err(|_| PathfindingError::ReachedMaxIterations)
    });

    commands.entity(task_target).insert(PathfindingTask(task));
}

/// Standalone function to spawn an async task for pathfinding
/// - Used to run path-finding tasks that may take longer than a single frame to
///   complete
pub fn spawn_pathfinding_task_full_tree(
    commands: &mut Commands,
    start: Vec2,
    end: Vec2,
    // smooth: bool,
    rrt_params: RRTSection,
    colliders: Colliders,
    task_target: Entity,
    rng_source: Option<Box<dyn RngCore + Send>>,
) {
    let mut rng_source: Box<dyn RngCore + Send> = match rng_source {
        Some(rng) => rng,
        None => Box::new(WyRand::from_entropy()),
    };

    let collision_solver =
        CollisionProblem::new(colliders).with_collision_radius(rrt_params.collision_radius.get());

    let task_pool = AsyncComputeTaskPool::get();

    let task = task_pool.spawn(async move {
        let start = [start.x as f64, start.y as f64];
        let end = [end.x as f64, end.y as f64];

        rrt::rrtstar::rrtstar(
            &start,
            &end,
            |x: &[f64]| collision_solver.is_feasible(x),
            // || collision_solver.random_sample(&mut *rng_source.lock().unwrap()),
            || collision_solver.random_sample(&mut *rng_source),
            rrt_params.step_size.get() as f64,
            rrt_params.max_iterations.get(),
            rrt_params.neighbourhood_radius.get() as f64,
            true,
        )
        .map(|res| {
            if let Some(goal_index) = res.goal_index {
                let resulting_path = {
                    let mut resulting_path = std::iter::once(vec![end[0], end[1]])
                        .chain(res.get_until_root(goal_index).into_iter())
                        .collect::<Vec<_>>();
                    if rrt_params.smoothing.enabled {
                        rrt::rrtstar::smooth_path(
                            &mut resulting_path,
                            |x| collision_solver.is_feasible(x),
                            rrt_params.step_size.get() as f64,
                            rrt_params.smoothing.max_iterations.get(),
                            &mut *rng_source,
                        );
                    }
                    resulting_path
                };

                Path(
                    resulting_path
                        .into_iter()
                        .map(|v| Vec2::new(v[0] as f32, v[1] as f32))
                        .collect::<Vec<_>>(),
                )
            } else {
                Path(vec![])
            }
        })
        .map_err(|_| PathfindingError::ReachedMaxIterations)
    });

    commands.entity(task_target).insert(PathfindingTask(task));
}
