use std::cell::RefCell;
use std::collections::{BTreeSet, VecDeque};
use std::rc::Rc;

// use nalgebra as na;

use crate::factorgraph::FactorGraph;
use crate::multivariate_normal::MultivariateNormal;
use crate::{IdGenerator, Key, Factor};
use crate::{Timestep, Variable};

use nalgebra::{DMatrix, DVector, dvector};
// use crate::{message::Message, FactorGraph};
use nutype::nutype;
use serde::{Deserialize, Serialize};

/// Used to uniquely identify each robot
pub type RobotId = usize;

pub type Position2d = nalgebra::Vector2<f32>;
pub type Velocity2d = nalgebra::Vector2<f32>;
// TODO: should maybe be a Pose2d, to ensure constraints about the heading the robot is expected to have at the waypoint
pub type Waypoint = Position2d;

#[derive(Debug)]
pub struct Pose2d {
    pub position: Position2d,
    pub orientation: f32,
}

/// How a robots state (that can change over time) is modelled in the gbpplanner paper.
#[derive(Debug)]
pub struct RobotState {
    pub pose: Pose2d,
    pub velocity: Velocity2d,
}

#[derive(Debug, Clone, Copy)]
pub struct Meter(pub f64);

// TOOD: move to lib.rs
/// Represents a probability value in the range [0,1]
#[nutype(
    validate(greater_or_equal = 0.0, less_or_equal = 1.0),
    derive(Debug, Clone, Copy)
)]
pub struct Probability(f64);

/// Sigma for Unary pose factor on current and horizon states
/// from **gbpplanner** `Globals.h`
const SIGMA_POSE_FIXED: f64 = 1e-15;

/// Characteristics of the communication media used by the robot.
/// This is used to model properties such as the maximum range of communication.
#[derive(Debug)]
pub struct CommunicationMedia {
    pub max_range: Meter,
    pub failure_probability: Probability,
}

#[derive(Debug, thiserror::Error)]
pub enum RobotInitError {
    #[error("No waypoints were provided")]
    NoWaypoints,
    #[error("No variable timesteps were provided")]
    NoVariableTimesteps,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GbpParameters {
    /// Sigma for Unary pose factor on current and horizon states
    pub sigma_pose_fixed: f32,
    /// Sigma for Dynamics factors
    pub sigma_factor_dynamics: f32,
    /// Sigma for Interrobot factor
    pub sigma_factor_interrobot: f32,
    /// Sigma for Static obstacle factors
    pub sigma_factor_obstacle: f32,
    /// Number of iterations of GBP per timestep
    pub num_iters: usize,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct RobotSettings {
    /// SI unit: s
    pub planning_horizon: f32,
    /// SI unit: m/s
    pub max_speed: f32,
    /// Degrees of freedom of the robot's state [x, y, x', y']
    pub dofs: usize,
    /// Inter-robot factors created if robots are within this range of each other
    /// SI unit: m
    pub communication_radius: f32, 
    // /// Simulation timestep interval
    // /// FIXME: does not belong to group of parameters, should be in SimulationSettings or something
    // pub delta_t: f32,
    pub gbp: GbpParameters,
    pub simulation: SimulationParameters,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationParameters {
    /// SI unit: s
    pub timestep: f32,
    /// Time between current state and next state of planned path
    /// SI unit: s
    pub t0: f32,
}

#[derive(Debug)]
pub struct Robot<'a> {
    /// Unique identifier for the robot.
    /// NOTE: it is up to the user to ensure that these are unique, among all robots.
    pub id: RobotId,
    /// The factor graph that the robot is part of, and uses to perform GBP message passing.
    factorgraph: FactorGraph,
    /// The current state of the robot
    state: RobotState,
    /// Waypoints used to instruct the robot to move to a specific position.
    /// A VecDeque is used to allow for efficient pop_front operations, and push_back operations.
    waypoints: VecDeque<Waypoint>,
    // communication_media: CommunicationMedia,
    // message_queue: VecDeque<Message>,
    /// Radius of the robot.
    /// If the robot is not a perfect circle, then set radius to be the smallest circle that fully encompass the shape of the robot.
    /// **constraint**: > 0.0
    pub radius: f64, // TODO: create new type that guarantees this constraint

    /// Optional shared pointer to image representing the obstacles in the environment as a SDF (Signed Distance Field) map.
    /// TODO(kpbaks): I do not think it has to be wrapped in a Rc<>, a 'a& should be enough
    obstacle_sdf: Option<Rc<image::RgbImage>>,

    /// List of robot ids that are within the communication radius of this robot.
    /// called `neighbours_` in **gbpplanner**.
    /// TODO: maybe change to a BTreeSet
    // ids_of_robots_within_comms_range: Vec<RobotId>,
    ids_of_robots_within_comms_range: BTreeSet<RobotId>,
    /// List of robot ids that are currently connected via inter-robot factors to this robot
    /// called `connected_r_ids_` in **gbpplanner**.
    /// TODO: maybe change to a BTreeSet
    // ids_of_robots_connected_with: Vec<RobotId>,
    ids_of_robots_connected_with: BTreeSet<RobotId>,
    // TODO: this property is the same for all robots in gbpplanner so should propably just be of type &[Timestep]
    // instead of allocating the same identical vector for each robot.
    // Variables representing the planned path are at timesteps which increase in spacing.
    // variable_timesteps: Vec<Timestep>,

    settings: &'a RobotSettings,
    id_generator: Rc<RefCell<IdGenerator>>,
}

impl<'a> Robot<'a> {
    #[must_use = "Constructor responsible for creating the robots factorgraph"]
    pub fn new(
        id: RobotId,
        initial_state: RobotState,
        waypoints: VecDeque<Waypoint>,
        radius: f64,
        planning_horizon: f32,
        max_speed: f32,
        variable_timesteps: &[Timestep],
        obstacle_sdf: Rc<image::RgbImage>,
        settings: &'a RobotSettings,
        mut id_generator: Rc<RefCell<IdGenerator>>,
    ) -> Result<Self, RobotInitError> {
        if waypoints.is_empty() {
            return Err(RobotInitError::NoWaypoints);
        }

        if variable_timesteps.is_empty() {
            return Err(RobotInitError::NoVariableTimesteps);
        }

        let start = &initial_state.pose.position;
        let goal = waypoints.front().expect("Know that waypoints has at least one element");

        // Initialise the horzion in the direction of the goal, at a distance T_HORIZON * MAX_SPEED from the start.
        let start2goal = goal - start;
        let horizon = start + f32::min(start2goal.norm(),
    planning_horizon * max_speed) * start2goal.normalize();

        let ndofs = 4; // [x, y, x', y']

        let mut factorgraph = FactorGraph::new();
    // /***************************************************************************/
    // /* Create Variables with fixed pose priors on start and horizon variables. */
    // /***************************************************************************/
    // Color var_color = color_; double sigma; int n = globals.N_DOFS;
    // Eigen::VectorXd mu(n); Eigen::VectorXd sigma_list(n);
    // for (int i = 0; i < num_variables_; i++){
    //     // Set initial mu and covariance of variable interpolated between start and horizon
    //     mu = start + (horizon - start) * (float)(variable_timesteps[i]/(float)variable_timesteps.back());
    //     // Start and Horizon state variables should be 'fixed' during optimisation at a timestep
    //     sigma = (i==0 || i==num_variables_-1) ? globals.SIGMA_POSE_FIXED : 0.;
    //     sigma_list.setConstant(sigma);

    //     // Create variable and add to robot's factor graph
    //     auto variable = std::make_shared<Variable>(sim->next_vid_++, rid_, mu, sigma_list, robot_radius_, n);
    //     variables_[variable->key_] = variable;
    // }

        let last_variable_timestep = *variable_timesteps.last().expect("Know that variable_timesteps has at least one element");
        for i in 0..variable_timesteps.len() {
            // Set initial mu and covariance of variable interpolated between start and horizon
            let mean = start + (horizon - start) * (variable_timesteps[i] as f32 / last_variable_timestep as f32);
            // Start and Horizon state variables should be 'fixed' during optimisation at a timestep
            let sigma = if i == 0 || i == variable_timesteps.len() - 1 {
                SIGMA_POSE_FIXED
            } else {
                0.0
            };

            let sigmas = DVector::<f32>::from_element(ndofs, sigma as f32);
            // FIXME: this is not the correct way to create a covariance matrix
            let covariance = DMatrix::<f32>::from_diagonal(&sigmas);
            let mean = DVector::<f32>::from_iterator(mean.nrows(), mean.into_iter().cloned());
            let prior = MultivariateNormal::from_mean_and_covariance(mean, covariance);
            let key = Key::new(id, id_generator.get_mut().next_variable_id());

            let variable = Variable::new(
                key,
                prior,
                ndofs,
            );

            factorgraph.variables.insert(variable.key, Rc::new(variable));
        }

        // Create Dynamics factors between variables 
        for i in 0..variable_timesteps.len() - 1 {
            // T0 is the timestep between the current state and the first planned state.
            let delta_t = settings.t0 * (variable_timesteps[i + 1] - variable_timesteps[i]) as f32;
            let adjacent_variables = vec![
                factorgraph
                    .get_variable_by_index(i)
                    .expect("The index is within [0, len)")
                    .clone(),
                factorgraph
                    .get_variable_by_index(i + 1)
                    .expect("The index is within [0, len)")
                    .clone(),
            ];
            let key = Key::new(id, id_generator.get_mut().next_factor_id());
            let measurement = DVector::<f32>::zeros(settings.dofs);
            let dynamic_factor = Factor::new_dynamic_factor(key, adjacent_variables, settings.gbp.sigma_factor_dynamics, &measurement, settings.dofs, delta_t);
            let dynamic_factor = Rc::new(dynamic_factor);

            // Add the factor to the variance's list of adjacent factor, as well as the robot's list of factors.
            for var in dynamic_factor.adjacent_variables.iter() {
                var.add_factor(Rc::clone(&dynamic_factor));
            }

            factorgraph.factors.insert(key, dynamic_factor);
        }

        // Create Obstacle factors for all variables excluding start, excluding horizon
        for i in 1..variable_timesteps.len() - 1 {
            let adjacent_variables = vec![
                factorgraph
                    .get_variable_by_index(i)
                    .expect("The index is within [0, len)")
                    .clone(),
            ];
            let key = Key::new(id, id_generator.get_mut().next_factor_id());
            let obstacle_factor = Factor::new_obstacle_factor(key, adjacent_variables,
                settings.gbp.sigma_factor_obstacle,
                dvector![0.0],
                settings.dofs,
                Rc::clone(&obstacle_sdf)
            );
            let obstacle_factor = Rc::new(obstacle_factor);

            // Add the factor to the variance's list of adjacent factor, as well as the robot's list of factors.
            for var in obstacle_factor.adjacent_variables.iter() {
                var.add_factor(Rc::clone(&obstacle_factor));
            }

            factorgraph.factors.insert(key, obstacle_factor);
        }

        Ok(Self {
            id,
            factorgraph,
            state: initial_state,
            waypoints,
            radius,
            obstacle_sdf: Some(obstacle_sdf),
            ids_of_robots_within_comms_range: BTreeSet::new(),
            ids_of_robots_connected_with: BTreeSet::new(),
            settings,
            id_generator,
        })
    }

    pub fn position(&self) -> &Position2d {
        &self.state.pose.position
    }

    pub fn position_mut(&mut self) -> &mut Position2d {
        &mut self.state.pose.position
    }

    pub fn orientation(&self) -> f32 {
        self.state.pose.orientation
    }

    pub fn velocity(&self) -> &Velocity2d {
        &self.state.velocity
    }

    pub fn velocity_mut(&mut self) -> &mut Velocity2d {
        &mut self.state.velocity
    }

    pub fn get_variable_by_index(&self, index: usize) -> Option<Rc<Variable>> {
        self.factorgraph.get_variable_by_index(index)
    }

//     /***************************************************************************************************/
// /* Change the prior of the Horizon state */
// /***************************************************************************************************/
// void Robot::updateHorizon(){
//     // Horizon state moves towards the next waypoint.
//     // The Horizon state's velocity is capped at MAX_SPEED
//     auto horizon = getVar(-1);      // get horizon state variable
//     Eigen::VectorXd dist_horz_to_goal = waypoints_.front()({0,1}) - horizon->mu_({0,1});
//     Eigen::VectorXd new_vel = dist_horz_to_goal.normalized() * std::min((double)globals.MAX_SPEED, dist_horz_to_goal.norm());
//     Eigen::VectorXd new_pos = horizon->mu_({0,1}) + new_vel*globals.TIMESTEP;

//     // Update horizon state with new pos and vel
//     horizon->mu_ << new_pos, new_vel;
//     horizon->change_variable_prior(horizon->mu_);

//     // If the horizon has reached the waypoint, pop that waypoint from the waypoints.
//     // Could add other waypoint behaviours here (maybe they might move, or change randomly).
//     if (dist_horz_to_goal.norm() < robot_radius_){
//         if (globals.FORMATION == "custom") {
//             waypoints_.front()(0) = sim_->random_int(-globals.WORLD_SZ/2, globals.WORLD_SZ/2);
//             waypoints_.front()(1) = sim_->random_int(-globals.WORLD_SZ/2, globals.WORLD_SZ/2);
//         } else {
//             if (waypoints_.size()>1) waypoints_.pop_front();
//         }
//     }
// }

    /// Update the prior of the horizon state
    pub fn update_horizon_prior(&mut self) {
        let horizon = self.factorgraph.variables.values().last().expect("There is at least one variable");
        let dist_horz_to_goal = self.waypoints.front().expect("There is at least one waypoint") - horizon.prior.mean();
        let new_velocity = dist_horz_to_goal.normalize() * f32::min(
            dist_horz_to_goal.norm(),
            self.settings.max_speed,
        );
        let new_position = horizon.prior.mean().columns(0, 2) + new_velocity * self.settings.t0;

        // Update horizon state with new pos and vel
        horizon.prior.mean().set_columns(0, 2, &new_position);
        horizon.prior.mean().set_columns(2, 4, &new_velocity);
        horizon.change_prior(horizon.prior.mean());

        // If the horizon has reached the waypoint, pop that waypoint from the waypoints.
        // Could add other waypoint behaviours here (maybe they might move, or change randomly).
        if dist_horz_to_goal.norm() < self.radius as f32 {
            if self.waypoints.len() > 1 {
                self.waypoints.pop_front();
            }
        }
    }

//     void Robot::updateHorizon(){
//     // Horizon state moves towards the next waypoint.
//     // The Horizon state's velocity is capped at MAX_SPEED
//     auto horizon = getVar(-1);      // get horizon state variable
//     Eigen::VectorXd dist_horz_to_goal = waypoints_.front()({0,1}) - horizon->mu_({0,1});                        
//     Eigen::VectorXd new_vel = dist_horz_to_goal.normalized() * std::min((double) globals.MAX_SPEED, dist_horz_to_goal.norm());
//     Eigen::VectorXd new_pos = horizon->mu_({0,1}) + new_vel * globals.TIMESTEP;
    
//     // Update horizon state with new pos and vel
//     horizon->mu_ << new_pos, new_vel;
//     horizon->change_variable_prior(horizon->mu_);

//     // If the horizon has reached the waypoint, pop that waypoint from the waypoints.
//     // Could add other waypoint behaviours here (maybe they might move, or change randomly).
//     if (dist_horz_to_goal.norm() < robot_radius_){
//         if (globals.FORMATION == "custom") {
//             waypoints_.front()(0) = sim_->random_int(-globals.WORLD_SZ/2, globals.WORLD_SZ/2);
//             waypoints_.front()(1) = sim_->random_int(-globals.WORLD_SZ/2, globals.WORLD_SZ/2);
//         } else {
//             if (waypoints_.size()>1) waypoints_.pop_front();
//         }
//     }
// }

    
    // void Robot::updateCurrent(){
    //     // Move plan: move plan current state by plan increment
    //     Eigen::VectorXd increment = ((*this)[1]->mu_ - (*this)[0]->mu_) * globals.TIMESTEP / globals.T0;
    //     // In GBP we do this by modifying the prior on the variable
    //     getVar(0)->change_variable_prior(getVar(0)->mu_ + increment);
    //     // Real pose update
    //     position_ = position_ + increment;

    // }

    /// Update the prior of the current state
    pub fn update_current_prior(&mut self) {
        let increment: DVector<f32> = {
            let mean = match self.factorgraph.variables.len() {
                0 => unreachable!(), // gbpplanner does not handle this case
                1 => {
                    // gbpplanner does not handle this case. I assume this is what the update should do in this case
                    self
                        .factorgraph
                        .get_variable_by_index(0)
                        .expect("There is one variable")
                        .belief.mean()
                        // .prior.mean()
                }
                _ => {
                    let first = &self.factorgraph.get_variable_by_index(0).expect("There are 2 or more variables");
                    let second = &self.factorgraph.get_variable_by_index(1).expect("There are 2 or more variables");
                    second.belief.mean() - first.belief.mean()
                }
            };
            mean * self.settings.simulation.timestep / self.settings.simulation.timestep
        };

        let mean = self.factorgraph.get_variable_by_index(0).expect("There is at least one variable").belief.mean();
        self.factorgraph.get_variable_by_index(0).expect("There is at least one variable")
            .change_prior(mean + increment);

        // Update pose
        self.state.pose.position += increment;
    }

    /// For new neighbours of a robot, create inter-robot factors if they don't exist.
    /// Delete existing inter-robot factors for faraway robots
    pub fn update_interrobot_factors(&mut self) {
        // Delete interrobot factors between connected neighbours not within range.
        self.ids_of_robots_connected_with
            .difference(&self.ids_of_robots_within_comms_range)
            .for_each(|robot_id| {
                // TODO: somehow call Robot::delete_interrobot_factors()
            });

        // Create interrobot factors between any robot within communication range, not already
        // connected with.
        self.ids_of_robots_within_comms_range
            .difference(&self.ids_of_robots_connected_with)
            .for_each(|robot_id| {
                // TODO: somehow call Robot::delete_interrobot_factors()
                // c++: if (!sim_->symmetric_factors) sim_->robots_.at(rid)->connected_r_ids_.push_back(rid_);
            });
    }

    pub fn create_interrobot_factors(&mut self, other_robot: Rc<Self>) {
        // Create Interrobot factors for all timesteps excluding current state
        for i in 1..self.factorgraph.variables.len() {
            let variable = self
                .factorgraph
                .get_variable_by_index(i)
                .expect("The index is within [0, len)")
                .clone();
            let other_robot_variable = other_robot
                .factorgraph
                .get_variable_by_index(i)
                .expect("The index is within [0, len)")
                .clone();

            // Create the interrobot factor
            let zeros = DVector::<f32>::zeros(variable.dofs);
            let
        }

        // Add the other robot to this robot's list of connected robots.
        self.ids_of_robots_connected_with.push(other_robot.id);

        unimplemented!()
    }

    pub fn delete_interrobot_factors(&mut self, other_robot: Rc<Self>) {
        unimplemented!()
    }
}
