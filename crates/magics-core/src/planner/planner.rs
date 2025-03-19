//! Core planning algorithms for the simulation
//!
//! This module contains the planner implementation for the GBP algorithm and
//! related functionality for robot planning and control.

use serde::{Serialize, Deserialize};
use super::{robot, collisions, tracking};
use crate::types::Vector2;
use crate::environment::GbpSimulationEnvironment;
use crate::factorgraph::factorgraph::FactorGraph;
use crate::error::SimulationError;

/// Planner trait for different planning algorithms
pub trait Planner {
    /// Initialize the planner
    fn initialize(&mut self) -> Result<(), SimulationError>;
    
    /// Plan the next steps for all agents
    fn plan(&mut self, dt: f64) -> Result<(), SimulationError>;
    
    /// Reset the planner
    fn reset(&mut self) -> Result<(), SimulationError>;
    
    /// Plan a path from start to goal position
    fn plan_path(&mut self, start: Vector2, goal: Vector2) -> Result<Path, PlannerError>;
}

/// GBP planner implementation
pub struct GbpPlanner {
    /// Factorgraph for the planner
    factorgraph: FactorGraph,
    /// Planning horizon
    horizon: f64,
    /// Time step for planning
    dt: f64,
}

impl GbpPlanner {
    /// Create a new GBP planner
    pub fn new(horizon: f64, dt: f64) -> Self {
        Self {
            factorgraph: FactorGraph::default(),
            horizon,
            dt,
        }
    }
}

impl Default for GbpPlanner {
    fn default() -> Self {
        Self::new(5.0, 0.1)
    }
}

impl Planner for GbpPlanner {
    fn initialize(&mut self) -> Result<(), SimulationError> {
        // In a real implementation, this would set up the factorgraph
        // and prepare for planning
        Ok(())
    }
    
    fn plan(&mut self, dt: f64) -> Result<(), SimulationError> {
        // In a real implementation, this would compute the next steps
        // for all agents using the factorgraph
        Ok(())
    }
    
    fn reset(&mut self) -> Result<(), SimulationError> {
        self.factorgraph = FactorGraph::default();
        Ok(())
    }
    
    fn plan_path(&mut self, start: Vector2, goal: Vector2) -> Result<Path, PlannerError> {
        // Simplified implementation for now - we'll expand this as needed
        if start == goal {
            return Ok(Path::new(vec![start]));
        }
        
        let direction = (goal - start).normalized();
        let distance = (goal - start).magnitude();
        
        // If goal is within horizon, go directly to it
        if distance <= self.horizon {
            return Ok(Path::new(vec![start, goal]));
        }
        
        // Otherwise, go as far as the horizon allows in the correct direction
        let intermediate = start + direction * self.horizon;
        Ok(Path::new(vec![start, intermediate]))
    }
}

/// A path consisting of waypoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Path {
    /// The waypoints that make up the path
    pub waypoints: Vec<Vector2>,
}

impl Path {
    /// Create a new path from a set of waypoints
    pub fn new(waypoints: Vec<Vector2>) -> Self {
        Self { waypoints }
    }
    
    /// Get the length of the path
    pub fn length(&self) -> f64 {
        if self.waypoints.len() < 2 {
            return 0.0;
        }
        
        let mut length = 0.0;
        for i in 0..self.waypoints.len() - 1 {
            length += (self.waypoints[i+1] - self.waypoints[i]).magnitude();
        }
        
        length
    }
    
    /// Check if the path is empty
    pub fn is_empty(&self) -> bool {
        self.waypoints.is_empty()
    }
    
    /// Get the number of waypoints in the path
    pub fn num_waypoints(&self) -> usize {
        self.waypoints.len()
    }
}

/// Error types for planning operations
#[derive(Debug, thiserror::Error)]
pub enum PlannerError {
    /// No valid path could be found
    #[error("No valid path found between start and goal")]
    NoPathFound,
    
    /// The planner reached its maximum iteration limit
    #[error("Planner reached iteration limit without finding a path")]
    IterationLimitReached,
    
    /// The start or goal position is invalid
    #[error("Invalid start or goal position")]
    InvalidPosition,
    
    /// Environment not properly initialized
    #[error("Environment not initialized")]
    EnvironmentNotInitialized,
    
    /// Other planning errors
    #[error("Planning error: {0}")]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_path_length() {
        let path = Path::new(vec![
            Vector2::new(0.0, 0.0),
            Vector2::new(3.0, 0.0),
            Vector2::new(3.0, 4.0),
        ]);
        
        assert_eq!(path.length(), 7.0);
    }
    
    #[test]
    fn test_gbp_planner_direct() {
        let mut planner = GbpPlanner::new(10.0, 0.1);
        let start = Vector2::new(0.0, 0.0);
        let goal = Vector2::new(5.0, 0.0);
        
        let path = planner.plan_path(start, goal).unwrap();
        assert_eq!(path.waypoints.len(), 2);
        assert_eq!(path.waypoints[0], start);
        assert_eq!(path.waypoints[1], goal);
    }
    
    #[test]
    fn test_gbp_planner_beyond_horizon() {
        let mut planner = GbpPlanner::new(10.0, 0.1);
        let start = Vector2::new(0.0, 0.0);
        let goal = Vector2::new(20.0, 0.0);
        
        let path = planner.plan_path(start, goal).unwrap();
        assert_eq!(path.waypoints.len(), 2);
        assert_eq!(path.waypoints[0], start);
        assert_eq!(path.waypoints[1], Vector2::new(10.0, 0.0));
    }
}
