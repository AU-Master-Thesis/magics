//! Library interface to the GBPPlanner
use bevy::ecs::schedule::States;

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
// pub mod prng;

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
    LoadingSimulationData,
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
