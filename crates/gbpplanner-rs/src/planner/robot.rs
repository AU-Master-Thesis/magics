use std::collections::{BTreeSet, HashMap, VecDeque};

use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use gbp_linalg::prelude::*;
use ndarray::{array, concatenate, s, Axis};

// use super::{
//     factor::{Factor, InterRobotConnection},
//     factorgraph::{FactorGraph, FactorId, VariableId},
//     variable::Variable,
//     NodeIndex,
// };
use crate::{
    bevy_utils::run_conditions::time::virtual_time_is_paused,
    factorgraph::{
        factor::{ExternalVariableId, FactorNode},
        factorgraph::{FactorGraph, NodeIndex, VariableIndex},
        id::{FactorId, VariableId},
    },
    simulation_loader::SdfImage,
};
use crate::{
    config::Config,
    factorgraph::{variable::VariableNode, DOFS},
    pause_play::PausePlay,
    utils::get_variable_timesteps,
};

pub type RobotId = Entity;

pub struct RobotPlugin;

impl Plugin for RobotPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VariableTimesteps>()
            // .init_resource::<ManualMode>()
            .insert_state(ManualModeState::Disabled)
            .add_event::<RobotSpawned>()
            .add_event::<RobotDespawned>()
            .add_event::<RobotReachedWaypoint>()
            .add_systems(PreUpdate, start_manual_step.run_if(virtual_time_is_paused))
            .add_systems(
                FixedUpdate,
                // Update,
                (
                    update_robot_neighbours,
                    delete_interrobot_factors,
                    create_interrobot_factors,
                    update_failed_comms,
                    iterate_gbp,
                    update_prior_of_horizon_state,
                    update_prior_of_current_state,
                    despawn_robots,
                    // finish_manual_step.run_if(ManualMode::enabled),
                    finish_manual_step.run_if(ManualModeState::enabled),
                    // finish_manual_step.run_if(in_state(ManualModeState::Enabled)),
                )
                    .chain()
                    .run_if(not(virtual_time_is_paused)),
            );
    }
}

/// Event emitted when a robot is spawned
#[derive(Debug, Event)]
pub struct RobotSpawned(pub RobotId);

/// Event emitted when a robot is despawned
#[derive(Debug, Event)]
pub struct RobotDespawned(pub RobotId);

/// Event emitted when a robot reaches a waypoint
#[derive(Event)]
pub struct RobotReachedWaypoint {
    pub robot_id:       RobotId,
    pub waypoint_index: usize,
}

fn despawn_robots(
    mut commands: Commands,
    // TODO: change query
    mut query: Query<(Entity, &mut FactorGraph, &mut RobotState)>,
    mut despawn_robot_event: EventReader<RobotDespawned>,
) {
    for RobotDespawned(robot_id) in despawn_robot_event.read() {
        for (_, mut factorgraph, _) in &mut query {
            let _ = factorgraph.remove_connection_to(*robot_id);
        }

        if let Some(mut entitycommand) = commands.get_entity(*robot_id) {
            // info!("despawning robot: {:?}", entitycommand.id());
            entitycommand.despawn();
        } else {
            error!(
                "A DespawnRobotEvent event was emitted with entity id: {:?} but the entity does \
                 not exist!",
                robot_id
            );
        }
    }
}

/// Resource that stores the horizon timesteps sequence
#[derive(Resource, Debug)]
pub struct VariableTimesteps {
    timesteps: Vec<u32>,
}

impl VariableTimesteps {
    // /// Returns the number of timesteps
    // #[inline(always)]
    // pub fn len(&self) -> usize {
    //     self.timesteps.len()
    // }

    /// Extracts a slice containing the entire vector.
    #[inline(always)]
    pub fn as_slice(&self) -> &[u32] {
        self.timesteps.as_slice()
    }
}

impl std::ops::Index<usize> for VariableTimesteps {
    type Output = u32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.timesteps[index]
    }
}

impl FromWorld for VariableTimesteps {
    fn from_world(world: &mut World) -> Self {
        // let config = world.resource::<Config>();

        // let lookahead_horizon = config.robot.planning_horizon / config.simulation.t0;
        let lookahead_horizon = 5.0 / 0.25;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        Self {
            timesteps: get_variable_timesteps(
                // lookahead_horizon.get() as u32,
                lookahead_horizon as u32,
                // config.gbp.lookahead_multiple as u32,
                3 as u32,
            ),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RobotInitError {
    #[error("No waypoints were provided")]
    NoWaypoints,
    #[error("No variable timesteps were provided")]
    NoVariableTimesteps,
}

/// Component for entities with a radius, used for robots
#[derive(Component, Debug, Deref, DerefMut)]
pub struct Radius(pub f32);

/// A waypoint is a position and velocity that the robot should move to and
/// achieve.
#[allow(clippy::similar_names)]
#[derive(Component, Debug)]
pub struct Waypoints(pub VecDeque<Vec4>);

/// A robot's state, consisting of other robots within communication range,
/// and other robots that are connected via inter-robot factors.
#[derive(Component, Debug)]
pub struct RobotState {
    /// List of robot ids that are within the communication radius of this
    /// robot. called `neighbours_` in **gbpplanner**.
    // ids_of_robots_within_comms_range: Vec<RobotId>,
    pub ids_of_robots_within_comms_range: BTreeSet<RobotId>,
    /// List of robot ids that are currently connected via inter-robot factors
    /// to this robot called `connected_r_ids_` in **gbpplanner**.
    pub ids_of_robots_connected_with: BTreeSet<RobotId>,
    // pub ids_of_robots_connected_with: Vec<RobotId>,
    /// Flag for whether this factorgraph/robot communicates with other robots
    pub interrobot_comms_active: bool,
}

impl RobotState {
    /// Create a new `RobotState`
    #[must_use]
    pub fn new() -> Self {
        Self {
            ids_of_robots_within_comms_range: BTreeSet::new(),
            ids_of_robots_connected_with: BTreeSet::new(),
            interrobot_comms_active: true,
        }
    }
}

impl Default for RobotState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Component, Deref)]
pub struct Ball(parry2d::shape::Ball);

#[derive(Bundle, Debug)]
pub struct RobotBundle {
    /// The factor graph that the robot is part of, and uses to perform GBP
    /// message passing.
    pub factorgraph: FactorGraph,
    /// Radius of the robot.
    /// If the robot is not a perfect circle, then set radius to be the smallest
    /// circle that fully encompass the shape of the robot. **constraint**:
    /// > 0.0
    pub radius: Radius,
    pub ball: Ball,
    /// The current state of the robot
    pub state: RobotState,
    /// Waypoints used to instruct the robot to move to a specific position.
    /// A `VecDeque` is used to allow for efficient `pop_front` operations, and
    /// `push_back` operations.
    pub waypoints: Waypoints,
}

/// State vector of a robot
/// [x, y, x', y']
#[derive(Debug, Clone, Copy, derive_more::Into, derive_more::Add)]
pub struct StateVector(bevy::math::Vec4);

impl StateVector {
    pub fn position(&self) -> Vec2 {
        self.0.xy()
    }

    pub fn velocity(&self) -> Vec2 {
        self.0.zw()
    }

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
        waypoints: VecDeque<Vec4>,
        variable_timesteps: &[u32],
        config: &Config,
        // obstacle_sdf: &'static Image,
        // obstacle_sdf: &Image,
        sdf: &SdfImage,
        // obstacle_sdf: Handle<Image>,
    ) -> Result<Self, RobotInitError> {
        if waypoints.is_empty() {
            return Err(RobotInitError::NoWaypoints);
        }

        if variable_timesteps.is_empty() {
            return Err(RobotInitError::NoVariableTimesteps);
        }

        let start: Vec4 = Vec4::from(initial_state);
        // let start = waypoints
        //     .pop_front()
        //     .expect("Waypoints has at least one element");

        let goal = waypoints
            .front()
            .expect("Waypoints has at least one element");

        // Initialise the horizon in the direction of the goal, at a distance T_HORIZON
        // * MAX_SPEED from the start.
        let start2goal: Vec4 = *goal - start;

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

        for (i, &variable_timestep) in variable_timesteps.iter().enumerate()
        // .take(n_variables)
        {
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

            let variable = VariableNode::new(mean, precision_matrix, DOFS);
            let variable_index = factorgraph.add_variable(variable);
            variable_node_indices.push(variable_index);
        }

        // Create Dynamics factors between variables
        for i in 0..variable_timesteps.len() - 1 {
            // T0 is the timestep between the current state and the first planned state.
            #[allow(clippy::cast_precision_loss)]
            let delta_t = config.simulation.t0.get()
                * (variable_timesteps[i + 1] - variable_timesteps[i]) as f32;

            let measurement = Vector::<Float>::zeros(config.robot.dofs.get());

            let dynamic_factor = FactorNode::new_dynamic_factor(
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

        // Create Obstacle factors for all variables excluding start, excluding horizon
        #[allow(clippy::needless_range_loop)]
        for i in 1..variable_timesteps.len() - 1 {
            let obstacle_factor = FactorNode::new_obstacle_factor(
                Float::from(config.gbp.sigma_factor_obstacle),
                array![0.0],
                // obstacle_sdf.clone(),
                sdf.clone(),
                // obstacle_sdf.clone_value(),
                Float::from(config.simulation.world_size.get()),
            );

            let factor_node_index = factorgraph.add_factor(obstacle_factor);
            let factor_id = FactorId::new(factorgraph.id(), factor_node_index);
            let _ = factorgraph.add_internal_edge(
                VariableId::new(factorgraph.id(), variable_node_indices[i]),
                factor_id,
            );
        }

        Ok(Self {
            factorgraph,
            radius: Radius(config.robot.radius.get()),
            ball: Ball(parry2d::shape::Ball::new(config.robot.radius.get())),
            state: RobotState::new(),
            waypoints: Waypoints(waypoints),
        })
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

    // dbg!(&robots_to_delete_interrobot_factors_between);

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
        let mut factorgraph1 = query
            .iter_mut()
            .find(|(id, _, _)| *id == robot1)
            .expect("the robot1 should be in the query")
            .1;

        factorgraph1.delete_interrobot_factors_connected_to(robot2);

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

        let Some((_, mut factorgraph2, _)) = query
            .iter_mut()
            .find(|(robot_id, _, _)| *robot_id == robot2)
        else {
            error!(
                "attempt to delete interrobot factors between robots: {:?} and {:?} failed, \
                 reason: {:?} does not exist!",
                robot1, robot2, robot2
            );
            continue;
        };

        factorgraph2.delete_messages_from_interrobot_factor_at(robot1);
    }
}

fn create_interrobot_factors(
    mut query: Query<(Entity, &mut FactorGraph, &mut RobotState)>,
    config: Res<Config>,
    variable_timesteps: Res<VariableTimesteps>,
) {
    // a mapping between a robot and the other robots it should create a interrobot
    // factor to e.g:
    // {a -> [b, c, d], b -> [a, c], c -> [a, b], d -> [c]}
    let new_connections_to_establish: HashMap<RobotId, Vec<RobotId>> = query
        .iter()
        .map(|(entity, _, robotstate)| {
            let new_connections = robotstate
                .ids_of_robots_within_comms_range
                .difference(&robotstate.ids_of_robots_connected_with)
                .copied()
                .collect::<Vec<_>>();

            (entity, new_connections)
        })
        .collect();

    let number_of_variables = variable_timesteps.timesteps.len();

    let variable_indices_of_each_factorgraph: HashMap<RobotId, Vec<NodeIndex>> = query
        .iter()
        .map(|(robot_id, factorgraph, _)| {
            let variable_indices = factorgraph
                .variable_indices_ordered_by_creation(1..number_of_variables)
                .expect("the factorgraph has up to `n_variables` variables");

            (robot_id, variable_indices)
        })
        .collect();

    let mut external_edges_to_add = Vec::new();

    for (robot_id, mut factorgraph, mut robotstate) in &mut query {
        for other_robot_id in new_connections_to_establish
            .get(&robot_id)
            .expect("the key is in the map")
        {
            let other_variable_indices = variable_indices_of_each_factorgraph
                .get(other_robot_id)
                .expect("the key is in the map");

            for i in 1..number_of_variables {
                let z = Vector::<Float>::zeros(DOFS);
                let eps = 0.2 * config.robot.radius.get();
                let safety_radius = 2.0f32.mul_add(config.robot.radius.get(), eps);
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
                    Float::from(config.gbp.sigma_factor_interrobot),
                    z,
                    Float::from(safety_radius)
                        .try_into()
                        .expect("safe radius is positive and finite"),
                    external_variable_id,
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
        let mut other_factorgraph = query
            .iter_mut()
            .find(|(id, _, _)| *id == other_robot_id)
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
        let mut factorgraph = query
            .iter_mut()
            .find(|(id, _, _)| *id == robot_id)
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
fn update_failed_comms(mut query: Query<&mut RobotState>, config: Res<Config>) {
    for mut state in &mut query {
        state.interrobot_comms_active =
            config.robot.communication.failure_rate < rand::random::<f32>();
    }
}

fn iterate_gbp(
    mut query: Query<(Entity, &mut FactorGraph), With<RobotState>>,
    config: Res<Config>,
) {
    for _ in 0..config.gbp.iterations_per_timestep {
        // pretty_print_title!(format!("GBP iteration: {}", i + 1));
        // ╭────────────────────────────────────────────────────────────────────────────────────────
        // │ Factor iteration
        let messages_to_external_variables = query
            .iter_mut()
            .map(|(_, mut factorgraph)| factorgraph.factor_iteration())
            .collect::<Vec<_>>();

        // Send messages to external variables
        for message in messages_to_external_variables.iter().flatten() {
            let (_, mut external_factorgraph) = query
                .iter_mut()
                .find(|(id, _)| *id == message.to.factorgraph_id)
                .expect("the factorgraph_id of the receiving variable should exist in the world");

            if let Some(variable) = external_factorgraph.get_variable_mut(message.to.variable_index)
            {
                variable.receive_message_from(message.from, message.message.clone());
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
        for message in messages_to_external_factors.iter().flatten() {
            let (_, mut external_factorgraph) = query
                .iter_mut()
                .find(|(id, _)| *id == message.to.factorgraph_id)
                .expect("the factorgraph_id of the receiving factor should exist in the world");

            if let Some(factor) = external_factorgraph.get_factor_mut(message.to.factor_index) {
                factor.receive_message_from(message.from, message.message.clone());
            }

            // external_factorgraph
            //     .factor_mut(message.to.factor_index)
            //     .receive_message_from(message.from, message.message.clone());
        }
    }
}

/// Called `Robot::updateHorizon` in **gbpplanner**
fn update_prior_of_horizon_state(
    mut query: Query<(Entity, &mut FactorGraph, &mut Waypoints), With<RobotState>>,
    config: Res<Config>,
    time: Res<Time>,
    mut despawn_robot_event: EventWriter<RobotDespawned>,
    mut robot_reached_waypoint_event: EventWriter<RobotReachedWaypoint>,
) {
    let delta_t = time.delta_seconds();

    let mut all_messages_to_external_factors = Vec::new();
    let mut ids_of_robots_to_despawn = Vec::new();

    for (robot_id, mut factorgraph, mut waypoints) in &mut query {
        let Some(current_waypoint) = waypoints
            .0
            .front()
            .map(|wp| array![Float::from(wp.x), Float::from(wp.y)])
        else {
            // info!("robot {:?}, has reached its final waypoint", robot_id);
            ids_of_robots_to_despawn.push(robot_id);

            continue;
        };

        // TODO: simplify this
        let (variable_index, new_mean, horizon2goal_dist) = {
            let (variable_index, horizon_variable) = factorgraph
                .last_variable()
                .expect("factorgraph has a horizon variable");

            debug_assert_eq!(horizon_variable.belief.mean.len(), 4);

            let estimated_position = horizon_variable.belief.mean.slice(s![..2]); // the mean is a 4x1 vector with [x, y, x', y']
                                                                                  // dbg!(&estimated_position);

            let horizon2goal_dir = current_waypoint - estimated_position;

            let horizon2goal_dist = horizon2goal_dir.euclidean_norm();
            // let horizon2goal_dist = dbg!(horizon2goal_dir.euclidean_norm());

            // Slow down if close to goal
            let new_velocity =
                Float::min(Float::from(config.robot.max_speed.get()), horizon2goal_dist)
                    * horizon2goal_dir.normalized();

            // dbg!(&new_velocity);
            let new_position =
                estimated_position.into_owned() + (&new_velocity * Float::from(delta_t));

            // pretty_print_subtitle!("HORIZON STATE UPDATE");
            // println!("horizon2goal_dir = {:?}", horizon2goal_dir);
            // pretty_print_vector!(&new_velocity);
            // pretty_print_vector!(&new_position);

            // dbg!(&new_position);

            // Update horizon state with new pos and vel
            // horizon->mu_ << new_pos, new_vel;
            // horizon->change_variable_prior(horizon->mu_);
            let new_mean = concatenate![Axis(0), new_position, new_velocity];

            // dbg!(&new_mean);

            debug_assert_eq!(new_mean.len(), 4);

            (variable_index, new_mean, horizon2goal_dist)
        };

        let (_, horizon_variable) = factorgraph
            .last_variable_mut()
            .expect("factorgraph has a horizon variable");

        horizon_variable.belief.mean.clone_from(&new_mean);

        let messages_to_external_factors =
            factorgraph.change_prior_of_variable(variable_index, new_mean);

        all_messages_to_external_factors.extend(messages_to_external_factors);

        // NOTE: this is weird, we think
        let horizon_has_reached_waypoint =
            horizon2goal_dist < Float::from(config.robot.radius.get());

        // println!("waypoints.0.len() = {:?}", waypoints.0.len());

        if horizon_has_reached_waypoint && !waypoints.0.is_empty() {
            // info!("robot {:?}, has reached its waypoint", robot_id);

            waypoints.0.pop_front();
            robot_reached_waypoint_event.send(RobotReachedWaypoint {
                robot_id,
                waypoint_index: 0,
            });
        }
    }

    // Send messages to external factors
    for message in all_messages_to_external_factors {
        let (_, mut external_factorgraph, _) = query
            .iter_mut()
            .find(|(id, _, _)| *id == message.to.factorgraph_id)
            .expect("the factorgraph_id of the receiving factor should exist in the world");

        if let Some(factor) = external_factorgraph.get_factor_mut(message.to.factor_index) {
            factor.receive_message_from(message.from, message.message.clone());
        }
        // external_factorgraph
        //     .factor_mut(message.to.factor_index)
        //     .receive_message_from(message.from, message.message.clone());
    }

    if !ids_of_robots_to_despawn.is_empty() {
        despawn_robot_event.send_batch(ids_of_robots_to_despawn.into_iter().map(RobotDespawned));
    }
}

/// Called `Robot::updateCurrent` in **gbpplanner**
fn update_prior_of_current_state(
    mut query: Query<(&mut FactorGraph, &mut Transform), With<RobotState>>,
    config: Res<Config>,
    time: Res<Time>,
) {
    let scale = time.delta_seconds() / config.simulation.t0.get();

    for (mut factorgraph, mut transform) in &mut query {
        let (current_variable_index, current_variable) = factorgraph
            .nth_variable(0)
            .expect("factorgraph should have a current variable");
        let (_, next_variable) = factorgraph
            .nth_variable(1)
            .expect("factorgraph should have a next variable");
        let mean_of_current_variable = current_variable.belief.mean.clone();
        let change_in_position =
            Float::from(scale) * (&next_variable.belief.mean - &mean_of_current_variable);

        factorgraph.change_prior_of_variable(
            current_variable_index,
            &mean_of_current_variable + &change_in_position,
        );

        #[allow(clippy::cast_possible_truncation)]
        let increment = Vec3::new(
            change_in_position[0] as f32,
            0.0,
            change_in_position[1] as f32,
        );

        // dbg!((&increment, scale));
        // pretty_print_subtitle!("CURRENT STATE UPDATE");
        // pretty_print_vector!(&array![increment.x, increment.y, increment.z]);
        // println!("scale = {:?}", scale);

        transform.translation += increment;
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
    pub fn enabled(state: Res<State<Self>>) -> bool {
        matches!(state.get(), Self::Enabled { .. })
    }

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
