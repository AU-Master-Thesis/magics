use bevy::prelude::*;
use gbp_config::{formation::{PlanningStrategy, ReachedWhen}, Config};
use gbp_linalg::prelude::*;
use ndarray::{array, concatenate, s, Axis};

use crate::{
    factorgraph::{
        factor::{ExternalVariableId, FactorNode},
        factorgraph::{FactorGraph, NodeIndex, VariableIndex},
        id::{FactorId, VariableId},
        variable::VariableNode,
        DOFS,
    },
    simulation_loader::SdfImage,
};

use super::{
    gbp::GbpIterationSchedule,
    mission::Mission,
    utils::RobotNumberGenerator, // Assuming utils.rs will exist
};

pub type RobotId = Entity;

/// Component for entities with a radius, used for robots
#[derive(Component, Debug, Deref, DerefMut)]
pub struct Radius(pub f32);

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
    pub robots_within_comms_range: std::collections::BTreeSet<RobotId>,
    /// List of robot ids that are currently connected via inter-robot factors
    /// to this robot called `connected_r_ids_` in **gbpplanner**.
    pub robots_connected_with:     std::collections::BTreeSet<RobotId>,
}

impl RobotConnections {
    /// Create a new `RobotState`
    #[must_use]
    pub fn new() -> Self {
        Self {
            robots_within_comms_range: std::collections::BTreeSet::new(),
            robots_connected_with:     std::collections::BTreeSet::new(),
        }
    }
}

// TODO: change to collider
#[derive(Debug, Component, Deref)]
pub struct Ball(parry2d::shape::Ball);

#[derive(Component, Debug)]
pub struct VariableTimesteps(pub Vec<u32>); // Made pub

#[derive(Component, Deref, Clone, Copy)]
pub struct T0(pub f32);

#[derive(Component, Debug, Default)]
pub struct FinishedPath(pub bool);

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
        let precision = 3;
        write!(
            f,
            "[{:.precision$}, {:.precision$}, {:.precision$}, {:.precision$}]",
            self.0.x,
            self.0.y,
            self.0.z,
            self.0.w,
            precision = precision,
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

    /// Time between t_i and t_i+1
    pub t0: T0,

    /// Boolean component used to keep track of whether the robot has finished
    /// its path by reaching its final waypoint. This flag exists to ensure
    /// that the robot is not detected as having finished more than once.
    /// TODO: should probably be modelled as an enum instead, too easier support
    /// additional states in the future
    finished_path: FinishedPath,

    pub mission: Mission,
    pub planning_strategy: PlanningStrategy,

    pub variable_timesteps: VariableTimesteps,
}

impl RobotBundle {
    /// Create a new `RobotBundle`
    #[must_use = "Constructor responsible for creating the robots factorgraph"]
    #[allow(clippy::missing_panics_doc)]
    pub fn new(
        robot_id: RobotId,
        initial_state: StateVector,
        variable_timesteps: &[u32],
        config: &Config,
        env_config: &gbp_environment::Environment,
        radius: f32,
        sdf: &SdfImage,
        started_at: f64,
        waypoints: min_len_vec::TwoOrMore<StateVector>,
        planning_strategy: PlanningStrategy,
        waypoint_reached_when_intersects: ReachedWhen,
        finished_when_intersects: ReachedWhen,
    ) -> Self {
        use itertools::Itertools; // Import trait for tuple_windows

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
                (config.robot.planning_horizon * config.robot.target_speed).get(),
            ) * start2goal.normalize();

        let mut factorgraph = FactorGraph::new(robot_id);
        // Initialize factor weights from config
        factorgraph.initialize_weights_from_config(config);

        let last_variable_timestep = *variable_timesteps
            .last()
            .expect("Know that variable_timesteps has at least one element");
        let n_variables = variable_timesteps.len();
        let mut variable_node_indices = Vec::with_capacity(n_variables);

        let mut init_variable_means = Vec::<Vector<Float>>::with_capacity(n_variables);
        for (i, &variable_timestep) in variable_timesteps.iter().enumerate() {
            // Set initial mean and covariance of variable interpolated between start and
            // horizon
            let mean = match planning_strategy {
                PlanningStrategy::OnlyLocal => {
                    start
                        + (horizon - start)
                            * (variable_timestep as f32 / last_variable_timestep as f32)
                }
                PlanningStrategy::RrtStar => start,
            };

            let sigma = if i == 0 || i == n_variables - 1 {
                // Start and Horizon state variables should be 'fixed' during optimisation at a
                // timestep SIGMA_POSE_FIXED
                1e30
            } else {
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

        let t0 = radius / 2.0 / config.robot.target_speed.get();

        // Create Dynamic factors between variables
        for i in 0..variable_timesteps.len() - 1 {
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
            PlanningStrategy::OnlyLocal => Mission::local(
                waypoints.try_into().unwrap(),
                started_at,
                finished_when_intersects,
                waypoint_reached_when_intersects,
            ),
            PlanningStrategy::RrtStar => Mission::global(
                waypoints.try_into().unwrap(),
                started_at,
                finished_when_intersects,
                waypoint_reached_when_intersects,
            ),
        };

        // Create Tracking factors for all variables, excluding the start
        for i in 1..variable_timesteps.len() - 1 {
            let init_linearisation_point =
                concatenate![Axis(0), init_variable_means[i].clone(), array![0.0, 0.0]];
            let initial_route = mission.active_route().unwrap();
            let waypoints = initial_route
                .waypoints() // Use the public waypoints() method
                .iter()
                .map(|w| w.position())
                .collect::<Vec<Vec2>>();
            let tracking_factor = FactorNode::new_tracking_factor(
                factorgraph.id(),
                Float::from(config.gbp.sigma_factor_tracking),
                array![0.0],
                init_linearisation_point,
                config.gbp.tracking.clone(),
                Some(waypoints.try_into().unwrap()),
                config.gbp.factors_enabled.tracking,
            );

            let factor_node_index = factorgraph.add_factor(tracking_factor);
            let factor_id = FactorId::new(factorgraph.id(), factor_node_index);
            let _ = factorgraph.add_internal_edge(
                VariableId::new(factorgraph.id(), variable_node_indices[i]),
                factor_id,
            );
        }

        Self {
            factorgraph,
            radius: Radius(radius),
            ball: Ball(parry2d::shape::Ball::new(radius)),
            antenna: RadioAntenna::new(config.robot.communication.radius.get(), true),
            connections: RobotConnections::new(),
            finished_path: FinishedPath::default(),
            t0: T0(t0),
            gbp_iteration_schedule: GbpIterationSchedule(config.gbp.iteration_schedule),
            mission,
            planning_strategy,
            variable_timesteps: VariableTimesteps(variable_timesteps.to_owned()),
        }
    }
}
