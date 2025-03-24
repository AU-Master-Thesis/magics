//! Message protocol definitions for API communication.
//!
//! This module defines the message formats used for communication between
//! the ZeroMQ server and clients.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use bevy::prelude::Entity;
use crate::api::state::FactorWeights;

/// Command enum for all supported API operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", content = "parameters")]
pub enum Command {
    /// Get the state of all agents in the simulation.
    GetAgentState,
    
    /// Get the state of the environment.
    GetEnvironmentState,
    
    /// Set factor graph weights.
    SetFactorWeights {
        /// Weights to set
        weights: FactorWeights,
        /// Optional agent ID for per-agent weights
        agent_id: Option<u32>,
    },
    
    /// Step the simulation forward by one frame.
    Step,
    
    /// Reset the simulation.
    Reset,
    
    /// Check if the API is active.
    IsApiActive,
    
    /// Set the API active state.
    SetApiActive {
        /// Whether to activate the API
        active: bool,
    },
    
    /// Set the number of iterations per step
    SetIterationsPerStep {
        /// The number of iterations per step
        iterations: usize,
    },
    
    /// Get the simulation Hz (frequency)
    GetSimulationHz,
    
    /// Set the simulation Hz (frequency)
    SetSimulationHz {
        /// The new Hz value
        hz: f64,
    },
}

/// Request message sent from client to server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// Command to execute
    #[serde(flatten)]
    pub command: Command,
    
    /// Request ID for matching responses
    #[serde(default)]
    pub request_id: Option<String>,
}

/// Alternative request format that can handle nested command structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeRequest {
    /// Command object containing the command and parameters
    pub command: Command,
    
    /// Request ID for matching responses
    #[serde(default)]
    pub request_id: Option<String>,
}

/// Status of a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Status {
    /// Request succeeded
    Success,
    /// Request failed
    Error,
}

/// Data returned in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum ResponseData {
    /// Agent states data
    AgentStates(HashMap<u32, SerializedAgentState>),
    
    /// Environment state data
    EnvironmentState(SerializedEnvironmentState),
    
    /// Boolean result
    Boolean(bool),
    
    /// Numeric result (float)
    Number(f64),
    
    /// No data
    None,
}

/// Response message sent from server to client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Status of the response
    pub status: Status,
    
    /// Optional data returned
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ResponseData>,
    
    /// Optional error message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    
    /// Request ID from the original request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

/// Serialized agent state for API communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedAgentState {
    /// Position of the agent [x, z]
    pub position: [f32; 2],
    
    /// Velocity of the agent [x, z]
    pub velocity: [f32; 2],
    
    /// Factor graph state
    pub factor_graph_state: SerializedFactorGraphState,
    
    /// IDs of connected neighbors
    pub connected_neighbors: Vec<u32>,
    
    /// Current mission state
    pub mission_state: SerializedMissionState,
    
    /// Planning strategy used by the agent
    pub planning_strategy: String,
    
    /// Radius of the agent
    pub radius: f32,
    
    /// Whether the agent's communication is active
    pub communication_active: bool,
    
    /// Communication radius of the agent
    pub communication_radius: f32,
    
    /// Target speed of the agent
    pub target_speed: f32,
    
    /// Index of the current waypoint
    pub current_waypoint_index: Option<usize>,
    
    /// Information about the next waypoint
    pub next_waypoint: Option<SerializedStateVectorInfo>,
    
    /// Position of the goal point
    pub goal_point: Option<[f32; 2]>,
    
    /// Mission progress information
    pub mission_progress: SerializedMissionProgress,
}

/// Serialized mission state for API communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum SerializedMissionState {
    /// Agent is idle, possibly waiting for waypoints.
    Idle { waiting_for_waypoints: bool },
    /// Agent is actively following a route.
    Active,
    /// Agent has completed its mission.
    Completed,
}

/// Serialized state vector information for API communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedStateVectorInfo {
    /// Position component of the state vector.
    pub position: [f32; 2],
    /// Velocity component of the state vector.
    pub velocity: [f32; 2],
}

/// Serialized mission progress information for API communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedMissionProgress {
    /// Time when the mission started.
    pub started_at: f64,
    /// Time when the mission finished, if completed.
    pub finished_at: Option<f64>,
    /// Index of the active route.
    pub active_route: usize,
    /// Total number of routes.
    pub total_routes: usize,
    /// Total number of waypoints.
    pub total_waypoints: usize,
    /// Number of remaining waypoints.
    pub remaining_waypoints: usize,
}

/// Serialized factor graph state for API communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedFactorGraphState {
    /// Weights of the factor graph
    pub weights: FactorWeights,
    
    /// Number of variables in the factor graph
    pub variable_count: usize,
    
    /// Number of factors in the factor graph
    pub factor_count: usize,
}

/// Serialized environment state for API communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedEnvironmentState {
    /// Positions of obstacles [x, z]
    pub obstacles: Vec<[f32; 2]>,
    
    /// Boundaries of the environment [min_x, min_z, max_x, max_z]
    pub boundaries: [[f32; 2]; 2],
}

/// Error type for API operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    /// ZeroMQ error
    #[error("ZMQ error: {0}")]
    Zmq(#[from] zmq::Error),
    
    /// API state error
    #[error("API state error: {0}")]
    ApiState(String),
    
    /// Command handling error
    #[error("Command error: {0}")]
    Command(String),
    
    /// Timeout error
    #[error("Timeout: {0}")]
    Timeout(String),
    
    /// Server not initialized
    #[error("Server not initialized")]
    NotInitialized,
}

impl From<&crate::api::state::AgentState> for SerializedAgentState {
    fn from(state: &crate::api::state::AgentState) -> Self {
        // Convert mission state
        let mission_state = match state.mission_state {
            crate::api::state::MissionState::Idle { waiting_for_waypoints } => {
                SerializedMissionState::Idle { waiting_for_waypoints }
            }
            crate::api::state::MissionState::Active => SerializedMissionState::Active,
            crate::api::state::MissionState::Completed => SerializedMissionState::Completed,
        };

        // Convert planning strategy
        let planning_strategy = match state.planning_strategy {
            crate::api::state::PlanningStrategy::OnlyLocal => "OnlyLocal".to_string(),
            crate::api::state::PlanningStrategy::RrtStar => "RrtStar".to_string(),
        };

        // Convert next waypoint if present
        let next_waypoint = state.next_waypoint.as_ref().map(|wp| SerializedStateVectorInfo {
            position: [wp.position.x, wp.position.y],
            velocity: [wp.velocity.x, wp.velocity.y],
        });

        // Convert goal point if present
        let goal_point = state.goal_point.map(|p| [p.x, p.y]);

        // Convert mission progress
        let mission_progress = SerializedMissionProgress {
            started_at: state.mission_progress.started_at,
            finished_at: state.mission_progress.finished_at,
            active_route: state.mission_progress.active_route,
            total_routes: state.mission_progress.total_routes,
            total_waypoints: state.mission_progress.total_waypoints,
            remaining_waypoints: state.mission_progress.remaining_waypoints,
        };

        SerializedAgentState {
            position: [state.position.x, state.position.y],
            velocity: [state.velocity.x, state.velocity.y],
            factor_graph_state: SerializedFactorGraphState {
                weights: state.factor_graph_state.weights,
                variable_count: state.factor_graph_state.variable_count,
                factor_count: state.factor_graph_state.factor_count,
            },
            connected_neighbors: state.connected_neighbors.iter().map(|e| e.index()).collect(),
            mission_state,
            planning_strategy,
            radius: state.radius,
            communication_active: state.communication_active,
            communication_radius: state.communication_radius,
            target_speed: state.target_speed,
            current_waypoint_index: state.current_waypoint_index,
            next_waypoint,
            goal_point,
            mission_progress,
        }
    }
}

impl From<&crate::api::state::EnvironmentState> for SerializedEnvironmentState {
    fn from(state: &crate::api::state::EnvironmentState) -> Self {
        SerializedEnvironmentState {
            obstacles: state.obstacles.iter().map(|v| [v.x, v.y]).collect(),
            boundaries: [
                [state.boundaries.0.x, state.boundaries.0.y],
                [state.boundaries.1.x, state.boundaries.1.y],
            ],
        }
    }
}
