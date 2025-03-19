//! Core simulation functionality for the Magics planner
//!
//! This crate provides the core simulation functionality for the Magics planner,
//! without any UI or rendering dependencies. It can be used to run simulations
//! in headless mode for faster experimentation and testing.

#![warn(missing_docs)]

pub mod types;
pub mod error;
pub mod output;
pub mod simulation;
pub mod environment;
pub mod factorgraph;
pub mod planner;
pub mod sdf;
pub mod state;

/// Prelude module that re-exports commonly used types and traits
pub mod prelude {
    pub use crate::types::{
        AgentId, SimTime, AgentState, EnvironmentState, SharedEnvironmentState, Vector2,
    };
    pub use crate::error::SimulationError;
    pub use crate::output::{OutputManager, OutputFormat};
    pub use crate::simulation::{
        Simulation, SimulationRunner, SimulationConfig, OutputSettings, GbpSimulation,
    };
    pub use crate::environment::{SimulationEnvironment, GbpSimulationEnvironment};
    pub use crate::sdf::Sdf;
    pub use crate::planner::{Planner, GbpPlanner, Path};
    pub use crate::planner::robot::{RobotId, Radius, RadioAntenna, RobotConnections, PlanningStrategy};
    pub use crate::planner::collisions::{Collision, RobotCollisionDetector, ObstacleCollisionDetector};
    pub use crate::planner::mission::{Mission, MissionTask, MissionTaskType};
    pub use crate::factorgraph::{
        factorgraph::FactorGraph,
        factor::{
            Factor, 
            FactorNode, 
            FactorKind,
            FactorState,
            SdfImage,
        },
        id::{FactorId, VariableId},
        message::{Message, MessageContent},
    };
    pub use crate::factorgraph::factor::dynamic::DynamicFactor;
    pub use crate::factorgraph::factor::obstacle::{ObstacleFactor, WorldSize};
    pub use crate::factorgraph::factor::tracking::TrackingFactor;
    pub use crate::factorgraph::factor::interrobot::{InterRobotFactor, ExternalVariableId};
    pub use crate::factorgraph::factor::pose::PoseFactor;
}
