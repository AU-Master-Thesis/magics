//! Robot state and planning for the simulation
//! 
//! This module defines types and structures related to robot state and planning.

use serde::{Serialize, Deserialize};
use crate::types::Vector2;

/// A robot identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RobotId(pub usize);

impl RobotId {
    /// Create a new robot ID
    pub fn new(id: usize) -> Self {
        Self(id)
    }
    
    /// Get the raw ID value
    pub fn value(&self) -> usize {
        self.0
    }
}

/// Component for entities with a radius, used for robots
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Radius(pub f32);

impl Radius {
    /// Create a new radius
    pub fn new(radius: f32) -> Self {
        Self(radius)
    }
    
    /// Get the radius value
    pub fn value(&self) -> f32 {
        self.0
    }
}

/// Radio antenna capabilities for communication between robots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadioAntenna {
    /// The radius that the radio antenna can cover
    pub radius: f32,
    /// Whether the antenna is currently active
    pub active: bool,
}

impl RadioAntenna {
    /// Creates a new radio antenna
    pub fn new(radius: f32, active: bool) -> Self {
        Self { radius, active }
    }

    /// Toggle the state of the antenna between on and off
    pub fn toggle(&mut self) {
        self.active = !self.active;
    }

    /// Check whether a given position is within the antenna's range
    pub fn within_range(&self, position: Vector2, robot_position: Vector2) -> bool {
        (position - robot_position).magnitude() < self.radius as f64
    }
}

/// Represents the connections between robots
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RobotConnections {
    /// List of robot ids that are within the communication radius of this robot
    pub robots_within_comms_range: Vec<RobotId>,
    /// List of robot ids that are currently connected via inter-robot factors
    pub robots_connected_with: Vec<RobotId>,
}

impl RobotConnections {
    /// Create a new empty connection state
    pub fn new() -> Self {
        Self {
            robots_within_comms_range: Vec::new(),
            robots_connected_with: Vec::new(),
        }
    }
    
    /// Add a robot to the communication range
    pub fn add_to_comms_range(&mut self, robot_id: RobotId) {
        if !self.robots_within_comms_range.contains(&robot_id) {
            self.robots_within_comms_range.push(robot_id);
        }
    }
    
    /// Remove a robot from the communication range
    pub fn remove_from_comms_range(&mut self, robot_id: RobotId) {
        self.robots_within_comms_range.retain(|id| *id != robot_id);
    }
    
    /// Add a robot to the connected list
    pub fn add_to_connected(&mut self, robot_id: RobotId) {
        if !self.robots_connected_with.contains(&robot_id) {
            self.robots_connected_with.push(robot_id);
        }
    }
    
    /// Remove a robot from the connected list
    pub fn remove_from_connected(&mut self, robot_id: RobotId) {
        self.robots_connected_with.retain(|id| *id != robot_id);
    }
}

/// Planning strategy for robot path planning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanningStrategy {
    /// Only local planning using belief propagation
    OnlyLocal,
    /// RRT* planning with belief propagation refinement
    RrtStar,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_robot_id() {
        let id = RobotId::new(42);
        assert_eq!(id.value(), 42);
    }
    
    #[test]
    fn test_radius() {
        let radius = Radius::new(1.5);
        assert_eq!(radius.value(), 1.5);
    }
    
    #[test]
    fn test_antenna() {
        let mut antenna = RadioAntenna::new(10.0, true);
        assert!(antenna.active);
        
        antenna.toggle();
        assert!(!antenna.active);
        
        assert!(antenna.within_range(
            Vector2::new(5.0, 0.0),
            Vector2::new(0.0, 0.0)
        ));
        
        assert!(!antenna.within_range(
            Vector2::new(15.0, 0.0),
            Vector2::new(0.0, 0.0)
        ));
    }
    
    #[test]
    fn test_robot_connections() {
        let mut connections = RobotConnections::new();
        let robot1 = RobotId::new(1);
        let robot2 = RobotId::new(2);
        
        connections.add_to_comms_range(robot1);
        connections.add_to_connected(robot2);
        
        assert!(connections.robots_within_comms_range.contains(&robot1));
        assert!(connections.robots_connected_with.contains(&robot2));
        
        connections.remove_from_comms_range(robot1);
        connections.remove_from_connected(robot2);
        
        assert!(!connections.robots_within_comms_range.contains(&robot1));
        assert!(!connections.robots_connected_with.contains(&robot2));
    }
}
