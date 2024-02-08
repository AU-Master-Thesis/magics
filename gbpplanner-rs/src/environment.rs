use bevy::prelude::*;

pub struct EnvironmentPlugin {
    config: Config,
}

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config.clone())
            .add_system(Startup, build_environment);
    }
}

fn build_environment() {
    todo!()
}