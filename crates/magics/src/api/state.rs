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

use bevy::{math::Vec2, prelude::*};

use crate::{factorgraph::factorgraph::FactorGraph, planner::robot::RobotConnections};

/// State of an agent in the simulation.
#[derive(Debug, Clone)]
pub struct AgentState {
    /// Position of the agent.
    pub position: Vec2,
    /// Velocity of the agent.
    pub velocity: Vec2,
    /// Factor graph state of the agent.
    pub factor_graph_state: FactorGraphState,
    /// IDs of connected neighbors.
    pub connected_neighbors: Vec<Entity>,
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

/// State of the environment in the simulation.
#[derive(Debug, Clone)]
pub struct EnvironmentState {
    /// Positions of obstacles in the environment.
    pub obstacles:  Vec<Vec2>,
    /// Boundaries of the environment.
    pub boundaries: (Vec2, Vec2),
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
    /// Number of iterations remaining in the current step
    pub step_iterations_remaining: Arc<AtomicUsize>,
    /// Number of iterations to use for each step
    pub iterations_per_step: Arc<AtomicUsize>,
}

impl Default for ApiState {
    fn default() -> Self {
        Self {
            agent_states: Arc::new(RwLock::new(HashMap::new())),
            environment_state: Arc::new(RwLock::new(EnvironmentState {
                obstacles:  Vec::new(),
                boundaries: (Vec2::ZERO, Vec2::ZERO),
            })),
            weight_requests: Arc::new(RwLock::new(Vec::new())),
            step_requested: Arc::new(AtomicBool::new(false)),
            step_completed: Arc::new(AtomicBool::new(false)),
            // Set api_active to true when the API feature is enabled
            #[cfg(feature = "api")]
            api_active: Arc::new(AtomicBool::new(true)),
            #[cfg(not(feature = "api"))]
            api_active: Arc::new(AtomicBool::new(false)),
            step_iterations_remaining: Arc::new(AtomicUsize::new(0)),
            // Default to 2 iterations per step
            iterations_per_step: Arc::new(AtomicUsize::new(2)),
        }
    }
}

impl ApiState {
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
}
