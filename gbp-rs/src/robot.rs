use std::collections::{BTreeSet, VecDeque};
use std::rc::Rc;

// use nalgebra as na;

use crate::{message::Message, FactorGraph};
use nutype::nutype;

/// Used to uniquely identify each robot
pub type RobotId = usize;

pub type Position2d = nalgebra::Vector2<f64>;
pub type Velocity2d = nalgebra::Vector2<f64>;
// TODO: should maybe be a Pose2d, to ensure constraints about the heading the robot is expected to have at the waypoint
pub type Waypoint = Position2d;


#[derive(Debug)]
pub struct Pose2d {
    pub position: Position2d,
    pub orientation: f64,
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

/// Characteristics of the communication media used by the robot.
/// This is used to model properties such as the maximum range of communication.
#[derive(Debug)]
pub struct CommunicationMedia {
    pub max_range: Meter,
    pub failure_probability: Probability,
}

// int num_variables_;                         // Number of variables in the planned path (assumed to be the same for all robots)

#[derive(Debug)]
pub struct Robot {
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
    communication_media: CommunicationMedia,
    message_queue: VecDeque<Message>,
    /// Radius of the robot.
    /// If the robot is not a perfect circle, then set radius to be the smallest circle that fully encompass the shape of the robot.
    /// **constraint**: > 0.0
    pub radius: f64, // TODO: create new type that guarantees this constraint

    /// Optional shared pointer to image representing the obstacles in the environment as a SDF (Signed Distance Field) map.
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
    /// Variables representing the planned path are at timesteps which increase in spacing.
    variable_timesteps: Vec<Timestep>,
}

impl Robot {
    pub fn position(&self) -> &Position2d {
        &self.state.pose.position
    }

    pub fn position_mut(&mut self) -> &mut Position2d {
        &mut self.state.pose.position
    }

    pub fn orientation(&self) -> f64 {
        self.state.pose.orientation
    }

    pub fn velocity(&self) -> &Velocity2d {
        &self.state.velocity
    }

    pub fn velocity_mut(&mut self) -> &mut Velocity2d {
        &mut self.state.velocity
    }

    // pub fn new(
    //     id: RobotId,
    //     factorgraph: FactorGraph,
    //     state: RobotState,
    //     communication_media: CommunicationMedia,
    //     bbox: B,
    // ) -> Self {
    //     Self {
    //         id,
    //         factorgraph,
    //         state,
    //         waypoints: VecDeque::new(),
    //         communication_media,
    //         message_queue: VecDeque::new(),
    //         bbox,
    //     }
    // }

    /// Update the prior of the horizon state
    pub fn update_horizon_prior(&mut self) {
        unimplemented!()
    }

    /// Update the prior of the current state
    pub fn update_current_prior(&mut self, cx: &GlobalContext) {
        let increment = {
            let mean = match self.factorgraph.variables.len() {
                0 => unreachable!(),
                1 => {
                    &self
                        .factorgraph
                        .variables
                        .first()
                        .expect("The .len() is 1, so the first element exist")
                        .mean
                }
                _ => {
                    let first = &self.factorgraph.variables[0];
                    let second = &self.factorgraph.variables[1];
                    second.mean - first.mean
                }
            };
            mean * cx.timestep / cx.t0
        };

        let mut current_var =
    }

    // TODO: test
    // fn ids_of_robots_connected_with_within_comms_range(&self) -> impl Iterator<Item = &usize> {
    //     self.ids_of_robots_connected_with
    //         .iter()
    //         .filter(|&connected_with_id| {
    //             !self
    //                 .ids_of_robots_within_comms_range
    //                 .iter()
    //                 .any(|within_comms_range_id| connected_with_id == within_comms_range_id)
    //         })
    // }

    // TODO: test
    // fn ids_of_robots_connected_with_not_within_comms_range(&self) -> impl Iterator<Item = &usize> {
    //     self.ids_of_robots_within_comms_range
    //         .iter()
    //         .filter(|&within_comms_range_id| {
    //             !self
    //                 .ids_of_robots_connected_with
    //                 .iter()
    //                 .any(|connected_with_id| within_comms_range_id == connected_with_id)
    //         })
    // }

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
        for i in 1..self.variable_timesteps.len() {}

        // Add the other robot to this robot's list of connected robots.
        self.ids_of_robots_connected_with.push(other_robot.id);

        unimplemented!()
    }

    pub fn delete_interrobot_factors(&mut self, other_robot: Rc<Self>) {
        unimplemented!()
    }
}
