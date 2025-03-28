//! State definitions for the API.
//!
//! This module defines the state structures that are shared between the
//! simulation and external API consumers.

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, RwLock,
    },
};

use bevy::{math::Vec2, prelude::*, time::Time};

use crate::{
    factorgraph::factorgraph::{FactorGraph, FactorGraphId},
    planner::robot::{RobotConnections, StateVector},
};

/// Mission state for an agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionState {
    /// Agent is idle, possibly waiting for waypoints.
    Idle { waiting_for_waypoints: bool },
    /// Agent is actively following a route.
    Active,
    /// Agent has completed its mission.
    Completed,
}

/// Planning strategy used by an agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanningStrategy {
    /// Agent uses only local planning.
    OnlyLocal,
    /// Agent uses RRT* for global planning.
    RrtStar,
}

/// Default mission state is Active.
impl Default for MissionState {
    fn default() -> Self {
        Self::Active
    }
}

/// Default planning strategy is OnlyLocal.
impl Default for PlanningStrategy {
    fn default() -> Self {
        Self::OnlyLocal
    }
}

/// Information about collisions for an agent.
#[derive(Debug, Clone, Default)]
pub struct CollisionInfo {
    /// Total number of collisions with other robots.
    pub robot_collisions_total: usize,
    /// Change in robot collisions since last state extraction.
    pub robot_collisions_delta: usize,
    /// Total number of collisions with the environment.
    pub environment_collisions_total: usize,
    /// Change in environment collisions since last state extraction.
    pub environment_collisions_delta: usize,
}

/// State of an agent in the simulation.
#[derive(Debug, Clone)]
pub struct AgentState {
    /// ID of the agent (Entity ID).
    pub agent_id: u32,
    /// ID of the factor graph.
    pub factorgraph_id: u32,
    /// Position of the agent.
    pub position: Vec2,
    /// Velocity of the agent.
    pub velocity: Vec2,
    /// Factor graph state of the agent.
    pub factor_graph_state: FactorGraphState,
    /// IDs of connected neighbors.
    pub connected_neighbors: Vec<Entity>,
    /// Current mission state of the agent.
    pub mission_state: MissionState,
    /// Planning strategy used by the agent.
    pub planning_strategy: PlanningStrategy,
    /// Radius of the agent.
    pub radius: f32,
    /// Whether the agent's communication is active.
    pub communication_active: bool,
    /// Communication radius of the agent.
    pub communication_radius: f32,
    /// Target speed of the agent.
    pub target_speed: f32,
    /// Index of the current waypoint.
    pub current_waypoint_index: Option<usize>,
    /// Information about the next waypoint.
    pub next_waypoint: Option<StateVectorInfo>,
    /// Position of the goal point.
    pub goal_point: Option<Vec2>,
    /// Mission progress information.
    pub mission_progress: MissionProgress,
    /// Detailed information about factor graph components.
    pub factor_details: FactorDetails,
    /// Information about collisions.
    pub collision_info: CollisionInfo,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            agent_id: 0,
            factorgraph_id: 0,
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            factor_graph_state: FactorGraphState::default(),
            connected_neighbors: Vec::new(),
            mission_state: MissionState::default(),
            planning_strategy: PlanningStrategy::default(),
            radius: 0.5,
            communication_active: true,
            communication_radius: 5.0,
            target_speed: 1.0,
            current_waypoint_index: None,
            next_waypoint: None,
            goal_point: None,
            mission_progress: MissionProgress::default(),
            factor_details: FactorDetails::default(),
            collision_info: CollisionInfo::default(),
        }
    }
}

/// Information about a state vector (position and velocity).
#[derive(Debug, Clone, Default)]
pub struct StateVectorInfo {
    /// Position component of the state vector.
    pub position: Vec2,
    /// Velocity component of the state vector.
    pub velocity: Vec2,
}

/// Mission progress information.
#[derive(Debug, Clone, Default)]
pub struct MissionProgress {
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

/// Detailed information about factor graph components.
#[derive(Debug, Clone, Default)]
pub struct FactorDetails {
    /// Information about variables in the factor graph.
    pub variables: Vec<VariableInfo>,
    /// Information about obstacle factors in the factor graph.
    pub obstacle_factors: Vec<ObstacleFactorInfo>,
    /// Information about inter-robot factors in the factor graph.
    pub interrobot_factors: Vec<InterRobotFactorInfo>,
    /// Information about tracking factors in the factor graph.
    pub tracking_factors: Vec<TrackingFactorInfo>,
    /// Information about dynamic factors in the factor graph.
    pub dynamic_factors: Vec<DynamicFactorInfo>,
}

/// Information about a variable in the factor graph.
#[derive(Debug, Clone)]
pub struct VariableInfo {
    /// Index of the variable.
    pub index: usize,
    /// ID of the factor graph this variable belongs to
    pub factorgraph_id: u32,
    /// Mean vector of the variable [x, y, vx, vy].
    pub mean: [f64; 4],
    /// Covariance matrix of the variable (4x4, flattened).
    pub covariance: [f64; 16],
    /// Estimated position from the variable.
    pub estimated_position: [f64; 2],
    /// Estimated velocity from the variable.
    pub estimated_velocity: [f64; 2],
}

/// Information about an obstacle factor in the factor graph.
#[derive(Debug, Clone)]
pub struct ObstacleFactorInfo {
    /// Index of the variable this factor is connected to.
    pub variable_index: usize,
    /// SDF value at the position, ranges from 0.0 (free space) to 1.0 (obstacle).
    pub sdf_value:      f64,
    /// Position where the SDF value was measured.
    pub position:       [f32; 2],
}

/// Information about an inter-robot factor in the factor graph.
#[derive(Debug, Clone)]
pub struct InterRobotFactorInfo {
    /// Index of the variable this factor is connected to.
    pub variable_index: usize,
    /// ID of the external robot this factor connects to.
    pub external_robot_id: u32,
    // ID of the external factor graph
    pub external_factorgraph_id: u32,
    /// Index of the variable in the external robot's factor graph.
    pub external_variable_index: usize,
    /// Safety distance for collision avoidance.
    pub safety_distance: f32,
    /// Current diff_between_estimated_positions value.
    pub distance_between_variables: f64,
    /// active (called skip)
    pub active: bool,
}

/// Information about a tracking factor in the factor graph.
#[derive(Debug, Clone)]
pub struct TrackingFactorInfo {
    /// Index of the variable this factor is connected to.
    pub variable_index:     usize,
    /// Path that the robot is tracking.
    pub tracking_path:      Vec<[f32; 2]>,
    /// Current index in the tracking path.
    pub tracking_index:     usize,
    /// Projected position on the path.
    pub projected_position: [f32; 2],
    /// Path deviation measurement (normalized distance).
    pub path_deviation:     f32,
    /// Distance from robot to projected point on path (in world units).
    pub distance_to_path:   f64,
}

/// Information about a dynamic factor in the factor graph.
#[derive(Debug, Clone)]
pub struct DynamicFactorInfo {
    /// Index of the source variable.
    pub from_variable_index: usize,
    /// Index of the destination variable.
    pub to_variable_index: usize,
    /// Time step between the variables.
    pub delta_t: f32,
}

/// Statistics about messages in the factor graph.
#[derive(Debug, Clone, Default)]
pub struct MessageStats {
    /// Number of internal messages.
    pub internal: usize,
    /// Number of external messages.
    pub external: usize,
}

/// Counts of different factor types in the factor graph.
#[derive(Debug, Clone, Default)]
pub struct FactorCounts {
    /// Number of obstacle factors.
    pub obstacle:   usize,
    /// Number of interrobot factors.
    pub interrobot: usize,
    /// Number of dynamic factors.
    pub dynamic:    usize,
    /// Number of tracking factors.
    pub tracking:   usize,
}

/// State of a factor graph.
#[derive(Debug, Clone)]
pub struct FactorGraphState {
    /// Current weights of the factor graph.
    pub weights: FactorWeights,
    /// Number of variables in the factor graph.
    pub variable_count: usize,
    /// Number of factors in the factor graph.
    pub factor_count: usize,
    /// Statistics about messages sent from the factor graph.
    pub messages_sent: MessageStats,
    /// Statistics about messages received by the factor graph.
    pub messages_received: MessageStats,
    /// Counts of different factor types.
    pub factor_counts: FactorCounts,
    /// All factors and variables in the factor graph.
    pub factor_details: FactorDetails,
}

impl Default for FactorGraphState {
    fn default() -> Self {
        Self {
            weights: FactorWeights::default(),
            variable_count: 0,
            factor_count: 0,
            messages_sent: MessageStats::default(),
            messages_received: MessageStats::default(),
            factor_counts: FactorCounts::default(),
            factor_details: FactorDetails::default(),
        }
    }
}

/// Weights for different factor types in the factor graph.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct FactorWeights {
    /// Weight for dynamic factors. 
    pub dynamic:    f32,
    /// Weight for obstacle factors.
    pub obstacle:   f32,
    /// Weight for inter-robot factors.
    pub interrobot: f32,
    /// Weight for tracking factors.
    pub tracking:   f32,
}

impl Default for FactorWeights {
    fn default() -> Self {
        Self {
            dynamic:    1.0,
            obstacle:   1.0,
            interrobot: 1.0,
            tracking:   1.0,
        }
    }
}

/// State of the environment in the simulation.
#[derive(Debug, Clone)]
pub struct EnvironmentState {
    /// Positions of obstacles in the environment.
    pub obstacles: Vec<Vec2>,
    /// Boundaries of the environment.
    pub boundaries: (Vec2, Vec2),
    /// Total number of agents in the environment.
    pub total_agents: usize,
    /// Optional agent density map.
    pub agent_density_map: Option<Vec<f32>>,
    /// Optional SDF resolution (width, height).
    pub sdf_resolution: Option<(usize, usize)>,
    /// Optional world size (width, height).
    pub world_size: Option<(f64, f64)>,
}

impl Default for EnvironmentState {
    fn default() -> Self {
        Self {
            obstacles: Vec::new(),
            boundaries: (Vec2::ZERO, Vec2::ZERO),
            total_agents: 0,
            agent_density_map: None,
            sdf_resolution: None,
            world_size: None,
        }
    }
}

/// Update to factor weights.
#[derive(Debug, Clone)]
pub struct WeightUpdate {
    /// ID of the agent to update weights for, or None for system-wide update.
    pub agent_id: Option<Entity>,
    /// New weights to apply.
    pub weights:  FactorWeights,
}

/// State for the API.
#[derive(Resource, Clone)]
pub struct ApiState {
    /// States of agents in the simulation.
    pub agent_states: Arc<RwLock<HashMap<Entity, AgentState>>>,
    /// State of the environment.
    pub environment_state: Arc<RwLock<EnvironmentState>>,
    /// Requests to update factor weights.
    pub weight_requests: Arc<RwLock<Vec<WeightUpdate>>>,
    /// Flag indicating whether a step has been requested.
    pub step_requested: Arc<AtomicBool>,
    /// Flag indicating whether a step has been completed.
    pub step_completed: Arc<AtomicBool>,
    /// Flag indicating whether the API is active.
    pub api_active: Arc<AtomicBool>,
    /// Flag indicating whether a reset has been requested.
    pub reset_requested: Arc<AtomicBool>,
    /// Flag indicating whether a reset has been completed.
    pub reset_completed: Arc<AtomicBool>,
    /// Flag indicating whether an environment load has been requested.
    pub load_environment_requested: Arc<RwLock<Option<String>>>,
    /// Flag indicating whether an environment load has been completed.
    pub load_environment_completed: Arc<AtomicBool>,
    /// Number of iterations remaining in the current step
    pub step_iterations_remaining: Arc<AtomicUsize>,
    /// Number of iterations to use for each step
    pub iterations_per_step: Arc<AtomicUsize>,
    /// Reference to the Config resource for accessing simulation parameters
    pub config: Option<Arc<RwLock<gbp_config::Config>>>,
    /// Reference to the Time<Fixed> resource for updating the fixed timestep
    pub time_fixed: Option<Arc<RwLock<Time<Fixed>>>>,
}

impl Default for ApiState {
    fn default() -> Self {
        Self {
            agent_states: Arc::new(RwLock::new(HashMap::new())),
            environment_state: Arc::new(RwLock::new(EnvironmentState::default())),
            weight_requests: Arc::new(RwLock::new(Vec::new())),
            step_requested: Arc::new(AtomicBool::new(false)),
            step_completed: Arc::new(AtomicBool::new(false)),
            // Set api_active to true when the API feature is enabled
            #[cfg(feature = "api")]
            api_active: Arc::new(AtomicBool::new(true)),
            #[cfg(not(feature = "api"))]
            api_active: Arc::new(AtomicBool::new(false)),
            reset_requested: Arc::new(AtomicBool::new(false)),
            reset_completed: Arc::new(AtomicBool::new(false)),
            load_environment_requested: Arc::new(RwLock::new(None)),
            load_environment_completed: Arc::new(AtomicBool::new(false)),
            step_iterations_remaining: Arc::new(AtomicUsize::new(0)),
            // Default to 2 iterations per step
            iterations_per_step: Arc::new(AtomicUsize::new(2)),
            config: None,
            time_fixed: None,
        }
    }
}

impl ApiState {
    /// Get the state of an agent with the given entity ID.
    pub fn get_agent_state(&self, entity: Entity) -> Option<AgentState> {
        if let Ok(agent_states) = self.agent_states.read() {
            agent_states.get(&entity).cloned()
        } else {
            None
        }
    }

    /// Get a reference to the environment state.
    pub fn get_environment_state(&self) -> Option<EnvironmentState> {
        if let Ok(env_state) = self.environment_state.read() {
            Some(env_state.clone())
        } else {
            None
        }
    }

    /// Check if the API is active.
    pub fn is_active(&self) -> bool {
        self.api_active.load(Ordering::SeqCst)
    }

    /// Set the API active state.
    pub fn set_active(&self, active: bool) {
        self.api_active.store(active, Ordering::SeqCst);
    }

    /// Request a step in the simulation.
    pub fn request_step(&self) {
        self.step_requested.store(true, Ordering::SeqCst);
    }

    /// Check if a step has been requested.
    pub fn is_step_requested(&self) -> bool {
        self.step_requested.load(Ordering::SeqCst)
    }

    /// Mark a step as completed.
    pub fn complete_step(&self) {
        self.step_completed.store(true, Ordering::SeqCst);
        self.step_requested.store(false, Ordering::SeqCst);
    }

    /// Check if a step has been completed.
    pub fn is_step_completed(&self) -> bool {
        self.step_completed.load(Ordering::SeqCst)
    }

    /// Reset step completion status.
    pub fn reset_step_completion(&self) {
        self.step_completed.store(false, Ordering::SeqCst);
    }

    /// Add a weight update request.
    pub fn add_weight_update(&self, update: WeightUpdate) {
        if let Ok(mut requests) = self.weight_requests.write() {
            requests.push(update);
        }
    }

    /// Get the number of iterations remaining in the current step.
    pub fn get_step_iterations_remaining(&self) -> usize {
        self.step_iterations_remaining.load(Ordering::SeqCst)
    }

    /// Set the number of iterations remaining in the current step.
    pub fn set_step_iterations_remaining(&self, iterations: usize) {
        self.step_iterations_remaining
            .store(iterations, Ordering::SeqCst);
    }

    /// Decrement the number of iterations remaining and return the previous
    /// value.
    pub fn decrement_step_iterations_remaining(&self) -> usize {
        self.step_iterations_remaining
            .fetch_sub(1, Ordering::SeqCst)
    }

    /// Get the number of iterations to use for each step.
    pub fn get_iterations_per_step(&self) -> usize {
        self.iterations_per_step.load(Ordering::SeqCst)
    }

    /// Set the number of iterations to use for each step.
    pub fn set_iterations_per_step(&self, iterations: usize) {
        self.iterations_per_step.store(iterations, Ordering::SeqCst);
    }

    /// Set the Config resource reference.
    pub fn set_config(&mut self, config: Arc<RwLock<gbp_config::Config>>) {
        self.config = Some(config);
    }

    /// Set the Time<Fixed> resource reference.
    pub fn set_time_fixed(&mut self, time_fixed: Arc<RwLock<Time<Fixed>>>) {
        self.time_fixed = Some(time_fixed);
    }

    /// Add or update an agent state.
    pub fn add_or_update_agent(
        &self,
        entity: Entity,
        agent_state: AgentState,
    ) -> Result<(), String> {
        if let Ok(mut agent_states) = self.agent_states.write() {
            agent_states.insert(entity, agent_state);
            Ok(())
        } else {
            Err("Failed to acquire write lock on agent_states".to_string())
        }
    }

    /// Get all agent entities.
    pub fn get_all_agent_entities(&self) -> Vec<Entity> {
        if let Ok(agent_states) = self.agent_states.read() {
            agent_states.keys().copied().collect()
        } else {
            Vec::new()
        }
    }

    /// Get the simulation Hz from the Config.
    pub fn get_simulation_hz(&self) -> f64 {
        if let Some(config) = &self.config {
            if let Ok(config_guard) = config.read() {
                return config_guard.simulation.hz;
            }
        }
        // Return a default value if config is not available
        60.0 // TODO: No please! Handle this better!
    }

    /// Set the simulation Hz in the Config and update the Time<Fixed> resource.
    pub fn set_simulation_hz(&self, hz: f64) -> Result<(), String> {
        // Update the config
        if let Some(config) = &self.config {
            if let Ok(mut config_guard) = config.write() {
                config_guard.simulation.hz = hz;
            } else {
                return Err("Failed to acquire write lock on config".to_string());
            }
        } else {
            return Err("Config not available".to_string());
        }

        // Update the Time<Fixed> resource
        if let Some(time_fixed) = &self.time_fixed {
            if let Ok(mut time_fixed_guard) = time_fixed.write() {
                *time_fixed_guard = Time::<Fixed>::from_hz(hz);
                return Ok(());
            } else {
                return Err("Failed to acquire write lock on Time<Fixed>".to_string());
            }
        }

        Err("Time<Fixed> resource not available".to_string())
    }

    /// Request a reset of the simulation.
    pub fn request_reset(&self) {
        self.reset_requested.store(true, Ordering::SeqCst);
        info!("API: Reset requested");
    }

    /// Check if a reset has been requested.
    pub fn is_reset_requested(&self) -> bool {
        self.reset_requested.load(Ordering::SeqCst)
    }

    /// Clear the reset request flag.
    pub fn clear_reset_request(&self) {
        self.reset_requested.store(false, Ordering::SeqCst);
        info!("API: Reset request cleared");
    }

    /// Request loading a specific environment.
    pub fn request_load_environment(&self, name: String) {
        if let Ok(mut env_name) = self.load_environment_requested.write() {
            *env_name = Some(name.clone());
            info!("API: Load environment '{}' requested", name);
        } else {
            error!("API: Failed to acquire write lock on load_environment_requested");
        }
    }

    /// Get the name of the environment to load, if any.
    pub fn get_load_environment_request(&self) -> Option<String> {
        if let Ok(env_name) = self.load_environment_requested.read() {
            env_name.clone()
        } else {
            None
        }
    }

    /// Clear the load environment request.
    pub fn clear_load_environment_request(&self) {
        if let Ok(mut env_name) = self.load_environment_requested.write() {
            *env_name = None;
            info!("API: Load environment request cleared");
        } else {
            error!("API: Failed to acquire write lock on load_environment_requested");
        }
    }

    /// Mark a reset as completed.
    pub fn complete_reset(&self) {
        self.reset_completed.store(true, Ordering::SeqCst);
        info!("API: Reset completed");
    }

    /// Check if a reset has been completed.
    pub fn is_reset_completed(&self) -> bool {
        self.reset_completed.load(Ordering::SeqCst)
    }

    /// Reset the reset completion status.
    pub fn reset_reset_completion(&self) {
        self.reset_completed.store(false, Ordering::SeqCst);
        info!("API: Reset completion status reset");
    }

    /// Mark an environment load as completed.
    pub fn complete_load_environment(&self) {
        self.load_environment_completed.store(true, Ordering::SeqCst);
        info!("API: Load environment completed");
    }

    /// Check if an environment load has been completed.
    pub fn is_load_environment_completed(&self) -> bool {
        self.load_environment_completed.load(Ordering::SeqCst)
    }

    /// Reset the environment load completion status.
    pub fn reset_load_environment_completion(&self) {
        self.load_environment_completed.store(false, Ordering::SeqCst);
        info!("API: Load environment completion status reset");
    }
}
