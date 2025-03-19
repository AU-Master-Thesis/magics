//! Mission planning for robot agents
//! 
//! This module defines mission-related structures for planning and executing
//! sequences of tasks for robot agents.

use std::collections::VecDeque;
use serde::{Serialize, Deserialize};

/// A mission consisting of a sequence of tasks to be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    /// Queue of mission tasks to be executed in order
    pub tasks: VecDeque<MissionTask>,
}

impl Mission {
    /// Create a new mission with no tasks
    pub fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
        }
    }

    /// Create a new mission with the given tasks
    pub fn with_tasks(tasks: Vec<MissionTask>) -> Self {
        Self {
            tasks: tasks.into(),
        }
    }

    /// Add a task to the mission
    pub fn add_task(&mut self, task: MissionTask) {
        self.tasks.push_back(task);
    }

    /// Get the next task in the mission, if any
    pub fn next_task(&mut self) -> Option<MissionTask> {
        self.tasks.pop_front()
    }

    /// Check if the mission has any remaining tasks
    pub fn is_complete(&self) -> bool {
        self.tasks.is_empty()
    }
}

impl Default for Mission {
    fn default() -> Self {
        Self::new()
    }
}

/// A task within a mission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionTask {
    /// Task identifier
    pub id: String,
    /// Task type
    pub task_type: MissionTaskType,
    /// Task parameters
    pub parameters: MissionTaskParameters,
}

/// Type of mission task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MissionTaskType {
    /// Move to a position
    MoveTo,
    /// Wait at current position
    Wait,
    /// Follow another agent
    Follow,
    /// Custom task type
    Custom(String),
}

/// Parameters for a mission task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionTaskParameters {
    /// Map of string keys to parameter values
    pub values: std::collections::HashMap<String, MissionTaskParameterValue>,
}

/// Value of a mission task parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MissionTaskParameterValue {
    /// String value
    String(String),
    /// Numeric value
    Number(f64),
    /// Boolean value
    Boolean(bool),
    /// Position value (x, y)
    Position(f64, f64),
    /// List of values
    List(Vec<MissionTaskParameterValue>),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mission_tasks() {
        let mut mission = Mission::new();
        
        // Add a MoveTo task
        let mut params = MissionTaskParameters { 
            values: std::collections::HashMap::new() 
        };
        params.values.insert(
            "position".to_string(), 
            MissionTaskParameterValue::Position(10.0, 20.0)
        );
        
        let task = MissionTask {
            id: "task1".to_string(),
            task_type: MissionTaskType::MoveTo,
            parameters: params,
        };
        
        mission.add_task(task);
        assert!(!mission.is_complete());
        
        let _next_task = mission.next_task();
        assert!(mission.is_complete());
    }
}
