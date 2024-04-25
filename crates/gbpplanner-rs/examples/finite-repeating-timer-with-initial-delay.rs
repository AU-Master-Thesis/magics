use std::{num::NonZeroUsize, time::Duration};

use bevy::{app::AppExit, prelude::*, time::TimePlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, create_formation_spawners)
        .add_systems(Update, (advance_time, exit_when_all_spawners_are_exhausted))
        .run();
}

fn exit_when_all_spawners_are_exhausted(
    q: Query<&FormationSpawner>,
    mut evw_app_exit: EventWriter<bevy::app::AppExit>,
) {
    if q.iter().all(|s| s.exhausted()) {
        evw_app_exit.send(AppExit);
    }
}

fn s(secs: u64) -> Duration {
    Duration::from_secs(secs)
}

fn create_formation_spawners(mut commands: Commands) {
    commands.spawn(FormationSpawner::new(
        0,
        Duration::from_secs(0),
        // RepeatingTimer::new(Duration::from_secs(2), RepeatTimes::ONCE),
        RepeatingTimer::new(Duration::from_secs(2), RepeatTimes::Finite(3)),
    ));

    commands.spawn(FormationSpawner::new(
        1,
        Duration::from_secs(1),
        RepeatingTimer::new(Duration::from_secs(1), RepeatTimes::Finite(5)),
    ));

    commands.spawn(FormationSpawner::new(
        2,
        Duration::from_secs(3),
        RepeatingTimer::new(Duration::from_secs(5), RepeatTimes::ONCE),
    ));
}

fn advance_time(time: Res<Time>, mut formation_spawners: Query<&mut FormationSpawner>) {
    for mut formation_spawner in formation_spawners.iter_mut() {
        formation_spawner.tick(time.delta());
        if formation_spawner.ready_to_spawn() {
            println!("time elapsed: {:?}", time.elapsed());
            println!("ready to spawn, {}", formation_spawner.formation_group_index);
        }
    }
}

/// Enum representing the number of times a formation should repeat.
#[derive(Debug, Clone, Copy, Default)]
pub enum RepeatTimes {
    #[default]
    Infinite,
    Finite(usize),
}

impl RepeatTimes {
    pub const ONCE: Self = RepeatTimes::Finite(1);

    /// Construct a new `RepeatTimes::Finite` variant
    pub fn finite(times: NonZeroUsize) -> Self {
        Self::Finite(times.into())
    }

    /// Construct a new `RepeatTimes::Infinite` variant
    pub fn infinite() -> Self {
        Self::Infinite
    }

    /// Returns true if there are one or more times left repeating
    pub fn exhausted(&self) -> bool {
        match self {
            RepeatTimes::Infinite => false,
            RepeatTimes::Finite(remaining) => *remaining == 0,
        }
    }

    pub fn decrement(&mut self) {
        match self {
            RepeatTimes::Finite(ref mut remaining) if *remaining > 0 => *remaining -= 1,
            _ => {} // RepeatTimes::Infinite => {},
        }
    }
}

#[derive(Debug, Clone)]
struct RepeatingTimer {
    timer:  Timer,
    repeat: RepeatTimes,
}

impl RepeatingTimer {
    fn new(duration: Duration, repeat: RepeatTimes) -> Self {
        let timer = Timer::new(duration, TimerMode::Repeating);
        Self { timer, repeat }
    }

    #[inline]
    pub fn exhausted(&self) -> bool {
        self.repeat.exhausted()
    }

    #[inline]
    pub fn tick(&mut self, delta: Duration) {
        self.timer.tick(delta);
        // if self.timer.just_finished() {
        //     self.repeat.decrement();
        // }
    }

    #[inline]
    pub fn just_finished(&mut self) -> bool {
        // self.timer.just_finished() && !self.repeat.exhausted()
        let finished = self.timer.just_finished() && !self.repeat.exhausted();
        if finished {
            self.repeat.decrement();
        }

        finished
    }
}

#[derive(Debug, Component)]
pub struct FormationSpawner {
    pub formation_group_index: usize,
    initial_delay: Timer,
    // timer: Timer,
    timer: RepeatingTimer,
}

impl FormationSpawner {
    #[must_use]
    pub fn new(formation_group_index: usize, initial_delay: Duration, timer: RepeatingTimer) -> Self {
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
    pub fn exhausted(&self) -> bool {
        self.timer.exhausted()
    }

    #[inline]
    fn ready_to_spawn(&mut self) -> bool {
        self.timer.just_finished()
    }

    #[inline]
    fn on_cooldown(&mut self) -> bool {
        !self.ready_to_spawn()
    }
}
