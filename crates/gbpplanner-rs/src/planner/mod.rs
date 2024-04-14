mod debug;
mod factor;
mod factorgraph;
mod marginalise_factor_distance;
mod message;
pub mod robot;
mod spawner;
mod variable;
mod visualiser;

use bevy::prelude::*;
pub use factorgraph::{graphviz::NodeKind, FactorGraph, NodeIndex};
pub use robot::{RobotId, RobotState};
pub use visualiser::{
    factorgraphs::VariableVisualiser, waypoints::WaypointVisualiser, RobotTracker,
};

use self::{
    debug::PlannerDebugPlugin, robot::RobotPlugin, spawner::SpawnerPlugin,
    visualiser::VisualiserPlugin,
};

pub struct PlannerPlugin;

impl Plugin for PlannerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RobotPlugin,
            SpawnerPlugin,
            VisualiserPlugin,
            PlannerDebugPlugin,
        ));
    }
}
