use std::collections::VecDeque;
use std::rc::Rc;

use nalgebra as na;

use crate::{bbox::BoundingBox, message::Message, FactorGraph};
use nutype::nutype;

/// Used to uniquely identify each robot
pub type RobotId = usize;

pub type Position2d = nalgebra::Vector2<f64>;
pub type Velocity2d = nalgebra::Vector2<f64>;
// TODO: should maybe be a Pose2d, to ensure constraints about the heading the robot is expected to have at the waypoint
pub type Waypoint = Position2d;

pub type Timestep = usize;

// struct LookaheadParams {
//     horizon: usize,
//     multiple: usize,
// }

// TODO: take a struct as argument for better names

/// Compute the timesteps at which variables in the planned path are placed.
/// For a lookahead_multiple of 3, variables are spaced at timesteps:
/// 0,  1, 2, 3,  5, 7, 9, 12, 15, 18, ...
/// e.g. variables ar in groups of size lookahead_multiple.
/// The spacing within a group increases by one each time (1, for the first group, 2 for the second etc.)
/// Seems convoluted, but the reasoning was:
/// - The first variable should always be at 1 timestep from the current state (0).
/// - The first few variables should be close together in time
/// - The variables should all be at integer timesteps, but the spacing should sort of increase exponentially.
/// ## Example:
/// ```rust
/// let lookahead_horizon = 20;
/// let lookahead_multiple = 3;
/// assert_eq!(
///     get_variable_timesteps(lookahead_horizon, lookahead_multiple),
///     vec![0, 1, 2, 3, 5, 7, 9, 12, 15, 18, 20]
/// );
/// ```
pub fn get_variable_timesteps(lookahead_horizon: u32, lookahead_multiple: u32) -> Vec<u32> {
    let estimated_capacity = (2.5 * f32::sqrt(lookahead_horizon as f32)) as usize;
    let mut timesteps = Vec::<u32>::with_capacity(estimated_capacity);
    timesteps.push(0);
    // TODO: use std::iter::successors instead
    for i in 1..lookahead_horizon {
        // timesteps[i] = timesteps[i-1] + (i - 1) / lookahead_multiple + 1;
        let ts = timesteps[timesteps.len() - 1] + ((i - 1) / lookahead_multiple) + 1;
        if ts >= lookahead_horizon {
            timesteps.push(lookahead_horizon);
            break;
        }
        timesteps.push(ts);
    }

    timesteps
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_get_variable_timesteps() {
        let lookahead_horizon = 30;
        let lookahead_multiple = 3;

        assert_eq!(
            get_variable_timesteps(lookahead_horizon, lookahead_multiple),
            vec![0, 1, 2, 3, 5, 7, 9, 12, 15, 18, 22, 26, 30]
        );

        let lookahead_horizon = 60;
        let lookahead_multiple = 3;
        assert_eq!(
            get_variable_timesteps(lookahead_horizon, lookahead_multiple),
            vec![0, 1, 2, 3, 5, 7, 9, 12, 15, 18, 22, 26, 30, 35, 40, 45, 51, 57, 60]
        );

        let lookahead_horizon = 10;
        let lookahead_multiple = 3;
        assert_eq!(
            get_variable_timesteps(lookahead_horizon, lookahead_multiple),
            vec![0, 1, 2, 3, 5, 7, 9, 10]
        );

        let lookahead_horizon = 20;
        let lookahead_multiple = 5;
        assert_eq!(
            get_variable_timesteps(lookahead_horizon, lookahead_multiple),
            vec![0, 1, 2, 3, 4, 5, 7, 9, 11, 13, 15, 18, 20],
        );
    }
}

// pub fn get_variable_timesteps(lookahead_horizon: usize, lookahead_multiple: usize) -> Vec<usize> {
//     let estimated_capacity = (2.5 * f32::sqrt(lookahead_horizon as f32)) as usize;

//     std::iter::successors(Some(0), |&x| {
//         Some(x + ((x - 1) / lookahead_multiple).saturating_add(1))
//             .filter(|&ts| ts < lookahead_horizon)
//     })
//     .take_while(|&ts| ts < lookahead_horizon)
//     .collect()
// }

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
    ids_of_robots_withim_comms_range: Vec<RobotId>,
    /// List of robot ids that are currently connected via inter-robot factors to this robot
    ids_of_robots_connected_with: Vec<RobotId>,
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
    pub fn update_current_prior(&mut self) {
        unimplemented!()
    }

    pub fn update_interrobot_factor(&mut self) {
        unimplemented!()
    }

    pub fn create_interrobot_factors(&mut self, other_robot: Rc<Self>) {
        unimplemented!()
    }

    pub fn delete_interrobot_factors(&mut self, other_robot: Rc<Self>) {
        unimplemented!()
    }
}
