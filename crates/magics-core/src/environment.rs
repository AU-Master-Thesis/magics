//! Environment-related types and functionality for the simulation

use gbp_environment::Environment as GbpEnvironment;
use crate::types::{EnvironmentState, AgentState, AgentId, SimTime, Vector2};
use crate::sdf::Sdf;
use gbp_config::FormationGroup;
use anyhow::Result;

/// Environment interface for the simulation
pub trait SimulationEnvironment {
    /// Initialize the environment
    fn initialize(&mut self) -> Result<(), crate::error::SimulationError>;
    
    /// Update the environment for the current time step
    fn update(&mut self, dt: f64) -> Result<(), crate::error::SimulationError>;
    
    /// Get the current state of the environment
    fn get_state(&self) -> EnvironmentState;
    
    /// Reset the environment to its initial state
    fn reset(&mut self) -> Result<(), crate::error::SimulationError>;
}

/// GBP environment wrapper
pub struct GbpSimulationEnvironment {
    /// The GBP environment
    env: GbpEnvironment,
    /// Current simulation time
    time: f64,
    /// Current agent states
    agents: Vec<AgentState>,
    /// SDF generated from the environment
    sdf: Option<Sdf>,
    /// Formation group for agent initialization
    formation: Option<FormationGroup>,
}

impl GbpSimulationEnvironment {
    /// Create a new GBP simulation environment
    pub fn new(env: GbpEnvironment) -> Self {
        Self {
            env,
            time: 0.0,
            agents: Vec::new(),
            sdf: None,
            formation: None,
        }
    }
    
    /// Set the formation group for agent initialization
    pub fn with_formation(mut self, formation: FormationGroup) -> Self {
        self.formation = Some(formation);
        self
    }
    
    /// Generate an SDF from the environment
    pub fn generate_sdf(&mut self) -> Result<&Sdf> {
        // Default SDF parameters
        let resolution = self.env.tiles.settings.sdf.resolution as u32;
        let expansion = self.env.tiles.settings.sdf.expansion as f64;
        let blur = self.env.tiles.settings.sdf.blur as f64;
        
        let sdf = Sdf::from_environment(&self.env, resolution, expansion, blur)?;
        self.sdf = Some(sdf);
        
        Ok(self.sdf.as_ref().unwrap())
    }
    
    /// Initialize agents based on the formation group
    pub fn initialize_agents(&mut self) {
        self.agents.clear();
        
            // If we have a formation group, use it to initialize agents
            if let Some(formation_group) = &self.formation {
                // This is just a basic placeholder implementation for now
                // In a real implementation, we would:
                // 1. Get world dimensions from the environment
                // 2. Get robot radii from the configuration
                // 3. Generate positions based on formation patterns
                
                // For now, use a simple placeholder implementation
                let mut agent_id = 0;
                
                // Since we don't have the full implementation yet,
                // create some default positions in a grid pattern
                for formation in formation_group.formations.iter() {
                    for i in 0..formation.robots {
                        // Simple grid layout
                        let x = (i % 3) as f64 * 2.0;
                        let y = (i / 3) as f64 * 2.0;
                        
                        let agent = AgentState {
                            id: AgentId(agent_id),
                            position: Vector2::new(x, y),
                            velocity: Vector2::new(0.0, 0.0),
                            target: Vector2::new(x + 5.0, y + 5.0), // Simple offset target
                        };
                        
                        self.agents.push(agent);
                        agent_id += 1;
                    }
                }
        } else {
            // Create some default agents if no formation is provided
            for i in 0..3 {
                let agent = AgentState {
                    id: AgentId(i),
                    position: Vector2::new(i as f64, i as f64),
                    velocity: Vector2::new(0.0, 0.0),
                    target: Vector2::new(i as f64 + 5.0, i as f64 + 5.0),
                };
                
                self.agents.push(agent);
            }
        }
    }
    
    /// Get a reference to the generated SDF
    pub fn sdf(&self) -> Option<&Sdf> {
        self.sdf.as_ref()
    }
    
    /// Get a reference to the GBP environment
    pub fn environment(&self) -> &GbpEnvironment {
        &self.env
    }
}

impl Default for GbpSimulationEnvironment {
    fn default() -> Self {
        Self {
            env: GbpEnvironment::default(),
            time: 0.0,
            agents: Vec::new(),
            sdf: None,
            formation: None,
        }
    }
}

impl SimulationEnvironment for GbpSimulationEnvironment {
    fn initialize(&mut self) -> Result<(), crate::error::SimulationError> {
        self.time = 0.0;
        
        // Generate SDF from environment if it doesn't exist
        if self.sdf.is_none() {
            self.generate_sdf().map_err(|err| {
                crate::error::SimulationError::Environment(format!("Failed to generate SDF: {}", err))
            })?;
        }
        
        // Initialize agents from formation or with defaults
        self.initialize_agents();
        
        Ok(())
    }
    
    fn update(&mut self, dt: f64) -> Result<(), crate::error::SimulationError> {
        // Update time
        self.time += dt;
        
        // In a full implementation, this would run the GBP algorithm
        // and update agent positions/velocities
        // For now, we'll just have a simple linear movement toward targets
        
        for agent in &mut self.agents {
            // Simple move toward target logic
            let dir = agent.target - agent.position;
            let distance = dir.magnitude();
            
            if distance > 0.1 {
                let normalized_dir = dir / distance;
                agent.velocity = normalized_dir * 0.5; // Simple constant velocity
                agent.position = agent.position + agent.velocity * dt;
            } else {
                agent.velocity = crate::types::Vector2::new(0.0, 0.0);
            }
        }
        
        Ok(())
    }
    
    fn get_state(&self) -> EnvironmentState {
        EnvironmentState {
            time: SimTime::new(self.time),
            agents: self.agents.clone(),
        }
    }
    
    fn reset(&mut self) -> Result<(), crate::error::SimulationError> {
        self.time = 0.0;
        self.initialize()
    }
}
