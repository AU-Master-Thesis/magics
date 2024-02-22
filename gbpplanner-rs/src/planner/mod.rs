mod robot;

use self::robot::RobotPlugin;
use bevy::prelude::*;

pub struct PlannerPlugin;

impl Plugin for PlannerPlugin {
    fn build(&self, app: &mut App) {
        app.add_resource(IdGenerator::new()).add_plugin(RobotPlugin);
    }
}

#[derive(Debug)]
struct IdGenerator {
    next_robot_id: RobotId,
    next_variable_id: NodeId,
    next_factor_id: NodeId,
}

impl IdGenerator {
    fn new() -> Self {
        Self {
            next_robot_id: 0,
            next_variable_id: 0,
            next_factor_id: 0,
        }
    }

    fn next_robot_id(&mut self) -> RobotId {
        let id = self.next_robot_id;
        self.next_robot_id += 1;
        id
    }

    fn next_variable_id(&mut self) -> NodeId {
        let id = self.next_variable_id;
        self.next_variable_id += 1;
        id
    }

    fn next_factor_id(&mut self) -> NodeId {
        let id = self.next_factor_id;
        self.next_factor_id += 1;
        id
    }
}
