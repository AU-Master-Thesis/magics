//! API module for external control of the simulation.
//!
//! This module provides functionality for controlling the simulation from external
//! sources, particularly for integration with external clients via ZeroMQ.

mod plugin;
mod state;
mod message;
mod zmq_server;
mod reset;
mod factor_details;
mod extract;
mod weights;
mod despawned_agents;
mod state_utils;

pub use plugin::ApiPlugin;
pub use state::{AgentState, ApiState, EnvironmentState, WeightUpdate, FactorWeights, FactorGraphState};
pub use zmq_server::{ZmqServer, DEFAULT_PORT};
pub use message::{Request, Response, Command, Status, ResponseData, Error};
pub use despawned_agents::DespawnedAgentsTracker;
pub use state_utils::create_agent_state;

/// Feature flag for enabling the API functionality
#[cfg(feature = "api")]
pub const API_ENABLED: bool = true;

/// Feature flag for enabling the API functionality
#[cfg(not(feature = "api"))]
pub const API_ENABLED: bool = false;
