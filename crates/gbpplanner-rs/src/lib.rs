//! Library interface to the GBPPlanner
use bevy::ecs::schedule::States;

pub(crate) mod asset_loader;
pub(crate) mod config;
pub(crate) mod diagnostic;
pub mod environment;
pub(crate) mod factorgraph;
pub mod input;
mod moveable_object;
pub mod movement;
pub(crate) mod pause_play;
mod planner;
mod robot_spawner;
pub mod theme;
pub mod ui;
pub(crate) mod utils;

pub(crate) mod escape_codes;
pub(crate) mod macros;

// TODO: use in app
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
    derive_more::IsVariant,
)]
pub enum SimulationState {
    #[default]
    #[display(fmt = "Loading")]
    Loading,
    #[display(fmt = "Starting")]
    Starting,
    #[display(fmt = "Running")]
    Running,
    #[display(fmt = "Paused")]
    Paused,
    #[display(fmt = "Finished")]
    Finished,
}

// TODO: use in app
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
    /* derive_more::IsVariant, */
)]
pub enum AppState {
    /// Start of the application where assets e.g. data in `./assets` is being
    /// loaded into memory
    #[default]
    #[display(fmt = "Loading")]
    Loading,
    // #[display(fmt = "Starting")]
    // Starting,
    /// A simulation is running in the application
    #[display(fmt = "Running")]
    Running,
    // #[display(fmt = "Paused")]
    // Paused,
    // #[display(fmt = "Finished")]
    // Finished,
}
