mod factor;
mod factorgraph;
mod marginalise_factor_distance;
mod message;
mod robot;
mod spawner;
mod variable;
mod visualiser;

pub use factorgraph::graphviz::NodeKind;
pub use factorgraph::FactorGraph;
pub use factorgraph::NodeIndex;
pub use robot::RobotId;
pub use robot::RobotState;
pub use visualiser::VariableVisualiser;
pub use visualiser::WaypointVisualiser;


use self::robot::RobotPlugin;
use self::spawner::SpawnerPlugin;
use self::visualiser::VisualiserPlugin;
use bevy::prelude::*;

pub struct PlannerPlugin;

impl Plugin for PlannerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((RobotPlugin, SpawnerPlugin, VisualiserPlugin));
        info!("built PlannerPlugin, added RobotPlugin, SpawnerPlugin, VisualiserPlugin");
    }
}
