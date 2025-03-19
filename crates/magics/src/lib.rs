//! Library interface to the GBPPlanner with UI components
//!
//! This crate provides a wrapper around the core simulation functionality
//! from `magics-core` and adds UI components for visualization and interaction.
//! It can run the simulation with a graphical interface, but can also be run
//! in headless mode for faster experimentation.

#![feature(iter_repeat_n)]
use bevy::ecs::schedule::States;

// Re-export magics-core for users of this crate
pub use magics_core;

pub mod asset_loader;
pub mod bevy_utils;
pub mod cli;
pub mod despawn_entity_after;
pub mod diagnostic;
pub mod environment;
pub mod export;
pub mod factorgraph;
pub mod goal_area;
pub mod input;
pub mod moveable_object;
pub mod movement;
pub mod pause_play;
pub mod planner;
pub mod simulation_loader;
pub mod theme;
pub mod ui;
pub(crate) mod utils;

pub(crate) mod escape_codes;
pub(crate) mod macros;

/// Set of distinct states the application can be in.
#[derive(
    Debug,
    Default,
    States,
    PartialEq,
    Eq,
    Hash,
    Clone,
    Copy,
    derive_more::Display,
)]
pub enum AppState {
    /// Start of the application where assets e.g. data in `./assets` is being
    /// loaded into memory
    #[default]
    #[display(fmt = "Loading")]
    LoadingSimulationData,
    /// A simulation is running in the application
    #[display(fmt = "Running")]
    Running,
}

/// Import common types and functions from magics-core and this crate
pub mod prelude {
    // Re-export everything from magics-core's prelude
    pub use magics_core::prelude::*;
    
    // Add magics-specific UI and visualization types
    pub use crate::asset_loader::AssetLoaderPlugin;
    pub use crate::environment::{Environment, EnvironmentPlugin};
    pub use crate::simulation_loader::{SimulationLoader, SimulationLoaderPlugin};
    pub use crate::theme::{ThemePlugin, DarkTheme, LightTheme};
    pub use crate::ui::{UiPlugin, UiState};
    pub use crate::AppState;
}
