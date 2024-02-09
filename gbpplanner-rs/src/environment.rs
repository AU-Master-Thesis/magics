use bevy::prelude::*;
use catppuccin::Flavour;

pub struct EnvironmentPlugin;
// {
//     config: Config,
// }

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        let (r, g, b) = Flavour::Latte.base().into();
        app.insert_resource(ClearColor(Color::rgb(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
        )))
        .insert_resource(AmbientLight {
            color: Color::default(),
            brightness: 0.75,
        });
        // .add_systems(Startup, build_environment);
    }
}

// fn build_environment(mut commands: Commands) {
//     commands.spawn(Camera2dBundle::default());
// }
