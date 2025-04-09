use bevy::{prelude::*, tasks::futures_lite::future};
use bevy_rand::component::EntropyComponent;
use bevy_prng::WyRand;
use gbp_config::{formation::{CheckIntersectionWith, IntersectionDistance, PlanningStrategy, ReachedWhen}, Config};
use gbp_global_planner::PathfindingTask;
use gbp_linalg::prelude::*;
use itertools::Itertools;
use ndarray::{array, concatenate, s, Axis};
use std::{num::NonZeroUsize, time::Duration};

use crate::{
    export::events::TakeSnapshotOfRobot,
    factorgraph::factorgraph::FactorGraph,
    simulation_loader::{LoadSimulation, ReloadSimulation},
};

use super::{
    bundle::{FinishedPath, Radius, RobotId, StateVector, VariableTimesteps},
    gbp::GbpIterationSchedule, // Keep GbpIterationSchedule struct definition here
    mission::{Mission, MissionState},
};

/// Event emitted when a robot is spawned
#[derive(Debug, Event)]
pub struct RobotSpawned(pub RobotId);

/// Event emitted when a robot is despawned
#[derive(Debug, Event)]
pub struct RobotDespawned(pub RobotId);

/// Event emitted when a robot reached its final waypoint and finished its path
#[derive(Debug, Event)]
pub struct RobotFinishedRoute(pub RobotId);

/// Event emitted when a robot reaches a waypoint
#[derive(Event)]
pub struct RobotReachedWaypoint {
    pub robot_id:       RobotId,
    pub waypoint_index: usize,
}

#[derive(Event)]
pub struct GbpScheduleChanged(pub GbpIterationSchedule);

impl From<gbp_config::GbpIterationSchedule> for GbpScheduleChanged {
    fn from(schedule: gbp_config::GbpIterationSchedule) -> Self {
        Self(GbpIterationSchedule(schedule))
    }
}

pub fn request_snapshot_of_robot_when_it_finishes_its_route(
    mut evr_robot_finished_route: EventReader<RobotFinishedRoute>,
    mut evw_take_snapshot_of_robot: EventWriter<TakeSnapshotOfRobot>,
) {
    for RobotFinishedRoute(robot_id) in evr_robot_finished_route.read() {
        evw_take_snapshot_of_robot.send(TakeSnapshotOfRobot(*robot_id));
    }
}

pub fn attach_despawn_timer_when_robot_finishes_route(
    mut commands: Commands,
    mut evr_robot_finished_route: EventReader<RobotFinishedRoute>,
    config: Res<Config>,
) {
    if !config.simulation.despawn_robot_when_final_waypoint_reached {
        return;
    }

    let duration = Duration::from_millis(100);
    for RobotFinishedRoute(robot_id) in evr_robot_finished_route.read() {
        info!(
            "attaching despawn timer to robot: {:?} with duration: {:?}",
            robot_id, duration
        );
        commands.spawn(
            crate::despawn_entity_after::components::DespawnEntityAfter::<Virtual>::new(
                *robot_id, duration,
            ),
        );
    }
}

pub fn progress_missions2(
    mut commands: Commands,
    mut q: Query<(Entity, &mut Mission, &PlanningStrategy)>,
    mut pathfinders: Query<(Entity, &mut EntropyComponent<WyRand>), Without<PathfindingTask>>,
    mut tasks: Query<&mut PathfindingTask>,
    mut factorgraphs: Query<(&mut FactorGraph, &VariableTimesteps)>,
    transforms: Query<&Transform>,
    config: Res<Config>,
    time: Res<Time>,
    colliders: Res<gbp_global_planner::Colliders>,
) {
    for (robot_entity, mut mission, plannning_strategy) in &mut q {
        match (mission.state, plannning_strategy) {
            (MissionState::Idle { .. }, PlanningStrategy::OnlyLocal) => {
                // no need to do anything
                mission.state = MissionState::Active;
            }
            (
                MissionState::Idle {
                    waiting_for_waypoints: false,
                },
                PlanningStrategy::RrtStar,
            ) => {
                if let Ok((pathfinder, prng)) = pathfinders.get_mut(robot_entity) {
                    // Get the current position from the transform if available
                    let start = if let Ok(transform) = transforms.get(robot_entity) {
                        transform.translation.xz()
                    } else {
                        // Fallback to mission.taskpoints if transform is not available
                        mission.taskpoints[mission.active_route].position()
                    };
                    
                    let end = mission.taskpoints[mission.active_route + 1].position();
                    debug!(
                        "starting pathfinding task for entity: {:?} from {:?} to {:?} #colliders \
                         {}",
                        robot_entity,
                        start,
                        end,
                        colliders.len()
                    );

                    // Use the enhanced pathfinding task with direct goal checking
                    gbp_global_planner::rrtstar::spawn_pathfinding_task_with_direct_goal_check(
                        &mut commands,
                        start,
                        end,
                        config.rrt.clone(),
                        colliders.clone(),
                        pathfinder,
                        Some(Box::new(prng.clone())),
                    );
                }

                mission.state = MissionState::Idle {
                    waiting_for_waypoints: true,
                };
            }
            (
                MissionState::Idle {
                    waiting_for_waypoints: true,
                },
                PlanningStrategy::RrtStar,
            ) => {
                if let Ok(mut task) = tasks.get_mut(robot_entity) {
                    if let Some(result) = future::block_on(future::poll_once(&mut task.0)) {
                        debug!("Pathfinding task completed for entity: {:?}", robot_entity);
                        commands.entity(robot_entity).remove::<PathfindingTask>();
                        match result {
                            Ok(new_path) => {
                                // Get all the information we need from mission before borrowing it mutably
                                let active_route_index = mission.active_route;
                                let start_pos = mission.taskpoints[active_route_index].position();
                                let goal_pos = mission.taskpoints[active_route_index + 1].position();
                                
                                // Ensure the path starts with the current position
                                let current_pos = if let Ok(transform) = transforms.get(robot_entity) {
                                    transform.translation.xz()
                                } else {
                                    start_pos
                                };
                                
                                // Create a new path that starts with the current position
                                let mut path_with_start = Vec::new();
                                path_with_start.push(current_pos);
                                
                                // Log the original path from RRT*
                                debug!("Original RRT* path: {:?}", new_path.0);
                                
                                // Add the rest of the path from RRT*
                                // Always include all points from the RRT* path
                                if !new_path.0.is_empty() {
                                    path_with_start.extend(new_path.0.iter().copied());
                                }
                                
                                // Ensure we have at least two points (start and goal)
                                if path_with_start.len() < 2 {
                                    // If we only have the start point, add the goal point
                                    path_with_start.push(goal_pos);
                                }
                                
                                // Now get the mutable reference to active_route
                                let active_route = mission.active_route_mut().unwrap();
                                
                                // Now create the waypoints with velocity information
                                let waypoints = path_with_start
                                    .iter()
                                    .tuple_windows()
                                    .map(|(from, to)| {
                                        let mut dir = (*to - *from).normalize();
                                        if dir.is_nan() {
                                            dir = Vec2::ZERO;
                                        }

                                        let vel = config.robot.target_speed.get() * dir;
                                        Vec4::new(from.x, from.y, vel.x, vel.y)
                                    })
                                    .map_into()
                                    .collect_vec();
                                
                                // Add the final waypoint (goal position with zero velocity)
                                let final_pos = path_with_start.last().unwrap();
                                let final_waypoint = StateVector::new(Vec4::new(final_pos.x, final_pos.y, 0.0, 0.0));
                                let waypoints = [waypoints, vec![final_waypoint]].concat();

                                if let Ok((mut fgraph, variable_timesteps)) =
                                    factorgraphs.get_mut(robot_entity)
                                {
                                    fgraph.modify_tracking_factors(|tracking| {
                                        let waypoints = waypoints
                                            .iter()
                                            .map(|wp: &StateVector| wp.position())
                                            .collect_vec();
                                        tracking.set_tracking_path(
                                            min_len_vec::TwoOrMore::new(waypoints).unwrap(),
                                        );
                                    });

                                    debug!(
                                        "updated tracking_path of each tracking factor of robot: \
                                         {:?}",
                                        robot_entity
                                    );

                                    let start: Vec4 = waypoints.first().copied().unwrap().into();
                                    let next: Vec4 = waypoints.get(1).copied().unwrap().into();
                                    let dir = next - start;
                                    let dir_normalized = dir.normalize();

                                    let n = variable_timesteps.0.len();
                                    let next = {
                                        let l = (config.robot.target_speed
                                            * config.robot.planning_horizon)
                                            .get();
                                        let max = dir.length() * 0.9;
                                        let s = if l < max { l } else { max };
                                        start + s * dir_normalized
                                    };

                                    let means = (0..n)
                                        .map(|i| i as f32 / n as f32)
                                        .map(|r| {
                                            let pos = start.xy().lerp(next.xy(), r);
                                            let vel =
                                                config.robot.target_speed.get() * dir_normalized;
                                            Vec4::new(pos.x, pos.y, vel.x, vel.y)
                                        })
                                        .map(|it| it.as_dvec4().to_array())
                                        .collect_vec();

                                    fgraph.reset_variables(&means, 1e30, Float::INFINITY);
                                    fgraph.reset_tracking_factors();
                                    error!(
                                        "updated variable positions of robot: {:?}",
                                        robot_entity
                                    );
                                }

                                debug!("updating route waypoints: {:?}", waypoints);
                                active_route.update_waypoints(waypoints.try_into().unwrap());
                                mission.state = MissionState::Active;
                            }
                            Err(e) => {
                                error!("Pathfinding error: {:?}", e);
                                mission.state = MissionState::Idle {
                                    waiting_for_waypoints: false, // try again
                                }
                            }
                        }
                    }
                }
            }
            (MissionState::Active, _) => {
                let route = mission.active_route().unwrap();
                if route.is_completed() {
                    info!("robot {:?} advancing to next route", robot_entity);
                    mission.next_route(&time);
                }
            }
            (MissionState::Completed, _) => {}
        }
    }
}

pub fn progress_missions(
    mut commands: Commands,
    mut q: Query<(Entity, &mut Mission, &PlanningStrategy)>,
    mut pathfinders: Query<(Entity, &mut EntropyComponent<WyRand>), Without<PathfindingTask>>,
    mut tasks: Query<&mut PathfindingTask>,
    mut factorgraphs: Query<(&mut FactorGraph, &VariableTimesteps)>,
    config: Res<Config>,
    time: Res<Time>,
    colliders: Res<gbp_global_planner::Colliders>,
) {
    for (robot_entity, mut mission, plannning_strategy) in &mut q {
        match (mission.state, plannning_strategy) {
            (MissionState::Idle { .. }, PlanningStrategy::OnlyLocal) => {
                // no need to do anything
                mission.state = MissionState::Active;
            }
            (
                MissionState::Idle {
                    waiting_for_waypoints: false,
                },
                PlanningStrategy::RrtStar,
            ) => {
                // disable tracking factors
                // factorgraphs.iter_mut().for_each(|(mut factorgraph, _)| {
                //    let mut settings = config.gbp.factors_enabled;
                //    settings.tracking = false;
                //    factorgraph.change_factor_enabled(settings);
                //});
                // info!("disabled tracking factors while idle");

                if let Ok((pathfinder, prng)) = pathfinders.get_mut(robot_entity) {
                    let active_route = mission.active_route().unwrap();
                    //    let start = active_route
                    //
                    //    .waypoints
                    //    .first()
                    //    .map(|w| w.position())
                    //    .unwrap();
                    // let end = active_route.waypoints.last().map(|w| w.position()).unwrap();
                    let start = mission.taskpoints[mission.active_route].position();
                    let end = mission.taskpoints[mission.active_route + 1].position();
                    // let start = mission.last_waypoint().unwrap().position();
                    // let end = mission.next_waypoint().unwrap().position();
                    info!(
                        "starting pathfinding task for entity: {:?} from {:?} to {:?} #colliders \
                         {}",
                        robot_entity,
                        start,
                        end,
                        colliders.len()
                    );

                    // dbg!(&colliders);
                    gbp_global_planner::rrtstar::spawn_pathfinding_task(
                        &mut commands,
                        start,
                        end,
                        config.rrt.clone(),
                        colliders.clone(),
                        pathfinder,
                        Some(Box::new(prng.clone())),
                    );
                }

                mission.state = MissionState::Idle {
                    waiting_for_waypoints: true,
                };

                // start rrt job TODO:
                // info!("(Idle {{ .. }}, RrtStar) => Active");
                // mission.state = RobotMissionState::Active;
            }
            (
                MissionState::Idle {
                    waiting_for_waypoints: true,
                },
                PlanningStrategy::RrtStar,
            ) => {
                // check if rrt job finished, and the advance to active state
                // TODO:

                if let Ok(mut task) = tasks.get_mut(robot_entity) {
                    // info!("polling task for entity: {:?}", robot_entity);
                    // if let Ok(result) = future::block_on(&mut task.0) {
                    if let Some(result) = future::block_on(future::poll_once(&mut task.0)) {
                        info!("Pathfinding task completed for entity: {:?}", robot_entity);
                        commands.entity(robot_entity).remove::<PathfindingTask>();
                        match result {
                            Ok(new_path) => {
                                let active_route = mission.active_route_mut().unwrap();
                                let waypoints = new_path
                                    .0
                                    .iter()
                                    .chain(new_path.0.last())
                                    .tuple_windows()
                                    .map(|(from, to)| {
                                        let mut dir = (*from - *to).normalize();
                                        if dir.is_nan() {
                                            dir = Vec2::ZERO;
                                        }

                                        let vel = config.robot.target_speed.get() * dir;
                                        Vec4::new(from.x, from.y, vel.x, vel.y)
                                    })
                                    .map_into()
                                    .collect_vec();

                                // dbg!(&waypoints);

                                if let Ok((mut fgraph, variable_timesteps)) =
                                    factorgraphs.get_mut(robot_entity)
                                {
                                    fgraph.modify_tracking_factors(|tracking| {
                                        let waypoints = waypoints
                                            .iter()
                                            .map(|wp: &StateVector| wp.position())
                                            .collect_vec();
                                        tracking.set_tracking_path(
                                            min_len_vec::TwoOrMore::new(waypoints).unwrap(),
                                        );
                                    });

                                    info!(
                                        "updated tracking_path of each tracking factor of robot: \
                                         {:?}",
                                        robot_entity
                                    );

                                    let start: Vec4 = waypoints.first().copied().unwrap().into();
                                    let next: Vec4 = waypoints.get(1).copied().unwrap().into();
                                    let dir = next - start;
                                    let dir_normalized = dir.normalize();
 
                                    let last_ts = variable_timesteps.0.last().unwrap().to_owned();
                                    // length of the path segment
                                    let n = variable_timesteps.0.len();
                                    // lerp positions between start and next, and have the
                                    // velocity
                                    // part be the normalized direction times max_speed
                                    // let next = next.length() * 0.8 * dir_normalized;
                                    let next = {
                                        let l = (config.robot.target_speed
                                            * config.robot.planning_horizon)
                                            .get();
                                        let max = dir.length() * 0.9;
                                        let s = if l < max { l } else { max };
                                        start + s * dir_normalized
                                    };

                                    let means = (0..n)
                                        .map(|i| i as f32 / n as f32)
                                        .map(|r| {
                                            let pos = start.xy().lerp(next.xy(), r);
                                            let vel =
                                                config.robot.target_speed.get() * dir_normalized;
                                            Vec4::new(pos.x, pos.y, vel.x, vel.y)
                                        })
                                        .map(|it| it.as_dvec4().to_array())
                                        .collect_vec();
                                    // means
                                    //    }
                                    //};

                                    fgraph.reset_variables(&means, 1e30, Float::INFINITY);
                                    fgraph.reset_tracking_factors();
                                    // fgraph.reset_variable_positions(positions.as_slice());
                                    error!(
                                        "updated variable positions of robot: {:?}",
                                        robot_entity
                                    );
                                }

                                info!("updating route waypoints: {:?}", waypoints);
                                active_route.update_waypoints(waypoints.try_into().unwrap());

                                // factorgraphs.iter_mut().for_each(|(mut factorgraph, _)| {
                                //    let mut settings = config.gbp.factors_enabled;
                                //    settings.tracking = true;
                                //    factorgraph.change_factor_enabled(settings);
                                //});

                                mission.state = MissionState::Active;
                                // TODO: update the variable placements of the
                                // factorgraph
                            }
                            Err(e) => {
                                error!("Pathfinding error: {:?}", e);
                                mission.state = MissionState::Idle {
                                    waiting_for_waypoints: false, // try again
                                }
                            }
                        }
                    }
                }
            }
            (MissionState::Active, _) => {
                // info!("(Active)");
                // check if route is completed, and advance to either completed or idle
                let route = mission.active_route().unwrap();
                if route.is_completed() {
                    info!("robot {:?} advancing to next route", robot_entity);
                    mission.next_route(&time);
                }
            }
            (MissionState::Completed, _) => {}
        }
    }
}


pub fn reached_waypoint(
    mut q: Query<(
        Entity,
        &mut FactorGraph,
        &Radius,
        &Transform,
        &mut Mission,
    )>,
    config: Res<Config>,
    time: Res<Time>,
    mut evw_robot_reached_waypoint: EventWriter<RobotReachedWaypoint>,
    mut evw_robot_despawned: EventWriter<RobotDespawned>,
    mut evw_robot_finalized_path: EventWriter<RobotFinishedRoute>,
) {
    for (robot_entity, mut fgraph, r, transform, mut mission) in &mut q {
        let Some(next_waypoint) = mission.next_waypoint() else {
            continue;
        };

        let r_sq = r.0 * r.0;

        let reached = {
            use CheckIntersectionWith::{Current, Horizon, Variable};
            let when_intersects = if mission.next_waypoint_is_last() {
                mission.finished_when_intersects
            } else {
                mission.taskpoint_reached_when_intersects
            };

            let variable = match when_intersects.intersects_with {
                Current => fgraph.first_variable(),
                Horizon => fgraph.last_variable(),
                Variable(ix) => {
                    let ix: usize = Into::into(ix);
                    fgraph.nth_variable(ix).or_else(|| fgraph.last_variable())
                }
            }
            .map(|(_, v)| v)
            .expect("variable exists");

            let estimated_pos = variable.estimated_position_vec2();
            let distance_squared = match when_intersects.distance {
                IntersectionDistance::RobotRadius => r_sq,
                IntersectionDistance::Meter(meter) => meter * meter,
            };

            let dist2waypoint = estimated_pos.distance_squared(next_waypoint.position());
            dist2waypoint < distance_squared
        };

        if reached {
            mission.advance_to_next_waypoint(&time);
            evw_robot_reached_waypoint.send(RobotReachedWaypoint {
                robot_id:       robot_entity,
                waypoint_index: 0, // TODO: This index seems wrong, should reflect actual waypoint
            });

            info!("robot: {:?} reached a waypoint", robot_entity);

            if let Some(current_waypoint_index) = mission.current_waypoint_index() {
                info!(
                    "updating waypoint index of tracking factors to {}",
                    current_waypoint_index
                );
                fgraph.modify_tracking_factors(|tracking| {
                    tracking.set_tracking_index(current_waypoint_index);
                });
            }
        }

        if mission.is_completed() {
            info!("robot {:?} completed its mission", robot_entity);
            evw_robot_finalized_path.send(RobotFinishedRoute(robot_entity));
            if config.simulation.despawn_robot_when_final_waypoint_reached {
                evw_robot_despawned.send(RobotDespawned(robot_entity));
            }
        }
    }
}
