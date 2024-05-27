use std::{
    collections::{BTreeSet, HashMap},
    num::NonZeroUsize,
    sync::{Arc, Mutex},
    time::Duration,
};

use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    tasks::futures_lite::future,
};
use bevy_prng::WyRand;
use bevy_rand::{component::EntropyComponent, prelude::GlobalEntropy};
use gbp_config::{
    formation::{PlanningStrategy, ReachedWhenIntersects},
    Config,
};
use gbp_global_planner::PathfindingTask;
use gbp_linalg::prelude::*;
use itertools::Itertools;
use ndarray::{array, concatenate, s, Axis};
use rand::Rng;

use super::{
    collisions::resources::{RobotEnvironmentCollisions, RobotRobotCollisions},
    spawner::RobotClickedOn,
};
use crate::{
    bevy_utils::run_conditions::time::virtual_time_is_paused,
    export::events::TakeSnapshotOfRobot,
    factorgraph::{
        factor::{ExternalVariableId, FactorNode},
        factorgraph::{FactorGraph, NodeIndex, VariableIndex},
        id::{FactorId, VariableId},
        message::{FactorToVariableMessage, VariableToFactorMessage},
        variable::VariableNode,
        DOFS,
    },
    pause_play::PausePlay,
    simulation_loader::{LoadSimulation, ReloadSimulation, SdfImage},
};

pub type RobotId = Entity;

pub struct RobotPlugin;

// #[derive(Debug, SystemSet, PartialEq, Eq, Hash, Clone, Copy)]
// struct GbpSystemSet;

impl Plugin for RobotPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GbpIterationSchedule>()
            .init_resource::<RobotNumberGenerator>()
            .insert_state(ManualModeState::Disabled)
            .add_event::<RobotSpawned>()
            .add_event::<RobotDespawned>()
            .add_event::<RobotFinishedRoute>()
            .add_event::<RobotReachedWaypoint>()
            .add_event::<GbpScheduleChanged>()
            .add_systems(PreUpdate, start_manual_step.run_if(virtual_time_is_paused))
            .add_systems(
                Update,
                reset_robot_number_generator
                    .run_if(on_event::<LoadSimulation>().or_else(on_event::<ReloadSimulation>())),
            )
            .add_systems(
                Update,
                (
                    on_robot_clicked,
                    on_gbp_schedule_changed,
                    attach_despawn_timer_when_robot_finishes_route,
                    request_snapshot_of_robot_when_it_finishes_its_route,
                    progress_missions.run_if(resource_exists::<gbp_global_planner::Colliders>),
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    reached_waypoint,
                    // progress_missions.run_if(resource_exists::<gbp_global_planner::Colliders>),
                )
                    .run_if(not(virtual_time_is_paused)),
            )
            .add_systems(
                FixedUpdate,
                // Update,
                (
                    update_robot_neighbours,
                    delete_interrobot_factors,
                    create_interrobot_factors,
                    update_failed_comms,
                    // iterate_gbp_internal,
                    // iterate_gbp_external,
                    // iterate_gbp_internal_sync,
                    // iterate_gbp_external_sync,
                    // iterate_gbp,
                    iterate_gbp_v2,
                    // update_prior_of_horizon_state_v2,
                    update_prior_of_horizon_state,
                    update_prior_of_current_state_v3,
                    // update_prior_of_current_state,
                    // despawn_robots,
                    finish_manual_step.run_if(ManualModeState::enabled),
                )
                    .chain()
                    .run_if(not(virtual_time_is_paused)),
            );
    }
}

fn request_snapshot_of_robot_when_it_finishes_its_route(
    mut evr_robot_finished_route: EventReader<RobotFinishedRoute>,
    mut evw_take_snapshot_of_robot: EventWriter<TakeSnapshotOfRobot>,
) {
    for RobotFinishedRoute(robot_id) in evr_robot_finished_route.read() {
        evw_take_snapshot_of_robot.send(TakeSnapshotOfRobot(*robot_id));
    }
}

#[derive(Resource)]
struct RobotNumberGenerator(usize);

impl Default for RobotNumberGenerator {
    fn default() -> Self {
        Self(1)
    }
}

impl RobotNumberGenerator {
    pub fn next(&mut self) -> NonZeroUsize {
        let next = self.0;
        self.0 += 1;
        next.try_into().unwrap()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

fn reset_robot_number_generator(mut robot_number_generator: ResMut<RobotNumberGenerator>) {
    robot_number_generator.reset();
}

#[derive(Event)]
pub struct GbpScheduleChanged(pub GbpIterationSchedule);

impl From<gbp_config::GbpIterationSchedule> for GbpScheduleChanged {
    fn from(schedule: gbp_config::GbpIterationSchedule) -> Self {
        Self(GbpIterationSchedule(schedule))
    }
}

/// Event emitted when a robot is spawned
#[derive(Debug, Event)]
pub struct RobotSpawned(pub RobotId);

/// Event emitted when a robot is despawned
#[derive(Debug, Event)]
pub struct RobotDespawned(pub RobotId);

/// Event emitted when a robot reached its final waypoint and finished its path
#[derive(Debug, Event)]
pub struct RobotFinishedRoute(pub RobotId);

fn attach_despawn_timer_when_robot_finishes_route(
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

/// Event emitted when a robot reaches a waypoint
#[derive(Event)]
pub struct RobotReachedWaypoint {
    pub robot_id:       RobotId,
    pub waypoint_index: usize,
}

// fn despawn_robots(
//     mut commands: Commands,
//     mut query: Query<&mut FactorGraph>,
//     mut evr_robot_despawned: EventReader<RobotDespawned>,
// ) {
//     for RobotDespawned(robot_id) in evr_robot_despawned.read() {
//         for mut factorgraph in &mut query {
//             let _ = factorgraph.remove_connection_to(*robot_id);
//         }
//
//         if let Some(mut entitycommand) = commands.get_entity(*robot_id) {
//             info!("despawning robot: {:?}", entitycommand.id());
//             entitycommand.despawn();
//         } else {
//             error!(
//                 "A DespawnRobotEvent event was emitted with entity id: {:?}
// but the entity does \                  not exist!",
//                 robot_id
//             );
//         }
//     }
// }

trait CreateVariableTimesteps {
    fn create_variable_timesteps(n: NonZeroUsize) -> Vec<u32>;
}

struct EvenlySpacedVariableTimesteps;

struct GbpplannerVariableTimesteps;

impl CreateVariableTimesteps for GbpplannerVariableTimesteps {
    fn create_variable_timesteps(n: NonZeroUsize) -> Vec<u32> {
        todo!()
        // get_variable_timesteps()
    }
}

// // #[derive(Component, Deref, DerefMut, derive_more::Index)]
// #[derive(Resource, Deref, DerefMut, derive_more::Index)]
// pub struct VariableTimesteps(pub Vec<u32>);
//
// impl VariableTimesteps {
//     /// Returns the number of timesteps
//     pub fn len(&self) -> usize {
//         self.0.len()
//     }
//
//     // pub fn from_config(config: &Config) -> Self {
//     //     let lookahead_horizon: u32 =
//     //         (config.robot.planning_horizon.get() /
// config.simulation.t0.get()) as     // u32;     let lookahead_multiple =
// config.gbp.lookahead_multiple as u32;     //     Self(get_variable_timesteps(
//     //         lookahead_horizon,
//     //         lookahead_multiple,
//     //     ))
//     // }
// }
//
// impl From<&crate::config::Config> for VariableTimesteps {
//     fn from(config: &crate::config::Config) -> Self {
//         let lookahead_horizon: u32 =
//             (config.robot.planning_horizon.get() /
// config.simulation.t0.get()) as u32;         let lookahead_multiple =
// config.gbp.lookahead_multiple as u32;         Self(get_variable_timesteps(
//             lookahead_horizon,
//             lookahead_multiple,
//         ))
//     }
// }
//
// impl FromWorld for VariableTimesteps {
//     fn from_world(world: &mut World) -> Self {
//         if let Some(config) = world.get_resource::<crate::config::Config>() {
//             Self::from(config)
//         } else {
//             Self(vec![])
//         }
//     }
// }

// /// Resource that stores the horizon timesteps sequence
// #[derive(Resource, Debug, Index)]
// pub struct VariableTimesteps {
//     #[index]
//     timesteps: Vec<u32>,
// }
//
// impl VariableTimesteps {
//     /// Extracts a slice containing the entire vector.
//     #[inline(always)]
//     pub fn as_slice(&self) -> &[u32] {
//         self.timesteps.as_slice()
//     }
// }
//
// impl FromWorld for VariableTimesteps {
//     // TODO: refactor
//     fn from_world(world: &mut World) -> Self {
//         // let config = world.resource::<Config>();
//
//         // let lookahead_horizon = config.robot.planning_horizon /
// config.simulation.t0;         let lookahead_horizon = 5.0 / 0.25;
//
//         #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
//         // FIXME(kpbaks): read settings from config
//         Self {
//             timesteps: get_variable_timesteps(
//                 // lookahead_horizon.get() as u32,
//                 lookahead_horizon as u32,
//                 // config.gbp.lookahead_multiple as u32,
//                 3,
//             ),
//         }
//     }
// }

// #[derive(Debug, thiserror::Error)]
// pub enum RobotInitError {
//     #[error("No waypoints were provided")]
//     NoWaypoints,
//     #[error("No variable timesteps were provided")]
//     NoVariableTimesteps,
// }

/// Component for entities with a radius, used for robots
#[derive(Component, Debug, Deref, DerefMut)]
pub struct Radius(pub f32);

/// Represents a robotic route consisting of several waypoints that define
/// positions and velocities the robot should achieve as it progresses along the
/// path.
#[allow(clippy::similar_names)]
#[derive(Component, Debug, derive_more::Index)]
pub struct Route {
    /// A list of state vectors representing waypoints.
    #[index]
    waypoints:    Vec<StateVector>,
    /// The index of the next target waypoint in the waypoints vector.
    target_index: usize,
    // /// Criteria determining when the robot is considered to have reached a
    // /// waypoint.
    // pub intersects_when: WaypointReachedWhenIntersects,
    /// The recorded time at the start of the route as a floating-point
    /// timestamp.
    started_at:   f64,
    /// Optional recorded time when the route was completed as a floating-point
    /// timestamp.
    finished_at:  Option<f64>,
}

impl std::fmt::Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "waypoints:")?;
        for wp in &self.waypoints {
            writeln!(f, "  {}", wp)?;
        }

        writeln!(f, "target_index: {}", self.target_index)?;
        writeln!(f, "started_at: {}", self.started_at)?;
        writeln!(f, "finished_at: {:?}", self.finished_at)
    }
}

// /// Represents the initial pose of the robot using a four-dimensional vector.
// #[derive(Component, Debug)]
// pub struct InitialPose(pub Vec4);

impl Route {
    /// Creates a new route from a specified set of waypoints and initial time.
    ///
    /// # Arguments
    /// * `waypoints` - A vector of `StateVector` that must contain at least two
    ///   elements.
    /// * `intersects_when` - Criteria to determine when a waypoint has been
    ///   reached.
    /// * `started_at` - The start time of the route as a floating-point
    ///   timestamp.
    pub fn new(
        waypoints: min_len_vec::TwoOrMore<StateVector>,
        // intersects_when: WaypointReachedWhenIntersects,
        started_at: f64,
    ) -> Self {
        Self {
            waypoints: waypoints.into(),
            target_index: 1, // skip first waypoint, as it is the initial pose
            // intersects_when,
            started_at,
            finished_at: None,
        }
    }

    pub fn update_waypoints(&mut self, waypoints: min_len_vec::TwoOrMore<StateVector>) {
        self.waypoints = waypoints.into();
        self.target_index = 1;
    }

    // pub fn upcoming(waypoints: min_len_vec::TwoOrMore<StateVector>) -> Self {
    //     Self {
    //         waypoints:    waypoints.into(),
    //         target_index: 1,
    //         started_at:   None,
    //         finished_at:  None,
    //     }
    // }

    /// Returns a reference to the next waypoint, if available.
    pub fn next_waypoint(&self) -> Option<&StateVector> {
        self.waypoints.get(self.target_index)
    }

    pub fn current_waypoint_index(&self) -> Option<usize> {
        if self.is_completed() {
            None
        } else {
            Some(self.target_index)
        }
        // self.target_index.checked_sub(1)
    }

    /// Returns a reference to the last waypoint, if available.
    pub fn last_waypoint(&self) -> Option<&StateVector> {
        self.waypoints.get(self.target_index - 1)
    }

    pub fn next_waypoint_is_last(&self) -> bool {
        self.target_index == self.waypoints.len() - 1
    }

    /// Advances to the next waypoint, updating the finished time if the route
    /// is completed.
    ///
    /// # Arguments
    /// * `elapsed` - The current time as a `std::time::Duration` since the
    ///   start.
    pub fn advance(&mut self, elapsed: std::time::Duration) {
        if self.target_index < self.waypoints.len() {
            self.target_index += 1;
        }
        if self.is_completed() && self.finished_at.is_none() {
            self.finished_at = Some(elapsed.as_secs_f64() + self.started_at);
        }
    }

    /// Returns the total number of waypoints.
    #[inline]
    pub fn len(&self) -> usize {
        self.waypoints.len()
    }

    /// Provides a slice of all waypoints.
    #[inline]
    pub fn waypoints(&self) -> &[StateVector] {
        &self.waypoints
    }

    /// Returns the start time of the route.
    #[inline]
    pub fn started_at(&self) -> f64 {
        self.started_at
    }

    /// Returns the finish time of the route, if completed.
    #[inline]
    pub fn finished_at(&self) -> Option<f64> {
        self.finished_at
    }

    /// Returns a reference to the first waypoint.
    #[inline]
    pub fn first(&self) -> &StateVector {
        // waypoints are guaranteed to have at least two elements
        &self.waypoints[0]
    }

    /// Returns a reference to the last waypoint.
    #[inline]
    pub fn last(&self) -> &StateVector {
        // waypoints are guaranteed to have at least two elements
        &self.waypoints[self.waypoints.len() - 1]
    }

    /// Checks whether all waypoints have been reached.
    #[inline]
    pub fn is_completed(&self) -> bool {
        self.target_index >= self.waypoints.len()
    }
}

/// Component for entities with a radio antenna
#[derive(Component, Debug)]
pub struct RadioAntenna {
    /// The radius that the radio antenna can cover
    pub radius: f32,
    /// Whether the antenna is currently active
    pub active: bool,
}

impl RadioAntenna {
    /// Creates a new radio antenna.
    pub fn new(radius: f32, active: bool) -> Self {
        Self { radius, active }
    }

    /// Toggle the state of the antenna between on and off
    pub fn toggle(&mut self) {
        self.active = !self.active;
    }

    /// Check whether a given position is within the antenna's range
    pub fn within_range(&self, position: Vec2) -> bool {
        position.length() < self.radius
    }
}

/// A robot's state, consisting of other robots within communication range,
/// and other robots that are connected via inter-robot factors.
#[derive(Component, Debug, Default)]
pub struct RobotConnections {
    /// List of robot ids that are within the communication radius of this
    /// robot. called `neighbours_` in **gbpplanner**.
    pub robots_within_comms_range: BTreeSet<RobotId>,
    /// List of robot ids that are currently connected via inter-robot factors
    /// to this robot called `connected_r_ids_` in **gbpplanner**.
    pub robots_connected_with:     BTreeSet<RobotId>,
}

impl RobotConnections {
    /// Create a new `RobotState`
    #[must_use]
    pub fn new() -> Self {
        Self {
            robots_within_comms_range: BTreeSet::new(),
            robots_connected_with:     BTreeSet::new(),
        }
    }
}

// TODO: change to collider
#[derive(Debug, Component, Deref)]
pub struct Ball(parry2d::shape::Ball);

#[derive(Clone, Copy, Debug, Component, Resource, derive_more::Into, derive_more::From)]
pub struct GbpIterationSchedule(pub gbp_config::GbpIterationSchedule);

impl GbpIterationSchedule {
    pub fn schedule(&self) -> Box<dyn gbp_schedule::GbpScheduleIterator> {
        let config = gbp_schedule::GbpScheduleParams {
            internal: self.0.internal as u8,
            external: self.0.external as u8,
        };
        self.0.schedule.get(config)
    }
}

impl FromWorld for GbpIterationSchedule {
    fn from_world(world: &mut World) -> Self {
        if let Some(config) = world.get_resource::<Config>() {
            Self(config.gbp.iteration_schedule)
        } else {
            Self(gbp_config::GbpIterationSchedule::default())
        }
    }
}

fn progress_missions(
    mut commands: Commands,
    mut q: Query<(Entity, &mut RobotMission, &PlanningStrategy)>,
    mut pathfinders: Query<(Entity, &mut EntropyComponent<WyRand>), Without<PathfindingTask>>,
    mut tasks: Query<&mut PathfindingTask>,
    mut factorgraphs: Query<&mut FactorGraph>,
    config: Res<Config>,
    time: Res<Time>,
    colliders: Res<gbp_global_planner::Colliders>,
) {
    for (robot_entity, mut mission, plannning_strategy) in &mut q {
        match (mission.state, plannning_strategy) {
            (RobotMissionState::Idle { .. }, PlanningStrategy::OnlyLocal) => {
                // no need to do anything
                info!("(Idle {{ .. }}, OnlyLocal) => Active");
                mission.state = RobotMissionState::Active;
            }
            (
                RobotMissionState::Idle {
                    waiting_for_waypoints: false,
                },
                PlanningStrategy::RrtStar,
            ) => {
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

                mission.state = RobotMissionState::Idle {
                    waiting_for_waypoints: true,
                };

                // start rrt job TODO:
                // info!("(Idle {{ .. }}, RrtStar) => Active");
                // mission.state = RobotMissionState::Active;
            }
            (
                RobotMissionState::Idle {
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

                                        let vel = config.robot.max_speed.get() * dir;
                                        Vec4::new(from.x, from.y, vel.x, vel.y)
                                    })
                                    .map_into()
                                    .collect_vec();

                                // dbg!(&waypoints);

                                if let Ok(mut fgraph) = factorgraphs.get_mut(robot_entity) {
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
                                }

                                info!("updating route waypoints: {:?}", waypoints);
                                active_route.update_waypoints(waypoints.try_into().unwrap());
                                mission.state = RobotMissionState::Active;
                            }
                            Err(e) => {
                                error!("Pathfinding error: {:?}", e);
                                mission.state = RobotMissionState::Idle {
                                    waiting_for_waypoints: false, // try again
                                }
                            }
                        }
                    }
                }
            }
            (RobotMissionState::Active, _) => {
                // info!("(Active)");
                // check if route is completed, and advance to either completed or idle
                let route = mission.active_route().unwrap();
                if route.is_completed() {
                    info!("robot {:?} advancing to next route", robot_entity);
                    mission.next_route(&time);
                }
            }
            (RobotMissionState::Completed, _) => {}
        }
    }
}

#[derive(Debug, Component)]
pub struct RobotMission {
    pub routes: Vec<Route>,
    pub taskpoints: Vec<StateVector>,
    active_route: usize,
    // total_routes: usize,
    started_at: f64,
    finished_at: Option<f64>,
    pub state: RobotMissionState,
    finished_when_intersects: ReachedWhenIntersects,
    taskpoint_reached_when_intersects: ReachedWhenIntersects,
}

// impl std::fmt::Display for RobotMission {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        writeln!(f, "robot-mission:")?;
//        writeln!(f, "  routes: {}", self.routes)?;
//        writeln!(f, "  waypoints: {}", self.waypoints)?;
//        writeln!(f, "")
//    }
//}

impl RobotMission {
    pub fn local(
        waypoints: min_len_vec::TwoOrMore<StateVector>,
        started_at: f64,
        finished_when_intersects: ReachedWhenIntersects,
        waypoint_reached_when_intersects: ReachedWhenIntersects,
    ) -> Self {
        let route = Route::new(
            waypoints.iter().copied().collect_vec().try_into().unwrap(),
            started_at,
        );
        Self {
            routes: vec![route],
            taskpoints: vec![*waypoints.first(), *waypoints.last()],
            active_route: 0,
            // total_routes: 1,
            started_at,
            finished_at: None,
            state: RobotMissionState::Active,
            finished_when_intersects,
            taskpoint_reached_when_intersects: waypoint_reached_when_intersects,
        }

        // Self::new(waypoints, started_at, RobotMissionState::Active)
    }

    pub fn global(
        waypoints: min_len_vec::TwoOrMore<StateVector>,
        started_at: f64,
        finished_when_intersects: ReachedWhenIntersects,
        waypoint_reached_when_intersects: ReachedWhenIntersects,
    ) -> Self {
        Self::new(
            waypoints,
            started_at,
            RobotMissionState::Idle {
                waiting_for_waypoints: false,
            },
            finished_when_intersects,
            waypoint_reached_when_intersects,
        )
    }

    fn new(
        waypoints: min_len_vec::TwoOrMore<StateVector>,
        started_at: f64,
        state: RobotMissionState,
        finished_when_intersects: ReachedWhenIntersects,
        waypoint_reached_when_intersects: ReachedWhenIntersects,
    ) -> Self {
        assert_ne!(state, RobotMissionState::Completed);
        let first_route = Route::new(
            waypoints
                .iter()
                .copied()
                .take(2)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            started_at,
        );
        Self {
            routes: vec![first_route],
            // total_routes: waypoints.len() - 1,
            taskpoints: waypoints.into(),
            active_route: 0,
            started_at,
            finished_at: None,
            state,
            finished_when_intersects,
            taskpoint_reached_when_intersects: waypoint_reached_when_intersects,
        }
    }

    /// Return the time at which the mission was started in seconds
    #[inline]
    pub fn started_at(&self) -> f64 {
        self.started_at
    }

    pub fn finished_at(&self) -> Option<f64> {
        self.finished_at
    }

    pub fn is_completed(&self) -> bool {
        self.state == RobotMissionState::Completed
    }

    pub fn next_waypoint(&self) -> Option<&StateVector> {
        self.routes
            .get(self.active_route)
            .and_then(|r| r.next_waypoint())
        // self.routes[self.active_route].next_waypoint()
    }

    pub fn current_waypoint_index(&self) -> Option<usize> {
        self.routes
            .get(self.active_route)
            .and_then(|r| r.current_waypoint_index())
    }

    pub fn last_waypoint(&self) -> Option<&StateVector> {
        self.routes
            .get(self.active_route)
            .and_then(|r| r.last_waypoint())
    }

    pub fn next_waypoint_is_last(&self) -> bool {
        self.active_route == self.routes.len() - 1
            && self
                .active_route()
                .is_some_and(|r| r.next_waypoint_is_last())
    }

    pub fn active_route(&self) -> Option<&Route> {
        if self.is_completed() {
            None
        } else {
            self.routes.get(self.active_route)
        }
    }

    pub fn active_route_mut(&mut self) -> Option<&mut Route> {
        if self.is_completed() {
            None
        } else {
            self.routes.get_mut(self.active_route)
        }
    }

    pub fn next_route(&mut self, time: &Time) {
        match self.state {
            RobotMissionState::Completed => {}
            _ => {
                self.active_route += 1;
                // if self.active_route >= self.routes.len() {
                if self.active_route >= self.taskpoints.len() - 1 {
                    // if self.active_route >= self.total_routes {
                    self.state = RobotMissionState::Completed;
                    self.finished_at = Some(time.elapsed().as_secs_f64());
                } else {
                    let waypoints: Vec<StateVector> = self
                        .taskpoints
                        .iter()
                        .skip(self.active_route)
                        .take(2)
                        .copied()
                        .collect_vec();
                    let next_route =
                        Route::new(waypoints.try_into().unwrap(), time.elapsed_seconds_f64());
                    self.routes.push(next_route);
                    self.state = RobotMissionState::Idle {
                        waiting_for_waypoints: false,
                    }
                }
            }
        }
    }

    pub fn advance_to_next_waypoint(&mut self, time: &Time) {
        match self.state {
            RobotMissionState::Active => {
                let current_route = self.routes.get_mut(self.active_route).unwrap();
                current_route.advance(time.elapsed());
                if current_route.is_completed() {
                    self.next_route(time);
                }
            }
            RobotMissionState::Completed | RobotMissionState::Idle { .. } => return,
        }
    }

    pub fn waypoints(&self) -> impl Iterator<Item = &StateVector> + '_ {
        self.routes.iter().flat_map(|r| r.waypoints())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RobotMissionState {
    Idle { waiting_for_waypoints: bool },
    Active,
    Completed,
}

impl RobotMissionState {
    pub fn idle(&self) -> bool {
        matches!(self, RobotMissionState::Idle { .. })
    }
}

#[derive(Bundle)]
pub struct RobotBundle {
    /// The factor graph that the robot is part of, and uses to perform GBP
    /// message passing.
    pub factorgraph: FactorGraph,
    /// The schedule that the robot uses to run internal and external GBP
    /// iterations
    pub gbp_iteration_schedule: GbpIterationSchedule,
    /// Radius of the robot.
    /// If the robot is not a perfect circle, then set radius to be the smallest
    /// circle that fully encompass the shape of the robot. **constraint**:
    /// > 0.0
    pub radius: Radius,

    pub ball: Ball,
    pub antenna: RadioAntenna,
    /// The current state of the robot
    pub connections: RobotConnections,

    // /// Waypoints used to instruct the robot to move to a specific position.
    // /// A `VecDeque` is used to allow for efficient `pop_front` operations, and
    // /// `push_back` operations.
    // pub route: Route,

    // /// Initial sate
    // pub initial_state: StateVector,
    /// Time between t_i and t_i+1
    pub t0: T0,

    /// Boolean component used to keep track of whether the robot has finished
    /// its path by reaching its final waypoint. This flag exists to ensure
    /// that the robot is not detected as having finished more than once.
    /// TODO: should probably be modelled as an enum instead, too easier support
    /// additional states in the future
    finished_path: FinishedPath,

    pub mission: RobotMission,
    // / Criteria determining when the robot is considered to have reached a
    // / waypoint.
    // pub intersects_when: ReachedWhenIntersects,
    pub planning_strategy: PlanningStrategy,
}

/// State vector of a robot
/// [x, y, x', y']
#[derive(
    Component,
    Debug,
    Clone,
    Copy,
    derive_more::Into,
    derive_more::From,
    derive_more::Add,
    derive_more::Sub,
)]
pub struct StateVector(pub bevy::math::Vec4);

impl std::fmt::Display for StateVector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{:.4}, {:.4}, {:.4}, {:.4}]",
            self.0.x, self.0.y, self.0.z, self.0.w
        )
    }
}

impl StateVector {
    /// Access the position vector of the robot state
    pub fn position(&self) -> Vec2 {
        self.0.xy()
    }

    /// Access the velocity vector of the robot state
    pub fn velocity(&self) -> Vec2 {
        self.0.zw()
    }

    /// Update the position vector of the robot state
    pub fn update_position(&mut self, position: Vec2) {
        self.0.x = position.x;
        self.0.y = position.y;
    }

    /// Update the velocity vector of the robot state
    pub fn update_velocity(&mut self, velocity: Vec2) {
        self.0.z = velocity.x;
        self.0.w = velocity.y;
    }

    /// Create a new `StateVector`
    #[must_use]
    pub const fn new(state: Vec4) -> Self {
        Self(state)
    }
}

impl RobotBundle {
    /// Create a new `RobotBundle`
    #[must_use = "Constructor responsible for creating the robots factorgraph"]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(
        robot_id: RobotId,
        initial_state: StateVector,
        // route: Route,
        variable_timesteps: &[u32],
        // variable_timesteps: Vec<u32>,
        // variable_timesteps: VariableTimesteps,
        config: &Config,
        env_config: &gbp_environment::Environment,
        radius: f32,
        sdf: &SdfImage,
        started_at: f64,
        waypoints: min_len_vec::TwoOrMore<StateVector>,
        // use_tracking: bool,
        planning_strategy: PlanningStrategy,
        waypoint_reached_when_intersects: ReachedWhenIntersects,
        finished_when_intersects: ReachedWhenIntersects,
    ) -> Self {
        assert!(
            !variable_timesteps.is_empty(),
            "Variable timesteps cannot be empty"
        );

        let start: Vec4 = waypoints.first().to_owned().into();

        let next_waypoint: Vec4 = waypoints[1].into();

        // Initialise the horizon in the direction of the goal, at a distance T_HORIZON
        // * MAX_SPEED from the start.
        let start2goal: Vec4 = next_waypoint - start;

        let horizon = start
            + f32::min(
                start2goal.length(),
                (config.robot.planning_horizon * config.robot.max_speed).get(),
            ) * start2goal.normalize();

        let mut factorgraph = FactorGraph::new(robot_id);
        let last_variable_timestep = *variable_timesteps
            .last()
            .expect("Know that variable_timesteps has at least one element");
        let n_variables = variable_timesteps.len();
        let mut variable_node_indices = Vec::with_capacity(n_variables);

        let mut init_variable_means = Vec::<Vector<Float>>::with_capacity(n_variables);
        for (i, &variable_timestep) in variable_timesteps.iter().enumerate() {
            // Set initial mean and covariance of variable interpolated between start and
            // horizon
            //#[allow(clippy::cast_precision_loss)]
            // let mean = start
            //    + (horizon - start) * (variable_timestep as f32 / last_variable_timestep
            //      as f32);

            let mean = match planning_strategy {
                PlanningStrategy::OnlyLocal => {
                    start
                        + (horizon - start)
                            * (variable_timestep as f32 / last_variable_timestep as f32)
                }
                // PlanningStrategy::RrtStar => Vec4::ZERO,
                // FIXME: why unwind like a worm?
                PlanningStrategy::RrtStar => start,
            };

            let sigma = if i == 0 || i == n_variables - 1 {
                // Start and Horizon state variables should be 'fixed' during optimisation at a
                // timestep SIGMA_POSE_FIXED
                1e30
                // 1e20
            } else {
                // 4e9
                // 0.0
                // 1e30
                Float::INFINITY
            };

            let precision_matrix = Matrix::<Float>::from_diag_elem(DOFS, sigma);

            let mean = array![
                Float::from(mean.x),
                Float::from(mean.y),
                Float::from(mean.z),
                Float::from(mean.w)
            ];
            init_variable_means.push(mean.slice(s![..2]).to_owned());

            let variable = VariableNode::new(factorgraph.id(), mean, precision_matrix, DOFS);
            let variable_index = factorgraph.add_variable(variable);
            variable_node_indices.push(variable_index);
        }

        let t0 = radius / 2.0 / config.robot.max_speed.get();

        // Create Dynamic factors between variables
        for i in 0..variable_timesteps.len() - 1 {
            // T0 is the timestep between the current state and the first planned state.
            #[allow(clippy::cast_precision_loss)]
            // let delta_t = config.simulation.t0.get()
            let delta_t = t0 * (variable_timesteps[i + 1] - variable_timesteps[i]) as f32;

            let measurement = Vector::<Float>::zeros(DOFS);

            let dynamic_factor = FactorNode::new_dynamic_factor(
                factorgraph.id(),
                Float::from(config.gbp.sigma_factor_dynamics),
                measurement,
                Float::from(delta_t),
                config.gbp.factors_enabled.dynamic,
            );

            let factor_node_index = factorgraph.add_factor(dynamic_factor);
            let factor_id = FactorId::new(factorgraph.id(), factor_node_index);
            // A dynamic factor connects two variables
            let _ = factorgraph.add_internal_edge(
                VariableId::new(factorgraph.id(), variable_node_indices[i + 1]),
                factor_id,
            );
            let _ = factorgraph.add_internal_edge(
                VariableId::new(factorgraph.id(), variable_node_indices[i]),
                factor_id,
            );
        }

        // Create Obstacle factors for all variables excluding start,
        // excluding horizon
        let tile_size = env_config.tiles.settings.tile_size as f64;
        let (nrows, ncols) = env_config.tiles.grid.shape();
        let world_size = crate::factorgraph::factor::obstacle::WorldSize {
            width:  tile_size * ncols as f64,
            height: tile_size * nrows as f64,
        };

        // Create Obstacle factors for all variables excluding start and
        // horizon state
        #[allow(clippy::needless_range_loop)]
        for i in 1..variable_timesteps.len() - 1 {
            let obstacle_factor = FactorNode::new_obstacle_factor(
                factorgraph.id(),
                Float::from(config.gbp.sigma_factor_obstacle),
                array![0.0],
                sdf.clone(),
                world_size,
                config.gbp.factors_enabled.obstacle,
            );

            let factor_node_index = factorgraph.add_factor(obstacle_factor);
            let factor_id = FactorId::new(factorgraph.id(), factor_node_index);
            let _ = factorgraph.add_internal_edge(
                VariableId::new(factorgraph.id(), variable_node_indices[i]),
                factor_id,
            );
        }

        let mission = match planning_strategy {
            PlanningStrategy::OnlyLocal => RobotMission::local(
                waypoints.try_into().unwrap(),
                started_at,
                finished_when_intersects,
                waypoint_reached_when_intersects,
            ),
            PlanningStrategy::RrtStar => RobotMission::global(
                waypoints.try_into().unwrap(),
                started_at,
                finished_when_intersects,
                waypoint_reached_when_intersects,
            ),
        };

        // dbg!(&mission);
        // Create Tracking factors for all variables, excluding the start
        // if config.gbp.factors_enabled.tracking {
        for i in 1..variable_timesteps.len() - 1 {
            // for var_ix in &variable_node_indices[1..] {
            let init_linearisation_point =
                concatenate![Axis(0), init_variable_means[i].clone(), array![0.0, 0.0]];
            // println!("init_linearisation_point: {:?}", init_linearisation_point);
            let initial_route = mission.active_route().unwrap();
            let waypoints = initial_route
                .waypoints
                .iter()
                .map(|w| w.position())
                .collect::<Vec<Vec2>>();
            let tracking_factor = FactorNode::new_tracking_factor(
                factorgraph.id(),
                Float::from(config.gbp.sigma_factor_tracking),
                array![0.0],
                init_linearisation_point,
                // config.gbp.tracking_smoothing as f64,
                config.gbp.tracking.clone(),
                Some(waypoints.try_into().unwrap()),
                config.gbp.factors_enabled.tracking,
            );

            let factor_node_index = factorgraph.add_factor(tracking_factor);
            let factor_id = FactorId::new(factorgraph.id(), factor_node_index);
            let _ = factorgraph.add_internal_edge(
                // VariableId::new(factorgraph.id(), variable_node_indices[i]),
                VariableId::new(factorgraph.id(), variable_node_indices[i]),
                factor_id,
            );
        }
        // }

        Self {
            factorgraph,
            radius: Radius(radius),
            ball: Ball(parry2d::shape::Ball::new(radius)),
            antenna: RadioAntenna::new(config.robot.communication.radius.get(), true),
            connections: RobotConnections::new(),
            // route,
            // initial_state,
            finished_path: FinishedPath::default(),
            t0: T0(t0),
            gbp_iteration_schedule: GbpIterationSchedule(config.gbp.iteration_schedule),
            // task_state:
            // mission: RobotMission::local(waypoints.try_into().unwrap(), started_at),
            mission,
            // intersects_when,
            planning_strategy,
        }
    }
}

/// Called `Simulator::calculateRobotNeighbours` in **gbpplanner**
fn update_robot_neighbours(
    robots: Query<(Entity, &Transform), With<RobotConnections>>,
    mut query: Query<(Entity, &Transform, &mut RobotConnections)>,
    config: Res<Config>,
) {
    // TODO: use kdtree to speed up, and to have something in the report
    for (robot_id, transform, mut robotstate) in &mut query {
        robotstate.robots_within_comms_range = robots
            .iter()
            .filter_map(|(other_robot_id, other_transform)| {
                if other_robot_id == robot_id
                    || config.robot.communication.radius.get()
                        < transform.translation.distance(other_transform.translation)
                {
                    // Do not compute the distance to self
                    None
                } else {
                    Some(other_robot_id)
                }
            })
            .collect();
    }
}

fn delete_interrobot_factors(mut query: Query<(Entity, &mut FactorGraph, &mut RobotConnections)>) {
    // the set of robots connected with will (possibly) be mutated
    // the robots factorgraph will (possibly) be mutated
    // the other robot with an interrobot factor connected will be mutated

    let mut robots_to_delete_interrobot_factors_between: HashMap<RobotId, RobotId> = HashMap::new();

    for (robot_id, _, mut robotstate) in &mut query {
        let ids_of_robots_connected_with_outside_comms_range: BTreeSet<_> = robotstate
            .robots_connected_with
            .difference(&robotstate.robots_within_comms_range)
            .copied()
            .collect();

        robots_to_delete_interrobot_factors_between.extend(
            ids_of_robots_connected_with_outside_comms_range
                .iter()
                .map(|id| (robot_id, *id)),
        );

        for id in ids_of_robots_connected_with_outside_comms_range {
            robotstate.robots_connected_with.remove(&id);
        }
    }

    for (robot1, robot2) in robots_to_delete_interrobot_factors_between {
        // Will delete both interrobot factors as,
        // (a, b)
        // (a, c)
        // (b, a)
        // (c, a)
        // (b, d)
        // (d, b)
        // etc.
        // deletes a's interrobot factor connecting to b, a -> b
        // deletes b's interrobot factor connecting to a, b -> a

        if let Ok((_, mut factorgraph1, _)) = query.get_mut(robot1) {
            factorgraph1.delete_interrobot_factors_connected_to(robot2);
        } else {
            error!("Could not find robot1 in the query");
        };

        if let Ok((_, mut factorgraph2, _)) = query.get_mut(robot2) {
            factorgraph2.delete_interrobot_factors_connected_to(robot1);
        } else {
            error!(
                "attempt to delete interrobot factors between robots: {:?} and {:?} failed, \
                 reason: {:?} does not exist!",
                robot1, robot2, robot2
            );
        };
    }
}

fn create_interrobot_factors(
    mut query: Query<(Entity, &mut FactorGraph, &mut RobotConnections, &Radius)>,
    config: Res<Config>,
    mut robot_number_gen: ResMut<RobotNumberGenerator>,
) {
    // a mapping between a robot and the other robots it should create a interrobot
    // factor to e.g:
    // {a -> [b, c, d], b -> [a, c], c -> [a, b], d -> [c]}
    let new_connections_to_establish: HashMap<RobotId, Vec<RobotId>> = query
        .iter()
        .map(|(entity, _, robotstate, _)| {
            let new_connections = robotstate
                .robots_within_comms_range
                .difference(&robotstate.robots_connected_with)
                .copied()
                .collect::<Vec<_>>();

            (entity, new_connections)
        })
        .collect();

    // let number_of_variables = variable_timesteps.len();

    // PERF(kpbaks): store a slice instead of a Vec<NodeIndex>
    let variable_indices_of_each_factorgraph: HashMap<RobotId, Vec<NodeIndex>> = query
        .iter()
        .map(|(robot_id, factorgraph, _, _)| {
            let variable_indices = factorgraph
                .variable_indices_ordered_by_creation()
                .skip(1) // skip current variable
                .collect::<Vec<_>>();

            let num_variables = factorgraph.node_count().variables;
            debug_assert_eq!(num_variables - 1, variable_indices.len());
            (robot_id, variable_indices)
        })
        .collect();

    // for (robot_id, node_indices) in &variable_indices_of_each_factorgraph {
    //     println!(
    //         "robot: {:?}, #node_indices = {}",
    //         robot_id,
    //         node_indices.len()
    //     );
    // }
    // debug_assert!(variable_indices_of_each_factorgraph.values().all_equal());

    let mut external_edges_to_add = Vec::new();

    for (robot_id, mut factorgraph, mut robotstate, radius) in &mut query {
        let num_variables = factorgraph.node_count().variables;
        for other_robot_id in new_connections_to_establish
            .get(&robot_id)
            .expect("the key is in the map")
        {
            let other_variable_indices = variable_indices_of_each_factorgraph
                .get(other_robot_id)
                .expect("the key is in the map");

            for i in 1..num_variables {
                let initial_measurement = Vector::<Float>::zeros(DOFS);
                // let eps = 0.2 * config.robot.radius.get();
                // let eps = 0.2 * radius.0;
                // let safety_radius = 2.0f32.mul_add(config.robot.radius.get(), eps);
                // let safety_radius = 2.0f32.mul_add(radius.0, eps);
                // TODO: should it be i - 1 or i?
                let external_variable_id = ExternalVariableId::new(
                    *other_robot_id,
                    VariableIndex(other_variable_indices[i - 1]),
                );
                // let connection =
                //     InterRobotFactorConnection::new(*other_robot_id, other_variable_indices[i
                // - 1]);
                //
                let interrobot_factor = FactorNode::new_interrobot_factor(
                    factorgraph.id(),
                    Float::from(config.gbp.sigma_factor_interrobot),
                    initial_measurement,
                    Float::from(radius.0).try_into().expect("> 0.0"),
                    Float::from(config.robot.inter_robot_safety_distance_multiplier.get())
                        .try_into()
                        .expect("> 0.0"),
                    // Float::from(safety_radius)
                    //     .try_into()
                    //     .expect("safe radius is positive and finite"),
                    external_variable_id,
                    robot_number_gen.next(),
                    config.gbp.factors_enabled.interrobot,
                );

                let factor_index = factorgraph.add_factor(interrobot_factor);

                let variable_index = factorgraph
                    .nth_variable_index(i)
                    .expect("there should be an i'th variable");

                let factor_id = FactorId::new(robot_id, factor_index);
                let graph_id = factorgraph.id();
                factorgraph.add_internal_edge(VariableId::new(graph_id, variable_index), factor_id);
                external_edges_to_add.push((robot_id, factor_index, *other_robot_id, i));
            }

            robotstate.robots_connected_with.insert(*other_robot_id);
        }
    }

    let mut temp = Vec::new();

    for (robot_id, factor_index, other_robot_id, i) in external_edges_to_add {
        // TODO: use query.get_mut()
        let mut other_factorgraph = query
            .iter_mut()
            .find(|(id, _, _, _)| *id == other_robot_id)
            .expect("the other_robot_id should be in the query")
            .1;

        other_factorgraph.add_external_edge(FactorId::new(robot_id, factor_index), i);

        let (nth_variable_index, nth_variable) = other_factorgraph
            .nth_variable(i)
            .expect("the i'th variable should exist");

        let variable_message = nth_variable.prepare_message();
        let variable_id = VariableId::new(other_robot_id, nth_variable_index);

        temp.push((robot_id, factor_index, variable_message, variable_id));
    }

    for (robot_id, factor_index, variable_message, variable_id) in temp {
        // TODO: use query.get_mut()
        let mut factorgraph = query
            .iter_mut()
            .find(|(id, _, _, _)| *id == robot_id)
            .expect("the robot_id should be in the query")
            .1;

        if let Some(factor) = factorgraph.get_factor_mut(factor_index) {
            factor.receive_message_from(variable_id, variable_message.clone());
        } else {
            error!(
                "factorgraph {:?} has no factor with index {:?}",
                robot_id, factor_index
            );
        }
    }
}

/// At random turn on/off the robots "radio".
/// When the radio is turned of the robot will not be able to communicate with
/// any other robot. The probability of failure is set by the user in the config
/// file. `config.robot.communication.failure_rate`
/// Called `Simulator::setCommsFailure` in **gbpplanner**
fn update_failed_comms(
    mut antennas: Query<&mut RadioAntenna>,
    config: Res<Config>,
    mut prng: ResMut<GlobalEntropy<WyRand>>,
) {
    for mut antenna in &mut antennas {
        antenna.active = !prng.gen_bool(config.robot.communication.failure_rate.into());
    }
}

fn iterate_gbp_internal(
    mut query: Query<&mut FactorGraph, With<RobotConnections>>,
    config: Res<Config>,
) {
    query.par_iter_mut().for_each(|mut factorgraph| {
        for _ in 0..config.gbp.iteration_schedule.internal {
            factorgraph.internal_factor_iteration();
            factorgraph.internal_variable_iteration();
        }
    });
}

fn iterate_gbp_internal_sync(
    mut query: Query<&mut FactorGraph, With<RobotConnections>>,
    config: Res<Config>,
) {
    for mut factorgraph in &mut query {
        for _ in 0..config.gbp.iteration_schedule.internal {
            factorgraph.internal_factor_iteration();
            factorgraph.internal_variable_iteration();
        }
    }
}

fn iterate_gbp_external(
    mut query: Query<(Entity, &mut FactorGraph, &RobotConnections, &RadioAntenna)>,
    config: Res<Config>,
) {
    // PERF: use Local<> to reuse arrays
    let messages_to_external_variables: Arc<Mutex<Vec<FactorToVariableMessage>>> =
        Default::default();
    let messages_to_external_factors: Arc<Mutex<Vec<VariableToFactorMessage>>> = Default::default();

    for _ in 0..config.gbp.iteration_schedule.external {
        query
            .par_iter_mut()
            .for_each(|(_, mut factorgraph, state, antenna)| {
                // if !state.interrobot_comms_active {
                if !antenna.active {
                    return;
                }

                let variable_messages = factorgraph.external_factor_iteration();
                if !variable_messages.is_empty() {
                    let mut guard = messages_to_external_variables.lock().expect("not poisoned");
                    guard.extend(variable_messages.into_iter());
                }

                let factor_messages = factorgraph.external_variable_iteration();
                if !factor_messages.is_empty() {
                    let mut guard = messages_to_external_factors.lock().expect("not poisoned");
                    guard.extend(factor_messages.into_iter());
                }
            });

        // Send messages to external variables
        let mut variable_messages = messages_to_external_variables.lock().expect("not poisoned");
        for message in variable_messages.iter() {
            let (_, mut factorgraph, _, _) = query
                .get_mut(message.to.factorgraph_id)
                .expect("the factorgraph of the receiving variable should exist in the world");

            if let Some(variable) = factorgraph.get_variable_mut(message.to.variable_index) {
                variable.receive_message_from(message.from, message.message.clone());
            } else {
                error!(
                    "variablegraph {:?} has no variable with index {:?}",
                    message.to.factorgraph_id, message.to.variable_index
                );
            }
        }

        variable_messages.clear();

        // Send messages to external factors
        let mut factor_messages = messages_to_external_factors.lock().expect("not poisoned");
        for message in factor_messages.iter() {
            let (_, mut factorgraph, _, _) = query
                .get_mut(message.to.factorgraph_id)
                .expect("the factorgraph of the receiving variable should exist in the world");

            if let Some(factor) = factorgraph.get_factor_mut(message.to.factor_index) {
                if !factor.enabled {
                    continue;
                }

                factor.receive_message_from(message.from, message.message.clone());
            }
        }

        factor_messages.clear();
    }
}

fn iterate_gbp_external_sync(
    mut query: Query<(Entity, &mut FactorGraph, &RobotConnections, &RadioAntenna)>,
    config: Res<Config>,
) {
    // PERF: use Local<> to reuse arrays
    let mut messages_to_external_variables: Vec<FactorToVariableMessage> = Default::default();
    let mut messages_to_external_factors: Vec<VariableToFactorMessage> = Default::default();

    for _ in 0..config.gbp.iteration_schedule.external {
        for (_, mut factorgraph, state, antenna) in &mut query {
            // if !state.interrobot_comms_active {
            if !antenna.active {
                return;
            }

            let variable_messages = factorgraph.external_factor_iteration();
            if !variable_messages.is_empty() {
                messages_to_external_variables.extend(variable_messages.into_iter());
                // let mut guard =
                // messages_to_external_variables.lock().expect("not poisoned");
                // guard.extend(variable_messages.into_iter());
            }

            let factor_messages = factorgraph.external_variable_iteration();
            if !factor_messages.is_empty() {
                messages_to_external_factors.extend(factor_messages);
                // let mut guard =
                // messages_to_external_factors.lock().expect("not poisoned");
                // guard.extend(factor_messages.into_iter());
            }
        }

        // Send messages to external variables
        // let mut variable_messages = messages_to_external_variables.lock().expect("not
        // poisoned");
        for message in messages_to_external_variables.iter() {
            let (_, mut factorgraph, _, _) = query
                .get_mut(message.to.factorgraph_id)
                .expect("the factorgraph of the receiving variable should exist in the world");

            if let Some(variable) = factorgraph.get_variable_mut(message.to.variable_index) {
                variable.receive_message_from(message.from, message.message.clone());
            } else {
                error!(
                    "variablegraph {:?} has no variable with index {:?}",
                    message.to.factorgraph_id, message.to.variable_index
                );
            }
        }

        // variable_messages.clear();

        // Send messages to external factors
        // let mut factor_messages = messages_to_external_factors.lock().expect("not
        // poisoned");
        for message in messages_to_external_factors.iter() {
            let (_, mut factorgraph, _, _) = query
                .get_mut(message.to.factorgraph_id)
                .expect("the factorgraph of the receiving variable should exist in the world");

            if let Some(factor) = factorgraph.get_factor_mut(message.to.factor_index) {
                factor.receive_message_from(message.from, message.message.clone());
            }
        }

        messages_to_external_factors.clear();
        messages_to_external_variables.clear();

        // factor_messages.clear();
    }
}

fn iterate_gbp_v2(
    mut query: Query<
        (
            Entity,
            &mut FactorGraph,
            &GbpIterationSchedule,
            &RadioAntenna,
        ),
        With<RobotConnections>,
    >,
    config: Res<Config>,
) {
    let schedule_config = gbp_schedule::GbpScheduleParams {
        internal: config.gbp.iteration_schedule.internal as u8,
        external: config.gbp.iteration_schedule.external as u8,
    };
    let schedule = config.gbp.iteration_schedule.schedule.get(schedule_config);

    for gbp_schedule::GbpScheduleAtIteration { internal, external } in schedule {
        if internal {
            query.par_iter_mut().for_each(|(_, mut factorgraph, _, _)| {
                factorgraph.internal_factor_iteration();
                factorgraph.internal_variable_iteration();
            });
        }

        if external {
            let mut messages_to_external_variables = vec![];
            for (_, mut factorgraph, _, antenna) in query.iter_mut() {
                if !antenna.active {
                    continue;
                }
                messages_to_external_variables
                    .extend(factorgraph.external_factor_iteration().drain(..));
            }

            // Send messages to external variables
            for message in messages_to_external_variables.into_iter() {
                let Ok((_, mut external_factorgraph, _, antenna)) =
                    query.get_mut(message.to.factorgraph_id)
                else {
                    continue;
                };

                // .expect(
                //     "the factorgraph_id of the receiving variable should exist in the world",
                // );

                if !antenna.active {
                    continue;
                }
                if let Some(variable) =
                    external_factorgraph.get_variable_mut(message.to.variable_index)
                {
                    variable.receive_message_from(message.from, message.message);
                }
            }

            let mut messages_to_external_factors = vec![];
            for (_, mut factorgraph, _, antenna) in query.iter_mut() {
                if !antenna.active {
                    continue;
                }
                messages_to_external_factors
                    .extend(factorgraph.external_variable_iteration().drain(..));
            }

            // Send messages to external factors
            for message in messages_to_external_factors.into_iter() {
                let Ok((_, mut external_factorgraph, _, antenna)) =
                    query.get_mut(message.to.factorgraph_id)
                else {
                    continue;
                };

                // .expect("the factorgraph_id of the receiving factor should exist in the
                // world");

                if !antenna.active {
                    continue;
                }

                if let Some(factor) = external_factorgraph.get_factor_mut(message.to.factor_index) {
                    factor.receive_message_from(message.from, message.message);
                }
            }
        }
    }
}

fn iterate_gbp(
    mut query: Query<(Entity, &mut FactorGraph), With<RobotConnections>>,
    config: Res<Config>,
) {
    // let mut  messages_to_external_variables = vec![];

    for _ in 0..config.gbp.iteration_schedule.internal {
        // pretty_print_title!(format!("GBP iteration: {}", i + 1));
        // ╭────────────────────────────────────────────────────────────────────────────────────────
        // │ Factor iteration
        let messages_to_external_variables = query
            .iter_mut()
            .map(|(_, mut factorgraph)| factorgraph.factor_iteration())
            .collect::<Vec<_>>();

        // Send messages to external variables
        for message in messages_to_external_variables.into_iter().flatten() {
            let (_, mut external_factorgraph) = query
                .get_mut(message.to.factorgraph_id)
                .expect("the factorgraph_id of the receiving variable should exist in the world");

            if let Some(variable) = external_factorgraph.get_variable_mut(message.to.variable_index)
            {
                variable.receive_message_from(message.from, message.message);
            } else {
                error!(
                    "variablegraph {:?} has no variable with index {:?}",
                    message.to.factorgraph_id, message.to.variable_index
                );
            }
        }

        // ╭────────────────────────────────────────────────────────────────────────────────────────
        // │ Variable iteration
        let messages_to_external_factors = query
            .iter_mut()
            .map(|(_, mut factorgraph)| factorgraph.variable_iteration())
            .collect::<Vec<_>>();

        // Send messages to external factors
        for message in messages_to_external_factors.into_iter().flatten() {
            let (_, mut external_factorgraph) = query
                .get_mut(message.to.factorgraph_id)
                .expect("the factorgraph_id of the receiving factor should exist in the world");

            if let Some(factor) = external_factorgraph.get_factor_mut(message.to.factor_index) {
                factor.receive_message_from(message.from, message.message);
            }
        }
    }
}

// #[derive(Component, Debug, Default)]
// pub struct FinishedPath(pub bool);

// #[derive(Component, Debug, Default)]
// pub enum FinishedPath {
//     #[default]
//     No,
//     Finished {
//         reported: bool,
//     },
// }

// impl FinishedPath {
//     fn transition(&mut self) {
//         use FinishedPath::{Finished, No};

//         match self {
//             No => {
//                 *self = Finished { reported: false };
//             }
//             Finished { reported: false } => {
//                 *self = Finished { reported: true };
//             }
//             _ => {}
//         }
//     }
// }

// /// Called `Robot::updateHorizon` in **gbpplanner**
// fn update_prior_of_horizon_state_v2(
//     config: Res<Config>,
//     time_fixed: Res<Time<Fixed>>,
//     mut query: Query<(Entity, &mut FactorGraph, &mut Waypoints, &mut
// FinishedPath), With<RobotState>>,     mut evw_robot_despawned:
// EventWriter<RobotDespawned>,     mut evw_robot_finalized_path:
// EventWriter<RobotFinishedPath>,     mut evw_robot_reached_waypoint:
// EventWriter<RobotReachedWaypoint>, ) {
//     // let t_start = std::time::Instant::now();

//     let delta_t = Float::from(time_fixed.delta_seconds());
//     // let robot_radius = config.robot.radius.get();
//     let robot_radius_squared = config.robot.radius.get().powi(2);
//     let max_speed = Float::from(config.robot.max_speed.get());

//     // let mut robots_to_despawn = Vec::new();
//     // let mut all_messages_to_external_factors = Vec::new();
//     let all_messages_to_external_factors: Arc<Mutex<Vec<_>>> =
// Default::default();     let robots_who_reached_waypoint:
// Arc<Mutex<Vec<RobotId>>> = Default::default();

//     query
//         .par_iter_mut()
//         .for_each(|(robot_id, mut factorgraph, mut waypoints, mut
// finished_path)| {             // for (robot_id, mut factorgraph, mut
// waypoints, mut finished_path) in &mut             // query {
//             let FinishedPath::Finished { reported: true } = *finished_path
// else {                 return;
//             };
//             // if finished_path.0 {
//             //     return;
//             // }

//             let Some(current_waypoint) = waypoints.front() else {
//                 // no more waypoints for the robot to move to
//                 // finished_path.0 = true;
//                 *finished_path = FinishedPath::Finished { reported: false };
//                 // robots_to_despawn.push(robot_id);
//                 return;
//             };

//             // 1. update the mean of the horizon variable
//             // 2. find the variable configured to use for the waypoint
// intersection check             let reached_waypoint = {
//                 let variable = match waypoints.intersects_when {
//                     WaypointReachedWhenIntersects::Current =>
// factorgraph.first_variable(),
// WaypointReachedWhenIntersects::Horizon => factorgraph.last_variable(),
//                     WaypointReachedWhenIntersects::Variable(ix) =>
// factorgraph.nth_variable(ix.into()),                 }
//                 .map(|(_, v)| v)
//                 .expect("variable exists");

//                 let estimated_pos = variable.estimated_position_vec2();
//                 // Use square distance comparison to avoid sqrt computation
//                 let dist2waypoint =
// estimated_pos.distance_squared(current_waypoint.xy());
// dist2waypoint < robot_radius_squared             };

//             let (variable_index, horizon_variable) = factorgraph
//                 .last_variable_mut()
//                 .expect("factorgraph has a horizon variable");
//             let estimated_position =
// horizon_variable.belief.mean.slice(s![..2]); // the mean is a 4x1 vector with
// [x, y, x', y']             let current_waypoint =
// array![Float::from(current_waypoint.x), Float::from(current_waypoint.y)];
//             let horizon2waypoint = current_waypoint - estimated_position;
//             let horizon2goal_dist = horizon2waypoint.euclidean_norm();

//             let new_velocity = Float::min(max_speed, horizon2goal_dist) *
// horizon2waypoint.normalized();             let new_position =
// estimated_position.into_owned() + (&new_velocity * delta_t);

//             // Update horizon state with new position and velocity
//             let new_mean = concatenate![Axis(0), new_position, new_velocity];
//             debug_assert_eq!(new_mean.len(), 4);

//             horizon_variable.belief.mean.clone_from(&new_mean);

//             let messages_to_external_factors =
// factorgraph.change_prior_of_variable(variable_index, new_mean);
// // all_messages_to_external_factors.extend(messages_to_external_factors);
//             all_messages_to_external_factors
//                 .lock()
//                 .unwrap()
//                 .extend(messages_to_external_factors);

//             if reached_waypoint && !waypoints.is_empty() {
//                 waypoints.pop_front();
//                 robots_who_reached_waypoint.lock().unwrap().push(robot_id);
//                 // evw_robot_reached_waypoint.send(RobotReachedWaypoint {
//                 //     robot_id,
//                 //     waypoint_index: 0,
//                 // });
//             }
//         });

//     // Send messages to external factors
//     for message in all_messages_to_external_factors.lock().unwrap().iter() {
//         let (_, mut external_factorgraph, _, _) = query
//             .get_mut(message.to.factorgraph_id)
//             .expect("the factorgraph of the receiving factor exists in the
// world");

//         if let Some(factor) =
// external_factorgraph.get_factor_mut(message.to.factor_index) {             //
// PERF: avoid the clone here

//             factor.receive_message_from(message.from,
// message.message.clone());             //
// factor.receive_message_from(message.from,             //
// message.message.acquire());         }
//     }

//     evw_robot_reached_waypoint.send_batch(robots_who_reached_waypoint.lock().
// unwrap().iter().map(|&robot_id| {         RobotReachedWaypoint {
//             robot_id,
//             waypoint_index: 0,
//         }
//     }));

//     for (robot_id, _, _, mut finished_path) in &mut query {
//         match *finished_path {
//             FinishedPath::No | FinishedPath::Finished { reported: true } =>
// {}             FinishedPath::Finished { reported: false } => {
//                 evw_robot_finalized_path.send(RobotFinishedPath(robot_id));
//                 if
// config.simulation.despawn_robot_when_final_waypoint_reached {
// evw_robot_despawned.send(RobotDespawned(robot_id));                 }

//                 *finished_path = FinishedPath::Finished { reported: true };
//             }
//         }
//     }
// }

fn reached_waypoint(
    mut q: Query<(
        Entity,
        &mut FactorGraph,
        &Radius,
        &mut RobotMission,
        //&ReachedWhenIntersects,
        //&PlanningStrategy,
    )>,
    // mut factorgraphs: Query<&mut FactorGraph>,
    config: Res<Config>,
    time: Res<Time>,
    mut evw_robot_reached_waypoint: EventWriter<RobotReachedWaypoint>,
    mut evw_robot_despawned: EventWriter<RobotDespawned>,
    mut evw_robot_finalized_path: EventWriter<RobotFinishedRoute>,
) {
    for (robot_entity, mut fgraph, r, mut mission) in &mut q {
        let Some(next_waypoint) = mission.next_waypoint() else {
            continue;
        };

        let r_sq = r.0 * r.0;
        let reached = {
            use ReachedWhenIntersects::{Current, Horizon, Variable};
            let when_intersects = if mission.next_waypoint_is_last() {
                mission.finished_when_intersects
            } else {
                mission.taskpoint_reached_when_intersects
            };

            let variable = match when_intersects {
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
            // Use square distance comparison to avoid sqrt computation
            let dist2waypoint = estimated_pos.distance_squared(next_waypoint.position());
            dist2waypoint < r_sq
        };

        if reached {
            mission.advance_to_next_waypoint(&time);
            evw_robot_reached_waypoint.send(RobotReachedWaypoint {
                robot_id:       robot_entity,
                waypoint_index: 0,
            });

            error!("robot: {:?} reached a waypoint", robot_entity);

            if let Some(current_waypoint_index) = mission.current_waypoint_index() {
                error!(
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

#[derive(Component, Debug, Default)]
pub struct FinishedPath(pub bool);

/// Called `Robot::updateHorizon` in **gbpplanner**
fn update_prior_of_horizon_state(
    config: Res<Config>,
    // time_virtual: Res<Time<Virtual>>,
    time: Res<Time>,
    mut query: Query<
        (
            Entity,
            &mut FactorGraph,
            // &mut Route,
            &RobotMission,
            &mut FinishedPath,
            &Radius,
            // &GbpIterationSchedule,
        ),
        With<RobotConnections>,
    >,
    mut evw_robot_despawned: EventWriter<RobotDespawned>,
    mut evw_robot_finalized_path: EventWriter<RobotFinishedRoute>,
    mut evw_robot_reached_waypoint: EventWriter<RobotReachedWaypoint>,
    // PERF: we reuse the same vector between system calls
    // the vector is cleared between calls, by calling .drain(..) at the end of every call
    mut all_messages_to_external_factors: Local<Vec<VariableToFactorMessage>>,
) {
    // println!("dt: {}", time.delta_seconds());
    let delta_t = Float::from(time.delta_seconds());
    let max_speed = Float::from(config.robot.max_speed.get());

    let mut robots_to_despawn = Vec::new();

    for (robot_id, mut factorgraph, mission, mut finished_path, radius) in &mut query {
        if finished_path.0 || mission.state.idle() {
            continue;
        }

        let Some(next_waypoint) = mission.next_waypoint() else {
            // no more waypoints for the robot to move to
            info!(
                "robot {:?} finished at {:?}",
                robot_id,
                mission.finished_at()
            );
            // dbg!(&mission);
            finished_path.0 = true;
            robots_to_despawn.push(robot_id);
            continue;
        };

        if config.gbp.iteration_schedule.internal == 0 {
            continue;
        }

        // let robot_radius_squared = radius.0.powi(2);

        // 1. update the mean of the horizon variable
        // 2. find the variable configured to use for the waypoint intersection check
        // let reached_waypoint = {
        //     let variable = match route.intersects_when {
        //         WaypointReachedWhenIntersects::Current =>
        // factorgraph.first_variable(),
        //         WaypointReachedWhenIntersects::Horizon =>
        // factorgraph.last_variable(),
        //         WaypointReachedWhenIntersects::Variable(ix) =>
        // factorgraph.nth_variable(ix.into()),     }
        //     .map(|(_, v)| v)
        //     .expect("variable exists");
        //
        //     let estimated_pos = variable.estimated_position_vec2();
        //     // Use square distance comparison to avoid sqrt computation
        //     let dist2waypoint =
        // estimated_pos.distance_squared(next_waypoint.position());
        //     dist2waypoint < robot_radius_squared
        // };

        let (horizon_variable_index, horizon_variable) = factorgraph
            .last_variable_mut()
            .expect("factorgraph has a horizon variable");
        let estimated_position = horizon_variable.belief.mean.slice(s![..2]); // the mean is a 4x1 vector with [x, y, x', y']

        let next_waypoint_pos = array![
            Float::from(next_waypoint.position().x),
            Float::from(next_waypoint.position().y)
        ];

        let horizon2waypoint = next_waypoint_pos - estimated_position;
        let horizon2goal_dist = horizon2waypoint.euclidean_norm();

        let new_velocity = Float::min(max_speed, horizon2goal_dist) * horizon2waypoint.normalized();
        let new_position = estimated_position.into_owned() + (&new_velocity * delta_t);

        // Update horizon state with new position and velocity
        let new_mean = concatenate![Axis(0), new_position, new_velocity];
        // debug_assert_eq!(new_mean.len(), 4);

        horizon_variable.belief.mean.clone_from(&new_mean);

        let messages_to_external_factors =
            factorgraph.change_prior_of_variable(horizon_variable_index, new_mean);
        all_messages_to_external_factors.extend(messages_to_external_factors);

        // if reached_waypoint && !route.is_completed() {
        //     route.advance(time.elapsed());
        //     evw_robot_reached_waypoint.send(RobotReachedWaypoint {
        //         robot_id,
        //         waypoint_index: 0,
        //     });
        // }
    }

    // Send messages to external factors
    for message in all_messages_to_external_factors.drain(..) {
        let Ok((_, mut external_factorgraph, _, _, _)) = query.get_mut(message.to.factorgraph_id)
        else {
            continue;
        };
        // .expect("the factorgraph of the receiving factor exists in the world");

        if let Some(factor) = external_factorgraph.get_factor_mut(message.to.factor_index) {
            factor.receive_message_from(message.from, message.message);
        }
    }

    // if !robots_to_despawn.is_empty() {
    //     evw_robot_finalized_path
    //         .send_batch(robots_to_despawn.iter().copied().
    // map(RobotFinishedRoute));     if config.simulation.
    // despawn_robot_when_final_waypoint_reached {
    //         evw_robot_despawned.send_batch(robots_to_despawn.into_iter().
    // map(RobotDespawned));     }
    // }
}

/// Called `Robot::updateCurrent` in **gbpplanner**
fn update_prior_of_current_state_v3(
    mut query: Query<
        (&mut FactorGraph, &mut Transform, &T0, &RobotMission),
        With<RobotConnections>,
    >,
    config: Res<Config>,
    time_fixed: Res<Time<Fixed>>,
) {
    // let time_scale = time_fixed.delta_seconds() / config.simulation.t0.get();

    let mut messages_to_external_factors: Vec<FactorToVariableMessage> = vec![];

    for (mut factorgraph, mut transform, &t0, mission) in &mut query {
        if mission.state.idle() {
            continue;
        }

        let time_scale = time_fixed.delta_seconds() / *t0;
        let (current_variable_index, current_variable) = factorgraph
            .nth_variable(0)
            .expect("factorgraph should have a current variable");
        let (_, next_variable) = factorgraph
            .nth_variable(1)
            .expect("factorgraph should have a next variable");

        let change_in_state =
            Float::from(time_scale) * (&next_variable.belief.mean - &current_variable.belief.mean);
        let mean_updated = &current_variable.belief.mean + &change_in_state;

        let external_factor_messages =
            factorgraph.change_prior_of_variable(current_variable_index, mean_updated);
        assert!(
            external_factor_messages.is_empty(),
            "the current variable is not connected to any external factors"
        );
        // messages_to_external_factors.extend(external_factor_messages);

        #[allow(clippy::cast_possible_truncation)]
        // bevy uses xzy coordinates, so the y component is put at the z coordinate
        let position_increment =
            Vec3::new(change_in_state[0] as f32, 0.0, change_in_state[1] as f32);

        transform.translation += position_increment;
    }

    // if !messages_to_external_factors.is_empty() {
    //     error!(
    //         "current prior sending messages to {:?}",
    //         messages_to_external_factors
    //     );
    // }
    //
    // // let guard = messages_to_external_factors.lock().unwrap();
    // // messages_to_external_factors.dra
    // for message in messages_to_external_factors.drain(..) {
    //     let (mut external_factorgraph, _) = query
    //         .get_mut(message.to.factorgraph_id)
    //         .expect("the factorgraph of the receiving factor exists in the
    // world");
    //
    //     if let Some(factor) =
    // external_factorgraph.get_factor_mut(message.to.factor_index) {
    //         error!("current prior sending message to {:?}", message.to);
    //         factor.receive_message_from(message.from, message.message);
    //     }
    // }
}

// /// Called `Robot::updateCurrent` in **gbpplanner**
// fn update_prior_of_current_state_v2(
//     mut query: Query<(&mut FactorGraph, &mut Transform), With<RobotState>>,
//     config: Res<Config>,
//     time_fixed: Res<Time<Fixed>>,
//     messages_to_external_factors: Local<Mutex<Vec<VariableToFactorMessage>>>,
// ) {
//     let time_scale = time_fixed.delta_seconds() / config.simulation.t0.get();
//     messages_to_external_factors.lock().unwrap().clear();
//
//     // let messages_to_external_factors:
// Arc<Mutex<Vec<VariableToFactorMessage>>> =     // Arc::default();
//     // let messages_to_external_factors: Mutex<Vec<VariableToFactorMessage>>
// =     // Default::default();
//
//     query
//         .par_iter_mut()
//         .for_each(|(mut factorgraph, mut transform)| {
//             let (current_variable_index, current_variable) = factorgraph
//                 .nth_variable(0)
//                 .expect("factorgraph should have a current variable");
//             let (_, next_variable) = factorgraph
//                 .nth_variable(1)
//                 .expect("factorgraph should have a next variable");
//
//             let change_in_state = Float::from(time_scale)
//                 * (&next_variable.belief.mean -
//                   &current_variable.belief.mean);
//             let mean_updated = &current_variable.belief.mean +
// &change_in_state;
//
//             let external_factor_messages =
//                 factorgraph.change_prior_of_variable(current_variable_index,
// mean_updated);
//
//             let mut guard = messages_to_external_factors.lock().unwrap();
//             guard.extend(external_factor_messages);
//
//             #[allow(clippy::cast_possible_truncation)]
//             // bevy uses xzy coordinates, so the y component is put at the z
// coordinate             let position_increment =
//                 Vec3::new(change_in_state[0] as f32, 0.0, change_in_state[1]
// as f32);
//
//             transform.translation += position_increment;
//         });
//
//     let guard = messages_to_external_factors.lock().unwrap();
//     for message in guard.iter() {
//         let (mut external_factorgraph, _) = query
//             .get_mut(message.to.factorgraph_id)
//             .expect("the factorgraph of the receiving factor exists in the
// world");
//
//         if let Some(factor) =
// external_factorgraph.get_factor_mut(message.to.factor_index) {             //
// PERF: avoid the clone here             error!("current prior sending message
// to {:?}", message.to);             factor.receive_message_from(message.from,
// message.message.clone());         }
//     }
// }

#[derive(Component, Deref, Clone, Copy)]
pub struct T0(pub f32);

/// Called `Robot::updateCurrent` in **gbpplanner**
fn update_prior_of_current_state(
    mut query: Query<(&mut FactorGraph, &mut Transform, &T0), With<RobotConnections>>,
    config: Res<Config>,
    time: Res<Time>,
) {
    // let scale = time.delta_seconds() / config.simulation.t0.get();

    for (mut factorgraph, mut transform, &t0) in &mut query {
        let scale: f32 = time.delta_seconds() / *t0;
        let (current_variable_index, current_variable) = factorgraph
            .nth_variable(0)
            .expect("factorgraph should have a current variable");
        let (_, next_variable) = factorgraph
            .nth_variable(1)
            .expect("factorgraph should have a next variable");
        let mean_of_current_variable = current_variable.belief.mean.clone();
        let change_in_state =
            Float::from(scale) * (&next_variable.belief.mean - &mean_of_current_variable);

        let messages = factorgraph.change_prior_of_variable(
            current_variable_index,
            &mean_of_current_variable + &change_in_state,
        );

        if !messages.is_empty() {
            error!(
                "{} messages from update_prior_of_current_state:",
                messages.len()
            );
            // continue;
        }

        #[allow(clippy::cast_possible_truncation)]
        let position_increment =
            Vec3::new(change_in_state[0] as f32, 0.0, change_in_state[1] as f32);

        transform.translation += position_increment;
    }
}

// boolean_bevy_resource!(ManualMode, default = false);

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ManualModeState {
    #[default]
    Disabled,
    Enabled {
        iterations_remaining: usize,
    },
}

impl ManualModeState {
    #[inline]
    pub fn enabled(state: Res<State<Self>>) -> bool {
        matches!(state.get(), Self::Enabled { .. })
    }

    #[inline]
    pub fn disabled(state: Res<State<Self>>) -> bool {
        matches!(state.get(), Self::Disabled)
    }
}

fn start_manual_step(
    config: Res<Config>,
    manual_mode_state: Res<State<ManualModeState>>,
    mut next_manual_mode_state: ResMut<NextState<ManualModeState>>,
    mut evr_keyboard_input: EventReader<KeyboardInput>,
    mut evw_pause_play: EventWriter<PausePlay>,
) {
    for event in evr_keyboard_input.read() {
        let (KeyCode::KeyM, ButtonState::Pressed) = (event.key_code, event.state) else {
            continue;
        };

        match manual_mode_state.get() {
            ManualModeState::Disabled => {
                next_manual_mode_state.set(ManualModeState::Enabled {
                    iterations_remaining: config.manual.timesteps_per_step.into(),
                });
                evw_pause_play.send(PausePlay::Play);
            }
            ManualModeState::Enabled { .. } => {
                warn!("manual step already in progress");
            }
        }
    }
}

fn finish_manual_step(
    // mut mode: ResMut<ManualMode>,
    state: Res<State<ManualModeState>>,
    mut next_state: ResMut<NextState<ManualModeState>>,
    mut pause_play_event: EventWriter<PausePlay>,
) {
    match state.get() {
        ManualModeState::Enabled {
            iterations_remaining,
        } if (0..=1).contains(iterations_remaining) => {
            next_state.set(ManualModeState::Disabled);
            pause_play_event.send(PausePlay::Pause);
        }
        ManualModeState::Enabled {
            iterations_remaining,
        } => {
            next_state.set(ManualModeState::Enabled {
                iterations_remaining: iterations_remaining - 1,
            });
        }
        ManualModeState::Disabled => {
            error!("manual step not in progress");
        }
    };
}

/// Event handler called whenever the mesh of a robot is clicked
/// Meant for debugging purposes
fn on_robot_clicked(
    mut evr_robot_clicked_on: EventReader<RobotClickedOn>,
    robots: Query<(
        Entity,
        &Transform,
        &FactorGraph,
        &RobotConnections,
        &Radius,
        &Ball,
        &RadioAntenna,
        &RobotMission,
        &PlanningStrategy,
    )>,
    robot_robot_collisions: Res<RobotRobotCollisions>,
    robot_environment_collisions: Res<RobotEnvironmentCollisions>,
) {
    use colored::Colorize;
    for RobotClickedOn(robot_id) in evr_robot_clicked_on.read() {
        let Ok((
            _,
            transform,
            factorgraph,
            robotstate,
            radius,
            ball,
            antenna,
            mission,
            planning_strategy,
        )) = robots.get(*robot_id)
        else {
            error!("robot_id {:?} does not exist", robot_id);
            continue;
        };

        println!("----- robot cliked on -----");
        println!("{}: {:?}", "robot".blue(), robot_id);
        println!("  {}: {}", "radius".magenta(), radius.0);
        println!("  {}:", "antenna".magenta());
        println!("    {}: {}", "radius".cyan(), antenna.radius);
        println!(
            "    {}: {}",
            "on".cyan(),
            if antenna.active {
                "true".green()
            } else {
                "false".red()
            }
        );
        println!("  {}:", "state".magenta());
        let (_, current_variable) = factorgraph
            .first_variable()
            .expect("factorgraph should have >= 2 variables");
        let [px, py] = current_variable.estimated_position();
        let [vx, vy] = current_variable.estimated_velocity();
        println!("    {}: [{:.4}, {:.4}]", "position".cyan(), px, py);
        println!("    {}: [{:.4}, {:.4}]", "velocity".cyan(), vx, vy);

        println!(
            "    {}: {:?}",
            "neighbours".cyan(),
            robotstate.robots_within_comms_range
        );
        println!(
            "    {}: {:?}",
            "connected".cyan(),
            robotstate.robots_connected_with
        );
        let node_counts = factorgraph.node_count();
        let edge_count = factorgraph.edge_count();
        println!("  {}:", "factorgraph".magenta());
        println!("    {}: {}", "edges".cyan(), edge_count);
        println!("    {}: {}", "nodes".cyan(), node_counts.total());
        println!(
            "      {}: {}",
            "variable".red(),
            node_counts.variables.to_string().red()
        );
        println!(
            "      {}:  {}",
            "factors".red(),
            node_counts.factors.to_string().blue()
        );
        let factor_counts = factorgraph.factor_count();
        println!(
            "        {}: {}",
            "obstacle".yellow(),
            factor_counts.obstacle
        );
        println!("        {}: {}", "dynamic".yellow(), factor_counts.dynamic);
        println!(
            "        {}: {}",
            "interrobot".yellow(),
            factor_counts.interrobot
        );
        println!("        {}: {}", "pose".yellow(), node_counts.variables); // bundled together
        println!(
            "        {}: {}",
            "tracking".yellow(),
            factor_counts.tracking
        );

        println!("  {}:", "messages".magenta());
        // let message_count = factorgraph.message_count();
        let messages_sent = factorgraph.messages_sent();
        let messages_received = factorgraph.messages_received();
        println!("    {}:", "sent".cyan());
        println!("      {}: {}", "internal".red(), messages_sent.internal);
        println!("      {}: {}", "external".red(), messages_sent.external);
        println!("    {}:", "received".cyan());
        println!("      {}: {}", "internal".red(), messages_received.internal);
        println!("      {}: {}", "external".red(), messages_received.external);
        println!("  {}:", "collisions".magenta());
        let robot_collisions = robot_robot_collisions.get(*robot_id).unwrap_or(0);
        println!("    {}: {}", "other-robots".cyan(), robot_collisions);
        let env_collisions = robot_environment_collisions.get(*robot_id).unwrap_or(0);
        println!("    {}: {}", "environment".cyan(), env_collisions);
        println!("  {}:", "aabb".magenta());
        let position =
            parry2d::na::Isometry2::translation(transform.translation.x, transform.translation.z);
        let aabb = ball.aabb(&position);
        println!(
            "    {}: [{:.4}, {:.4}]",
            "min".cyan(),
            aabb.mins.x,
            aabb.mins.y
        );
        println!(
            "    {}: [{:.4}, {:.4}]",
            "max".cyan(),
            aabb.maxs.x,
            aabb.maxs.y
        );
        println!("    {}: {:.4} m^2", "area".cyan(), aabb.volume());
        println!(
            "  {}: {:?}",
            "planning_strategy".magenta(),
            planning_strategy
        );
        println!("  {}:", "mission".magenta());
        println!("    {}:", "taskpoints".cyan());
        mission.taskpoints.iter().for_each(|wp| {
            println!("      - {}", wp);
        });
        println!("    {}:", "routes".cyan());
        mission.routes.iter().for_each(|route| {
            println!("      - {}: {}", "target_index".red(), route.target_index);
            println!("      - {}:", "waypoints".red());
            route.waypoints.iter().for_each(|wp| {
                println!("        - {}", wp);
            });
        });
        println!("    {}: {}s", "started_at".cyan(), mission.started_at());
        println!("    {}: {:?}", "finished_at".cyan(), mission.finished_at());
        println!("    {}: {:?}", "state".cyan(), mission.state);
        println!("    {}: {}", "active_route".cyan(), mission.active_route);
        println!(
            "    {}: {:?}",
            "finished_when_intersects".cyan(),
            mission.finished_when_intersects
        );
        println!(
            "    {}: {:?}",
            "taskpoint_reached_when_intersects".cyan(),
            mission.taskpoint_reached_when_intersects
        );

        // println!("    {}: {}", "active_route".cyan(), mission.)

        // println!("{:#?}", mission);
    }
}

fn on_gbp_schedule_changed(
    mut evr_gbp_schedule_changed: EventReader<GbpScheduleChanged>,
    mut q_robots: Query<(Entity, &mut GbpIterationSchedule)>,
) {
    for GbpScheduleChanged(new_schedule) in evr_gbp_schedule_changed.read() {
        for (entity, mut schedule) in q_robots.iter_mut() {
            *schedule = *new_schedule;
            info!(
                "changed gbp-schedule to: {:?} of entity {:?}",
                new_schedule, entity
            );
            // *schedule = GbpIterationSchedule(*new_schedule.into());
        }
    }
}
