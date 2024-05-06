use std::time::Duration;

use bevy::{prelude::*, time::TimePlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // .add_plugins(DefaultPlugins.set(WindowPlugin { primary_window: None, ..default() }))
        // .add_plugins(TimePlugin)
        .add_systems(Startup, create_formation_spawners)
        .add_systems(Update, advance_time)
        .run();
}

fn s(secs: u64) -> Duration {
    Duration::from_secs(secs)
}

fn create_formation_spawners(mut commands: Commands) {
    commands.spawn(FormationSpawner::new(
        0,
        Duration::from_secs(0),
        Timer::from_seconds(2.0, TimerMode::Repeating),
    ));

    commands.spawn(FormationSpawner::new(
        1,
        Duration::from_secs(1),
        Timer::from_seconds(2.0, TimerMode::Repeating),
    ));

    commands.spawn(FormationSpawner::new(
        2,
        Duration::from_secs(3),
        Timer::from_seconds(3.0, TimerMode::Once),
    ));
}

fn advance_time(time: Res<Time>, mut formation_spawners: Query<&mut FormationSpawner>) {
    for mut formation_spawner in formation_spawners.iter_mut() {
        formation_spawner.tick(time.delta());
        if formation_spawner.ready_to_spawn() {
            println!("time elapsed: {:?}", time.elapsed());
            println!(
                "ready to spawn, {}",
                formation_spawner.formation_group_index
            );
        }
    }
}

#[derive(Debug, Component)]
pub struct FormationSpawner {
    pub formation_group_index: usize,
    initial_delay: Timer,
    timer: Timer,
}

impl std::fmt::Display for FormationSpawner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FormationSpawner {{ formation_group_index: {}, initial_delay: {:?}, timer: {:?} }}",
            self.formation_group_index, self.initial_delay, self.timer
        )
    }
}

impl FormationSpawner {
    #[must_use]
    pub fn new(formation_group_index: usize, initial_delay: Duration, timer: Timer) -> Self {
        Self {
            formation_group_index,
            initial_delay: Timer::new(initial_delay, TimerMode::Once),
            timer,
        }
    }

    #[inline]
    fn is_active(&self) -> bool {
        self.initial_delay.finished()
    }

    fn tick(&mut self, delta: Duration) {
        if self.is_active() {
            self.timer.tick(delta);
        } else {
            self.initial_delay.tick(delta);
        }
    }

    #[inline]
    fn ready_to_spawn(&self) -> bool {
        // self.timer.finished()
        self.timer.just_finished()
    }

    #[inline]
    fn on_cooldown(&self) -> bool {
        !self.ready_to_spawn()
    }
}
