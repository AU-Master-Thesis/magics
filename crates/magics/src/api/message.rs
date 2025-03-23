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
        SerializedAgentState {
            position: [state.position.x, state.position.y],
            velocity: [state.velocity.x, state.velocity.y],
            factor_graph_state: SerializedFactorGraphState {
                weights: state.factor_graph_state.weights,
                variable_count: state.factor_graph_state.variable_count,
                factor_count: state.factor_graph_state.factor_count,
            },
            connected_neighbors: state.connected_neighbors.iter().map(|e| e.index()).collect(),
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
