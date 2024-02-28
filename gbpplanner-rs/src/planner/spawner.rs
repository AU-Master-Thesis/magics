use bevy::prelude::*;

use crate::config::Config;

pub struct SpawnerPlugin;

impl Plugin for SpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.ass_systems(spawn);
    }
}

fn spawn(mut commands: Commands, config: Res<Config>) {}
