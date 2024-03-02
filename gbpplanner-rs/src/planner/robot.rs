use std::collections::{BTreeSet, VecDeque};
// use std::sync::{Arc, OnceLock};

use crate::config::Config;
use crate::utils::get_variable_timesteps;

use super::factor::{Factor, InterRobotConnection};
use super::factorgraph::{FactorGraph, MessagePassingMode};
use super::multivariate_normal::MultivariateNormal;
use super::variable::Variable;
use super::{Matrix, NdarrayVectorExt, NodeIndex, Timestep, Vector, VectorNorm};
use bevy::prelude::*;
use ndarray::{array, concatenate, Axis};
use std::collections::HashMap;

pub struct RobotPlugin;

pub type RobotId = Entity;

/// Sigma for Unary pose factor on current and horizon states
/// from **gbpplanner** `Globals.h`
const SIGMA_POSE_FIXED: f64 = 1e-15;

#[derive(Resource)]
struct VariableTimestepsResource {
    timesteps: Vec<u32>,
}

impl FromWorld for VariableTimestepsResource {
    fn from_world(world: &mut World) -> Self {
        let config = world.resource::<Config>();
        let lookahead_horizon = config.robot.planning_horizon / config.simulation.t0;

        Self {
            timesteps: get_variable_timesteps(
                lookahead_horizon as u32,
                config.gbp.lookahead_multiple as u32,
            ),
        }
    }
}

impl Plugin for RobotPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VariableTimestepsResource>()
            .add_systems(
                Update,
                (
                    update_robot_neighbours_system,
                    delete_interrobot_factors_system,
                    create_interrobot_factors_system,
                    update_failed_comms_system,
                    // iterate_gbp_system,
                    // iterate_gbp_internal_system,
                    // iterate_gbp_external_system,
                    // update_prior_of_horizon_state_system,
                    // update_prior_of_current_state_system,
                )
                    .chain(),
            );
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
pub struct Waypoints(pub VecDeque<Vec2>);

/// A robot's state, consiting of other robots within communication range,
/// and other robots that are connected via inter-robot factors.
#[derive(Component, Debug)]
pub struct RobotState {
    /// List of robot ids that are within the communication radius of this robot.
    /// called `neighbours_` in **gbpplanner**.
    /// TODO: maybe change to a BTreeSet
    // ids_of_robots_within_comms_range: Vec<RobotId>,
    pub ids_of_robots_within_comms_range: BTreeSet<RobotId>,
    /// List of robot ids that are currently connected via inter-robot factors to this robot
    /// called `connected_r_ids_` in **gbpplanner**.
    pub ids_of_robots_connected_with: BTreeSet<RobotId>,
    // pub ids_of_robots_connected_with: Vec<RobotId>,
    /// Flag for whether this factorgraph/robot communicates with other robots
    pub interrobot_comms_active: bool,
}

impl RobotState {
    pub fn new() -> Self {
        Self {
            ids_of_robots_within_comms_range: BTreeSet::new(),
            ids_of_robots_connected_with: BTreeSet::new(),
            interrobot_comms_active: true,
        }
    }
}

#[derive(Bundle)]
pub struct RobotBundle {
    /// The factor graph that the robot is part of, and uses to perform GBP message passing.
    pub factorgraph: FactorGraph,
    /// Radius of the robot.
    /// If the robot is not a perfect circle, then set radius to be the smallest circle that fully encompass the shape of the robot.
    /// **constraint**: > 0.0
    pub radius: Radius, // TODO: create new type that guarantees this constraint
    /// The current state of the robot
    pub state: RobotState,
    // pub transform: Transform,
    /// Waypoints used to instruct the robot to move to a specific position.
    /// A VecDeque is used to allow for efficient pop_front operations, and push_back operations.
    pub waypoints: Waypoints,
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
        mut waypoints: VecDeque<Vec2>,
        // transform: Transform,
        variable_timesteps: &[Timestep],
        config: &Config,
        // obstacle_sdf: Arc<image::RgbImage>,
        // obstacle_sdf: &OnceLock<Image>,
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
        // let transform = Transform::from_translation(Vec3::new(start.x, 0.0, start.y));
        let goal = waypoints
            .front()
            .expect("Waypoints has at least two elements");

        // Initialise the horzion in the direction of the goal, at a distance T_HORIZON * MAX_SPEED from the start.
        let start2goal = *goal - start;
        let horizon = start
            + f32::min(
                start2goal.length(),
                config.robot.planning_horizon * config.robot.max_speed,
            ) * start2goal.normalize();

        let ndofs = 4; // [x, y, x', y']

        let mut factorgraph = FactorGraph::new();

        let last_variable_timestep = *variable_timesteps
            .last()
            .expect("Know that variable_timesteps has at least one element");

        let mut variable_node_indices = Vec::with_capacity(variable_timesteps.len());
        for i in 0..variable_timesteps.len() {
            // Set initial mean and covariance of variable interpolated between start and horizon
            let mean = start
                + (horizon - start)
                    * (variable_timesteps[i] as f32 / last_variable_timestep as f32);
            // Start and Horizon state variables should be 'fixed' during optimisation at a timestep
            let sigma = if i == 0 || i == variable_timesteps.len() - 1 {
                SIGMA_POSE_FIXED
            } else {
                0.0
            };

            let sigmas: Vector<f32> = {
                let elem = if sigma == 0.0 {
                    f32::MAX
                } else {
                    1.0 / (sigma as f32).powi(2)
                };
                Vector::<f32>::from_shape_fn(ndofs, |_| elem)
            };

            let covariance = Matrix::<f32>::from_diag(&sigmas);
            let prior = MultivariateNormal::from_mean_and_covariance(
                array![mean.x, mean.y, 0.0, 0.0], // initial velocity (x', y') is zero
                covariance,
            );

            let variable = Variable::new(prior, ndofs);
            let variable_index = factorgraph.add_variable(variable);
            variable_node_indices.push(variable_index);
        }

        // Create Dynamics factors between variables
        for i in 0..variable_timesteps.len() - 1 {
            // T0 is the timestep between the current state and the first planned state.
            let delta_t = config.simulation.t0
                * (variable_timesteps[i + 1] - variable_timesteps[i]) as f32;
            let measurement = Vector::<f32>::zeros(config.robot.dofs);
            let dynamic_factor = Factor::new_dynamic_factor(
                config.gbp.sigma_factor_dynamics,
                measurement,
                config.robot.dofs,
                delta_t,
            );

            let factor_node_index = factorgraph.add_factor(dynamic_factor);
            let _ = factorgraph.add_edge(variable_node_indices[i], factor_node_index);
            let _ = factorgraph.add_edge(variable_node_indices[i + 1], factor_node_index);
        }

        // Create Obstacle factors for all variables excluding start, excluding horizon
        for i in 1..variable_timesteps.len() - 1 {
            let obstacle_factor = Factor::new_obstacle_factor(
                config.gbp.sigma_factor_obstacle,
                array![0.0],
                config.robot.dofs,
                obstacle_sdf,
                config.simulation.world_size,
            );

            let factor_node_index = factorgraph.add_factor(obstacle_factor);
            let _ = factorgraph.add_edge(variable_node_indices[i], factor_node_index);
        }

        Ok(Self {
            factorgraph,
            radius: Radius(config.robot.radius),
            state: RobotState::new(),
            // transform,
            waypoints: Waypoints(waypoints),
        })
    }
}

/// Called `Simulator::calculateRobotNeighbours` in **gbpplanner**
fn update_robot_neighbours_system(
    // query: Query<(Entity, &Transform, &mut RobotState)>,
    robots: Query<(Entity, &Transform), With<RobotState>>,
    mut states: Query<(Entity, &Transform, &mut RobotState)>,
    config: Res<Config>,
) {
    // TODO: use kdtree to speed up, and to have something in the report
    for (entity_id, transform, mut robotstate) in states.iter_mut() {
        // TODO: maybe use clear() instead
        // unsafe {
        //     state.ids_of_robots_within_comms_range.set_len(0);
        // }
        robotstate.ids_of_robots_within_comms_range = robots
            .iter()
            .filter_map(|(other_entity_id, other_transform)| {
                // Do not compute the distance to self
                if other_entity_id == entity_id
                    || config.robot.communication.radius
                        < transform.translation.distance(other_transform.translation)
                {
                    None
                } else {
                    Some(other_entity_id)
                }
            })
            .collect();
        // .for_each(|other_entity_id| {
        //     state.ids_of_robots_within_comms_range.push(other_entity_id)
        // });
    }
}

// FIXME: more that one interrobot is created in `create_interrobot_factors`
fn delete_interrobot_factors_system(
    mut query: Query<(Entity, &mut FactorGraph, &mut RobotState)>,
) {
    // the set of robots connected with will (possibly) be mutated
    // the robots factorgraph will (possibly) be mutated
    // the other robot with an interrobot factor connected will be mutated

    let mut interrobot_factors_to_delete: HashMap<Entity, Entity> = HashMap::new();
    for (entity, _, mut robotstate) in query.iter_mut() {
        let ids_of_robots_connected_with_outside_comms_range: BTreeSet<_> = robotstate
            .ids_of_robots_connected_with
            .difference(&robotstate.ids_of_robots_within_comms_range)
            .cloned()
            .collect();

        interrobot_factors_to_delete.extend(
            ids_of_robots_connected_with_outside_comms_range
                .iter()
                .map(|id| (entity, *id)),
        );

        let robotstate = robotstate.as_mut();
        for id in ids_of_robots_connected_with_outside_comms_range {
            robotstate.ids_of_robots_connected_with.remove(&id);
        }
    }

    for (a, b) in interrobot_factors_to_delete {
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
        // TODO: use par_iter_mut
        for (entity, mut graph, _) in query.iter_mut() {
            if entity != a {
                continue;
            }

            if let Err(err) = graph.as_mut().delete_interrobot_factor_connected_to(b) {
                error!("Could not delete interrobot factor between {:?} -> {:?}, with error msg: {}", a, b, err);
            }
        }
    }
}

fn create_interrobot_factors_system(
    mut query: Query<(Entity, &mut FactorGraph, &mut RobotState)>,
    config: Res<Config>,
    variable_timesteps: Res<VariableTimestepsResource>,
) {
    // a mapping between a robot and the other robots it should create a interrobot factor to
    // e.g:
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

    let n_variables = variable_timesteps.timesteps.len();

    let variable_indices_of_each_factorgraph: HashMap<RobotId, Vec<NodeIndex>> = query
        .iter()
        .map(|(robot_id, factorgraph, _)| {
            let varible_indices = factorgraph
                .variable_indices_ordered_by_creation(1..n_variables)
                .expect("the factorgraph has up to `n_variables` variables");
            (robot_id, varible_indices)
        })
        .collect();

    for (robot_id, mut factorgraph, mut robotstate) in query.iter_mut() {
        for other_robot_id in new_connections_to_establish
            .get(&robot_id)
            .expect("the key is in the map")
        {
            let other_varible_indices = variable_indices_of_each_factorgraph
                .get(other_robot_id)
                .expect("the key is in the map");
            for i in 1..n_variables {
                // TODO: do not hardcode
                let dofs = 4;
                let z = Vector::<f32>::zeros(dofs);
                let eps = 0.2 * config.robot.radius;
                let safety_radius = 2.0 * config.robot.radius + eps;
                let connection = InterRobotConnection {
                    id_of_robot_connected_with: *other_robot_id,
                    index_of_connected_variable_in_other_robots_factorgraph:
                        other_varible_indices[i - 1],
                };
                let interrobot_factor = Factor::new_interrobot_factor(
                    config.gbp.sigma_factor_interrobot,
                    z,
                    dofs,
                    safety_radius,
                    connection,
                );
                let factor_index = factorgraph.add_factor(interrobot_factor);
                let variable_index = factorgraph
                    .nth_variable_index(i)
                    .expect("there should be an i'th variable");
                factorgraph.add_edge(factor_index, variable_index);
            }

            robotstate
                .ids_of_robots_connected_with
                .insert(*other_robot_id);
        }
    }
}

/// Called `Simulator::setCommsFailure` in **gbpplanner**
fn update_failed_comms_system(mut query: Query<&mut RobotState>, config: Res<Config>) {
    for mut state in query.iter_mut() {
        state.interrobot_comms_active =
            config.robot.communication.failure_rate > rand::random::<f32>();
    }
}

macro_rules! iterate_gbp_impl {
    ($name:ident, $mode:expr) => {
        fn $name(
            mut query: Query<(Entity, &mut FactorGraph), With<RobotState>>,
            config: Res<Config>,
        ) {
            query
                .par_iter_mut()
                .for_each(|(robot_id, mut factorgraph)| {
                    factorgraph.variable_iteration(robot_id, $mode);
                });
            query
                .par_iter_mut()
                .for_each(|(robot_id, mut factorgraph)| {
                    factorgraph.factor_iteration(robot_id, $mode);
                });
            // for (robot_id, mut factorgraph) in query.par_iter_mut() {
            //     factorgraph.factor_iteration(robot_id, $mode);
            // }
            // for (robot_id, mut factorgraph) in query.iter_mut() {
            //     factorgraph.variable_iteration(robot_id, $mode);
            // }
        }
    };
}

iterate_gbp_impl!(iterate_gbp_internal_system, MessagePassingMode::Internal);
iterate_gbp_impl!(iterate_gbp_external_system, MessagePassingMode::External);

// fn iterate_gbp_system(
//     mut query: Query<(Entity, &mut FactorGraph), With<RobotState>>,
//     config: Res<Config>,
// ) {

//     query.par_iter_mut().for_each(|(robot_id, mut factorgraph)| {
//         factorgraph.factor_iteration(robot_id, MessagePassingMode::Internal);
//     });
//     query.par_iter_mut().for_each(|(robot_id, mut factorgraph)| {
//         factorgraph.variable_iteration(robot_id, MessagePassingMode::Internal);
//     });
//     // for (robot_id, mut factorgraph) in query.par_iter_mut() {
//     //     factorgraph.factor_iteration(robot_id, MessagePassingMode::Internal);
//     // }
//     // for (robot_id, mut factorgraph) in query.iter_mut() {
//     //     factorgraph.variable_iteration(robot_id, MessagePassingMode::Internal);
//     // }
// }

// fn iterate_gbp(query: Query<&mut FactorGraph>, config: Res<Config>) {}

fn update_prior_of_horizon_state_system(
    mut query: Query<(Entity, &mut FactorGraph, &mut Waypoints), With<RobotState>>,
    config: Res<Config>,
    time: Res<Time>,
) {
    let delta_t = time.delta_seconds();
    for (entity, mut factorgraph, mut waypoints) in query.iter_mut() {
        let Some(current_waypoint) = waypoints.0.front().map(|wp| array![wp.x, wp.y])
        else {
            warn!("robot {:?}, has reached its final waypoint", entity);
            continue;
        };

        let horizon_variable = factorgraph
            .last_variable_mut()
            .expect("factorgraph has a horizon variable");
        let mean_of_horizon_variable = horizon_variable.belief.mean();
        let direction_from_horizon_to_goal = current_waypoint - &mean_of_horizon_variable;
        let distance_from_horizon_to_goal =
            direction_from_horizon_to_goal.euclidean_norm();
        let new_velocity =
            f32::min(config.robot.max_speed, distance_from_horizon_to_goal)
                * direction_from_horizon_to_goal.normalized();
        let new_position = mean_of_horizon_variable + &new_velocity * delta_t;

        // Update horizon state with new pos and vel
        // horizon->mu_ << new_pos, new_vel;
        // horizon->change_variable_prior(horizon->mu_);
        let new_mean = concatenate![Axis(0), new_position, new_velocity];
        // TODO: cache the mean ...
        // horizon_variable.belief.mean()

        // TODO: create a separate method on Variable so we do not have to abuse the interface, and call it with a empty

        // vector, and get an empty HashMap as a return value.
        let _ = horizon_variable.change_prior(new_mean, vec![]);

        // NOTE: this is weird, we think
        let horizon_has_reached_waypoint =
            distance_from_horizon_to_goal < config.robot.radius;
        if horizon_has_reached_waypoint && !waypoints.0.is_empty() {
            waypoints.0.pop_front();
        }
    }
}

fn update_prior_of_current_state_system(
    mut query: Query<(&mut FactorGraph, &mut Transform), With<RobotState>>,
    config: Res<Config>,
    time: Res<Time>,
) {
    let scale = time.delta_seconds() / config.simulation.t0;

    for (mut factorgraph, mut transform) in query.iter_mut() {
        let (mean_of_current_variable, increment) = {
            let current_variable = factorgraph
                .nth_variable(0)
                .expect("factorgraph should have a current variable");
            let next_variable = factorgraph
                .nth_variable(1)
                .expect("factorgraph should have a next variable");

            let mean_of_current_variable = current_variable.belief.mean();
            let increment =
                scale * (next_variable.belief.mean() - &mean_of_current_variable);

            (mean_of_current_variable, increment)
        };

        // TODO: create a separate method on Variable so we do not have to abuse the interface, and call it with a empty
        // vector, and get an empty HashMap as a return value.
        let _ = factorgraph
            .nth_variable_mut(0)
            .expect("factorgraph should have a current variable")
            .change_prior(mean_of_current_variable + &increment, vec![]);
        let increment = Vec3::new(increment[0], 0.0, increment[1]);
        transform.translation += increment;
    }
}
