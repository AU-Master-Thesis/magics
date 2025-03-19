//! Collision detection and handling for robot agents
//! 
//! This module provides collision detection functionality between robots and
//! between robots and environment obstacles.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::types::Vector2;
use crate::planner::robot::{RobotId, Radius};

/// Represents a collision between two entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collision {
    /// Time when the collision occurred
    pub time: f64,
    /// Location of the collision
    pub position: Vector2,
    /// IDs of the entities involved in the collision
    pub entities: Vec<EntityId>,
}

/// Represents an entity ID which could be a robot or an obstacle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityId {
    /// A robot entity
    Robot(RobotId),
    /// An obstacle entity
    Obstacle(usize),
}

/// State of a collision between entities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionState {
    /// No collision
    Free,
    /// Currently colliding
    Colliding,
}

/// Status of a collision update
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionStatus {
    /// A new collision has started
    Hit,
    /// An existing collision is ongoing
    Colliding,
    /// A collision has ended
    End,
    /// No collision
    Free,
}

/// History of collisions between two entities
#[derive(Debug, Clone)]
pub struct CollisionHistory {
    /// Number of collisions that have occurred
    pub count: usize,
    /// Current state of the collision
    pub state: CollisionState,
    /// Collection of all collision events
    pub events: Vec<Collision>,
}

impl CollisionHistory {
    /// Create a new collision history
    pub fn new() -> Self {
        Self {
            count: 0,
            state: CollisionState::Free,
            events: Vec::new(),
        }
    }

    /// Update the collision state and return the status
    pub fn update(&mut self, is_colliding: bool, time: f64, position: Vector2, entities: Vec<EntityId>) -> CollisionStatus {
        match (self.state, is_colliding) {
            (CollisionState::Colliding, true) => CollisionStatus::Colliding,
            (CollisionState::Colliding, false) => {
                self.state = CollisionState::Free;
                CollisionStatus::End
            }
            (CollisionState::Free, false) => CollisionStatus::Free,
            (CollisionState::Free, true) => {
                self.state = CollisionState::Colliding;
                self.count += 1;
                
                let collision = Collision {
                    time,
                    position,
                    entities,
                };
                self.events.push(collision);
                
                CollisionStatus::Hit
            }
        }
    }
}

impl Default for CollisionHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Detector for robot-robot collisions
#[derive(Debug, Default)]
pub struct RobotCollisionDetector {
    /// History of collisions between robots
    collision_history: HashMap<(RobotId, RobotId), CollisionHistory>,
    /// Total number of collisions detected
    total_collisions: usize,
}

impl RobotCollisionDetector {
    /// Create a new robot collision detector
    pub fn new() -> Self {
        Self {
            collision_history: HashMap::new(),
            total_collisions: 0,
        }
    }
    
    /// Check for collisions between robots
    pub fn check_collisions(
        &mut self,
        robots: &[(RobotId, Vector2, Radius)],
        current_time: f64,
    ) -> Vec<Collision> {
        let mut new_collisions = Vec::new();
        
        // Check each pair of robots for collisions
        for i in 0..robots.len() {
            let (id1, pos1, radius1) = robots[i];
            
            for j in (i+1)..robots.len() {
                let (id2, pos2, radius2) = robots[j];
                
                // Simple distance-based collision detection
                let distance = (pos2 - pos1).magnitude();
                let is_colliding = distance < (radius1.0 as f64 + radius2.0 as f64);
                
                let key = if id1.0 < id2.0 {
                    (id1, id2)
                } else {
                    (id2, id1)
                };
                
                let entry = self.collision_history
                    .entry(key)
                    .or_insert_with(CollisionHistory::new);
                
                let collision_position = pos1 + (pos2 - pos1) * 0.5;
                let entities = vec![EntityId::Robot(id1), EntityId::Robot(id2)];
                
                let status = entry.update(is_colliding, current_time, collision_position, entities.clone());
                
                if status == CollisionStatus::Hit {
                    self.total_collisions += 1;
                    
                    if let Some(collision) = entry.events.last() {
                        new_collisions.push(collision.clone());
                    }
                }
            }
        }
        
        new_collisions
    }
    
    /// Get the total number of collisions that have occurred
    pub fn total_collisions(&self) -> usize {
        self.total_collisions
    }
    
    /// Get the number of collisions for a specific robot
    pub fn collisions_for_robot(&self, robot_id: RobotId) -> usize {
        self.collision_history
            .iter()
            .filter_map(|((id1, id2), history)| {
                if *id1 == robot_id || *id2 == robot_id {
                    Some(history.count)
                } else {
                    None
                }
            })
            .sum()
    }
    
    /// Reset the collision detector
    pub fn reset(&mut self) {
        self.collision_history.clear();
        self.total_collisions = 0;
    }
}

/// Detector for robot-obstacle collisions
#[derive(Debug, Default)]
pub struct ObstacleCollisionDetector {
    /// History of collisions between robots and obstacles
    collision_history: HashMap<(RobotId, usize), CollisionHistory>,
    /// Total number of collisions detected
    total_collisions: usize,
}

impl ObstacleCollisionDetector {
    /// Create a new obstacle collision detector
    pub fn new() -> Self {
        Self {
            collision_history: HashMap::new(),
            total_collisions: 0,
        }
    }
    
    /// Check for collisions between robots and obstacles
    /// 
    /// This is a simplified version - in a real implementation, obstacles would have
    /// proper shapes and collision detection would be more sophisticated
    pub fn check_collisions(
        &mut self,
        robots: &[(RobotId, Vector2, Radius)],
        obstacles: &[(usize, Vector2, f32)],
        current_time: f64,
    ) -> Vec<Collision> {
        let mut new_collisions = Vec::new();
        
        for (robot_id, robot_pos, robot_radius) in robots {
            for (obstacle_id, obstacle_pos, obstacle_radius) in obstacles {
                let distance = (*robot_pos - *obstacle_pos).magnitude();
                let is_colliding = distance < (robot_radius.0 as f64 + *obstacle_radius as f64);
                
                let entry = self.collision_history
                    .entry((*robot_id, *obstacle_id))
                    .or_insert_with(CollisionHistory::new);
                
                let collision_position = *robot_pos + (*obstacle_pos - *robot_pos) * 0.5;
                let entities = vec![
                    EntityId::Robot(*robot_id), 
                    EntityId::Obstacle(*obstacle_id)
                ];
                
                let status = entry.update(is_colliding, current_time, collision_position, entities.clone());
                
                if status == CollisionStatus::Hit {
                    self.total_collisions += 1;
                    
                    if let Some(collision) = entry.events.last() {
                        new_collisions.push(collision.clone());
                    }
                }
            }
        }
        
        new_collisions
    }
    
    /// Get the total number of collisions that have occurred
    pub fn total_collisions(&self) -> usize {
        self.total_collisions
    }
    
    /// Get the number of collisions for a specific robot
    pub fn collisions_for_robot(&self, robot_id: RobotId) -> usize {
        self.collision_history
            .iter()
            .filter_map(|((id, _), history)| {
                if *id == robot_id {
                    Some(history.count)
                } else {
                    None
                }
            })
            .sum()
    }
    
    /// Get the number of collisions for a specific obstacle
    pub fn collisions_for_obstacle(&self, obstacle_id: usize) -> usize {
        self.collision_history
            .iter()
            .filter_map(|((_, id), history)| {
                if *id == obstacle_id {
                    Some(history.count)
                } else {
                    None
                }
            })
            .sum()
    }
    
    /// Reset the collision detector
    pub fn reset(&mut self) {
        self.collision_history.clear();
        self.total_collisions = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_robot_collision_detection() {
        let mut detector = RobotCollisionDetector::new();
        
        let robot1 = (RobotId(1), Vector2::new(0.0, 0.0), Radius(1.0));
        let robot2 = (RobotId(2), Vector2::new(0.5, 0.0), Radius(1.0));
        let robot3 = (RobotId(3), Vector2::new(10.0, 10.0), Radius(1.0));
        
        let robots = vec![robot1, robot2, robot3];
        
        let collisions = detector.check_collisions(&robots, 1.0);
        assert_eq!(collisions.len(), 1);
        assert_eq!(detector.total_collisions(), 1);
        
        // Robot 1 and 2 collide, Robot 3 is far away
        assert_eq!(detector.collisions_for_robot(RobotId(1)), 1);
        assert_eq!(detector.collisions_for_robot(RobotId(2)), 1);
        assert_eq!(detector.collisions_for_robot(RobotId(3)), 0);
    }
    
    #[test]
    fn test_obstacle_collision_detection() {
        let mut detector = ObstacleCollisionDetector::new();
        
        let robot1 = (RobotId(1), Vector2::new(0.0, 0.0), Radius(1.0));
        let robot2 = (RobotId(2), Vector2::new(10.0, 10.0), Radius(1.0));
        
        let obstacle1 = (1, Vector2::new(0.5, 0.0), 1.0);
        let obstacle2 = (2, Vector2::new(5.0, 5.0), 1.0);
        
        let robots = vec![robot1, robot2];
        let obstacles = vec![obstacle1, obstacle2];
        
        let collisions = detector.check_collisions(&robots, &obstacles, 1.0);
        assert_eq!(collisions.len(), 1);
        assert_eq!(detector.total_collisions(), 1);
        
        // Robot 1 collides with Obstacle 1
        assert_eq!(detector.collisions_for_robot(RobotId(1)), 1);
        assert_eq!(detector.collisions_for_robot(RobotId(2)), 0);
        assert_eq!(detector.collisions_for_obstacle(1), 1);
        assert_eq!(detector.collisions_for_obstacle(2), 0);
    }
}
