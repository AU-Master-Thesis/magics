//! Core data types for the simulation

use std::sync::Arc;
use serde::{Serialize, Deserialize};

/// Agent identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub usize);

/// Simulation time
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct SimTime {
    /// Raw time value in seconds
    time: f64,
}

impl SimTime {
    /// Create a new simulation time
    pub fn new(time: f64) -> Self {
        Self { time }
    }
    
    /// Get the raw time value
    pub fn get(&self) -> f64 {
        self.time
    }
}

/// A 2D vector representation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl Vector2 {
    /// Create a new vector
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    
    /// Calculate the Euclidean distance between this vector and another
    pub fn distance(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
    
    /// Calculate the magnitude (length) of the vector
    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    
    /// Normalize the vector (make it unit length)
    pub fn normalized(&self) -> Self {
        let mag = self.magnitude();
        if mag == 0.0 {
            *self
        } else {
            Self {
                x: self.x / mag,
                y: self.y / mag,
            }
        }
    }
}

impl std::ops::Add for Vector2 {
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::Sub for Vector2 {
    type Output = Self;
    
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl std::ops::Mul<f64> for Vector2 {
    type Output = Self;
    
    fn mul(self, scalar: f64) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl std::ops::Div<f64> for Vector2 {
    type Output = Self;
    
    fn div(self, scalar: f64) -> Self {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

/// State of an agent in the environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    /// Agent identifier
    pub id: AgentId,
    /// Current position
    pub position: Vector2,
    /// Current velocity
    pub velocity: Vector2,
    /// Target position
    pub target: Vector2,
}

/// State of the entire environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentState {
    /// Current simulation time
    pub time: SimTime,
    /// States of all agents in the environment
    pub agents: Vec<AgentState>,
}

/// Shared environment state that can be accessed concurrently
pub type SharedEnvironmentState = Arc<EnvironmentState>;
