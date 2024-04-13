#![allow(missing_docs)]

use std::time::Duration;

use bevy::prelude::*;

const SPAWNERS: usize = 3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Countdown>()
        .add_systems(Startup, (setup,))
        .add_systems(Update, (countdown, advance_time, spawn_formation))
        .run();

    Ok(())
}

#[derive(Debug, Resource)]
pub struct Countdown(Timer);

#[derive(Debug, Component, Deref, DerefMut)]
pub struct FormationSpawnerCountdown(Timer);

fn setup(mut commands: Commands) {
    for i in 0..SPAWNERS {
        info!("Spawning spawner {}", i);
        commands.spawn(FormationSpawnerCountdown(Timer::from_seconds(
            (i + 1) as f32,
            TimerMode::Repeating,
        )));
    }
}

fn advance_time(time: Res<Time>, mut query: Query<&mut FormationSpawnerCountdown>) {
    for mut countdown in query.iter_mut() {
        countdown.tick(time.delta());
    }
}

fn spawn_formation(query: Query<(Entity, &FormationSpawnerCountdown)>) {
    for (entity, countdown) in query.iter() {
        if countdown.0.just_finished() {
            info!("entity: {:?} is spawning a formation!", entity);
        }
    }
}
impl Countdown {
    pub fn new(duration: Duration) -> Self {
        Self(Timer::from_seconds(
            duration.as_secs_f32(),
            TimerMode::Repeating,
        ))
    }
}

impl Default for Countdown {
    fn default() -> Self {
        Self(Timer::from_seconds(5.0, TimerMode::Repeating))
    }
}

fn countdown(time: Res<Time>, mut countdown: ResMut<Countdown>) {
    countdown.0.tick(time.delta());
    if countdown.0.finished() {
        info!("Countdown finished!");
    }
}
