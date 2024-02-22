use std::collections::{BTreeSet, VecDeque};
use std::rc::Rc;

use nalgebra::{DMatrix, DVector};

use crate::config::Config;

use super::factorgraph::FactorGraph;
use super::multivariate_normal::MultivariateNormal;
use super::variable::Variable;
use super::Timestep;
use bevy::prelude::*;

pub struct RobotPlugin;

/// Sigma for Unary pose factor on current and horizon states
/// from **gbpplanner** `Globals.h`
const SIGMA_POSE_FIXED: f64 = 1e-15;

impl Plugin for RobotPlugin {
    fn build(&self, app: &mut App) {
        todo!()
        // app.add_system();
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
    pub ids_of_robots_within_comms_range: BTreeSet<Entity>,
    /// List of robot ids that are currently connected via inter-robot factors to this robot
    /// called `connected_r_ids_` in **gbpplanner**.
    pub ids_of_robots_connected_with: BTreeSet<Entity>,
}

impl RobotState {
    pub fn new() -> Self {
        Self {
            ids_of_robots_within_comms_range: BTreeSet::new(),
            ids_of_robots_connected_with: BTreeSet::new(),
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

            let sigmas = DVector::<f32>::from_fn(ndofs, |_, _| {
                if sigma == 0.0 {
                    f32::MAX
                } else {
                    1.0 / (sigma as f32).powi(2)
                }
            });
            let covariance = DMatrix::<f32>::from_diagonal(&sigmas);
            let prior = MultivariateNormal::from_mean_and_covariance(mean, covariance);

            let variable = Variable::new(prior, ndofs);
            let variable_index = factorgraph.add_variable(variable);
            variable_node_indices.push(variable_index);
        }

        // Create Dynamics factors between variables
        for i in 0..variable_timesteps.len() - 1 {
            // T0 is the timestep between the current state and the first planned state.
            let delta_t = config.simulation.t0
                * (variable_timesteps[i + 1] - variable_timesteps[i]) as f32;
            let measurement = DVector::<f32>::zeros(config.dofs);
            let dynamic_factor = Factor::new_dynamic_factor(
                config.gbp.sigma_factor_dynamics,
                &measurement,
                config.dofs,
                delta_t,
            );
            let dynamic_factor = Rc::new(dynamic_factor);
            let factor_node_index = factorgraph.add_factor(dynamic_factor);
            let _ = factorgraph.add_edge(variable_node_indices[i], factor_node_index);
            let _ = factorgraph.add_edge(variable_node_indices[i + 1], factor_node_index);
        }

        // Create Obstacle factors for all variables excluding start, excluding horizon
        for i in 1..variable_timesteps.len() - 1 {
            let obstacle_factor = Factor::new_obstacle_factor(
                config.gbp.sigma_factor_obstacle,
                dvector![0.0],
                config.dofs,
                Rc::clone(&obstacle_sdf),
                config.simulation.world_size,
            );

            let obstacle_factor = Rc::new(obstacle_factor);
            let factor_node_index = factorgraph.add_factor(obstacle_factor);
            let _ = factorgraph.add_edge(variable_node_indices[i], factor_node_index);
        }

        Ok(Self {
            factorgraph,
            radius: Radius(config.robot.radius),
            state: RobotState::new(),
            transform,
            waypoints,
        })
    }
}
