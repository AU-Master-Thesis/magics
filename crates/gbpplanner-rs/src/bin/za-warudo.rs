//! This example illustrates how to load and play an audio file.

use bevy::prelude::*;

fn main() {
    better_panic::debug_install();
    App::new().add_plugins(DefaultPlugins).add_systems(Startup, setup).run();
}

fn setup(asset_server: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(AudioBundle {
        source:   asset_server.load("audio/za-warudo.ogg"),
        settings: PlaybackSettings::LOOP,
        // ..default(),
    });
}
