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
pub use visualiser::{factorgraphs::VariableVisualiser, waypoints::WaypointVisualiser};

use self::{robot::RobotPlugin, spawner::SpawnerPlugin, visualiser::VisualiserPlugin};

pub struct PlannerPlugin;

impl Plugin for PlannerPlugin {
    fn build(&self, app: &mut App) {
        info!("built PlannerPlugin, added RobotPlugin, SpawnerPlugin, VisualiserPlugin");
        app.init_resource::<PausePlay>().add_plugins((
            RobotPlugin,
            SpawnerPlugin,
            VisualiserPlugin,
        ));
    }
}

/// **Bevy** [`Resource`] for pausing or playing the simulation
#[derive(Default, Resource)]
pub struct PausePlay(bool);

impl PausePlay {
    pub fn pause(&mut self) {
        self.0 = false;
    }

    pub fn play(&mut self) {
        self.0 = true;
    }

    pub fn toggle(&mut self) {
        self.0 = !self.0;
    }

    pub fn is_paused(&self) -> bool {
        !self.0
    }
}
