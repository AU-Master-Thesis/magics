//! Python bindings for the Magics simulation.
//!
//! This module provides Python bindings for the Magics simulation using PyO3.
//! It allows controlling the simulation from Python, particularly for
//! reinforcement learning applications.

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3::exceptions::PyRuntimeError;
use numpy::{PyArray, PyArray1, PyArray2};
use ndarray::Array1;

use magics::api::{AgentState, ApiState, EnvironmentState, WeightUpdate, FactorWeights, FactorGraphState};

/// Shared state between Rust and Python.
static mut SHARED_STATE: Option<Arc<ApiState>> = None;

// We'll use a different approach to initialize the API
// The ApiState is already initialized in the ApiPlugin
// and we'll access it directly from Python

/// Get the shared state.
///
/// This function returns the shared state, or raises a Python exception if it's not initialized.
fn get_shared_state() -> PyResult<&'static Arc<ApiState>> {
    unsafe {
        SHARED_STATE.as_ref().ok_or_else(|| {
            PyRuntimeError::new_err("API not initialized. The simulation must be running with the API plugin enabled.")
        })
    }
}

/// Python module for the Magics API.
#[pymodule]
fn magics_api(_py: Python, m: &PyModule) -> PyResult<()> {
    // Initialize the shared state if it's not already initialized
    unsafe {
        if SHARED_STATE.is_none() {
            // Create a new ApiState
            let api_state = Arc::new(ApiState::default());
            SHARED_STATE = Some(api_state);
        }
    }
    
    m.add_function(wrap_pyfunction!(get_agent_state, m)?)?;
    m.add_function(wrap_pyfunction!(get_environment_state, m)?)?;
    m.add_function(wrap_pyfunction!(set_factor_weights, m)?)?;
    m.add_function(wrap_pyfunction!(step, m)?)?;
    m.add_function(wrap_pyfunction!(reset, m)?)?;
    m.add_function(wrap_pyfunction!(is_api_active, m)?)?;
    m.add_function(wrap_pyfunction!(set_api_active, m)?)?;
    
    Ok(())
}

/// Check if the API is active.
#[pyfunction]
fn is_api_active(_py: Python) -> PyResult<bool> {
    let state = get_shared_state()?;
    Ok(state.is_active())
}

/// Set the API active state.
#[pyfunction]
fn set_api_active(_py: Python, active: bool) -> PyResult<()> {
    let state = get_shared_state()?;
    state.set_active(active);
    Ok(())
}

/// Get the state of all agents in the simulation.
///
/// Returns a dictionary mapping agent IDs to agent states.
#[pyfunction]
fn get_agent_state(py: Python) -> PyResult<PyObject> {
    let state = get_shared_state()?;
    
    // Create a Python dictionary to hold the agent states
    let agent_states = PyDict::new(py);
    
    // Get the agent states from the shared state
    if let Ok(states) = state.agent_states.read() {
        for (entity, agent_state) in states.iter() {
            let agent_dict = PyDict::new(py);
            
            // Position
            let position = PyArray1::from_array(py, &Array1::from_vec(vec![
                agent_state.position.x,
                agent_state.position.y,
            ]));
            agent_dict.set_item("position", position)?;
            
            // Velocity
            let velocity = PyArray1::from_array(py, &Array1::from_vec(vec![
                agent_state.velocity.x,
                agent_state.velocity.y,
            ]));
            agent_dict.set_item("velocity", velocity)?;
            
            // Factor graph state
            let factor_graph_dict = PyDict::new(py);
            factor_graph_dict.set_item("variable_count", agent_state.factor_graph_state.variable_count)?;
            factor_graph_dict.set_item("factor_count", agent_state.factor_graph_state.factor_count)?;
            
            // Weights
            let weights_dict = PyDict::new(py);
            weights_dict.set_item("dynamic", agent_state.factor_graph_state.weights.dynamic)?;
            weights_dict.set_item("obstacle", agent_state.factor_graph_state.weights.obstacle)?;
            weights_dict.set_item("interrobot", agent_state.factor_graph_state.weights.interrobot)?;
            weights_dict.set_item("tracking", agent_state.factor_graph_state.weights.tracking)?;
            factor_graph_dict.set_item("weights", weights_dict)?;
            
            agent_dict.set_item("factor_graph_state", factor_graph_dict)?;
            
            // Connected neighbors
            let neighbors = PyList::new(py, &agent_state.connected_neighbors.iter().map(|&id| id.index()).collect::<Vec<_>>());
            agent_dict.set_item("connected_neighbors", neighbors)?;
            
            // Add the agent state to the dictionary
            agent_states.set_item(entity.index().to_string(), agent_dict)?;
        }
    }
    
    Ok(agent_states.into())
}

/// Get the state of the environment.
///
/// Returns a dictionary containing the environment state.
#[pyfunction]
fn get_environment_state(py: Python) -> PyResult<PyObject> {
    let state = get_shared_state()?;
    
    // Create a Python dictionary to hold the environment state
    let env_dict = PyDict::new(py);
    
    // Get the environment state from the shared state
    if let Ok(env_state) = state.environment_state.read() {
        // Obstacles
        let obstacles = env_state.obstacles.iter().map(|pos| {
            let pos_array = PyArray1::from_array(py, &Array1::from_vec(vec![pos.x, pos.y]));
            pos_array.to_object(py)
        }).collect::<Vec<_>>();
        env_dict.set_item("obstacles", PyList::new(py, &obstacles))?;
        
        // Boundaries
        let min_bounds = PyArray1::from_array(py, &Array1::from_vec(vec![
            env_state.boundaries.0.x,
            env_state.boundaries.0.y,
        ]));
        let max_bounds = PyArray1::from_array(py, &Array1::from_vec(vec![
            env_state.boundaries.1.x,
            env_state.boundaries.1.y,
        ]));
        let boundaries = PyDict::new(py);
        boundaries.set_item("min", min_bounds)?;
        boundaries.set_item("max", max_bounds)?;
        env_dict.set_item("boundaries", boundaries)?;
    }
    
    Ok(env_dict.into())
}

/// Set the factor graph weights.
///
/// Args:
///     weights: A dictionary containing the weights for different factor types.
///     agent_id: Optional agent ID to set weights for a specific agent.
///
/// If agent_id is None, the weights will be applied to all agents.
#[pyfunction]
fn set_factor_weights(py: Python, weights: &PyDict, agent_id: Option<usize>) -> PyResult<()> {
    let state = get_shared_state()?;
    
    // Extract weights from the Python dictionary
    let dynamic = weights.get_item("dynamic")
        .and_then(|w| w.extract::<f32>().ok())
        .unwrap_or(1.0);
    
    let obstacle = weights.get_item("obstacle")
        .and_then(|w| w.extract::<f32>().ok())
        .unwrap_or(1.0);
    
    let interrobot = weights.get_item("interrobot")
        .and_then(|w| w.extract::<f32>().ok())
        .unwrap_or(1.0);
    
    let tracking = weights.get_item("tracking")
        .and_then(|w| w.extract::<f32>().ok())
        .unwrap_or(1.0);
    
    // Create a weight update
    let factor_weights = FactorWeights {
        dynamic,
        obstacle,
        interrobot,
        tracking,
    };
    
    // Convert agent_id to Entity if provided
    let entity = agent_id.map(|id| bevy::ecs::entity::Entity::from_raw(id as u32));
    
    // Create a weight update
    let update = WeightUpdate {
        agent_id: entity,
        weights: factor_weights,
    };
    
    // Add the update to the shared state
    state.add_weight_update(update);
    
    Ok(())
}

/// Step the simulation forward by one frame.
///
/// This function requests a step in the simulation and waits for it to complete.
#[pyfunction]
fn step(_py: Python) -> PyResult<()> {
    let state = get_shared_state()?;
    
    // Check if the API is active
    if !state.is_active() {
        return Err(PyRuntimeError::new_err("API is not active. Call set_api_active(True) first."));
    }
    
    // Reset step completion status
    state.reset_step_completion();
    
    // Request a step
    state.request_step();
    
    // Wait for the step to complete with a longer timeout
    let mut attempts = 0;
    let max_attempts = 500; // Increase timeout to 5 seconds (500 * 10ms)
    
    while !state.is_step_completed() && attempts < max_attempts {
        thread::sleep(Duration::from_millis(10));
        attempts += 1;
        
        // Print debug info every 100 attempts
        if attempts % 100 == 0 {
            println!("Waiting for step to complete... Attempt {}/{}", attempts, max_attempts);
            println!("Step requested: {}", state.is_step_requested());
            println!("Step completed: {}", state.is_step_completed());
        }
    }
    
    if !state.is_step_completed() {
        return Err(PyRuntimeError::new_err(
            format!("Step timed out after {} attempts. The simulation may be paused or not running. \
                    Step requested: {}, Step completed: {}", 
                    attempts, state.is_step_requested(), state.is_step_completed())
        ));
    }
    
    Ok(())
}

/// Reset the simulation.
///
/// This is a placeholder for now, as the actual reset functionality
/// will depend on how the simulation is designed to be reset.
#[pyfunction]
fn reset(_py: Python) -> PyResult<()> {
    let state = get_shared_state()?;
    
    // Check if the API is active
    if !state.is_active() {
        return Err(PyRuntimeError::new_err("API is not active. Call set_api_active(True) first."));
    }
    
    // TODO: Implement actual reset functionality
    
    Ok(())
}
