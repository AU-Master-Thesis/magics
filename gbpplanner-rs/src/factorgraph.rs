// use factorgraph::FactorGraph;
use bevy::prelude::*;

pub struct FactorGraphPlugin {
    config: Config,
}

impl Plugin for FactorGraphPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config.clone())
            .add_systems(Update, update_factor_graph);
    }
}

fn update_factor_graph() {
    todo!()
}
