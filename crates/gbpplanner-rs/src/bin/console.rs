#![allow(missing_docs)]

use bevy::{log::LogPlugin, prelude::*};
use bevy_dev_console::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            // Start capturing logs before the default plugins initiate.
            ConsoleLogPlugin::default(),
            // Add the default plugins without the LogPlugin.
            // Not removing the LogPlugin will cause a panic!
            DefaultPlugins.build().disable::<LogPlugin>(),
            // Add the dev console plugin itself.
            DevConsolePlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, countdown)
        .run();
}

#[derive(Debug, Component)]
pub struct Countdown(Timer);

impl Default for Countdown {
    fn default() -> Self {
        Self(Timer::from_seconds(2.0, TimerMode::Repeating))
    }
}

fn setup(mut commands: Commands) {
    for _ in 0..10 {
        commands.spawn(Countdown::default());
    }
}

fn countdown(time: Res<Time>, mut q: Query<&mut Countdown>) {
    for mut countdown in &mut q {
        countdown.0.tick(time.delta());
        if countdown.0.just_finished() {
            info!("just finished");
        }
    }
}
