use std::collections::{BTreeSet, HashMap, VecDeque};

use bevy::prelude::*;
use gbp_linalg::{prelude::*, pretty_print_matrix, pretty_print_vector};
use ndarray::{array, concatenate, s, Axis};

// use super::multivariate_normal::MultivariateNormal;
use super::variable::Variable;
use super::{
    factor::{Factor, InterRobotConnection},
    factorgraph::{FactorGraph, FactorId, VariableId},
    NodeIndex,
};
use crate::{config::Config, utils::get_variable_timesteps};

pub struct RobotPlugin;

pub type RobotId = Entity;

// /// Sigma for Unary pose factor on current and horizon states
// /// from **gbpplanner** `Globals.h`
// const SIGMA_POSE_FIXED: f32 = 1e-6;

#[derive(Resource)]

pub struct VariableTimestepsResource {
    pub timesteps: Vec<u32>,
}

impl FromWorld for VariableTimestepsResource {
    fn from_world(world: &mut World) -> Self {
        let config = world.resource::<Config>();

        let lookahead_horizon = config.robot.planning_horizon / config.simulation.t0;

        #[allow(clippy::cast_possible_truncation)]
        Self {
            timesteps: get_variable_timesteps(
                lookahead_horizon.get() as u32,
                config.gbp.lookahead_multiple as u32,
            ),
        }
    }
}

impl Plugin for RobotPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VariableTimestepsResource>()
            .add_systems(
                // FixedUpdate,
                Update,
                (
                    update_robot_neighbours,
                    delete_interrobot_factors,
                    create_interrobot_factors,
                    update_failed_comms,
                    iterate_gbp,
                    update_prior_of_horizon_state,
                    update_prior_of_current_state,
                )
                    .chain()
                    .run_if(time_is_not_paused),
            );

        info!("built RobotPlugin, added:");
        eprintln!(" - Resource: VariableTimestepsResource");
        eprintln!(" - system:Update");
        eprintln!("   - update_robot_neighbours_system");
        eprintln!("   - delete_interrobot_factors_system");
        eprintln!("   - create_interrobot_factors_system");
        eprintln!("   - update_failed_comms_system");
        eprintln!("   - iterate_gbp_internal_system");
        eprintln!("   - iterate_gbp_external_system");
        eprintln!("   - update_prior_of_horizon_state_system");
        eprintln!("   - update_prior_of_current_state_system");
    }
}

#[derive(Debug, thiserror::Error)]

pub enum RobotInitError {
    #[error("No waypoints were provided")]
    NoWaypoints,
    #[error("No variable timesteps were provided")]
    NoVariableTimesteps,
}

/// A component to encapsulate the idea of a radius.
#[derive(Component, Debug)]

pub struct Radius(pub f32);

/// A waypoint is a position that the robot should move to.
#[allow(clippy::similar_names)]
#[derive(Component, Debug)]

pub struct Waypoints(pub VecDeque<Vec4>);

// pub struct Waypoints(pub VecDeque<Vec2>);

/// A robot's state, consisting of other robots within communication range,
/// and other robots that are connected via inter-robot factors.
#[derive(Component, Debug)]

pub struct RobotState {
    /// List of robot ids that are within the communication radius of this
    /// robot. called `neighbours_` in **gbpplanner**.
    /// TODO: maybe change to a BTreeSet
    // ids_of_robots_within_comms_range: Vec<RobotId>,
    pub ids_of_robots_within_comms_range: BTreeSet<RobotId>,
    /// List of robot ids that are currently connected via inter-robot factors
    /// to this robot called `connected_r_ids_` in **gbpplanner**.
    pub ids_of_robots_connected_with:     BTreeSet<RobotId>,
    // pub ids_of_robots_connected_with: Vec<RobotId>,
    /// Flag for whether this factorgraph/robot communicates with other robots
    pub interrobot_comms_active:          bool,
}

impl RobotState {
    pub fn new() -> Self {
        Self {
            ids_of_robots_within_comms_range: BTreeSet::new(),
            ids_of_robots_connected_with:     BTreeSet::new(),
            interrobot_comms_active:          true,
        }
    }
}

#[derive(Bundle, Debug)]

pub struct RobotBundle {
    /// The factor graph that the robot is part of, and uses to perform GBP
    /// message passing.
    pub factorgraph: FactorGraph,
    /// Radius of the robot.
    /// If the robot is not a perfect circle, then set radius to be the smallest
    /// circle that fully encompass the shape of the robot. **constraint**:
    /// > 0.0
    pub radius:      Radius, // TODO: create new type that guarantees this constraint
    /// The current state of the robot
    pub state:       RobotState,
    // pub transform: Transform,
    /// Waypoints used to instruct the robot to move to a specific position.
    /// A VecDeque is used to allow for efficient pop_front operations, and
    /// push_back operations.
    pub waypoints:   Waypoints,
    // NOTE: Using the **Bevy** entity id as the robot id
    // pub id: RobotId,
    // NOTE: These are accessible as **Bevy** resources
    // obstacle_sdf: Option<Rc<image::RgbImage>>,
    // settings: &'a RobotSettings,
    // id_generator: Rc<RefCell<IdGenerator>>,
}

impl RobotBundle {
    #[must_use = "Constructor responsible for creating the robots factorgraph"]

    pub fn new(
        robot_id: RobotId,
        mut waypoints: VecDeque<Vec4>,
        variable_timesteps: &[u32],
        config: &Config,
        obstacle_sdf: &'static Image,
    ) -> Result<Self, RobotInitError> {
        if waypoints.is_empty() {
            return Err(RobotInitError::NoWaypoints);
        }

        if variable_timesteps.is_empty() {
            return Err(RobotInitError::NoVariableTimesteps);
        }

        let start = waypoints
            .pop_front()
            .expect("Waypoints has at least one element");

        let goal = waypoints
            .front()
            .expect("Waypoints has at least two elements");

        // Initialise the horizon in the direction of the goal, at a distance T_HORIZON
        // * MAX_SPEED from the start.
        let start2goal = *goal - start;

        let horizon = start
            + f32::min(
                start2goal.length(),
                (config.robot.planning_horizon * config.robot.max_speed).get(),
            ) * start2goal.normalize();

        let ndofs = 4; // [x, y, x', y']

        let mut factorgraph = FactorGraph::new(robot_id);

        let last_variable_timestep = *variable_timesteps
            .last()
            .expect("Know that variable_timesteps has at least one element");

        let n_variables = variable_timesteps.len();

        let mut variable_node_indices = Vec::with_capacity(n_variables);

        for (i, &variable_timestep) in variable_timesteps.iter().enumerate().take(n_variables) {
            // Set initial mean and covariance of variable interpolated between start and
            // horizon
            let mean = start
                + (horizon - start) * (variable_timestep as f32 / last_variable_timestep as f32);

            let sigma = if i == 0 || i == n_variables - 1 {
                // Start and Horizon state variables should be 'fixed' during optimisation at a
                // timestep SIGMA_POSE_FIXED
                1e30
            } else {
                // 4e9
                Float::INFINITY
            };

            let precision_matrix = Matrix::<Float>::from_diag_elem(ndofs, sigma);
            pretty_print_matrix!(&precision_matrix);

            let mean = array![
                mean.x as Float,
                mean.y as Float,
                mean.z as Float,
                mean.w as Float
            ];
            pretty_print_vector!(&mean);

            let variable = Variable::new(mean, precision_matrix, ndofs);
            let variable_index = factorgraph.add_variable(variable);
            variable_node_indices.push(variable_index);
        }

        // Create Dynamics factors between variables
        for i in 0..variable_timesteps.len() - 1 {
            // T0 is the timestep between the current state and the first planned state.
            let delta_t = config.simulation.t0.get()
                * (variable_timesteps[i + 1] - variable_timesteps[i]) as f32;

            let measurement = Vector::<Float>::zeros(config.robot.dofs.get());

            let dynamic_factor = Factor::new_dynamic_factor(
                config.gbp.sigma_factor_dynamics as Float,
                measurement,
                config.robot.dofs,
                delta_t as Float,
            );

            let factor_node_index = factorgraph.add_factor(dynamic_factor);
            // A dynamic factor connects two variables
            let _ = factorgraph.add_internal_edge(variable_node_indices[i], factor_node_index);
            let _ = factorgraph.add_internal_edge(variable_node_indices[i + 1], factor_node_index);
        }

        // Create Obstacle factors for all variables excluding start, excluding horizon
        #[allow(clippy::needless_range_loop)]
        for i in 1..variable_timesteps.len() - 1 {
            let obstacle_factor = Factor::new_obstacle_factor(
                config.gbp.sigma_factor_obstacle as Float,
                array![0.0],
                config.robot.dofs,
                obstacle_sdf,
                config.simulation.world_size.get() as Float,
            );

            let factor_node_index = factorgraph.add_factor(obstacle_factor);
            let _ = factorgraph.add_internal_edge(variable_node_indices[i], factor_node_index);
        }

        Ok(Self {
            factorgraph,
            radius: Radius(config.robot.radius.get()),
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
    for (robot_id, transform, mut robotstate) in query.iter_mut() {
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

    for (robot_id, _, mut robotstate) in query.iter_mut() {
        let ids_of_robots_connected_with_outside_comms_range: BTreeSet<_> = robotstate
            .ids_of_robots_connected_with
            .difference(&robotstate.ids_of_robots_within_comms_range)
            .cloned()
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

        match factorgraph1.delete_interrobot_factors_connected_to(robot2) {
            Ok(_) => info!(
                "Deleted interrobot factor between factorgraph {:?} and {:?}",
                robot1, robot2
            ),
            Err(err) => {
                error!(
                    "Could not delete interrobot factor between {:?} -> {:?}, with error msg: {}",
                    robot1, robot2, err
                );
            }
        }

        let mut factorgraph2 = query
            .iter_mut()
            .find(|(id, _, _)| *id == robot2)
            .expect("the robot2 should be in the query")
            .1;

        factorgraph2.delete_messages_from_interrobot_factor_at(robot1);
    }
}

fn create_interrobot_factors(
    mut query: Query<(Entity, &mut FactorGraph, &mut RobotState)>,
    config: Res<Config>,
    variable_timesteps: Res<VariableTimestepsResource>,
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
                .cloned()
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

    for (robot_id, mut factorgraph, mut robotstate) in query.iter_mut() {
        for other_robot_id in new_connections_to_establish
            .get(&robot_id)
            .expect("the key is in the map")
        {
            let other_variable_indices = variable_indices_of_each_factorgraph
                .get(other_robot_id)
                .expect("the key is in the map");

            for i in 1..number_of_variables {
                let dofs = 4;
                let z = Vector::<Float>::zeros(dofs);
                let eps = 0.2 * config.robot.radius.get();
                let safety_radius = 2.0 * config.robot.radius.get() + eps;
                // TODO: should it be i - 1 or i?
                let connection =
                    InterRobotConnection::new(*other_robot_id, other_variable_indices[i - 1]);

                let interrobot_factor = Factor::new_interrobot_factor(
                    config.gbp.sigma_factor_interrobot as Float,
                    z,
                    dofs.try_into().expect("dofs > 0"),
                    (safety_radius as Float)
                        .try_into()
                        .expect("safe radius is positive and finite"),
                    connection,
                )
                .expect("safe radius is positive and finite");

                let factor_index = factorgraph.add_factor(interrobot_factor);

                let variable_index = factorgraph
                    .nth_variable_index(i)
                    .expect("there should be an i'th variable");

                factorgraph.add_internal_edge(variable_index, factor_index);
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

        factorgraph
            .factor_mut(factor_index)
            .send_message(variable_id, variable_message);
        info!(
            "sent initial message from {:?} to {:?}",
            robot_id, variable_id
        );
    }
}

/// Called `Simulator::setCommsFailure` in **gbpplanner**

fn update_failed_comms(mut query: Query<&mut RobotState>, config: Res<Config>) {
    for mut state in query.iter_mut() {
        state.interrobot_comms_active =
            config.robot.communication.failure_rate > rand::random::<f32>();
    }
}

fn iterate_gbp(
    mut query: Query<(Entity, &mut FactorGraph), With<RobotState>>,
    config: Res<Config>,
) {
    for _ in 0..config.gbp.iterations_per_timestep {
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

            external_factorgraph
                .variable_mut(message.to.variable_index)
                .send_message(message.from, message.message.clone());
        }

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

            external_factorgraph
                .factor_mut(message.to.factor_index)
                .send_message(message.from, message.message.clone());
        }
    }
}

/// Called `Robot::updateHorizon` in **gbpplanner**

fn update_prior_of_horizon_state(
    // mut commands: Commands,
    mut query: Query<(Entity, &mut FactorGraph, &mut Waypoints), With<RobotState>>,
    config: Res<Config>,
    time: Res<Time>,
) {
    let delta_t = time.delta_seconds();

    let mut all_messages_to_external_factors = Vec::new();

    let mut ids_of_robots_to_despawn = Vec::new();
    for (robot_id, mut factorgraph, mut waypoints) in query.iter_mut() {
        let Some(current_waypoint) = waypoints
            .0
            .front()
            .map(|wp| array![wp.x as Float, wp.y as Float])
        else {
            warn_once!("robot {:?}, has reached its final waypoint", robot_id);
            // despawn robot
            // robot_id.despawn();
            ids_of_robots_to_despawn.push(robot_id);

            continue;
        };

        // TODO: simplify this
        let (variable_index, new_mean, horizon2goal_dist) = {
            let (variable_index, horizon_variable) = factorgraph
                .last_variable()
                .expect("factorgraph has a horizon variable");

            debug_assert_eq!(horizon_variable.mu.len(), 4);

            let estimated_position = horizon_variable.mu.slice(s![..2]); // the mean is a 4x1 vector with [x, y, x', y']
            let horizon2goal_dir = current_waypoint - estimated_position;

            let horizon2goal_dist = horizon2goal_dir.euclidean_norm();

            // Slow down if close to goal
            let new_velocity = Float::min(config.robot.max_speed.get() as Float, horizon2goal_dist)
                * horizon2goal_dir.normalized();

            let new_position = estimated_position.into_owned() + (&new_velocity * delta_t as Float);

            // Update horizon state with new pos and vel
            // horizon->mu_ << new_pos, new_vel;
            // horizon->change_variable_prior(horizon->mu_);
            let new_mean = concatenate![Axis(0), new_position, new_velocity];

            debug_assert_eq!(new_mean.len(), 4);

            (variable_index, new_mean, horizon2goal_dist)
        };

        let (_, horizon_variable) = factorgraph
            .last_variable_mut()
            .expect("factorgraph has a horizon variable");

        horizon_variable.mu.clone_from(&new_mean);

        let messages_to_external_factors =
            factorgraph.change_prior_of_variable(variable_index, new_mean);

        all_messages_to_external_factors.extend(messages_to_external_factors);

        // NOTE: this is weird, we think
        let horizon_has_reached_waypoint = horizon2goal_dist < config.robot.radius.get() as Float;

        if horizon_has_reached_waypoint && !waypoints.0.is_empty() {
            info!("robot {:?}, has reached its waypoint", robot_id);

            waypoints.0.pop_front();
        }
    }

    // Send messages to external factors
    for message in all_messages_to_external_factors {
        let (_, mut external_factorgraph, _) = query
            .iter_mut()
            .find(|(id, _, _)| *id == message.to.factorgraph_id)
            .expect("the factorgraph_id of the receiving factor should exist in the world");

        external_factorgraph
            .factor_mut(message.to.factor_index)
            .send_message(message.from, message.message.clone());
    }

    // FIXME: sometimes this causes a panic
    // for id in ids_of_robots_to_despawn {
    //     commands.entity(id).despawn_recursive();
    // }
}

/// run criteria if time is not paused

fn time_is_not_paused(time: Res<Time<Virtual>>) -> bool {
    !time.is_paused()
}

/// Called `Robot::updateCurrent` in **gbpplanner**

fn update_prior_of_current_state(
    mut query: Query<(&mut FactorGraph, &mut Transform), With<RobotState>>,
    config: Res<Config>,
    time: Res<Time>,
) {
    let scale = time.delta_seconds() / config.simulation.t0.get();

    for (mut factorgraph, mut transform) in query.iter_mut() {
        let (current_variable_index, mean_of_current_variable, increment) = {
            let (current_index, current_variable) = factorgraph
                .nth_variable(0)
                .expect("factorgraph should have a current variable");

            let (_, next_variable) = factorgraph
                .nth_variable(1)
                .expect("factorgraph should have a next variable");

            // let mean_of_current_variable = current_variable.belief.mean().clone();
            let mean_of_current_variable = current_variable.mu.clone();

            let increment = scale as Float * (&next_variable.mu - &mean_of_current_variable);

            (current_index, mean_of_current_variable, increment)
        };

        // dbg!(&increment);
        // TODO: create a separate method on Variable so we do not have to abuse the
        // interface, and call it with a empty vector, and get an empty HashMap
        // as a return value.
        factorgraph.change_prior_of_variable(
            current_variable_index,
            mean_of_current_variable + &increment,
        );

        #[allow(clippy::cast_possible_truncation)]
        let increment = Vec3::new(increment[0] as f32, 0.0, increment[1] as f32);

        transform.translation += increment;
    }
}
