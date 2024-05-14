use std::{
    collections::{BTreeSet, HashMap},
    num::NonZeroUsize,
    sync::{Arc, Mutex},
    time::Duration,
};

use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use bevy_prng::WyRand;
use bevy_rand::prelude::{EntropyComponent, ForkableRng, GlobalEntropy};
use derive_more::Index;
use gbp_linalg::prelude::*;
use gbp_schedule::GbpSchedule;
use itertools::Itertools;
use ndarray::{array, concatenate, s, Axis};
use rand::{thread_rng, Rng};

use super::{
    collisions::resources::{RobotEnvironmentCollisions, RobotRobotCollisions},
    spawner::RobotClickedOn,
};
use crate::{
    bevy_utils::run_conditions::time::virtual_time_is_paused,
    config::{formation::WaypointReachedWhenIntersects, Config},
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
    utils::get_variable_timesteps,
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
                ),
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

impl From<crate::config::GbpIterationSchedule> for GbpScheduleChanged {
    fn from(schedule: crate::config::GbpIterationSchedule) -> Self {
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
) {
    let duration = Duration::from_secs(2);
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
    waypoints: Vec<StateVector>,
    /// The index of the next target waypoint in the waypoints vector.
    target_index: usize,
    /// Criteria determining when the robot is considered to have reached a
    /// waypoint.
    pub intersects_when: WaypointReachedWhenIntersects,
    /// The recorded time at the start of the route as a floating-point
    /// timestamp.
    started_at: f64,
    /// Optional recorded time when the route was completed as a floating-point
    /// timestamp.
    finished_at: Option<f64>,
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
        intersects_when: WaypointReachedWhenIntersects,
        started_at: f64,
    ) -> Self {
        Self {
            waypoints: waypoints.into(),
            target_index: 1, // skip first waypoint, as it is the initial pose
            intersects_when,
            started_at,
            finished_at: None,
        }
    }

    /// Returns a reference to the next waypoint, if available.
    pub fn next_waypoint(&self) -> Option<&StateVector> {
        self.waypoints.get(self.target_index)
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
            self.finished_at = Some(elapsed.as_secs_f64() - self.started_at);
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
#[derive(Component, Debug)]
pub struct RobotState {
    /// List of robot ids that are within the communication radius of this
    /// robot. called `neighbours_` in **gbpplanner**.
    pub ids_of_robots_within_comms_range: BTreeSet<RobotId>,
    /// List of robot ids that are currently connected via inter-robot factors
    /// to this robot called `connected_r_ids_` in **gbpplanner**.
    pub ids_of_robots_connected_with:     BTreeSet<RobotId>,
}

impl RobotState {
    /// Create a new `RobotState`
    #[must_use]
    pub fn new() -> Self {
        Self {
            ids_of_robots_within_comms_range: BTreeSet::new(),
            ids_of_robots_connected_with:     BTreeSet::new(),
        }
    }
}

impl Default for RobotState {
    fn default() -> Self {
        Self::new()
    }
}

// TODO: change to collider
#[derive(Debug, Component, Deref)]
pub struct Ball(parry2d::shape::Ball);

// #[derive(Component)]
// pub struct GbpIterationSchedule {
//     pub external_iterations: u32,
//     pub internal_iterations: u32,
//     pub schedule: Arc<dyn gbp_schedule::GbpSchedule>,
// }

#[derive(Clone, Copy, Debug, Component, Resource, derive_more::Into, derive_more::From)]
pub struct GbpIterationSchedule(pub crate::config::GbpIterationSchedule);

impl GbpIterationSchedule {
    pub fn schedule(&self) -> Box<dyn gbp_schedule::GbpScheduleIterator> {
        let config = gbp_schedule::GbpScheduleConfig {
            internal: self.0.internal as u8,
            external: self.0.external as u8,
        };
        self.0.schedule.get(config)
    }
}

// {
//     kind:       crate::config::GbpIterationScheduleKind,
//     iterations: crate::config::GbpIterationSchedule,
// }

// #[derive(Component, Resource)]
// pub struct GbpIterationSchedule {
//     kind:       crate::config::GbpIterationScheduleKind,
//     iterations: crate::config::GbpIterationSchedule,
// }

impl FromWorld for GbpIterationSchedule {
    fn from_world(world: &mut World) -> Self {
        if let Some(config) = world.get_resource::<Config>() {
            Self(config.gbp.iteration_schedule)
        } else {
            Self(crate::config::GbpIterationSchedule::default())
        }
    }
}

// impl Default for GbpIterationSchedule {
//     fn default() -> Self {
//         Self {
//             kind:
// crate::config::GbpIterationScheduleKind::SoonAsPossible,
// iterations: crate::config::GbpIterationsPerTimestep {
// internal: 10,                 external: 10,
//             },
//         }
//     }
// }

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
    pub state: RobotState,
    /// Waypoints used to instruct the robot to move to a specific position.
    /// A `VecDeque` is used to allow for efficient `pop_front` operations, and
    /// `push_back` operations.
    pub route: Route,

    /// Initial sate
    pub initial_state: StateVector,

    /// Time between t_i and t_i+1
    pub t0: T0,

    // pub variable_timesteps: VariableTimesteps,
    /// Boolean component used to keep track of whether the robot has finished
    /// its path by reaching its final waypoint. This flag exists to ensure
    /// that the robot is not detected as having finished more than once.
    /// TODO: should probably be modelled as an enum instead, too easier support
    /// additional states in the future
    finished_path: FinishedPath,
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
    ///
    /// # Errors
    ///
    /// Will return `Err` if:
    /// - `waypoints` is empty
    /// - `variable_timesteps` is empty
    #[must_use = "Constructor responsible for creating the robots factorgraph"]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(
        robot_id: RobotId,
        initial_state: StateVector,
        route: Route,
        variable_timesteps: &[u32],
        // variable_timesteps: Vec<u32>,
        // variable_timesteps: VariableTimesteps,
        config: &Config,
        env_config: &gbp_environment::Environment,
        radius: f32,
        sdf: &SdfImage,
        use_tracking: bool,
    ) -> Self {
        assert!(
            !variable_timesteps.is_empty(),
            "Variable timesteps cannot be empty"
        );

        let start: Vec4 = route.first().to_owned().into();

        let next_waypoint: Vec4 = route[1].into();

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
            #[allow(clippy::cast_precision_loss)]
            let mean = start
                + (horizon - start) * (variable_timestep as f32 / last_variable_timestep as f32);

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

        #[allow(clippy::needless_range_loop)]
        for i in 1..variable_timesteps.len() - 1 {
            let obstacle_factor = FactorNode::new_obstacle_factor(
                factorgraph.id(),
                Float::from(config.gbp.sigma_factor_obstacle),
                array![0.0],
                sdf.clone(),
                world_size,
                // crate::factorgraph::factor::obstacle::WorldSize {
                //     width: env_config.tiles.settings.tile_size *
                //     height: todo!(),
                // },
                // Float::from(config.simulation.world_size.get()),
            );

            let factor_node_index = factorgraph.add_factor(obstacle_factor);
            let factor_id = FactorId::new(factorgraph.id(), factor_node_index);
            let _ = factorgraph.add_internal_edge(
                VariableId::new(factorgraph.id(), variable_node_indices[i]),
                factor_id,
            );
        }

        // Create Tracking factors for all variables, excluding the start
        if use_tracking {
            for i in 1..variable_timesteps.len() {
                let tracking_factor = FactorNode::new_tracking_factor(
                    factorgraph.id(),
                    Float::from(config.gbp.sigma_factor_tracking),
                    // init_variable_means[i].clone(),
                    array![0.0],
                    route
                        .waypoints()
                        .iter()
                        .map(|wp| wp.position())
                        .collect::<Vec<Vec2>>(),
                );

                let factor_node_index = factorgraph.add_factor(tracking_factor);
                let factor_id = FactorId::new(factorgraph.id(), factor_node_index);
                let _ = factorgraph.add_internal_edge(
                    VariableId::new(factorgraph.id(), variable_node_indices[i]),
                    factor_id,
                );
            }
        }

        Self {
            factorgraph,
            radius: Radius(radius),
            ball: Ball(parry2d::shape::Ball::new(radius)),
            antenna: RadioAntenna::new(config.robot.communication.radius.get(), true),
            state: RobotState::new(),
            route,
            initial_state,
            finished_path: FinishedPath::default(),
            t0: T0(t0),
            gbp_iteration_schedule: GbpIterationSchedule(config.gbp.iteration_schedule),
        }
    }
}

/// Called `Simulator::calculateRobotNeighbours` in **gbpplanner**
fn update_robot_neighbours(
    robots: Query<(Entity, &Transform), With<RobotState>>,
    mut query: Query<(Entity, &Transform, &mut RobotState)>,
    config: Res<Config>,
) {
    // TODO: use kdtree to speed up, and to have something in the report
    for (robot_id, transform, mut robotstate) in &mut query {
        robotstate.ids_of_robots_within_comms_range = robots
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

fn delete_interrobot_factors(mut query: Query<(Entity, &mut FactorGraph, &mut RobotState)>) {
    // the set of robots connected with will (possibly) be mutated
    // the robots factorgraph will (possibly) be mutated
    // the other robot with an interrobot factor connected will be mutated

    let mut robots_to_delete_interrobot_factors_between: HashMap<RobotId, RobotId> = HashMap::new();

    for (robot_id, _, mut robotstate) in &mut query {
        let ids_of_robots_connected_with_outside_comms_range: BTreeSet<_> = robotstate
            .ids_of_robots_connected_with
            .difference(&robotstate.ids_of_robots_within_comms_range)
            .copied()
            .collect();

        robots_to_delete_interrobot_factors_between.extend(
            ids_of_robots_connected_with_outside_comms_range
                .iter()
                .map(|id| (robot_id, *id)),
        );

        for id in ids_of_robots_connected_with_outside_comms_range {
            robotstate.ids_of_robots_connected_with.remove(&id);
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

        // let mut factorgraph1 = query
        //     .iter_mut()
        //     .find(|(id, _, _)| *id == robot1)
        //     .expect("the robot1 should be in the query")
        //     .1;

        //         match factorgraph1.delete_interrobot_factors_connected_to(robot2) {

        //     Ok(()) => (),
        //     // info!(
        //     //     "Deleted interrobot factor between factorgraph {:?} and {:?}",
        //     //     robot1, robot2
        //     // ),
        //     Err(err) => {
        //         error!(
        //             "Could not delete interrobot factor between {:?} -> {:?}, with
        // error msg: {}",             robot1, robot2, err
        //         );
        //     }
        // }

        if let Ok((_, mut factorgraph2, _)) = query.get_mut(robot2) {
            factorgraph2.delete_interrobot_factors_connected_to(robot1);
        } else {
            error!(
                "attempt to delete interrobot factors between robots: {:?} and {:?} failed, \
                 reason: {:?} does not exist!",
                robot1, robot2, robot2
            );
        };

        // let Some((_, mut factorgraph2, _)) = query
        //     .iter_mut()
        //     .find(|(robot_id, _, _)| *robot_id == robot2)
        // else {
        //     error!(
        //         "attempt to delete interrobot factors between robots: {:?}
        // and {:?} failed, \          reason: {:?} does not exist!",
        //         robot1, robot2, robot2
        //     );
        //     continue;
        // };
        //
        // factorgraph2.delete_messages_from_interrobot_factor_at(robot1);
    }
}

fn create_interrobot_factors(
    mut query: Query<(Entity, &mut FactorGraph, &mut RobotState, &Radius)>,
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
                .ids_of_robots_within_comms_range
                .difference(&robotstate.ids_of_robots_connected_with)
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
    debug_assert!(variable_indices_of_each_factorgraph.values().all_equal());

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

            robotstate
                .ids_of_robots_connected_with
                .insert(*other_robot_id);
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

fn iterate_gbp_internal(mut query: Query<&mut FactorGraph, With<RobotState>>, config: Res<Config>) {
    query.par_iter_mut().for_each(|mut factorgraph| {
        for _ in 0..config.gbp.iteration_schedule.internal {
            factorgraph.internal_factor_iteration();
            factorgraph.internal_variable_iteration();
        }
    });
}

fn iterate_gbp_internal_sync(
    mut query: Query<&mut FactorGraph, With<RobotState>>,
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
    mut query: Query<(Entity, &mut FactorGraph, &RobotState, &RadioAntenna)>,
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
                factor.receive_message_from(message.from, message.message.clone());
            }
        }

        factor_messages.clear();
    }
}

fn iterate_gbp_external_sync(
    mut query: Query<(Entity, &mut FactorGraph, &RobotState, &RadioAntenna)>,
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
        With<RobotState>,
    >,
    config: Res<Config>,
) {
    let schedule_config = gbp_schedule::GbpScheduleConfig {
        internal: config.gbp.iteration_schedule.internal as u8,
        external: config.gbp.iteration_schedule.external as u8,
    };
    let schedule = config.gbp.iteration_schedule.schedule.get(schedule_config);

    for gbp_schedule::GbpScheduleAtTimestep { internal, external } in schedule {
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
                let (_, mut external_factorgraph, _, antenna) =
                    query.get_mut(message.to.factorgraph_id).expect(
                        "the factorgraph_id of the receiving variable should exist in the world",
                    );

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
                let (_, mut external_factorgraph, _, antenna) = query
                    .get_mut(message.to.factorgraph_id)
                    .expect("the factorgraph_id of the receiving factor should exist in the world");

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
    mut query: Query<(Entity, &mut FactorGraph), With<RobotState>>,
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

#[derive(Component, Debug, Default)]
pub struct FinishedPath(pub bool);

/// Called `Robot::updateHorizon` in **gbpplanner**
fn update_prior_of_horizon_state(
    config: Res<Config>,
    time_virtual: Res<Time<Virtual>>,
    mut query: Query<
        (
            Entity,
            &mut FactorGraph,
            &mut Route,
            &mut FinishedPath,
            &Radius,
            // &GbpIterationSchedule,
        ),
        With<RobotState>,
    >,
    mut evw_robot_despawned: EventWriter<RobotDespawned>,
    mut evw_robot_finalized_path: EventWriter<RobotFinishedRoute>,
    mut evw_robot_reached_waypoint: EventWriter<RobotReachedWaypoint>,
    // PERF: we reuse the same vector between system calls
    // the vector is cleared between calls, by calling .drain(..) at the end of every call
    mut all_messages_to_external_factors: Local<Vec<VariableToFactorMessage>>,
) {
    let delta_t = Float::from(time_virtual.delta_seconds());
    let max_speed = Float::from(config.robot.max_speed.get());

    let mut robots_to_despawn = Vec::new();

    for (robot_id, mut factorgraph, mut route, mut finished_path, radius) in &mut query {
        if finished_path.0 {
            continue;
        }

        let Some(next_waypoint) = route.next_waypoint() else {
            // no more waypoints for the robot to move to
            info!("robot {:?} finished at {:?}", robot_id, route.finished_at());
            finished_path.0 = true;
            robots_to_despawn.push(robot_id);
            continue;
        };

        if config.gbp.iteration_schedule.internal == 0 {
            continue;
        }

        let robot_radius_squared = radius.0.powi(2);

        // 1. update the mean of the horizon variable
        // 2. find the variable configured to use for the waypoint intersection check
        let reached_waypoint = {
            let variable = match route.intersects_when {
                WaypointReachedWhenIntersects::Current => factorgraph.first_variable(),
                WaypointReachedWhenIntersects::Horizon => factorgraph.last_variable(),
                WaypointReachedWhenIntersects::Variable(ix) => factorgraph.nth_variable(ix.into()),
            }
            .map(|(_, v)| v)
            .expect("variable exists");

            let estimated_pos = variable.estimated_position_vec2();
            // Use square distance comparison to avoid sqrt computation
            let dist2waypoint = estimated_pos.distance_squared(next_waypoint.position());
            dist2waypoint < robot_radius_squared
        };

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

        if reached_waypoint && !route.is_completed() {
            route.advance(time_virtual.elapsed());
            evw_robot_reached_waypoint.send(RobotReachedWaypoint {
                robot_id,
                waypoint_index: 0,
            });
        }
    }

    // Send messages to external factors
    for message in all_messages_to_external_factors.drain(..) {
        let (_, mut external_factorgraph, _, _, _) = query
            .get_mut(message.to.factorgraph_id)
            .expect("the factorgraph of the receiving factor exists in the world");

        if let Some(factor) = external_factorgraph.get_factor_mut(message.to.factor_index) {
            factor.receive_message_from(message.from, message.message);
        }
    }

    if !robots_to_despawn.is_empty() {
        evw_robot_finalized_path
            .send_batch(robots_to_despawn.iter().copied().map(RobotFinishedRoute));
        if config.simulation.despawn_robot_when_final_waypoint_reached {
            evw_robot_despawned.send_batch(robots_to_despawn.into_iter().map(RobotDespawned));
        }
    }
}

/// Called `Robot::updateCurrent` in **gbpplanner**
fn update_prior_of_current_state_v3(
    mut query: Query<(&mut FactorGraph, &mut Transform, &T0), With<RobotState>>,
    config: Res<Config>,
    time_fixed: Res<Time<Fixed>>,
) {
    // let time_scale = time_fixed.delta_seconds() / config.simulation.t0.get();

    let mut messages_to_external_factors: Vec<FactorToVariableMessage> = vec![];

    for (mut factorgraph, mut transform, &t0) in &mut query {
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
    mut query: Query<(&mut FactorGraph, &mut Transform, &T0), With<RobotState>>,
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
        &RobotState,
        &Radius,
        &Ball,
        &RadioAntenna,
    )>,
    robot_robot_collisions: Res<RobotRobotCollisions>,
    robot_environment_collisions: Res<RobotEnvironmentCollisions>,
) {
    // let print_line = |text: &str,
    //     println!("{}", text);
    // };

    use colored::Colorize;
    for RobotClickedOn(robot_id) in evr_robot_clicked_on.read() {
        let Ok((_, transform, factorgraph, robotstate, radius, ball, antenna)) =
            robots.get(*robot_id)
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
            robotstate.ids_of_robots_within_comms_range
        );
        println!(
            "    {}: {:?}",
            "connected".cyan(),
            robotstate.ids_of_robots_connected_with
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
        println!("    {}: {:.4} m^2", "volume".cyan(), aabb.volume());
        println!("");
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
