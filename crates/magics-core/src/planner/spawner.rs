//! Robot spawning functionality for simulation
//!
//! This module provides functionality to create and spawn robots
//! in the simulation environment.

use serde::{Serialize, Deserialize};
use crate::types::Vector2;
use crate::planner::robot::{RobotId, Radius, RadioAntenna, PlanningStrategy};

/// Configuration for spawning a robot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobotSpawnConfig {
    /// ID of the robot to spawn
    pub id: RobotId,
    /// Initial position of the robot
    pub position: Vector2,
    /// Radius of the robot
    pub radius: f32,
    /// Radio antenna configuration
    pub antenna: Option<RadioAntenna>,
    /// Planning strategy for the robot
    pub planning_strategy: PlanningStrategy,
}

impl RobotSpawnConfig {
    /// Create a new robot spawn configuration
    pub fn new(
        id: RobotId,
        position: Vector2,
        radius: f32,
        planning_strategy: PlanningStrategy,
    ) -> Self {
        Self {
            id,
            position,
            radius,
            antenna: Some(RadioAntenna::new(10.0, true)),
            planning_strategy,
        }
    }
    
    /// Create a new robot spawn configuration with custom antenna settings
    pub fn with_antenna(
        id: RobotId,
        position: Vector2,
        radius: f32,
        antenna_radius: f32,
        antenna_active: bool,
        planning_strategy: PlanningStrategy,
    ) -> Self {
        Self {
            id,
            position,
            radius,
            antenna: Some(RadioAntenna::new(antenna_radius, antenna_active)),
            planning_strategy,
        }
    }
    
    /// Create a new robot spawn configuration without an antenna
    pub fn without_antenna(
        id: RobotId,
        position: Vector2,
        radius: f32,
        planning_strategy: PlanningStrategy,
    ) -> Self {
        Self {
            id,
            position,
            radius,
            antenna: None,
            planning_strategy,
        }
    }
}

/// Spawner for creating robots in the simulation
#[derive(Debug, Default)]
pub struct RobotSpawner {
    /// Counter for assigning robot IDs
    next_id: usize,
}

impl RobotSpawner {
    /// Create a new robot spawner
    pub fn new() -> Self {
        Self { next_id: 0 }
    }
    
    /// Get the next available robot ID
    pub fn next_id(&mut self) -> RobotId {
        let id = self.next_id;
        self.next_id += 1;
        RobotId::new(id)
    }
    
    /// Create a new robot spawn configuration
    pub fn create_robot_config(
        &mut self,
        position: Vector2,
        radius: f32,
        planning_strategy: PlanningStrategy,
    ) -> RobotSpawnConfig {
        let id = self.next_id();
        RobotSpawnConfig::new(id, position, radius, planning_strategy)
    }
    
    /// Create multiple robot spawn configurations in a grid pattern
    pub fn create_robot_grid(
        &mut self,
        start_pos: Vector2,
        rows: usize,
        cols: usize,
        spacing: f64,
        radius: f32,
        planning_strategy: PlanningStrategy,
    ) -> Vec<RobotSpawnConfig> {
        let mut configs = Vec::with_capacity(rows * cols);
        
        for r in 0..rows {
            for c in 0..cols {
                let x = start_pos.x + (c as f64 * spacing);
                let y = start_pos.y + (r as f64 * spacing);
                let position = Vector2::new(x, y);
                
                configs.push(self.create_robot_config(
                    position,
                    radius,
                    planning_strategy,
                ));
            }
        }
        
        configs
    }
    
    /// Create multiple robot spawn configurations in a circle pattern
    pub fn create_robot_circle(
        &mut self,
        center: Vector2,
        radius: f64,
        count: usize,
        robot_radius: f32,
        planning_strategy: PlanningStrategy,
    ) -> Vec<RobotSpawnConfig> {
        let mut configs = Vec::with_capacity(count);
        
        for i in 0..count {
            let angle = 2.0 * std::f64::consts::PI * (i as f64) / (count as f64);
            let x = center.x + radius * angle.cos();
            let y = center.y + radius * angle.sin();
            let position = Vector2::new(x, y);
            
            configs.push(self.create_robot_config(
                position,
                robot_radius,
                planning_strategy,
            ));
        }
        
        configs
    }
    
    /// Reset the spawner's ID counter
    pub fn reset(&mut self) {
        self.next_id = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_robot_spawner() {
        let mut spawner = RobotSpawner::new();
        let id1 = spawner.next_id();
        let id2 = spawner.next_id();
        
        assert_eq!(id1.value(), 0);
        assert_eq!(id2.value(), 1);
        
        spawner.reset();
        let id3 = spawner.next_id();
        assert_eq!(id3.value(), 0);
    }
    
    #[test]
    fn test_robot_grid() {
        let mut spawner = RobotSpawner::new();
        let configs = spawner.create_robot_grid(
            Vector2::new(0.0, 0.0),
            2,
            3,
            1.0,
            0.5,
            PlanningStrategy::RrtStar,
        );
        
        assert_eq!(configs.len(), 6);
        
        // Check first row positions
        assert_eq!(configs[0].position, Vector2::new(0.0, 0.0));
        assert_eq!(configs[1].position, Vector2::new(1.0, 0.0));
        assert_eq!(configs[2].position, Vector2::new(2.0, 0.0));
        
        // Check second row positions
        assert_eq!(configs[3].position, Vector2::new(0.0, 1.0));
        assert_eq!(configs[4].position, Vector2::new(1.0, 1.0));
        assert_eq!(configs[5].position, Vector2::new(2.0, 1.0));
    }
    
    #[test]
    fn test_robot_circle() {
        let mut spawner = RobotSpawner::new();
        let configs = spawner.create_robot_circle(
            Vector2::new(0.0, 0.0),
            10.0,
            4,
            0.5,
            PlanningStrategy::OnlyLocal,
        );
        
        assert_eq!(configs.len(), 4);
        
        // Check positions around the circle
        assert!(
            (configs[0].position.x - 10.0).abs() < 1e-10 && 
            configs[0].position.y.abs() < 1e-10
        );
        assert!(
            configs[1].position.x.abs() < 1e-10 && 
            (configs[1].position.y - 10.0).abs() < 1e-10
        );
        assert!(
            (configs[2].position.x + 10.0).abs() < 1e-10 && 
            configs[2].position.y.abs() < 1e-10
        );
        assert!(
            configs[3].position.x.abs() < 1e-10 && 
            (configs[3].position.y + 10.0).abs() < 1e-10
        );
    }
}
