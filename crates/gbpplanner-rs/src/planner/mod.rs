pub mod collisions;
pub mod mission;
pub mod robot;
pub mod spawner;
pub mod tracking;
mod visualiser;

use bevy::prelude::*;
pub use robot::{RobotConnections, RobotId};
pub use visualiser::{
    factorgraphs::VariableVisualiser, waypoints::WaypointVisualiser, RobotTracker,
};

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
            mission::MissionPlugin,
        ));
    }
}
