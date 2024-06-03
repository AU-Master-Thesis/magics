use bevy::{
    ecs::{entity::Entity, system::Commands},
    math::Vec2,
    tasks::AsyncComputeTaskPool,
};
use bevy_prng::WyRand;
use gbp_config::RRTSection;
use rand::{RngCore, SeedableRng};

use crate::{Colliders, CollisionProblem, Path, PathfindingError, PathfindingTask};

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
