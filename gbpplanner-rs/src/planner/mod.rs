mod factor;
mod factorgraph;
mod marginalise_factor_distance;
mod message;
pub mod robot;
mod spawner;
mod variable;
mod visualiser;

pub use factorgraph::graphviz::NodeKind;
pub use factorgraph::FactorGraph;
pub use factorgraph::NodeIndex;
pub use robot::RobotId;
pub use robot::RobotState;
pub use visualiser::factorgraphs::VariableVisualiser;
pub use visualiser::waypoints::WaypointVisualiser;

use self::robot::RobotPlugin;
use self::spawner::SpawnerPlugin;
use self::visualiser::VisualiserPlugin;
use bevy::prelude::*;

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
