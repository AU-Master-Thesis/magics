use std::collections::VecDeque;

use nutype::nutype;
use crate::{bbox::BoundingBox, message::Message, FactorGraph};

pub type RobotId = usize;

pub type Position2d = nalgebra::Vector2<f64>;
pub type Velocity2d = nalgebra::Vector2<f64>;
pub type Waypoint = Position2d;

#[derive(Debug)]
pub struct Pose2d {
    pub position: Position2d,
    pub orientation: f64,
}

/// How a robots state (that can change over time) is modelled in the gbpplanner paper.
#[derive(Debug)]
pub struct RobotState {
    // pub position: Position2d,
    pub pose: Pose2d,
    pub velocity: Velocity2d,
}


#[derive(Debug, Clone, Copy)]
pub struct Meter(pub f64);

/// Represents a probability value in the range [0,1]
#[nutype(
    validate(greater_or_equal = 0.0, less_or_equal = 1.0),
    derive(Debug, Clone, Copy),
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
pub struct Robot<B: BoundingBox> {
    /// Unique identifier for the robot.
    /// NOTE: it is up to the user to ensure that these are unique, among all robots.
    id: RobotId,
    /// The factor graph that the robot is part of, and uses to perform GBP message passing.
    factorgraph: FactorGraph,
    /// The current state of the robot
    state: RobotState,
    /// Waypoints used to instruct the robot to move to a specific position.
    /// A VecDeque is used to allow for efficient pop_front operations, and push_back operations.
    waypoints: VecDeque<Waypoint>,
    communication_media: CommunicationMedia,
    message_queue: VecDeque<Message>,
    /// The bounding box of the robot, used to model the physical space occupied by the robot.
    bbox: B,
}

impl <B: BoundingBox> Robot <B> {
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

    pub fn new(id: RobotId, factorgraph: FactorGraph, state: RobotState, communication_media: CommunicationMedia, bbox: B) -> Self {
        Self {
            id,
            factorgraph,
            state,
            waypoints: VecDeque::new(),
            communication_media,
            message_queue: VecDeque::new(),
            bbox,
        }
    }

    /// Update the prior of the horizon state
    pub fn update_horizon_prior(&mut self) {
        unimplemented!()
    }

    /// Update the prior of the current state
    pub fn update_current_prior(&mut self) {
        unimplemented!()
    }
}
