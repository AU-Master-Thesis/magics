//! Library interface to the GBPPlanner
use bevy::ecs::schedule::States;

pub(crate) mod asset_loader;
pub(crate) mod config;
pub mod environment;
pub mod input;
mod moveable_object;
pub mod movement;
mod planner;
mod robot_spawner;
pub mod theme;
mod toggle_fullscreen;
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
