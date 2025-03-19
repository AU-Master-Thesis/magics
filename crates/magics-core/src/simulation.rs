//! Core simulation logic for running simulations without UI dependencies

use std::sync::Arc;
use serde::{Serialize, Deserialize};

use crate::error::SimulationError;
use crate::types::{AgentId, AgentState, EnvironmentState, SimTime, SharedEnvironmentState};
use crate::output::OutputManager;
use crate::environment::{GbpSimulationEnvironment, SimulationEnvironment};

/// Configuration for the simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    /// Time step for the simulation
    pub time_step: f64,
    /// Maximum simulation time
    pub max_time: f64,
    /// Random seed for reproducibility
    pub random_seed: u64,
    /// Output settings
    pub output: OutputSettings,
}

/// Output settings for the simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSettings {
    /// Should output to stdout
    pub stdout: bool,
    /// Output file path (if any)
    pub file_path: Option<std::path::PathBuf>,
    /// Output frequency (every N steps)
    pub frequency: usize,
}

impl Default for OutputSettings {
    fn default() -> Self {
        Self {
            stdout: true,
            file_path: None,
            frequency: 1,
        }
    }
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            time_step: 0.1,
            max_time: 100.0,
            random_seed: 42,
            output: OutputSettings::default(),
        }
    }
}

/// The core simulation trait that must be implemented by simulation runners
pub trait Simulation: std::any::Any {
    /// Initialize the simulation with the given configuration
    fn initialize(&mut self, config: SimulationConfig) -> Result<(), SimulationError>;
    
    /// Step the simulation forward by one time step
    fn step(&mut self) -> Result<(), SimulationError>;
    
    /// Reset the simulation to its initial state
    fn reset(&mut self) -> Result<(), SimulationError>;
    
    /// Get the current state of the simulation
    fn get_state(&self) -> SharedEnvironmentState;
    
    /// Get the current simulation time
    fn get_time(&self) -> SimTime;
    
    /// Check if the simulation is complete
    fn is_complete(&self) -> bool;
    
    /// Used for downcasting to concrete simulation types
    fn as_any(&self) -> &dyn std::any::Any;
    
    /// Used for downcasting to concrete simulation types (mutable)
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    
    /// Check if the simulation has finished (different from is_complete)
    fn is_finished(&self) -> bool;
}

/// The simulation runner that manages the simulation and output
pub struct SimulationRunner {
    /// The actual simulation implementation
    simulation: Box<dyn Simulation + Send>,
    /// Configuration for the simulation
    config: SimulationConfig,
    /// Output manager
    output_manager: OutputManager,
    /// Current step count
    step_count: usize,
}

impl SimulationRunner {
    /// Create a new simulation runner with the given simulation implementation
    pub fn new(simulation: Box<dyn Simulation + Send>, config: SimulationConfig) -> Result<Self, SimulationError> {
        let output_manager = OutputManager::new(&config.output)?;
        
        let mut runner = Self {
            simulation,
            config,
            output_manager,
            step_count: 0,
        };
        
        runner.simulation.initialize(runner.config.clone())?;
        
        Ok(runner)
    }
    
    /// Run the simulation to completion
    pub fn run(&mut self) -> Result<(), SimulationError> {
        // Write initial state
        let state = self.simulation.get_state();
        self.output_manager.write(&state)?;
        
        // Run simulation until complete
        while !self.simulation.is_complete() {
            self.step()?;
        }
        
        Ok(())
    }
    
    /// Step the simulation forward by one time step
    pub fn step(&mut self) -> Result<(), SimulationError> {
        self.simulation.step()?;
        self.step_count += 1;
        
        // Output state if needed
        if self.step_count % self.config.output.frequency == 0 {
            let state = self.simulation.get_state();
            self.output_manager.write(&state)?;
        }
        
        Ok(())
    }
    
    /// Reset the simulation
    pub fn reset(&mut self) -> Result<(), SimulationError> {
        self.simulation.reset()?;
        self.step_count = 0;
        Ok(())
    }
    
    /// Get the current simulation state
    pub fn get_state(&self) -> SharedEnvironmentState {
        self.simulation.get_state()
    }
    
    /// Check if the simulation is complete
    pub fn is_complete(&self) -> bool {
        self.simulation.is_complete()
    }
}

/// A basic GBP simulation implementation that wraps the GBP environment and config
pub struct GbpSimulation {
    /// Environment configuration
    env: GbpSimulationEnvironment,
    /// Simulation configuration
    config: SimulationConfig,
    /// Current simulation state
    state: Arc<EnvironmentState>,
    /// Current simulation time
    time: f64,
    /// Maximum simulation time
    max_time: f64,
    /// Output settings
    output_settings: OutputSettings,
    /// Planner instance
    planner: Option<crate::planner::GbpPlanner>,
    /// Is the simulation finished
    finished: bool,
}

impl GbpSimulation {
    /// Create a new GBP simulation
    pub fn new(mut env: GbpSimulationEnvironment, config: SimulationConfig) -> Self {
        // Initialize the environment first to create agents
        if let Err(e) = env.initialize() {
            log::error!("Failed to initialize environment: {}", e);
        }
        
        // Get initial state from environment
        let env_state = env.get_state();
        
        // Initialize with agent states from environment
        let state = Arc::new(env_state);
        
        let output_settings = config.output.clone();
        let max_time = config.max_time;
        
        log::info!("Created simulation with {} agents", state.agents.len());
        
        Self {
            env,
            config,
            state,
            time: 0.0,
            max_time,
            output_settings,
            planner: None,
            finished: false,
        }
    }
    
    /// Set the output settings
    pub fn set_output_settings(&mut self, settings: OutputSettings) {
        self.output_settings = settings;
    }
    
    /// Set the planner
    pub fn set_planner(&mut self, planner: crate::planner::GbpPlanner) {
        self.planner = Some(planner);
    }
    
    /// Check if the simulation is finished
    pub fn is_finished(&self) -> bool {
        self.finished
    }
    
    /// Mark the simulation as finished
    pub fn set_finished(&mut self, finished: bool) {
        self.finished = finished;
    }
}

impl Simulation for GbpSimulation {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    
    fn is_finished(&self) -> bool {
        self.finished
    }
    
    fn initialize(&mut self, config: SimulationConfig) -> Result<(), SimulationError> {
        // Set max time from config
        self.max_time = config.max_time;
        
        // TODO: Initialize agents from environment and config
        // For now, just create a simple placeholder state
        
        let mut agents = Vec::new();
        
        // Create some test agents - this would normally come from GBP environment
        // This is a placeholder until we properly integrate with GBP
        let agent_count = 5;
        for i in 0..agent_count {
            let position = crate::types::Vector2::new(i as f64, i as f64);
            let velocity = crate::types::Vector2::new(0.0, 0.0);
            let target = crate::types::Vector2::new(10.0, 10.0);
            
            agents.push(AgentState {
                id: AgentId(i),
                position,
                velocity,
                target,
            });
        }
        
        let state = EnvironmentState {
            time: SimTime::new(self.time),
            agents,
        };
        
        self.state = Arc::new(state);
        
        Ok(())
    }
    
    fn step(&mut self) -> Result<(), SimulationError> {
        // Advance time
        self.time += self.config.time_step;
        
        // Get current state
        let current_state = Arc::make_mut(&mut self.state);
        
        // Update time
        current_state.time = SimTime::new(self.time);
        
        // Update agents - this would be more sophisticated in the real implementation
        for agent in &mut current_state.agents {
            // Simple move toward target logic
            let dir = agent.target - agent.position;
            let distance = dir.magnitude();
            
            if distance > 0.1 {
                let normalized_dir = dir / distance;
                agent.velocity = normalized_dir * 0.5; // Simple constant velocity
                agent.position = agent.position + agent.velocity * self.config.time_step;
            } else {
                agent.velocity = crate::types::Vector2::new(0.0, 0.0);
            }
        }
        
        Ok(())
    }
    
    fn reset(&mut self) -> Result<(), SimulationError> {
        self.time = 0.0;
        self.initialize(SimulationConfig {
            max_time: self.max_time,
            ..Default::default()
        })
    }
    
    fn get_state(&self) -> SharedEnvironmentState {
        self.state.clone()
    }
    
    fn get_time(&self) -> SimTime {
        SimTime::new(self.time)
    }
    
    fn is_complete(&self) -> bool {
        self.time >= self.max_time
    }
}
