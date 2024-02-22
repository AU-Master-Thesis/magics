mod factor;
mod factorgraph;
mod multivariate_normal;
mod robot;
mod variable;

pub type Timestep = u32;

use self::robot::RobotPlugin;
use bevy::prelude::*;

pub struct PlannerPlugin;

impl Plugin for PlannerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RobotPlugin);
    }
}
