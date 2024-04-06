use bevy::prelude::*;

use super::{FactorGraph, RobotState};

pub struct PlannerDebugPlugin;

impl Plugin for PlannerDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, debug);
    }
}

fn debug(q: Query<(Entity, &RobotState, &FactorGraph)>) {
    for (entity, state, graph) in q.iter() {
        // println!("{:?}", entity);
        // println!("{:?}", state);
        // println!("{:?}", graph);
        graph.variables().for_each(|(index, variable)| {
            // println!("variable {:?}: mu {}", index, variable.belief.mu);
        })
    }
    // std::process
}
