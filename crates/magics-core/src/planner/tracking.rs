//! Tracking functionality for robot trajectories
//!
//! This module provides types and functionality for tracking robot movements
//! and storing position and velocity history.

use std::time::Duration;
use serde::{Serialize, Deserialize};
use crate::types::Vector2;

/// A position measurement with timestamp
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PositionMeasurement {
    /// The position at the time of measurement
    pub position: Vector2,
    /// The timestamp of the measurement (seconds)
    pub timestamp: f64,
}

/// A velocity measurement with timestamp
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VelocityMeasurement {
    /// The velocity at the time of measurement
    pub velocity: Vector2,
    /// The timestamp of the measurement (seconds)
    pub timestamp: f64,
    /// Time period over which the velocity was measured
    pub measured_over: f64,
}

/// A trajectory of a robot consisting of position and velocity measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trajectory {
    /// Position measurements over time
    positions: Vec<PositionMeasurement>,
    /// Velocity measurements over time
    velocities: Vec<VelocityMeasurement>,
    /// Maximum number of measurements to store (ring buffer size)
    capacity: usize,
    /// Time interval between measurements
    measurement_interval: f64,
    /// First measurement timestamp
    first_measurement_time: Option<f64>,
    /// Last position recorded (used for velocity calculation)
    previous_position: Option<(Vector2, f64)>,
}

impl Trajectory {
    /// Create a new trajectory tracker with specified capacity and update interval
    pub fn new(capacity: usize, measurement_interval: f64) -> Self {
        Self {
            positions: Vec::with_capacity(capacity),
            velocities: Vec::with_capacity(capacity),
            capacity,
            measurement_interval,
            first_measurement_time: None,
            previous_position: None,
        }
    }
    
    /// Get the interval between measurements in seconds
    pub fn measurement_interval(&self) -> f64 {
        self.measurement_interval
    }
    
    /// Set the interval between measurements in seconds
    pub fn set_measurement_interval(&mut self, interval: f64) {
        self.measurement_interval = interval;
    }
    
    /// Record a new position measurement
    ///
    /// If the buffer is full, the oldest measurement will be removed
    pub fn record_position(&mut self, position: Vector2, timestamp: f64) {
        // Record first measurement time if this is the first measurement
        if self.first_measurement_time.is_none() {
            self.first_measurement_time = Some(timestamp);
        }
        
        // Add the position measurement
        let measurement = PositionMeasurement {
            position,
            timestamp,
        };
        
        if self.positions.len() >= self.capacity {
            // Remove oldest measurement if at capacity
            self.positions.remove(0);
        }
        self.positions.push(measurement);
        
        // Calculate velocity if we have a previous position
        if let Some((prev_pos, prev_time)) = self.previous_position {
            let dt = timestamp - prev_time;
            
            // Only calculate velocity if enough time has passed
            if dt >= self.measurement_interval {
                let velocity = (position - prev_pos) / dt;
                
                let velocity_measurement = VelocityMeasurement {
                    velocity,
                    timestamp,
                    measured_over: dt,
                };
                
                if self.velocities.len() >= self.capacity {
                    // Remove oldest measurement if at capacity
                    self.velocities.remove(0);
                }
                self.velocities.push(velocity_measurement);
                
                // Update previous position for next velocity calculation
                self.previous_position = Some((position, timestamp));
            }
        } else {
            // First position measurement
            self.previous_position = Some((position, timestamp));
        }
    }
    
    /// Get all position measurements
    pub fn positions(&self) -> &[PositionMeasurement] {
        &self.positions
    }
    
    /// Get all velocity measurements
    pub fn velocities(&self) -> &[VelocityMeasurement] {
        &self.velocities
    }
    
    /// Get position at specific time (nearest measurement)
    pub fn position_at(&self, time: f64) -> Option<Vector2> {
        if self.positions.is_empty() {
            return None;
        }
        
        // Find measurement closest to requested time
        let closest = self.positions
            .iter()
            .min_by(|a, b| {
                let a_diff = (a.timestamp - time).abs();
                let b_diff = (b.timestamp - time).abs();
                a_diff.partial_cmp(&b_diff).unwrap_or(std::cmp::Ordering::Equal)
            })?;
        
        Some(closest.position)
    }
    
    /// Get velocity at specific time (nearest measurement)
    pub fn velocity_at(&self, time: f64) -> Option<Vector2> {
        if self.velocities.is_empty() {
            return None;
        }
        
        // Find measurement closest to requested time
        let closest = self.velocities
            .iter()
            .min_by(|a, b| {
                let a_diff = (a.timestamp - time).abs();
                let b_diff = (b.timestamp - time).abs();
                a_diff.partial_cmp(&b_diff).unwrap_or(std::cmp::Ordering::Equal)
            })?;
        
        Some(closest.velocity)
    }
    
    /// Clear all trajectory data
    pub fn clear(&mut self) {
        self.positions.clear();
        self.velocities.clear();
        self.first_measurement_time = None;
        self.previous_position = None;
    }
    
    /// Check if there are any position measurements
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
    
    /// Get the number of position measurements
    pub fn len(&self) -> usize {
        self.positions.len()
    }
    
    /// Get the first measurement time
    pub fn first_measurement_time(&self) -> Option<f64> {
        self.first_measurement_time
    }
    
    /// Get the latest position measurement
    pub fn latest_position(&self) -> Option<PositionMeasurement> {
        self.positions.last().copied()
    }
    
    /// Get the latest velocity measurement
    pub fn latest_velocity(&self) -> Option<VelocityMeasurement> {
        self.velocities.last().copied()
    }
    
    /// Calculate the total distance traveled
    pub fn total_distance(&self) -> f64 {
        if self.positions.len() < 2 {
            return 0.0;
        }
        
        let mut total = 0.0;
        for i in 1..self.positions.len() {
            total += (self.positions[i].position - self.positions[i-1].position).magnitude();
        }
        
        total
    }
    
    /// Calculate the average speed
    pub fn average_speed(&self) -> Option<f64> {
        if self.velocities.is_empty() {
            return None;
        }
        
        let sum: f64 = self.velocities
            .iter()
            .map(|v| v.velocity.magnitude())
            .sum();
        
        Some(sum / self.velocities.len() as f64)
    }
}

/// A tracker for multiple robot trajectories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryTracker {
    /// Trajectories for each robot
    trajectories: HashMap<crate::planner::robot::RobotId, Trajectory>,
    /// Default capacity for new trajectories
    default_capacity: usize,
    /// Default measurement interval for new trajectories
    default_interval: f64,
}

impl TrajectoryTracker {
    /// Create a new tracker for multiple robot trajectories
    pub fn new(default_capacity: usize, default_interval: f64) -> Self {
        Self {
            trajectories: HashMap::new(),
            default_capacity,
            default_interval,
        }
    }
    
    /// Record a position for a specific robot
    pub fn record_position(&mut self, robot_id: crate::planner::robot::RobotId, position: Vector2, timestamp: f64) {
        let trajectory = self.trajectories
            .entry(robot_id)
            .or_insert_with(|| Trajectory::new(self.default_capacity, self.default_interval));
        
        trajectory.record_position(position, timestamp);
    }
    
    /// Get a trajectory for a specific robot
    pub fn get_trajectory(&self, robot_id: &crate::planner::robot::RobotId) -> Option<&Trajectory> {
        self.trajectories.get(robot_id)
    }
    
    /// Get a mutable trajectory for a specific robot
    pub fn get_trajectory_mut(&mut self, robot_id: &crate::planner::robot::RobotId) -> Option<&mut Trajectory> {
        self.trajectories.get_mut(robot_id)
    }
    
    /// Clear all trajectories
    pub fn clear(&mut self) {
        self.trajectories.clear();
    }
    
    /// Get the number of robots being tracked
    pub fn robot_count(&self) -> usize {
        self.trajectories.len()
    }
    
    /// Get all robot IDs being tracked
    pub fn robot_ids(&self) -> impl Iterator<Item = &crate::planner::robot::RobotId> {
        self.trajectories.keys()
    }
}

use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::robot::RobotId;
    
    #[test]
    fn test_trajectory_recording() {
        let mut trajectory = Trajectory::new(100, 0.1);
        
        // Record some positions
        trajectory.record_position(Vector2::new(0.0, 0.0), 0.0);
        trajectory.record_position(Vector2::new(1.0, 0.0), 0.1);
        trajectory.record_position(Vector2::new(2.0, 0.0), 0.2);
        
        // Check positions were recorded
        assert_eq!(trajectory.len(), 3);
        assert_eq!(trajectory.positions()[0].position.x, 0.0);
        assert_eq!(trajectory.positions()[1].position.x, 1.0);
        assert_eq!(trajectory.positions()[2].position.x, 2.0);
        
        // Check velocities were calculated
        assert_eq!(trajectory.velocities().len(), 2);
        assert_eq!(trajectory.velocities()[0].velocity.x, 10.0); // (1.0 - 0.0) / 0.1
        assert_eq!(trajectory.velocities()[1].velocity.x, 10.0); // (2.0 - 1.0) / 0.1
    }
    
    #[test]
    fn test_trajectory_tracker() {
        let mut tracker = TrajectoryTracker::new(100, 0.1);
        
        // Record positions for two robots
        let robot1 = RobotId(1);
        let robot2 = RobotId(2);
        
        tracker.record_position(robot1, Vector2::new(0.0, 0.0), 0.0);
        tracker.record_position(robot1, Vector2::new(1.0, 0.0), 0.1);
        tracker.record_position(robot2, Vector2::new(0.0, 0.0), 0.0);
        tracker.record_position(robot2, Vector2::new(0.0, 1.0), 0.1);
        
        // Check trajectories were created
        assert_eq!(tracker.robot_count(), 2);
        
        // Check positions were recorded
        let traj1 = tracker.get_trajectory(&robot1).unwrap();
        let traj2 = tracker.get_trajectory(&robot2).unwrap();
        
        assert_eq!(traj1.len(), 2);
        assert_eq!(traj2.len(), 2);
        
        // Check directions
        assert_eq!(traj1.velocities()[0].velocity.x, 10.0);
        assert_eq!(traj1.velocities()[0].velocity.y, 0.0);
        assert_eq!(traj2.velocities()[0].velocity.x, 0.0);
        assert_eq!(traj2.velocities()[0].velocity.y, 10.0);
    }
}
