use bevy::prelude::*;

pub struct EnvironmentPlugin;
// {
//     config: Config,
// }

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, build_environment);
    }
}

fn build_environment(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
