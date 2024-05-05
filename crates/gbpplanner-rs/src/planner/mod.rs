// mod debug;
// mod factor;
// mod factorgraph;
// mod marginalise_factor_distance;
// mod message;
pub mod collisions;
pub mod robot;
mod spawner;
pub mod tracking;
// mod variable;
mod run_schedule;
mod visualiser;

use bevy::prelude::*;
// pub use factorgraph::{graphviz::NodeKind, FactorGraph, NodeIndex};
pub use robot::{RobotId, RobotState};
pub use visualiser::{factorgraphs::VariableVisualiser, waypoints::WaypointVisualiser, RobotTracker};

use self::{robot::RobotPlugin, spawner::RobotSpawnerPlugin, visualiser::VisualiserPlugin};

pub struct PlannerPlugin;

impl Plugin for PlannerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RobotPlugin,
            RobotSpawnerPlugin,
            VisualiserPlugin,
            collisions::RobotCollisionsPlugin,
            tracking::TrackingPlugin,
            // PlannerDebugPlugin,
        ));
    }
}
