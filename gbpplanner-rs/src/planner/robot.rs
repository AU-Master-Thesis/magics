use std::collections::{BTreeSet, VecDeque};
use std::sync::Arc;

use crate::config::Config;

use super::factor::Factor;
use super::factorgraph::FactorGraph;
use super::multivariate_normal::MultivariateNormal;
use super::variable::Variable;
use super::{Matrix, Timestep, Vector};
use bevy::prelude::*;
use ndarray::array;

pub struct RobotPlugin;

pub type RobotId = Entity;

/// Sigma for Unary pose factor on current and horizon states
/// from **gbpplanner** `Globals.h`
const SIGMA_POSE_FIXED: f64 = 1e-15;

impl Plugin for RobotPlugin {
    fn build(&self, app: &mut App) {
        // TODO: add rest
        app.add_systems(
            Update,
            (
                update_robot_neighbours,
                update_interrobot_factors,
                iterate_gbp_internal,
                iterate_gbp_external,
                update_horizon,
                update_current,
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
    pub transform: Transform,
    /// Waypoints used to instruct the robot to move to a specific position.
    /// A VecDeque is used to allow for efficient pop_front operations, and push_back operations.
    pub waypoints: Waypoints,
    // NOTE: Using the Bevy entity id as the robot id
    // pub id: RobotId,
    // NOTE: These are accessible as Bevy resources
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
        obstacle_sdf: Arc<image::RgbImage>,
    ) -> Result<Self, RobotInitError> {
        if waypoints.is_empty() {
            return Err(RobotInitError::NoWaypoints);
        }

        if variable_timesteps.is_empty() {
            return Err(RobotInitError::NoVariableTimesteps);
        }

        let start = waypoints
            .pop_front()
            .expect("Know that waypoints has at least one element");
        let transform = Transform::from_translation(Vec3::new(start.x, 0.0, start.y));
        let goal = waypoints
            .front()
            .expect("Know that waypoints has at least one element");

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
                array![mean.x, mean.y],
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
                &measurement,
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
                Arc::clone(&obstacle_sdf),
                config.simulation.world_size,
            );

            let factor_node_index = factorgraph.add_factor(obstacle_factor);
            let _ = factorgraph.add_edge(variable_node_indices[i], factor_node_index);
        }

        Ok(Self {
            factorgraph,
            radius: Radius(config.robot.radius),
            state: RobotState::new(),
            transform,
            waypoints: Waypoints(waypoints),
        })
    }
}

/// Called `Simulator::calculateRobotNeighbours` in **gbpplanner**
fn update_robot_neighbours(
    // query: Query<(Entity, &Transform, &mut RobotState)>,
    robots: Query<(Entity, &Transform), With<RobotState>>,
    mut states: Query<(Entity, &Transform, &mut RobotState)>,
    config: Res<Config>,
) {
    // TODO: use kdtree to speed up, and to have something in the report
    for (entity_id, transform, mut state) in states.iter_mut() {
        // TODO: maybe use clear() instead
        // unsafe {
        //     state.ids_of_robots_within_comms_range.set_len(0);
        // }
        state.ids_of_robots_within_comms_range = robots
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

/// Called `Robot::updateInterrobotFactors` in **gbpplanner**
/// For new neighbours of a robot, create inter-robot factors if they don't exist.
/// Delete existing inter-robot factors for faraway robots
fn update_interrobot_factors(
    mut primary_query: Query<(Entity, &mut FactorGraph, &RobotState)>,
    mut secondary_query: Query<(Entity, &mut FactorGraph, &RobotState)>,
) {
    // 1. For every robot
    //    1.1 Find the ids of all the robots that it is connected to but, are outside its
    //        communication range.
    //        1.1.1 Delete the robots interrobot factor associated with the other robot
    //        1.1.2 Delete the interrobot factor
    for (entity_id, mut factorgraph, state) in primary_query.iter_mut() {
        let ids_of_robots_connected_with_outside_comms_range: BTreeSet<_> = state
            .ids_of_robots_connected_with
            .difference(&state.ids_of_robots_within_comms_range)
            .cloned()
            .collect();

        for id in ids_of_robots_connected_with_outside_comms_range.iter() {
            if let Some((_, graph, _)) = secondary_query
                .iter_mut()
                .find(|(other_id, _, _)| other_id == id)
            {}

            // flag/store entity_id
        }
        // state.ids_of_robots_connected_with.iter().zip(state.ids_of_robots_within_comms_range.iter()).
    }

    // // Delete interrobot factors between connected neighbours not within range.
    // self.ids_of_robots_connected_with
    //     .difference(&self.ids_of_robots_within_comms_range)
    //     .for_each(|&robot_id| {
    //         if let Some(robot_ptr) = world.robot_with_id(robot_id) {
    //             self.delete_interrobot_factors(robot_ptr);
    //         }
    //         // if let Some(robot_ptr) = world.robots.iter().find(|&it| it.id == robot_id) {
    //         //     self.delete_interrobot_factors(Rc::clone(robot_ptr));
    //         // }
    //     });

    // // Create interrobot factors between any robot within communication range, not already
    // // connected with.
    // self.ids_of_robots_within_comms_range
    //     .difference(&self.ids_of_robots_connected_with)
    //     .for_each(|&robot_id| {
    //         if let Some(mut robot_ptr) = world.robot_with_id_mut(robot_id) {
    //             self.create_interrobot_factors(robot_ptr);
    //             // if (!sim_->symmetric_factors) sim_->robots_.at(rid)->connected_r_ids_.push_back(rid_);
    //             if !self.settings.symmetric_factors {
    //                 robot_ptr.ids_of_robots_connected_with.insert(self.id);
    //             }
    //         }
    //     });
}

/// Called `Simulator::setCommsFailure` in **gbpplanner**
fn update_failed_comms(mut query: Query<&mut RobotState>, config: Res<Config>) {
    for mut state in query.iter_mut() {
        state.interrobot_comms_active =
            config.robot.communication.failure_rate > rand::random::<f32>();
    }
}

macro_rules! iterate_gbp_impl {
    ($name:ident, $mode:expr) => {
        fn $name(mut query: Query<&mut FactorGraph>, config: Res<Config>) {}
    };
}

iterate_gbp_impl!(iterate_gbp_internal, MessagePassingMode::Internal);
iterate_gbp_impl!(iterate_gbp_external, MessagePassingMode::External);

// fn iterate_gbp(query: Query<&mut FactorGraph>, config: Res<Config>) {}

fn update_horizon(
    query: Query<(&mut FactorGraph, &mut Waypoints), With<RobotState>>,
    config: Res<Config>,
) {
}
fn update_current(
    query: Query<(&mut FactorGraph, &mut Transform), With<RobotState>>,
    config: Res<Config>,
) {
}
