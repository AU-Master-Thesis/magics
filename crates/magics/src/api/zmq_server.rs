//! ZeroMQ server implementation for API.
//!
//! This module provides a ZeroMQ server that handles API requests from clients.

use std::sync::{Arc, RwLock, atomic::{AtomicBool, Ordering}};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use std::collections::HashMap;

use bevy::prelude::*;
use zmq;
use serde_json;

use super::state::{ApiState, AgentState, EnvironmentState, WeightUpdate, FactorWeights};
use super::message::{Request, Response, Command, Status, ResponseData, Error, SerializedAgentState, SerializedEnvironmentState};

/// Default port for the ZeroMQ server.
pub const DEFAULT_PORT: u16 = 5555;

/// Resource for the ZeroMQ server.
#[derive(Resource)]
pub struct ZmqServer {
    /// API state shared with the server thread.
    state: Arc<ApiState>,
    
    /// Server thread handle.
    thread: Option<JoinHandle<()>>,
    
    /// Flag indicating whether the server is running.
    running: Arc<AtomicBool>,
    
    /// Port the server is listening on.
    port: u16,
}

impl ZmqServer {
    /// Create a new ZeroMQ server.
    pub fn new(state: Arc<ApiState>, port: Option<u16>) -> Self {
        Self {
            state,
            thread: None,
            running: Arc::new(AtomicBool::new(false)),
            port: port.unwrap_or(DEFAULT_PORT),
        }
    }
    
    /// Start the server in a separate thread.
    pub fn start(&mut self) -> Result<(), Error> {
        if self.thread.is_some() {
            return Ok(());
        }
        
        self.running.store(true, Ordering::SeqCst);
        
        let state = self.state.clone();
        let running = self.running.clone();
        let port = self.port;
        
        info!("Starting ZeroMQ API server on port {}...", self.port);
        
        self.thread = Some(thread::spawn(move || {
            if let Err(err) = Self::server_loop(state, running, port) {
                error!("ZeroMQ server error: {:?}", err);
            }
        }));
        
        info!("✅ ZeroMQ API server successfully started on tcp://127.0.0.1:{}", self.port);
        info!("💡 Python clients can now connect to this address");
        
        Ok(())
    }
    
    /// Stop the server and join the thread.
    pub fn stop(&mut self) {
        if let Some(thread) = self.thread.take() {
            self.running.store(false, Ordering::SeqCst);
            
            if let Err(err) = thread.join() {
                error!("Failed to join ZeroMQ server thread: {:?}", err);
            }
            
            info!("ZeroMQ server stopped");
        }
    }
    
    /// Main server loop.
    fn server_loop(state: Arc<ApiState>, running: Arc<AtomicBool>, port: u16) -> Result<(), Error> {
        let context = zmq::Context::new();
        let socket = context.socket(zmq::REP)?;
        
        let address = format!("tcp://127.0.0.1:{}", port);
        socket.bind(&address)?;
        
        info!("ZeroMQ server loop started on {}", address);
        
        // Set socket to non-blocking mode
        socket.set_rcvtimeo(100)?;
        
        while running.load(Ordering::Relaxed) {
            // Try to receive a message
            match socket.recv_string(zmq::DONTWAIT) {
                Ok(Ok(message)) => {
                    // Process message
                    let response = match Self::handle_message(&message, &state) {
                        Ok(response) => response,
                        Err(err) => {
                            let error_response = Response {
                                status: Status::Error,
                                data: None,
                                error: Some(err.to_string()),
                                request_id: None,
                            };
                            serde_json::to_string(&error_response).unwrap_or_else(|_| 
                                r#"{"status":"error","data":null,"error":"Failed to serialize error"}"#.to_string()
                            )
                        }
                    };
                    
                    // Send response
                    if let Err(err) = socket.send(response.as_bytes(), 0) {
                        error!("Failed to send response: {:?}", err);
                    }
                },
                Ok(Err(err)) => {
                    error!("Received invalid UTF-8 string: {:?}", err);
                    // Send error response
                    let error_response = Response {
                        status: Status::Error,
                        data: None,
                        error: Some("Invalid UTF-8 in request".to_string()),
                        request_id: None,
                    };
                    let response = serde_json::to_string(&error_response).unwrap();
                    if let Err(err) = socket.send(response.as_bytes(), 0) {
                        error!("Failed to send error response: {:?}", err);
                    }
                },
                Err(zmq::Error::EAGAIN) => {
                    // No message available, just continue
                    thread::sleep(Duration::from_millis(10));
                },
                Err(err) => {
                    error!("ZMQ error: {:?}", err);
                    return Err(err.into());
                }
            }
        }
        
        // Clean up
        drop(socket);
        drop(context);
        
        info!("ZMQ server loop stopped");
        
        Ok(())
    }
    
    /// Handle a message from a client.
    fn handle_message(message: &str, api_state: &Arc<ApiState>) -> Result<String, Error> {
        let request: Request = serde_json::from_str(message)?;
        
        // Log the incoming request with more details
        let request_id = request.request_id.as_deref().unwrap_or("anonymous");
        let command_name = match &request.command {
            Command::GetAgentState => "GetAgentState".to_string(),
            Command::GetEnvironmentState => "GetEnvironmentState".to_string(),
            Command::SetFactorWeights { weights, agent_id } => {
                let agent_str = match agent_id {
                    Some(id) => format!("agent {}", id),
                    None => "all agents".to_string(),
                };
                
                format!("SetFactorWeights for {} (dynamic: {:.2}, obstacle: {:.2}, interrobot: {:.2}, tracking: {:.2})",
                    agent_str, weights.dynamic, weights.obstacle, weights.interrobot, weights.tracking)
            },
            Command::Step => "Step".to_string(),
            Command::Reset => "Reset".to_string(),
            Command::IsApiActive => "IsApiActive".to_string(),
            Command::SetApiActive { active } => format!("SetApiActive({})", active),
        };
        
        info!("📥 Received API request: {} (ID: {})", command_name, request_id);
        debug!("Request details: {}", serde_json::to_string(&request).unwrap_or_else(|_| "Failed to serialize request".to_string()));
        
        let response = match request.command {
            Command::GetAgentState => {
                // Get agent states
                if let Ok(agent_states) = api_state.agent_states.read() {
                    let mut serialized_states = HashMap::new();
                    
                    for (entity, state) in agent_states.iter() {
                        serialized_states.insert(entity.index(), SerializedAgentState::from(state));
                    }
                    
                Response {
                    status: Status::Success,
                    data: Some(ResponseData::AgentStates(serialized_states)),
                    error: None,
                    request_id: request.request_id.clone(),
                }
                } else {
                    return Err(Error::ApiState("Failed to read agent states".to_string()));
                }
            },
            Command::GetEnvironmentState => {
                // Get environment state
                if let Ok(env_state) = api_state.environment_state.read() {
                    Response {
                        status: Status::Success,
                        data: Some(ResponseData::EnvironmentState(SerializedEnvironmentState::from(&*env_state))),
                        error: None,
                        request_id: request.request_id.clone(),
                    }
                } else {
                    return Err(Error::ApiState("Failed to read environment state".to_string()));
                }
            },
            Command::SetFactorWeights { weights, agent_id } => {
                // Set factor weights
                let update = WeightUpdate {
                    agent_id: agent_id.map(Entity::from_raw),
                    weights,
                };
                
                api_state.add_weight_update(update);
                
                Response {
                    status: Status::Success,
                    data: Some(ResponseData::None),
                    error: None,
                    request_id: request.request_id.clone(),
                }
            },
            Command::Step => {
                // Request a step
                api_state.reset_step_completion();
                api_state.request_step();
                
                // Wait for step to complete with timeout
                let start_time = Instant::now();
                let timeout = Duration::from_secs(5);
                
                while !api_state.is_step_completed() {
                    if start_time.elapsed() > timeout {
                        return Err(Error::Timeout("Step command timed out".to_string()));
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                
                // Step completed successfully
                Response {
                    status: Status::Success,
                    data: Some(ResponseData::None),
                    error: None,
                    request_id: request.request_id.clone(),
                }
            },
            Command::Reset => {
                // TODO: Implement reset functionality
                Response {
                    status: Status::Success,
                    data: Some(ResponseData::None),
                    error: None,
                    request_id: request.request_id.clone(),
                }
            },
            Command::IsApiActive => {
                // Check if API is active
                let is_active = api_state.is_active();
                
                Response {
                    status: Status::Success,
                    data: Some(ResponseData::Boolean(is_active)),
                    error: None,
                    request_id: request.request_id.clone(),
                }
            },
            Command::SetApiActive { active } => {
                // Set API active state
                api_state.set_active(active);
                
                Response {
                    status: Status::Success,
                    data: Some(ResponseData::None),
                    error: None,
                    request_id: request.request_id.clone(),
                }
            },
        };
        // Log the successful handling of the request with detailed response information
        let request_id = request.request_id.as_deref().unwrap_or("anonymous");
        
        // Prepare detailed response logs based on response type
        let response_details = match &response.data {
            Some(ResponseData::AgentStates(states)) => {
                format!("Returning data for {} agents", states.len())
            },
            Some(ResponseData::EnvironmentState(env)) => {
                format!("Returning environment with {} obstacles, boundaries: [{:?},{:?}] to [{:?},{:?}]",
                    env.obstacles.len(),
                    env.boundaries[0][0], env.boundaries[0][1],
                    env.boundaries[1][0], env.boundaries[1][1])
            },
            Some(ResponseData::Boolean(val)) => {
                format!("Returning boolean value: {}", val)
            },
            _ => "No detailed data to display".to_string()
        };
        
        // Log summary of the processed request with basic info
        info!("📤 Successfully processed API request: {} (ID: {})", 
            match &request.command {
                Command::GetAgentState => "GetAgentState",
                Command::GetEnvironmentState => "GetEnvironmentState",
                Command::SetFactorWeights { .. } => "SetFactorWeights",
                Command::Step => "Step",
                Command::Reset => "Reset",
                Command::IsApiActive => "IsApiActive",
                Command::SetApiActive { active } => if *active { "SetApiActive(true)" } else { "SetApiActive(false)" },
            },
            request_id);
        
        // Log the detailed response info
        info!("🔍 Response details: {}", response_details);
        
        // Log the complete response at debug level for even more detail
        debug!("Complete response: {}", 
            serde_json::to_string_pretty(&response)
                .unwrap_or_else(|_| "Failed to serialize response".to_string()));
        
        serde_json::to_string(&response).map_err(|e| {
            error!("Failed to serialize response: {:?}", e);
            e.into()
        })
    }
}

impl Drop for ZmqServer {
    fn drop(&mut self) {
        self.stop();
    }
}
