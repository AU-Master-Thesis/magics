//! Module for pausing and resuming the simulation.

use bevy::prelude::*;

/// Plugin for pausing and resuming the simulation.
pub struct PausePlayPlugin;

impl Plugin for PausePlayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(PausedState::default())
            .add_event::<PausePlay>()
            .add_systems(PreUpdate, pause_play_virtual_time);
    }
}

/// State keeping track of whether the simulation is paused or not.
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash, derive_more::IsVariant)]
pub enum PausedState {
    #[default]
    Running,
    Paused,
}

/// Event for pausing and resuming the simulation.
#[derive(Debug, Clone, Copy, Default, Event)]
pub enum PausePlay {
    #[default]
    Toggle,
    Pause,
    Play,
}

/// System that reacts to events for pausing and resuming the simulation.
fn pause_play_virtual_time(
    state: ResMut<State<PausedState>>,
    mut next_state: ResMut<NextState<PausedState>>,
    mut pause_play_event: EventReader<PausePlay>,
    mut time: ResMut<Time<Virtual>>,
) {
    for pause_play_event in pause_play_event.read() {
        debug!("received event: {:?}", pause_play_event);
        match (pause_play_event, state.get()) {
            (PausePlay::Pause, PausedState::Paused) | (PausePlay::Play, PausedState::Running) => {
                warn!("ignoring duplicate event: {:?}", pause_play_event);
                continue;
            }
            (PausePlay::Pause | PausePlay::Toggle, PausedState::Running) => {
                next_state.set(PausedState::Paused);
                time.pause();
            }
            (PausePlay::Play | PausePlay::Toggle, PausedState::Paused) => {
                next_state.set(PausedState::Running);
                time.unpause();
            }
        };
    }
}
